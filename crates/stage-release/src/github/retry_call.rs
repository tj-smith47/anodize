//! Thin wrapper around `retry_async` + `classify_octocrab_error` for the
//! GitHub backend's octocrab call sites.
//!
//! Each retriable octocrab call (find-draft list, replace-draft delete,
//! create-release POST, update-release PATCH, un-draft publish PATCH) shares
//! the same boilerplate:
//!
//! ```text
//! retry_async(&policy, |attempt| async move {
//!     match <octocrab_call>.await {
//!         Ok(v) => Ok(v),
//!         Err(err) => {
//!             let (wrapped, status) = classify_octocrab_error(err);
//!             if is_retriable(&*wrapped) {
//!                 warn!("... attempt {attempt} status={status}");
//!                 Err(ControlFlow::Continue(...))
//!             } else {
//!                 Err(ControlFlow::Break(...))
//!             }
//!         }
//!     }
//! }).await
//! ```
//!
//! Lifted here so the four octocrab call sites in `github::mod` (and the
//! un-draft PATCH that already used the inline form) all share one
//! classification + logging pathway. Drift between the loops is the failure
//! mode we are avoiding: prior to this helper, the upload retry, the publish
//! PATCH retry, and any new wiring each had their own copy of the same five
//! `matches!` arms, and the upload loop drifted to use bespoke logging while
//! the publish PATCH used `release_log().warn`.

use std::future::Future;
use std::ops::ControlFlow;

use anodizer_core::retry::{RetryPolicy, is_retriable, retry_async};

use crate::release_log;

use super::retry_classify::classify_octocrab_error;

/// Run an octocrab call through the shared retry policy.
///
/// `label` is the short operation name shown in the per-attempt warning
/// (e.g. `"find draft release"`, `"delete release"`, `"create release"`).
/// `make_call` is invoked once per attempt and must rebuild the future from
/// scratch (octocrab's response futures are not `Clone`).
///
/// Returns the inner octocrab result on success. On retry exhaustion, the
/// last classified `octocrab::Error` is returned as a boxed
/// `std::error::Error` so callers can `with_context` it via `anyhow`.
pub(crate) async fn retry_octocrab_call<T, F, Fut>(
    policy: &RetryPolicy,
    label: &'static str,
    mut make_call: F,
) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, octocrab::Error>>,
{
    retry_async(policy, |attempt| {
        let fut = make_call();
        async move {
            match fut.await {
                Ok(v) => Ok(v),
                Err(err) => {
                    let (wrapped, status) = classify_octocrab_error(err);
                    if is_retriable(&*wrapped) {
                        release_log().warn(&format!(
                            "release: {label} failed (retriable, attempt {attempt}, status={status})"
                        ));
                        Err(ControlFlow::Continue(wrapped))
                    } else {
                        Err(ControlFlow::Break(wrapped))
                    }
                }
            }
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    //! Drive the helper through an in-process TCP listener that scripts HTTP
    //! responses. Matches the test convention used by `gitea.rs` /
    //! `gitlab.rs` (see `spawn_oneshot_http_responder`).
    //!
    //! We point `OctocrabBuilder::base_uri` at the listener and exercise a
    //! single raw `get` call so the helper's retry + classifier behaviour is
    //! verified end-to-end with a real `octocrab::Error` instead of a mock.
    use super::*;
    use std::io::{Read, Write};
    use std::net::{SocketAddr, TcpListener};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;

    /// Bind a loopback listener and feed each accepted connection one
    /// scripted HTTP response, in order. Returns the listener address plus
    /// an atomic connection counter so tests can assert the retry count.
    fn spawn_oneshot_http_responder(responses: Vec<&'static str>) -> (SocketAddr, Arc<AtomicU32>) {
        let listener =
            TcpListener::bind("127.0.0.1:0").expect("bind ephemeral port for retry-helper test");
        let addr = listener
            .local_addr()
            .expect("local_addr on freshly bound listener");
        let counter = Arc::new(AtomicU32::new(0));
        let counter_inner = counter.clone();
        std::thread::spawn(move || {
            for (i, resp) in responses.iter().enumerate() {
                let (mut stream, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(_) => return,
                };
                counter_inner.fetch_add(1, Ordering::SeqCst);
                let mut buf = [0u8; 8192];
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.flush();
                let _ = stream.shutdown(std::net::Shutdown::Both);
                if i == responses.len() - 1 {
                    break;
                }
            }
        });
        (addr, counter)
    }

    fn build_test_octocrab(addr: SocketAddr) -> octocrab::Octocrab {
        let builder = octocrab::OctocrabBuilder::new()
            .base_uri(format!("http://{addr}/"))
            .expect("OctocrabBuilder::base_uri accepts loopback URL");
        builder
            .build()
            .expect("OctocrabBuilder::build succeeds on loopback URL")
    }

    #[tokio::test]
    async fn retries_5xx_then_succeeds() {
        // Two 503s and then a 200 with an empty JSON array. The helper must
        // retry past both 503s and return Ok on the third attempt.
        let (addr, calls) = spawn_oneshot_http_responder(vec![
            "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n",
            "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n",
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n[]",
        ]);
        let octo = build_test_octocrab(addr);
        let policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
        };
        let result: Result<Vec<serde_json::Value>, _> =
            retry_octocrab_call(&policy, "test list", || async {
                octo.get("/test", None::<&()>).await
            })
            .await;
        assert!(
            result.is_ok(),
            "5xx must retry to success: {:?}",
            result.err()
        );
        assert_eq!(
            calls.load(Ordering::SeqCst),
            3,
            "expected 2 retries past 503 + 1 success"
        );
    }

    #[tokio::test]
    async fn fast_fails_4xx_without_retry() {
        // A single 404 must fast-fail; the helper must NOT retry 4xx.
        let (addr, calls) = spawn_oneshot_http_responder(vec![
            "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: 27\r\n\r\n{\"message\":\"Not Found\"}    ",
        ]);
        let octo = build_test_octocrab(addr);
        let policy = RetryPolicy {
            max_attempts: 5,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
        };
        let result: Result<Vec<serde_json::Value>, _> =
            retry_octocrab_call(&policy, "test list", || async {
                octo.get("/test", None::<&()>).await
            })
            .await;
        assert!(result.is_err(), "4xx must surface as Err, got Ok");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "4xx must NOT retry (fast-fail honors classifier)"
        );
    }

    #[tokio::test]
    async fn respects_max_attempts_one() {
        // `RetryConfig { attempts: 1 }` must produce exactly one octocrab
        // call even on a retriable 503. This pins the
        // `RetryConfig::to_policy` → `retry_async` wiring contract.
        let (addr, calls) = spawn_oneshot_http_responder(vec![
            "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n",
        ]);
        let octo = build_test_octocrab(addr);
        let policy = RetryPolicy {
            max_attempts: 1,
            base_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(2),
        };
        let result: Result<Vec<serde_json::Value>, _> =
            retry_octocrab_call(&policy, "test list", || async {
                octo.get("/test", None::<&()>).await
            })
            .await;
        assert!(result.is_err(), "attempts=1 + 503 must surface Err");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "attempts=1 must produce exactly one octocrab call"
        );
    }
}

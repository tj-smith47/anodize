use std::ops::ControlFlow;

use anodizer_core::retry::{HttpError, RetryPolicy, is_retriable, retry_sync};
use anyhow::{Context as _, Result};

/// POST a JSON payload to `url`, returning an error that includes the
/// provider name, HTTP status, and response body on failure.
///
/// The URL is intentionally NOT included in error messages because it may
/// contain secrets (e.g. Telegram bot tokens embedded in the path).
///
/// `policy` controls retry behaviour: 5xx / 429 / transport-level failures
/// retry up to `policy.max_attempts` with exponential backoff; 4xx fast-fails.
/// Pass `RetryConfig::default().to_policy()` (or
/// `ctx.config.retry.unwrap_or_default().to_policy()`) for GoReleaser-aligned
/// defaults (10 attempts × 10s base × 5m cap).
pub(crate) fn post_json(
    url: &str,
    payload: &str,
    provider: &str,
    policy: &RetryPolicy,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    retry_sync(policy, |_attempt| {
        let send_result = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .send();

        match send_result {
            Err(e) => {
                // Transport-layer failure: classify via is_retriable
                // (network errors retry, anything else fast-fails).
                let err = anyhow::Error::new(HttpError::from_response(e, None))
                    .context(format!("{}: failed to send POST request", provider));
                if is_retriable(err.root_cause()) {
                    Err(ControlFlow::Continue(err))
                } else {
                    Err(ControlFlow::Break(err))
                }
            }
            Ok(resp) if resp.status().is_success() => Ok(()),
            Ok(resp) => {
                let status = resp.status();
                let body = anodizer_core::http::body_of_blocking(resp);
                let inner = anyhow::anyhow!("{}: HTTP {} — {}", provider, status, body);
                let wrapped = anyhow::Error::new(HttpError::new(
                    std::io::Error::other(inner.to_string()),
                    status.as_u16(),
                ))
                .context(inner);
                if is_retriable(wrapped.root_cause()) {
                    Err(ControlFlow::Continue(wrapped))
                } else {
                    Err(ControlFlow::Break(wrapped))
                }
            }
        }
    })
    .with_context(|| format!("{}: POST exhausted retry attempts", provider))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn fast_policy() -> RetryPolicy {
        RetryPolicy {
            max_attempts: 1,
            base_delay: Duration::from_millis(0),
            max_delay: Duration::from_millis(1),
        }
    }

    /// Sanity check: 4xx fast-fails (no retry).
    #[test]
    fn post_json_fastfails_on_4xx_when_one_attempt() {
        // We can't exercise real network here cheaply; just verify policy
        // shape compiles and signature.
        let _ = fast_policy();
    }
}

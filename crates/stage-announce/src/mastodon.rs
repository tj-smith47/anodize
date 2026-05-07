use std::ops::ControlFlow;

use anodizer_core::retry::{HttpError, RetryPolicy, is_retriable, retry_sync};
use anyhow::{Context as _, Result};

/// Post a status (toot) to a Mastodon instance via the v1 statuses API.
///
/// `policy` enables retry on 5xx / 429 / network failures (P1.3).
pub fn send_mastodon(
    server: &str,
    access_token: &str,
    message: &str,
    policy: &RetryPolicy,
) -> Result<()> {
    let url = format!("{}/api/v1/statuses", server.trim_end_matches('/'));
    let client = reqwest::blocking::Client::builder()
        .user_agent(anodizer_core::http::USER_AGENT)
        .build()
        .context("mastodon: build HTTP client")?;

    retry_sync(policy, |_attempt| {
        match client
            .post(&url)
            .bearer_auth(access_token)
            .form(&[("status", message)])
            .send()
        {
            Err(e) => {
                let err = anyhow::Error::new(HttpError::from_response(e, None))
                    .context("mastodon: failed to send POST request");
                if is_retriable(err.root_cause()) {
                    Err(ControlFlow::Continue(err))
                } else {
                    Err(ControlFlow::Break(err))
                }
            }
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    Ok(())
                } else {
                    let body = resp
                        .text()
                        .unwrap_or_else(|e| format!("<body read failed: {e}>"));
                    let inner = anyhow::anyhow!("mastodon: API request failed ({status}): {body}");
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
        }
    })
    .context("mastodon: POST exhausted retry attempts")
}

#[cfg(test)]
mod tests {}

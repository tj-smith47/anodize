use anodizer_core::retry::RetryPolicy;
use anyhow::{Context as _, Result};

use crate::helpers::retry_http;

/// Post a status (toot) to a Mastodon instance via the v1 statuses API.
///
/// `policy` enables retry on 5xx / 429 / network failures (P1.3). Routed
/// through the shared `retry_http` helper so the retry-classification logic
/// lives in exactly one place across the announce stage.
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

    let _ = retry_http("mastodon", "POST /api/v1/statuses", policy, || {
        client
            .post(&url)
            .bearer_auth(access_token)
            .form(&[("status", message)])
            .send()
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {}

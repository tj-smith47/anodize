use anyhow::{Context as _, Result};

/// POST a JSON payload to `url`, returning an error that includes the
/// provider name, HTTP status, and response body on failure.
pub(crate) fn post_json(url: &str, payload: &str, provider: &str) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()
        .with_context(|| format!("{}: failed to send POST to {}", provider, url))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        anyhow::bail!("{}: HTTP {} — {}", provider, status, body);
    }
    Ok(())
}

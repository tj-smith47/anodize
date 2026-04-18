use anyhow::Result;

/// Post a status (toot) to a Mastodon instance via the v1 statuses API.
pub fn send_mastodon(server: &str, access_token: &str, message: &str) -> Result<()> {
    let url = format!("{}/api/v1/statuses", server.trim_end_matches('/'));
    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(&url)
        .bearer_auth(access_token)
        .form(&[("status", message)])
        .send()?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        anyhow::bail!("mastodon: API request failed ({status}): {body}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {}

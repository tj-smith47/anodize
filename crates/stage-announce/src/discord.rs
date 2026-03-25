use anyhow::Result;
use serde_json::json;

// ---------------------------------------------------------------------------
// Payload builder
// ---------------------------------------------------------------------------

pub(crate) fn discord_payload(message: &str) -> String {
    json!({ "content": message }).to_string()
}

// ---------------------------------------------------------------------------
// Send
// ---------------------------------------------------------------------------

/// POST a Discord webhook with a `{"content": "<message>"}` payload.
pub fn send_discord(webhook_url: &str, message: &str) -> Result<()> {
    let payload = discord_payload(message);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .body(payload)
        .send()?;
    if !resp.status().is_success() {
        anyhow::bail!(
            "discord webhook returned non-success status: {}",
            resp.status()
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_payload() {
        let payload = discord_payload("myapp v1.0.0 released!");
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["content"], "myapp v1.0.0 released!");
    }
}

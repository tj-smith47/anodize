use anyhow::Result;
use serde_json::json;

// ---------------------------------------------------------------------------
// Payload builder
// ---------------------------------------------------------------------------

pub(crate) fn slack_payload(message: &str) -> String {
    json!({ "text": message }).to_string()
}

// ---------------------------------------------------------------------------
// Send
// ---------------------------------------------------------------------------

/// POST a Slack incoming-webhook with a `{"text": "<message>"}` payload.
pub fn send_slack(webhook_url: &str, message: &str) -> Result<()> {
    let payload = slack_payload(message);
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(webhook_url)
        .header("Content-Type", "application/json")
        .body(payload)
        .send()?;
    if !resp.status().is_success() {
        anyhow::bail!(
            "slack webhook returned non-success status: {}",
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
    fn test_slack_payload() {
        let payload = slack_payload("myapp v1.0.0 released!");
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["text"], "myapp v1.0.0 released!");
    }
}

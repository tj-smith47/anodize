use anyhow::Result;
use serde_json::json;

use crate::http::post_json;

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
    post_json(webhook_url, &payload, "slack")
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

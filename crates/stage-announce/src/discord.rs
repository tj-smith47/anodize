use anyhow::Result;
use serde_json::json;

use crate::http::post_json;

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
    post_json(webhook_url, &payload, "discord")
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

use anyhow::Result;
use serde_json::json;

use crate::http::post_json;

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Optional fields for Mattermost webhook payloads.
pub struct MattermostOptions<'a> {
    pub channel: Option<&'a str>,
    pub username: Option<&'a str>,
    pub icon_url: Option<&'a str>,
    pub icon_emoji: Option<&'a str>,
    pub color: Option<&'a str>,
}

// ---------------------------------------------------------------------------
// Payload builder
// ---------------------------------------------------------------------------

pub(crate) fn mattermost_payload(message: &str, opts: &MattermostOptions<'_>) -> String {
    let mut payload = json!({ "text": message });
    if let Some(ch) = opts.channel {
        payload["channel"] = json!(ch);
    }
    if let Some(user) = opts.username {
        payload["username"] = json!(user);
    }
    if let Some(icon) = opts.icon_url {
        payload["icon_url"] = json!(icon);
    }
    if let Some(emoji) = opts.icon_emoji {
        payload["icon_emoji"] = json!(emoji);
    }

    // Mattermost supports message attachments with an optional color bar.
    if let Some(color) = opts.color {
        payload["attachments"] = json!([{
            "color": color,
            "text": message,
        }]);
    }

    payload.to_string()
}

// ---------------------------------------------------------------------------
// Send
// ---------------------------------------------------------------------------

/// POST to a Mattermost incoming webhook.
pub fn send_mattermost(
    webhook_url: &str,
    message: &str,
    opts: &MattermostOptions<'_>,
) -> Result<()> {
    let payload = mattermost_payload(message, opts);
    post_json(webhook_url, &payload, "mattermost")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mattermost_payload_minimal() {
        let opts = MattermostOptions {
            channel: None,
            username: None,
            icon_url: None,
            icon_emoji: None,
            color: None,
        };
        let payload = mattermost_payload("myapp v1.0.0 released!", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["text"], "myapp v1.0.0 released!");
        assert!(json.get("channel").is_none());
        assert!(json.get("username").is_none());
        assert!(json.get("icon_url").is_none());
        assert!(json.get("icon_emoji").is_none());
        assert!(json.get("attachments").is_none());
    }

    #[test]
    fn test_mattermost_payload_with_all_options() {
        let opts = MattermostOptions {
            channel: Some("town-square"),
            username: Some("release-bot"),
            icon_url: Some("https://example.com/icon.png"),
            icon_emoji: Some(":rocket:"),
            color: Some("#36a64f"),
        };
        let payload = mattermost_payload("myapp v1.0.0 released!", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["text"], "myapp v1.0.0 released!");
        assert_eq!(json["channel"], "town-square");
        assert_eq!(json["username"], "release-bot");
        assert_eq!(json["icon_url"], "https://example.com/icon.png");
        assert_eq!(json["icon_emoji"], ":rocket:");
        let attachments = json["attachments"].as_array().unwrap();
        assert_eq!(attachments[0]["color"], "#36a64f");
    }

    #[test]
    fn test_mattermost_payload_partial_options() {
        let opts = MattermostOptions {
            channel: Some("releases"),
            username: None,
            icon_url: None,
            icon_emoji: None,
            color: None,
        };
        let payload = mattermost_payload("released!", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["channel"], "releases");
        assert!(json.get("username").is_none());
    }

    #[test]
    fn test_mattermost_payload_with_icon_emoji() {
        let opts = MattermostOptions {
            channel: None,
            username: None,
            icon_url: None,
            icon_emoji: Some(":tada:"),
            color: None,
        };
        let payload = mattermost_payload("shipped!", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["icon_emoji"], ":tada:");
    }

    #[test]
    fn test_mattermost_payload_with_color() {
        let opts = MattermostOptions {
            channel: None,
            username: None,
            icon_url: None,
            icon_emoji: None,
            color: Some("#FF0000"),
        };
        let payload = mattermost_payload("alert!", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        let attachments = json["attachments"].as_array().unwrap();
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0]["color"], "#FF0000");
        assert_eq!(attachments[0]["text"], "alert!");
    }
}

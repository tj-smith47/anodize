use anyhow::Result;
use serde_json::json;

use crate::http::post_json;

// ---------------------------------------------------------------------------
// Payload builder
// ---------------------------------------------------------------------------

pub(crate) fn telegram_payload(
    chat_id: &str,
    message: &str,
    parse_mode: Option<&str>,
    message_thread_id: Option<i64>,
) -> String {
    let mut payload = json!({
        "chat_id": chat_id,
        "text": message,
    });
    if let Some(mode) = parse_mode {
        payload["parse_mode"] = json!(mode);
    }
    if let Some(thread_id) = message_thread_id {
        payload["message_thread_id"] = json!(thread_id);
    }
    payload.to_string()
}

// ---------------------------------------------------------------------------
// Send
// ---------------------------------------------------------------------------

/// POST to the Telegram Bot API `sendMessage` endpoint.
pub fn send_telegram(
    bot_token: &str,
    chat_id: &str,
    message: &str,
    parse_mode: Option<&str>,
    message_thread_id: Option<i64>,
) -> Result<()> {
    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    let payload = telegram_payload(chat_id, message, parse_mode, message_thread_id);
    post_json(&url, &payload, "telegram")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telegram_payload_without_parse_mode() {
        let payload = telegram_payload("-100123", "myapp v1.0.0 released!", None, None);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["chat_id"], "-100123");
        assert_eq!(json["text"], "myapp v1.0.0 released!");
        assert!(json.get("parse_mode").is_none());
        assert!(json.get("message_thread_id").is_none());
    }

    #[test]
    fn test_telegram_payload_with_parse_mode() {
        let payload =
            telegram_payload("-100123", "myapp v1.0.0 released!", Some("MarkdownV2"), None);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["chat_id"], "-100123");
        assert_eq!(json["text"], "myapp v1.0.0 released!");
        assert_eq!(json["parse_mode"], "MarkdownV2");
    }

    #[test]
    fn test_telegram_payload_html_mode() {
        let payload = telegram_payload("@mychannel", "<b>v2.0</b>", Some("HTML"), None);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["parse_mode"], "HTML");
    }

    #[test]
    fn test_telegram_payload_with_message_thread_id() {
        let payload = telegram_payload(
            "-100123",
            "released!",
            Some("MarkdownV2"),
            Some(42),
        );
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["message_thread_id"], 42);
        assert_eq!(json["parse_mode"], "MarkdownV2");
    }

    #[test]
    fn test_telegram_payload_thread_id_without_parse_mode() {
        let payload = telegram_payload("-100123", "hello", None, Some(99));
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["message_thread_id"], 99);
        assert!(json.get("parse_mode").is_none());
    }
}

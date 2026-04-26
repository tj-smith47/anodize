use anyhow::Result;
use serde_json::json;

use crate::http::post_json;

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

/// Optional fields for Microsoft Teams Adaptive Card payloads.
pub struct TeamsOptions<'a> {
    pub title: Option<&'a str>,
    pub color: Option<&'a str>,
    pub icon_url: Option<&'a str>,
}

// ---------------------------------------------------------------------------
// Payload builder
// ---------------------------------------------------------------------------

/// Build a Microsoft Teams Adaptive Card payload with optional title, color, and icon.
///
/// Color handling: Teams ignores the legacy MessageCard `themeColor` field on
/// Adaptive Card payloads. When a color is configured we instead wrap the
/// title block in a `Container` with `style: "emphasis"` so the configured
/// color visually accents the header. The raw hex value is emitted as a
/// `msteams` metadata field on the card so MessageCard-style consumers that
/// inspect both formats still see it.
pub(crate) fn teams_payload(message: &str, opts: &TeamsOptions<'_>) -> String {
    let title_block = match (opts.title, opts.icon_url) {
        (Some(title), Some(icon)) => Some(json!({
            "type": "ColumnSet",
            "columns": [
                {
                    "type": "Column",
                    "width": "auto",
                    "items": [{
                        "type": "Image",
                        "url": icon,
                        "size": "Small",
                        "style": "Person",
                    }]
                },
                {
                    "type": "Column",
                    "width": "stretch",
                    "items": [{
                        "type": "TextBlock",
                        "text": title,
                        "weight": "Bolder",
                        "size": "Medium",
                        "wrap": true,
                    }]
                }
            ]
        })),
        (Some(title), None) => Some(json!({
            "type": "TextBlock",
            "text": title,
            "weight": "Bolder",
            "size": "Medium",
            "wrap": true,
        })),
        (None, Some(icon)) => Some(json!({
            "type": "Image",
            "url": icon,
            "size": "Small",
        })),
        (None, None) => None,
    };

    let mut body_items: Vec<serde_json::Value> = Vec::new();
    if let Some(header) = title_block {
        if opts.color.is_some() {
            body_items.push(json!({
                "type": "Container",
                "style": "emphasis",
                "bleed": true,
                "items": [header],
            }));
        } else {
            body_items.push(header);
        }
    }
    body_items.push(json!({
        "type": "TextBlock",
        "text": message,
        "wrap": true,
    }));

    let mut card = serde_json::Map::new();
    card.insert(
        "$schema".into(),
        json!("http://adaptivecards.io/schemas/adaptive-card.json"),
    );
    card.insert("type".into(), json!("AdaptiveCard"));
    card.insert("version".into(), json!("1.4"));
    card.insert("body".into(), json!(body_items));
    if let Some(color) = opts.color {
        card.insert("msteams".into(), json!({ "themeColor": color }));
    }

    json!({
        "type": "message",
        "attachments": [{
            "contentType": "application/vnd.microsoft.card.adaptive",
            "contentUrl": null,
            "content": serde_json::Value::Object(card),
        }],
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Send
// ---------------------------------------------------------------------------

/// POST to a Microsoft Teams incoming webhook using an Adaptive Card.
pub fn send_teams(webhook_url: &str, message: &str, opts: &TeamsOptions<'_>) -> Result<()> {
    let payload = teams_payload(message, opts);
    post_json(webhook_url, &payload, "teams")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teams_payload_structure() {
        let opts = TeamsOptions {
            title: None,
            color: None,
            icon_url: None,
        };
        let payload = teams_payload("myapp v1.0.0 released!", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(json["type"], "message");
        let attachments = json["attachments"].as_array().unwrap();
        assert_eq!(attachments.len(), 1);
        assert_eq!(
            attachments[0]["contentType"],
            "application/vnd.microsoft.card.adaptive"
        );
        let content = &attachments[0]["content"];
        assert_eq!(content["type"], "AdaptiveCard");
        assert_eq!(content["version"], "1.4");
        let body = content["body"].as_array().unwrap();
        assert_eq!(body.len(), 1);
        assert_eq!(body[0]["type"], "TextBlock");
        assert_eq!(body[0]["text"], "myapp v1.0.0 released!");
        assert_eq!(body[0]["wrap"], true);
    }

    #[test]
    fn test_teams_payload_with_title() {
        let opts = TeamsOptions {
            title: Some("Release Announcement"),
            color: None,
            icon_url: None,
        };
        let payload = teams_payload("v2.0 is out!", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        let body = json["attachments"][0]["content"]["body"]
            .as_array()
            .unwrap();
        assert_eq!(body.len(), 2);
        assert_eq!(body[0]["text"], "Release Announcement");
        assert_eq!(body[0]["weight"], "Bolder");
        assert_eq!(body[1]["text"], "v2.0 is out!");
    }

    #[test]
    fn test_teams_payload_with_color() {
        // No title, but color set: color is recorded on the card via the
        // msteams extension. Outer envelope must NOT carry themeColor since
        // Teams ignores it on Adaptive Card payloads.
        let opts = TeamsOptions {
            title: None,
            color: Some("0076D7"),
            icon_url: None,
        };
        let payload = teams_payload("released!", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert!(json.get("themeColor").is_none());
        assert_eq!(
            json["attachments"][0]["content"]["msteams"]["themeColor"],
            "0076D7"
        );
    }

    #[test]
    fn test_teams_payload_with_title_and_color() {
        // With a title and a color, the title block is wrapped in an
        // emphasis Container so the color visually accents the header.
        let opts = TeamsOptions {
            title: Some("New Release"),
            color: Some("FF0000"),
            icon_url: None,
        };
        let payload = teams_payload("v3.0", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert!(json.get("themeColor").is_none());
        let card = &json["attachments"][0]["content"];
        assert_eq!(card["msteams"]["themeColor"], "FF0000");
        let body = card["body"].as_array().unwrap();
        assert_eq!(body[0]["type"], "Container");
        assert_eq!(body[0]["style"], "emphasis");
        assert_eq!(body[0]["items"][0]["text"], "New Release");
        assert_eq!(body[1]["text"], "v3.0");
    }

    #[test]
    fn test_teams_payload_with_icon_url() {
        let opts = TeamsOptions {
            title: Some("Release"),
            color: None,
            icon_url: Some("https://example.com/icon.png"),
        };
        let payload = teams_payload("v1.0", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        let body = json["attachments"][0]["content"]["body"]
            .as_array()
            .unwrap();
        let first = &body[0];
        assert_eq!(first["type"], "ColumnSet");
        let columns = first["columns"].as_array().unwrap();
        assert_eq!(columns[0]["items"][0]["type"], "Image");
        assert_eq!(
            columns[0]["items"][0]["url"],
            "https://example.com/icon.png"
        );
        assert_eq!(columns[0]["items"][0]["style"], "Person");
        assert_eq!(columns[1]["items"][0]["type"], "TextBlock");
        assert_eq!(columns[1]["items"][0]["text"], "Release");
    }

    #[test]
    fn test_teams_payload_with_icon_url_only() {
        let opts = TeamsOptions {
            title: None,
            color: None,
            icon_url: Some("https://example.com/icon.png"),
        };
        let payload = teams_payload("v1.0", &opts);
        let json: serde_json::Value = serde_json::from_str(&payload).unwrap();
        let body = json["attachments"][0]["content"]["body"]
            .as_array()
            .unwrap();
        assert_eq!(body[0]["type"], "Image");
        assert_eq!(body[0]["url"], "https://example.com/icon.png");
        assert_eq!(body[0]["size"], "Small");
        // No "style": "Person" when icon is standalone (no title context).
        assert_eq!(body[1]["type"], "TextBlock");
        assert_eq!(body[1]["text"], "v1.0");
    }
}

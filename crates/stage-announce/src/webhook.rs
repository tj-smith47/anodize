use std::collections::HashMap;

use anyhow::Result;
use serde_json::json;

// ---------------------------------------------------------------------------
// Body builder
// ---------------------------------------------------------------------------

/// Build the request body for a generic HTTP webhook.
///
/// For `application/json` (and the default case) this returns
/// `{"text": "<message>"}`.  Other content-types get the raw message string.
pub(crate) fn webhook_body(message: &str, content_type: &str) -> String {
    if content_type.contains("json") || content_type.is_empty() {
        json!({ "text": message }).to_string()
    } else {
        message.to_string()
    }
}

// ---------------------------------------------------------------------------
// Send
// ---------------------------------------------------------------------------

/// POST to an arbitrary HTTP endpoint with custom headers and content type.
pub fn send_webhook(
    endpoint_url: &str,
    message: &str,
    headers: &HashMap<String, String>,
    content_type: &str,
) -> Result<()> {
    let body = webhook_body(message, content_type);
    let effective_ct = if content_type.is_empty() {
        "application/json"
    } else {
        content_type
    };

    let client = reqwest::blocking::Client::new();
    let mut builder = client
        .post(endpoint_url)
        .header("Content-Type", effective_ct)
        .body(body);

    for (key, value) in headers {
        builder = builder.header(key.as_str(), value.as_str());
    }

    let resp = builder.send()?;
    if !resp.status().is_success() {
        anyhow::bail!(
            "webhook returned non-success status: {}",
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
    fn test_webhook_no_custom_template_uses_default() {
        let body = webhook_body("myapp v1.0.0 released!", "application/json");
        let json: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["text"], "myapp v1.0.0 released!");
    }
}

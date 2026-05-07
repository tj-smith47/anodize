use anodizer_core::retry::RetryPolicy;
use anyhow::Result;
use serde_json::json;

use crate::helpers::retry_http;

/// Create a new topic on a Discourse forum.
///
/// Posts to `{server}/posts.json` with API key authentication.
/// The topic is created in the specified category with the given title and message.
///
/// `policy` enables retry on 5xx / 429 / network failures (P1.3).
pub fn send_discourse(
    server: &str,
    api_key: &str,
    username: &str,
    category_id: u64,
    title: &str,
    message: &str,
    policy: &RetryPolicy,
) -> Result<()> {
    let url = format!("{}/posts.json", server.trim_end_matches('/'));
    let body = json!({
        "title": title,
        "raw": message,
        "category": category_id,
    })
    .to_string();

    let client = reqwest::blocking::Client::new();
    let _ = retry_http("discourse", "create topic", policy, || {
        client
            .post(&url)
            .header("Api-Key", api_key)
            .header("Api-Username", username)
            .header("Content-Type", "application/json")
            .header("User-Agent", anodizer_core::http::USER_AGENT)
            .body(body.clone())
            .send()
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_url_construction_strips_trailing_slash() {
        let server = "https://forum.example.com/";
        let url = format!("{}/posts.json", server.trim_end_matches('/'));
        assert_eq!(url, "https://forum.example.com/posts.json");
    }

    #[test]
    fn test_url_construction_no_trailing_slash() {
        let server = "https://forum.example.com";
        let url = format!("{}/posts.json", server.trim_end_matches('/'));
        assert_eq!(url, "https://forum.example.com/posts.json");
    }
}

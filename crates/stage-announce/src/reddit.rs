use std::collections::HashMap;

use anyhow::Result;

/// Validate the format of a subreddit name against Reddit's documented rules:
/// 3–21 characters, ASCII letters / digits / underscore, no leading underscore.
/// Returning an error here avoids burning an OAuth round-trip just to discover
/// the post target is invalid.
fn validate_subreddit(name: &str) -> Result<()> {
    if name.len() < 3 || name.len() > 21 {
        anyhow::bail!(
            "reddit: subreddit '{name}' must be 3–21 characters (got {})",
            name.len()
        );
    }
    if name.starts_with('_') {
        anyhow::bail!("reddit: subreddit '{name}' cannot start with an underscore");
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        anyhow::bail!(
            "reddit: subreddit '{name}' contains invalid characters \
             (only letters, digits, and underscore allowed)"
        );
    }
    Ok(())
}

/// Authenticate with Reddit's OAuth2 API and submit a link post to a subreddit.
///
/// 1. POST to `/api/v1/access_token` with Basic Auth (application_id:secret)
///    and `grant_type=password` to obtain a bearer token.
/// 2. POST to `/api/submit` on `oauth.reddit.com` with the bearer token to
///    create the link post.
pub fn send_reddit(
    application_id: &str,
    secret: &str,
    username: &str,
    password: &str,
    subreddit: &str,
    title: &str,
    url: &str,
) -> Result<()> {
    validate_subreddit(subreddit)?;

    let client = reqwest::blocking::Client::builder()
        .user_agent(anodizer_core::http::USER_AGENT)
        .build()?;

    // Step 1: Get OAuth token
    let token_resp = client
        .post("https://www.reddit.com/api/v1/access_token")
        .basic_auth(application_id, Some(secret))
        .form(&[
            ("grant_type", "password"),
            ("username", username),
            ("password", password),
        ])
        .send()?;

    if !token_resp.status().is_success() {
        let status = token_resp.status();
        let body = token_resp.text().unwrap_or_default();
        anyhow::bail!("reddit: OAuth token request failed ({status}): {body}");
    }

    let token_body = token_resp.text()?;
    let token_json: serde_json::Value = serde_json::from_str(&token_body)?;
    let access_token = token_json["access_token"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("reddit: missing access_token in OAuth response"))?;

    // Step 2: Submit link
    let mut form = HashMap::new();
    form.insert("api_type", "json");
    form.insert("kind", "link");
    form.insert("sr", subreddit);
    form.insert("title", title);
    form.insert("url", url);

    let submit_resp = client
        .post("https://oauth.reddit.com/api/submit")
        .bearer_auth(access_token)
        .form(&form)
        .send()?;

    if !submit_resp.status().is_success() {
        let status = submit_resp.status();
        let body = submit_resp.text().unwrap_or_default();
        anyhow::bail!("reddit: submit failed ({status}): {body}");
    }

    // Reddit returns 200 even on failure — check json.errors
    let submit_body: serde_json::Value = serde_json::from_str(&submit_resp.text()?)?;
    if let Some(errors) = submit_body
        .get("json")
        .and_then(|j| j.get("errors"))
        .and_then(|e| e.as_array())
        && !errors.is_empty()
    {
        anyhow::bail!("reddit: submit returned errors: {errors:?}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_subreddit;

    #[test]
    fn accepts_valid_names() {
        validate_subreddit("rust").unwrap();
        validate_subreddit("rust_lang").unwrap();
        validate_subreddit("AnodizerRel123").unwrap();
    }

    #[test]
    fn rejects_too_short() {
        let err = validate_subreddit("ab").unwrap_err().to_string();
        assert!(err.contains("3–21"), "{err}");
    }

    #[test]
    fn rejects_too_long() {
        let err = validate_subreddit(&"a".repeat(22)).unwrap_err().to_string();
        assert!(err.contains("3–21"), "{err}");
    }

    #[test]
    fn rejects_leading_underscore() {
        let err = validate_subreddit("_oops").unwrap_err().to_string();
        assert!(err.contains("underscore"), "{err}");
    }

    #[test]
    fn rejects_invalid_characters() {
        let err = validate_subreddit("has-hyphen").unwrap_err().to_string();
        assert!(err.contains("invalid characters"), "{err}");
        let err = validate_subreddit("has space").unwrap_err().to_string();
        assert!(err.contains("invalid characters"), "{err}");
    }
}

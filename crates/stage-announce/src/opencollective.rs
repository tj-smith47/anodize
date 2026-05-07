use std::ops::ControlFlow;

use anodizer_core::retry::{HttpError, RetryPolicy, is_retriable, retry_sync};
use anyhow::{Context as _, Result};
use serde_json::json;

const GRAPHQL_URL: &str = "https://api.opencollective.com/graphql/v2";

pub const DEFAULT_TITLE_TEMPLATE: &str = "{{ Tag }}";
pub const DEFAULT_MESSAGE_TEMPLATE: &str = r#"{{ ProjectName }} {{ Tag }} is out!<br/>Check it out at <a href="{{ ReleaseURL }}">{{ ReleaseURL }}</a>"#;

/// Validate an OpenCollective collective slug. Slugs are lowercase
/// alphanumeric with hyphens, 1–48 characters, no leading/trailing hyphen
/// and no consecutive hyphens. Catching format errors here avoids a wasted
/// GraphQL round-trip for an unresolvable slug.
pub fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() || slug.len() > 48 {
        anyhow::bail!(
            "opencollective: slug {slug:?} must be 1–48 characters (got {})",
            slug.len()
        );
    }
    if slug.starts_with('-') || slug.ends_with('-') {
        anyhow::bail!("opencollective: slug {slug:?} must not start or end with '-'");
    }
    if slug.contains("--") {
        anyhow::bail!("opencollective: slug {slug:?} must not contain consecutive hyphens");
    }
    if !slug
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        anyhow::bail!(
            "opencollective: slug {slug:?} must contain only lowercase letters, digits, and hyphens"
        );
    }
    Ok(())
}

/// Loose check on the Personal-Token header value. OpenCollective tokens are
/// long opaque strings; reject anything obviously malformed (whitespace,
/// non-printable bytes, very short) so we surface the misconfiguration before
/// the API rejects us with an opaque 401.
pub fn validate_token_shape(token: &str) -> Result<()> {
    crate::util::validate_token_min_length("opencollective", "OPENCOLLECTIVE_TOKEN", token, 16)?;
    if token.chars().any(|c| c.is_whitespace() || c.is_control()) {
        anyhow::bail!(
            "opencollective: OPENCOLLECTIVE_TOKEN contains whitespace or control characters \
             — check for stray quotes or line wraps"
        );
    }
    Ok(())
}

const CREATE_QUERY: &str =
    r#"mutation($update: UpdateCreateInput!) { createUpdate(update: $update) { id } }"#;

const PUBLISH_QUERY: &str = r#"mutation($id: String!, $audience: UpdateAudience) { publishUpdate(id: $id, notificationAudience: $audience) { id } }"#;

pub(crate) fn build_create_body(slug: &str, title: &str, html: &str) -> serde_json::Value {
    json!({
        "query": CREATE_QUERY,
        "variables": {
            "update": {
                "title": title,
                "html": html,
                "account": { "slug": slug }
            }
        }
    })
}

pub(crate) fn build_publish_body(update_id: &str) -> serde_json::Value {
    json!({
        "query": PUBLISH_QUERY,
        "variables": {"id": update_id, "audience": "ALL"}
    })
}

/// Categorise an OpenCollective HTTP response into a structured error.
///
/// Q7.1 mirror of upstream commit 206120a (#6512): callers see distinct
/// messages for 401-unauthorized, 5xx-server-error, and other 4xx
/// rejections. GraphQL APIs return HTTP 200 even on mutation failures —
/// errors are in the response body, not the status code — so this only
/// classifies HTTP-level failures (`!status.is_success()`).
pub(crate) fn classify_opencollective_status(
    stage: &str,
    status: reqwest::StatusCode,
    body: &str,
) -> String {
    match status.as_u16() {
        401 => format!(
            "opencollective: {stage} unauthorized (401) — check OPENCOLLECTIVE_TOKEN: {body}"
        ),
        403 => format!(
            "opencollective: {stage} forbidden (403) — token lacks the required scope: {body}"
        ),
        s if (500..600).contains(&s) => format!(
            "opencollective: {stage} server error ({status}) — upstream is unhealthy, retrying: {body}"
        ),
        _ => format!("opencollective: {stage} failed ({status}): {body}"),
    }
}

/// Single-shot HTTP POST with retry + categorised error wrapping.
fn do_mutation(
    client: &reqwest::blocking::Client,
    stage: &str,
    token: &str,
    body_payload: String,
    policy: &RetryPolicy,
) -> Result<String> {
    retry_sync(policy, |_attempt| {
        match client
            .post(GRAPHQL_URL)
            .header("Personal-Token", token)
            .header("Content-Type", "application/json")
            .body(body_payload.clone())
            .send()
        {
            Err(e) => {
                let err = anyhow::Error::new(HttpError::from_response(e, None))
                    .context(format!("opencollective: {stage} transport error"));
                if is_retriable(err.as_ref()) {
                    Err(ControlFlow::Continue(err))
                } else {
                    Err(ControlFlow::Break(err))
                }
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp
                    .text()
                    .unwrap_or_else(|e| format!("<body read failed: {e}>"));
                if status.is_success() {
                    Ok(body)
                } else {
                    let msg = classify_opencollective_status(stage, status, &body);
                    let wrapped = anyhow::Error::new(HttpError::new(
                        std::io::Error::other(msg.clone()),
                        status.as_u16(),
                    ))
                    .context(msg);
                    if is_retriable(wrapped.as_ref()) {
                        Err(ControlFlow::Continue(wrapped))
                    } else {
                        Err(ControlFlow::Break(wrapped))
                    }
                }
            }
        }
    })
    .with_context(|| format!("opencollective: {stage} exhausted retry attempts"))
}

/// Create and publish an update on OpenCollective.
///
/// Two-step GraphQL flow:
/// 1. `createUpdate` mutation — creates a draft update with title and HTML body
/// 2. `publishUpdate` mutation — publishes the update to all collective members
///
/// Q7.1 — error categorisation mirrors upstream commit 206120a: 401, 5xx, and
/// other 4xx rejections all surface distinct messages, GraphQL `errors` arrays
/// are decoded and reported, and malformed JSON responses are caught with a
/// dedicated error rather than panicking.
///
/// The publish step is unconditionally attempted whenever step 1 yields a valid
/// `update_id`, even if the response also includes a non-fatal `errors` array.
/// A draft created with warnings is still publishable, and silently abandoning
/// it would leave the collective with an unpublished update.
pub fn send_opencollective(
    token: &str,
    slug: &str,
    title: &str,
    html: &str,
    policy: &RetryPolicy,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();

    let resp_text = do_mutation(
        &client,
        "createUpdate",
        token,
        build_create_body(slug, title, html).to_string(),
        policy,
    )?;
    let resp_json: serde_json::Value = serde_json::from_str(&resp_text).with_context(|| {
        format!("opencollective: createUpdate response was not valid JSON: {resp_text}")
    })?;
    let update_id = resp_json["data"]["createUpdate"]["id"].as_str();
    let create_errors = resp_json.get("errors");
    let update_id = match (update_id, create_errors) {
        (Some(id), _) => id.to_string(),
        (None, Some(errs)) => {
            anyhow::bail!("opencollective: createUpdate returned errors: {errs}")
        }
        (None, None) => {
            anyhow::bail!("opencollective: missing update ID in createUpdate response")
        }
    };

    let publish_text = do_mutation(
        &client,
        "publishUpdate",
        token,
        build_publish_body(&update_id).to_string(),
        policy,
    )?;
    let publish_json: serde_json::Value =
        serde_json::from_str(&publish_text).with_context(|| {
            format!("opencollective: publishUpdate response was not valid JSON: {publish_text}")
        })?;
    if let Some(errors) = publish_json.get("errors") {
        anyhow::bail!("opencollective: publishUpdate returned errors: {errors}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_create_body_shape() {
        let body = build_create_body("my-project", "v1.0.0", "Project v1.0.0 is out!");
        assert_eq!(body["query"], CREATE_QUERY);
        assert_eq!(body["variables"]["update"]["account"]["slug"], "my-project");
        assert_eq!(body["variables"]["update"]["title"], "v1.0.0");
        assert!(
            body["variables"]["update"]["html"]
                .as_str()
                .unwrap()
                .contains("is out!")
        );
    }

    #[test]
    fn test_build_publish_body_shape() {
        let body = build_publish_body("UPD-123");
        assert_eq!(body["query"], PUBLISH_QUERY);
        assert_eq!(body["variables"]["id"], "UPD-123");
        assert_eq!(body["variables"]["audience"], "ALL");
    }

    #[test]
    fn slug_accepts_well_formed() {
        validate_slug("my-project").unwrap();
        validate_slug("opensource").unwrap();
        validate_slug("a1-b2-c3").unwrap();
    }

    #[test]
    fn slug_rejects_bad_format() {
        assert!(validate_slug("").is_err());
        assert!(validate_slug("-leading").is_err());
        assert!(validate_slug("trailing-").is_err());
        assert!(validate_slug("double--hyphen").is_err());
        assert!(validate_slug("UpperCase").is_err());
        assert!(validate_slug("under_score").is_err());
        assert!(validate_slug(&"x".repeat(49)).is_err());
    }

    #[test]
    fn token_shape_accepts_long_opaque() {
        validate_token_shape(&"a".repeat(64)).unwrap();
    }

    #[test]
    fn token_shape_rejects_short() {
        assert!(validate_token_shape("short").is_err());
    }

    #[test]
    fn token_shape_rejects_whitespace() {
        let err = validate_token_shape("token with spaces inside it 123456789012345")
            .unwrap_err()
            .to_string();
        assert!(err.contains("whitespace"), "{err}");
    }
}

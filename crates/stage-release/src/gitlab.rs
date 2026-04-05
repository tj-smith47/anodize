//! GitLab release backend — creates releases, uploads assets, and publishes
//! releases via the GitLab REST API.
//!
//! GitLab does not support draft releases (unlike GitHub), so `PublishRelease`
//! is a no-op.  Asset uploads use either the Generic Package Registry (PUT) or
//! Project Markdown Uploads (POST multipart), then create a release link to
//! the uploaded file.
//!
//! Reference: GoReleaser `internal/client/gitlab.go`.

use std::path::Path;

use anyhow::{Context as _, Result, bail};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::Client;

use crate::compose_body_for_mode;

// ---------------------------------------------------------------------------
// URL-encoding helpers
// ---------------------------------------------------------------------------

/// Characters that must be percent-encoded in a GitLab project path segment.
/// GitLab requires the full project path (e.g. `group/project`) to be encoded
/// so that `/` becomes `%2F`.
const PATH_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.');

/// Percent-encode a GitLab project ID path segment.
///
/// `owner/name` becomes `owner%2Fname`.
fn encode_project_id(project_id: &str) -> String {
    utf8_percent_encode(project_id, PATH_ENCODE_SET).to_string()
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// Build the GitLab project ID string from owner and name.
///
/// If `owner` is empty, only the name is returned (GitLab supports projects
/// without a namespace prefix in some API calls).
pub(crate) fn gitlab_project_id(owner: &str, name: &str) -> String {
    if owner.is_empty() {
        name.to_string()
    } else {
        format!("{}/{}", owner, name)
    }
}

/// Build the release page URL on the GitLab web UI.
pub(crate) fn gitlab_release_url(
    download_url: &str,
    owner: &str,
    name: &str,
    tag: &str,
) -> String {
    let base = download_url.trim_end_matches('/');
    if owner.is_empty() {
        format!("{}/{}/-/releases/{}", base, name, tag)
    } else {
        format!("{}/{}/{}/-/releases/{}", base, owner, name, tag)
    }
}

/// Build the GitLab auth header name and value for the given token.
fn auth_header(use_job_token: bool) -> &'static str {
    if use_job_token {
        "JOB-TOKEN"
    } else {
        "PRIVATE-TOKEN"
    }
}

/// Build a [`reqwest::Client`] configured for GitLab API access.
///
/// - `token`: the GITLAB_TOKEN or CI_JOB_TOKEN value.
/// - `skip_tls_verify`: when true, disable TLS certificate verification.
/// - `use_job_token`: when true, use `JOB-TOKEN` header instead of `PRIVATE-TOKEN`.
///
/// The token is set as a default header on all requests from the returned client.
pub(crate) fn build_gitlab_client(
    token: &str,
    skip_tls_verify: bool,
    use_job_token: bool,
) -> Result<Client> {
    let header_name = auth_header(use_job_token);
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::HeaderName::from_bytes(header_name.as_bytes())
            .context("gitlab: invalid auth header name")?,
        reqwest::header::HeaderValue::from_str(token)
            .context("gitlab: invalid token value for header")?,
    );

    let builder = Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(skip_tls_verify)
        .timeout(std::time::Duration::from_secs(300));

    builder.build().context("gitlab: build HTTP client")
}

// ---------------------------------------------------------------------------
// Create / update release
// ---------------------------------------------------------------------------

/// Create or update a GitLab release.
///
/// Checks whether the release already exists for the given tag. If it does,
/// applies mode-based body composition (keep-existing / append / prepend /
/// replace) and updates via PUT. If it does not exist, creates via POST.
///
/// Returns the tag name (GitLab's release identifier).
#[allow(clippy::too_many_arguments)]
pub(crate) async fn gitlab_create_release(
    client: &Client,
    api_url: &str,
    project_id: &str,
    tag: &str,
    name: &str,
    body: &str,
    commit: &str,
    release_mode: &str,
) -> Result<String> {
    let api = api_url.trim_end_matches('/');
    let encoded = encode_project_id(project_id);

    // Try to get the existing release for this tag.
    let get_url = format!("{}/projects/{}/releases/{}", api, encoded, tag);
    let get_resp = client.get(&get_url).send().await.context(
        "gitlab: GET release by tag",
    )?;

    let status = get_resp.status().as_u16();

    if status == 403 || status == 404 {
        // Release does not exist — create it.
        let create_url = format!("{}/projects/{}/releases", api, encoded);
        let payload = serde_json::json!({
            "name": name,
            "description": body,
            "ref": commit,
            "tag_name": tag,
        });

        let resp = client
            .post(&create_url)
            .json(&payload)
            .send()
            .await
            .context("gitlab: POST create release")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!(
                "gitlab: create release failed (HTTP {}): {}",
                status,
                text
            );
        }
    } else if get_resp.status().is_success() {
        // Release exists — update it with mode-based body composition.
        let existing: serde_json::Value = get_resp
            .json()
            .await
            .context("gitlab: parse existing release JSON")?;
        let existing_body = existing["description"].as_str();
        let final_body = compose_body_for_mode(release_mode, existing_body, body);

        let update_url = format!("{}/projects/{}/releases/{}", api, encoded, tag);
        let payload = serde_json::json!({
            "name": name,
            "description": final_body,
        });

        let resp = client
            .put(&update_url)
            .json(&payload)
            .send()
            .await
            .context("gitlab: PUT update release")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!(
                "gitlab: update release failed (HTTP {}): {}",
                status,
                text
            );
        }
    } else {
        // Unexpected error.
        let text = get_resp.text().await.unwrap_or_default();
        bail!(
            "gitlab: check existing release failed (HTTP {}): {}",
            status,
            text
        );
    }

    Ok(tag.to_string())
}

// ---------------------------------------------------------------------------
// Upload asset + create release link
// ---------------------------------------------------------------------------

/// Upload a file to GitLab and create a release link for it.
///
/// When `use_package_registry` is true (or when using job tokens), the file is
/// uploaded to the GitLab Generic Package Registry via PUT. Otherwise, it is
/// uploaded via the Project Markdown Uploads endpoint (POST multipart).
///
/// After the upload, a release link is created pointing to the uploaded file.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn gitlab_upload_asset(
    client: &Client,
    api_url: &str,
    project_id: &str,
    tag: &str,
    file_path: &Path,
    file_name: &str,
    project_name: &str,
    version: &str,
    use_package_registry: bool,
    download_url: &str,
) -> Result<()> {
    let api = api_url.trim_end_matches('/');
    let encoded = encode_project_id(project_id);

    let link_url = if use_package_registry {
        upload_via_package_registry(client, api, &encoded, project_name, version, file_name, file_path)
            .await?
    } else {
        upload_via_project_uploads(client, api, &encoded, project_id, file_path, file_name, download_url)
            .await?
    };

    // Create a release link for the uploaded asset.
    let link_api = format!(
        "{}/projects/{}/releases/{}/assets/links",
        api, encoded, tag
    );
    let direct_asset_path = format!("/{}", file_name);
    let payload = serde_json::json!({
        "name": file_name,
        "url": link_url,
        "direct_asset_path": direct_asset_path,
    });

    let resp = client
        .post(&link_api)
        .json(&payload)
        .send()
        .await
        .context("gitlab: POST create release link")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!(
            "gitlab: create release link for '{}' failed (HTTP {}): {}",
            file_name,
            status,
            text
        );
    }

    Ok(())
}

/// Upload a file via the GitLab Generic Package Registry.
///
/// ```text
/// PUT {api}/projects/{id}/packages/generic/{package}/{version}/{filename}
/// ```
async fn upload_via_package_registry(
    client: &Client,
    api: &str,
    encoded_project_id: &str,
    project_name: &str,
    version: &str,
    file_name: &str,
    file_path: &Path,
) -> Result<String> {
    let data = tokio::fs::read(file_path)
        .await
        .with_context(|| format!("gitlab: read file {}", file_path.display()))?;

    let upload_url = format!(
        "{}/projects/{}/packages/generic/{}/{}/{}",
        api, encoded_project_id, project_name, version, file_name
    );

    let resp = client
        .put(&upload_url)
        .header("Content-Type", "application/octet-stream")
        .body(data)
        .send()
        .await
        .with_context(|| format!("gitlab: PUT upload '{}' to package registry", file_name))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!(
            "gitlab: package registry upload '{}' failed (HTTP {}): {}",
            file_name,
            status,
            text
        );
    }

    // The link URL for package registry assets is the same upload URL.
    Ok(upload_url)
}

/// Upload a file via the GitLab Project Markdown Uploads endpoint.
///
/// ```text
/// POST {api}/projects/{id}/uploads
/// Content-Type: multipart/form-data
/// ```
///
/// Returns the full download URL constructed from the download base URL and
/// the returned `full_path` field.
async fn upload_via_project_uploads(
    client: &Client,
    api: &str,
    encoded_project_id: &str,
    project_id: &str,
    file_path: &Path,
    file_name: &str,
    download_url: &str,
) -> Result<String> {
    let data = tokio::fs::read(file_path)
        .await
        .with_context(|| format!("gitlab: read file {}", file_path.display()))?;

    let upload_url = format!("{}/projects/{}/uploads", api, encoded_project_id);

    let file_part = reqwest::multipart::Part::bytes(data)
        .file_name(file_name.to_string())
        .mime_str("application/octet-stream")
        .context("gitlab: set MIME type for upload")?;

    let form = reqwest::multipart::Form::new().part("file", file_part);

    let resp = client
        .post(&upload_url)
        .multipart(form)
        .send()
        .await
        .with_context(|| format!("gitlab: POST upload '{}' as project attachment", file_name))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!(
            "gitlab: project upload '{}' failed (HTTP {}): {}",
            file_name,
            status,
            text
        );
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .context("gitlab: parse upload response JSON")?;

    // GitLab returns `{ "full_path": "/uploads/...", "url": "/uploads/...", ... }`.
    // GoReleaser uses `projectFile.FullPath` and prepends `download_url + "/" + full_path`.
    let full_path = body["full_path"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("gitlab: upload response missing 'full_path' field"))?;

    let base = download_url.trim_end_matches('/');

    // GitLab's upload response returns a `full_path` field:
    // - Modern API: `/{owner}/{name}/uploads/<hash>/<filename>` (project path included)
    // - Older API: `/uploads/<hash>/<filename>` (just the upload path, no project prefix)
    //
    // GoReleaser constructs the link as: `gitlabBaseURL + "/" + projectFile.FullPath`.
    // We detect the older format and insert the project path ourselves.
    let link = if full_path.starts_with("/uploads/") {
        // Older API — full_path is just `/uploads/<hash>/<filename>`.
        // Prefix with project_id to get the correct download path.
        format!("{}/{}{}", base, project_id, full_path)
    } else if full_path.starts_with('/') {
        // Modern API — full_path already includes the project path.
        format!("{}{}", base, full_path)
    } else {
        format!("{}/{}", base, full_path)
    };

    Ok(link)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- gitlab_project_id ---------------------------------------------------

    #[test]
    fn project_id_with_owner_and_name() {
        assert_eq!(gitlab_project_id("mygroup", "myproject"), "mygroup/myproject");
    }

    #[test]
    fn project_id_with_empty_owner() {
        assert_eq!(gitlab_project_id("", "myproject"), "myproject");
    }

    #[test]
    fn project_id_with_nested_group() {
        assert_eq!(
            gitlab_project_id("org/subgroup", "repo"),
            "org/subgroup/repo"
        );
    }

    // -- encode_project_id ---------------------------------------------------

    #[test]
    fn encode_simple_project_id() {
        assert_eq!(encode_project_id("mygroup/myproject"), "mygroup%2Fmyproject");
    }

    #[test]
    fn encode_nested_project_id() {
        assert_eq!(
            encode_project_id("org/subgroup/repo"),
            "org%2Fsubgroup%2Frepo"
        );
    }

    #[test]
    fn encode_project_id_no_slash() {
        // A project without an owner should pass through mostly unchanged.
        assert_eq!(encode_project_id("myproject"), "myproject");
    }

    // -- gitlab_release_url --------------------------------------------------

    #[test]
    fn release_url_with_owner() {
        let url = gitlab_release_url("https://gitlab.com", "mygroup", "myproject", "v1.0.0");
        assert_eq!(
            url,
            "https://gitlab.com/mygroup/myproject/-/releases/v1.0.0"
        );
    }

    #[test]
    fn release_url_without_owner() {
        let url = gitlab_release_url("https://gitlab.com", "", "myproject", "v1.0.0");
        assert_eq!(url, "https://gitlab.com/myproject/-/releases/v1.0.0");
    }

    #[test]
    fn release_url_trailing_slash_stripped() {
        let url = gitlab_release_url("https://gitlab.example.com/", "org", "repo", "v2.0.0");
        assert_eq!(
            url,
            "https://gitlab.example.com/org/repo/-/releases/v2.0.0"
        );
    }

    // -- build_gitlab_client -------------------------------------------------

    #[test]
    fn build_client_with_private_token() {
        let client = build_gitlab_client("glpat-xxxx", false, false);
        assert!(client.is_ok());
    }

    #[test]
    fn build_client_with_job_token() {
        let client = build_gitlab_client("job-token-value", false, true);
        assert!(client.is_ok());
    }

    #[test]
    fn build_client_with_skip_tls() {
        let client = build_gitlab_client("glpat-xxxx", true, false);
        assert!(client.is_ok());
    }

    #[test]
    fn build_client_with_all_options() {
        let client = build_gitlab_client("job-token", true, true);
        assert!(client.is_ok());
    }

    // -- auth_header ---------------------------------------------------------

    #[test]
    fn auth_header_private_token() {
        assert_eq!(auth_header(false), "PRIVATE-TOKEN");
    }

    #[test]
    fn auth_header_job_token() {
        assert_eq!(auth_header(true), "JOB-TOKEN");
    }
}

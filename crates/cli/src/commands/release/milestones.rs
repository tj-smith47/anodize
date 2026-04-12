use anodize_core::config::Config;
use anodize_core::context::Context;
use anodize_core::log::StageLogger;
use anyhow::{Context as _, Result};

/// Close milestones on the VCS provider after a release.
///
/// For each milestone config with `close: true`, renders the name template,
/// resolves the repo owner/name, and calls the GitHub/GitLab/Gitea API to
/// close the milestone. Errors are logged as warnings unless `fail_on_error` is set.
pub(super) fn close_milestones(
    milestones: &[anodize_core::config::MilestoneConfig],
    ctx: &mut Context,
    dry_run: bool,
    log: &StageLogger,
) -> Result<()> {
    let token = ctx.options.token.clone().unwrap_or_default();

    for milestone_cfg in milestones {
        if !milestone_cfg.close.unwrap_or(false) {
            continue;
        }

        let name_template = milestone_cfg
            .name_template
            .as_deref()
            .unwrap_or("{{ Tag }}");
        let milestone_name = ctx
            .render_template(name_template)
            .context("milestone: render name_template")?;

        if milestone_name.is_empty() {
            log.verbose("milestone: skipping empty name");
            continue;
        }

        // Determine repo owner/name from milestone config or release config
        let (owner, repo_name) = resolve_milestone_repo(milestone_cfg, &ctx.config);

        if owner.is_empty() || repo_name.is_empty() {
            if milestone_cfg.fail_on_error.unwrap_or(false) {
                anyhow::bail!("milestone: repo owner/name not configured");
            }
            log.warn("milestone: skipping — repo owner/name not configured");
            continue;
        }

        if dry_run {
            log.status(&format!(
                "(dry-run) would close milestone '{}' on {}/{}",
                milestone_name, owner, repo_name
            ));
            continue;
        }

        log.status(&format!(
            "closing milestone '{}' on {}/{}",
            milestone_name, owner, repo_name
        ));

        // GoReleaser parity: close milestones on GitHub, GitLab, and Gitea.
        let provider = resolve_milestone_provider(milestone_cfg, &ctx.config);
        let api_url = resolve_milestone_api_url(milestone_cfg, &ctx.config);
        let close_result = match provider.as_str() {
            "github" => close_milestone_github(&token, &owner, &repo_name, &milestone_name),
            "gitlab" => close_milestone_gitlab(
                &token,
                &owner,
                &repo_name,
                &milestone_name,
                api_url.as_deref(),
            ),
            "gitea" => close_milestone_gitea(
                &token,
                &owner,
                &repo_name,
                &milestone_name,
                api_url.as_deref(),
            ),
            other => {
                let msg = format!(
                    "milestone: unknown provider '{}' — cannot close milestone",
                    other
                );
                if milestone_cfg.fail_on_error.unwrap_or(false) {
                    anyhow::bail!("{}", msg);
                }
                log.warn(&msg);
                continue;
            }
        };
        match close_result {
            Ok(()) => {
                log.status(&format!("milestone '{}' closed", milestone_name));
            }
            Err(e) => {
                if milestone_cfg.fail_on_error.unwrap_or(false) {
                    return Err(
                        e.context(format!("milestone: failed to close '{}'", milestone_name))
                    );
                }
                log.warn(&format!(
                    "milestone: could not close '{}': {}",
                    milestone_name, e
                ));
            }
        }
    }
    Ok(())
}

fn resolve_milestone_repo(
    milestone_cfg: &anodize_core::config::MilestoneConfig,
    config: &Config,
) -> (String, String) {
    if let Some(ref repo_cfg) = milestone_cfg.repo
        && !repo_cfg.owner.is_empty()
        && !repo_cfg.name.is_empty()
    {
        return (repo_cfg.owner.clone(), repo_cfg.name.clone());
    }

    // Fall back to the first crate's release config
    for crate_cfg in &config.crates {
        if let Some(ref release_cfg) = crate_cfg.release {
            if let Some(ref gh) = release_cfg.github {
                return (gh.owner.clone(), gh.name.clone());
            }
            if let Some(ref gl) = release_cfg.gitlab {
                return (gl.owner.clone(), gl.name.clone());
            }
            if let Some(ref gt) = release_cfg.gitea {
                return (gt.owner.clone(), gt.name.clone());
            }
        }
    }

    (String::new(), String::new())
}

/// Determine the SCM provider type for milestone operations.
/// Returns "github", "gitlab", "gitea", or "unknown".
fn resolve_milestone_provider(
    milestone_cfg: &anodize_core::config::MilestoneConfig,
    config: &Config,
) -> String {
    // If the milestone config specifies a repo, check what provider type the
    // first crate's release config uses (since MilestoneConfig.repo doesn't
    // have a provider field).
    let _ = milestone_cfg;
    for crate_cfg in &config.crates {
        if let Some(ref release_cfg) = crate_cfg.release {
            if release_cfg.github.is_some() {
                return "github".to_string();
            }
            if release_cfg.gitlab.is_some() {
                return "gitlab".to_string();
            }
            if release_cfg.gitea.is_some() {
                return "gitea".to_string();
            }
        }
    }
    "unknown".to_string()
}

/// Close a GitHub milestone by name using the REST API.
fn close_milestone_github(
    token: &str,
    owner: &str,
    repo: &str,
    milestone_name: &str,
) -> Result<()> {
    if token.is_empty() {
        anyhow::bail!("no authentication token available for milestone close");
    }

    let rt = tokio::runtime::Runtime::new().context("milestone: create tokio runtime")?;
    rt.block_on(async {
        let client = reqwest::Client::new();

        // List milestones with pagination to find the one with the matching title.
        // GitHub returns at most 100 per page.
        let mut page = 1u32;
        let mut milestone_number: Option<u64> = None;

        loop {
            let url = format!(
                "https://api.github.com/repos/{}/{}/milestones?state=open&per_page=100&page={}",
                owner, repo, page
            );
            let resp = client
                .get(&url)
                .header("Authorization", format!("Bearer {}", token))
                .header("Accept", "application/vnd.github+json")
                .header("User-Agent", "anodize")
                .send()
                .await
                .context("milestone: list milestones request failed")?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!(
                    "milestone: list milestones failed (HTTP {}): {}",
                    status,
                    body
                );
            }

            let milestones: Vec<serde_json::Value> = resp
                .json()
                .await
                .context("milestone: parse milestones response")?;

            if milestones.is_empty() {
                break;
            }

            if let Some(m) = milestones.iter().find(|m| {
                m.get("title")
                    .and_then(|t| t.as_str())
                    .is_some_and(|t| t == milestone_name)
            }) {
                milestone_number = m.get("number").and_then(|n| n.as_u64());
                break;
            }

            // If we got fewer than 100 results, there are no more pages.
            if milestones.len() < 100 {
                break;
            }
            page += 1;
        }

        let milestone_number = match milestone_number {
            Some(n) => n,
            None => {
                // Milestone not found -- treat as success (may have been closed already)
                return Ok(());
            }
        };

        // Close the milestone
        let close_url = format!(
            "https://api.github.com/repos/{}/{}/milestones/{}",
            owner, repo, milestone_number
        );
        let resp = client
            .patch(&close_url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "anodize")
            .json(&serde_json::json!({ "state": "closed" }))
            .send()
            .await
            .context("milestone: close milestone request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("milestone: close failed (HTTP {}): {}", status, body);
        }

        Ok(())
    })
}

/// Simple percent-encoding for URL path segments.
fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Resolve the API base URL for milestone operations on GitLab/Gitea.
fn resolve_milestone_api_url(
    _milestone_cfg: &anodize_core::config::MilestoneConfig,
    config: &Config,
) -> Option<String> {
    // Check top-level gitlab_urls / gitea_urls config
    if let Some(ref gitlab) = config.gitlab_urls
        && let Some(ref api) = gitlab.api
    {
        // Strip trailing /api/v4/ to get base URL
        let base = api.trim_end_matches('/').trim_end_matches("/api/v4");
        return Some(base.to_string());
    }
    if let Some(ref gitea) = config.gitea_urls
        && let Some(ref api) = gitea.api
    {
        let base = api.trim_end_matches('/').trim_end_matches("/api/v1");
        return Some(base.to_string());
    }
    None
}

/// Close a GitLab milestone by name using the REST API.
fn close_milestone_gitlab(
    token: &str,
    owner: &str,
    repo: &str,
    milestone_name: &str,
    api_url: Option<&str>,
) -> Result<()> {
    if token.is_empty() {
        anyhow::bail!("no authentication token available for GitLab milestone close");
    }
    let base = api_url.unwrap_or("https://gitlab.com");

    let rt = tokio::runtime::Runtime::new().context("milestone: create tokio runtime")?;
    rt.block_on(async {
        let client = reqwest::Client::new();
        let project_path = format!("{}/{}", owner, repo);
        let encoded_path = url_encode(&project_path);

        // List milestones to find matching title
        let url = format!(
            "{}/api/v4/projects/{}/milestones?title={}",
            base,
            encoded_path,
            url_encode(milestone_name)
        );
        let resp = client
            .get(&url)
            .header("PRIVATE-TOKEN", token)
            .header("User-Agent", "anodize")
            .send()
            .await
            .context("milestone: GitLab list milestones failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "milestone: GitLab list milestones failed (HTTP {}): {}",
                status,
                body
            );
        }

        let milestones: Vec<serde_json::Value> = resp
            .json()
            .await
            .context("milestone: parse GitLab milestones")?;

        let milestone_id = milestones
            .iter()
            .find(|m| {
                m.get("title")
                    .and_then(|t| t.as_str())
                    .is_some_and(|t| t == milestone_name)
            })
            .and_then(|m| m.get("id").and_then(|i| i.as_u64()));

        let milestone_id = match milestone_id {
            Some(id) => id,
            None => return Ok(()), // Not found — may be already closed
        };

        // Close the milestone (GoReleaser: StateEvent = "close")
        let close_url = format!(
            "{}/api/v4/projects/{}/milestones/{}",
            base, encoded_path, milestone_id
        );
        let resp = client
            .put(&close_url)
            .header("PRIVATE-TOKEN", token)
            .header("User-Agent", "anodize")
            .json(&serde_json::json!({ "state_event": "close" }))
            .send()
            .await
            .context("milestone: GitLab close milestone failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("milestone: GitLab close failed (HTTP {}): {}", status, body);
        }
        Ok(())
    })
}

/// Close a Gitea milestone by name using the REST API.
fn close_milestone_gitea(
    token: &str,
    owner: &str,
    repo: &str,
    milestone_name: &str,
    api_url: Option<&str>,
) -> Result<()> {
    if token.is_empty() {
        anyhow::bail!("no authentication token available for Gitea milestone close");
    }
    let base = api_url.unwrap_or("https://gitea.com");

    let rt = tokio::runtime::Runtime::new().context("milestone: create tokio runtime")?;
    rt.block_on(async {
        let client = reqwest::Client::new();

        // List milestones to find matching title
        let url = format!(
            "{}/api/v1/repos/{}/{}/milestones?state=open&name={}",
            base,
            owner,
            repo,
            url_encode(milestone_name)
        );
        let resp = client
            .get(&url)
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "anodize")
            .send()
            .await
            .context("milestone: Gitea list milestones failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "milestone: Gitea list milestones failed (HTTP {}): {}",
                status,
                body
            );
        }

        let milestones: Vec<serde_json::Value> = resp
            .json()
            .await
            .context("milestone: parse Gitea milestones")?;

        let milestone_id = milestones
            .iter()
            .find(|m| {
                m.get("title")
                    .and_then(|t| t.as_str())
                    .is_some_and(|t| t == milestone_name)
            })
            .and_then(|m| m.get("id").and_then(|i| i.as_u64()));

        let milestone_id = match milestone_id {
            Some(id) => id,
            None => return Ok(()), // Not found — may be already closed
        };

        // Close the milestone (GoReleaser: state = "closed")
        let close_url = format!(
            "{}/api/v1/repos/{}/{}/milestones/{}",
            base, owner, repo, milestone_id
        );
        let resp = client
            .patch(&close_url)
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "anodize")
            .json(&serde_json::json!({ "state": "closed", "title": milestone_name }))
            .send()
            .await
            .context("milestone: Gitea close milestone failed")?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            // GoReleaser parity: 404 means milestone not found
            return Ok(());
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("milestone: Gitea close failed (HTTP {}): {}", status, body);
        }
        Ok(())
    })
}

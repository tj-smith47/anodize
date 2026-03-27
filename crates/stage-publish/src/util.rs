use anodize_core::artifact::ArtifactKind;
use anodize_core::context::Context;
use anyhow::{Context as _, Result};
use std::process::Command;

/// Run a command with the given program and arguments, failing with `context_msg`
/// on spawn failure or non-zero exit.
pub(crate) fn run_cmd(program: &str, args: &[&str], context_msg: &str) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("{}: spawn", context_msg))?;
    if !status.success() {
        anyhow::bail!("{}: exited with {}", context_msg, status);
    }
    Ok(())
}

/// Run a command in a specific working directory, failing with `context_msg`
/// on spawn failure or non-zero exit.
pub(crate) fn run_cmd_in(
    dir: &std::path::Path,
    program: &str,
    args: &[&str],
    context_msg: &str,
) -> Result<()> {
    let status = Command::new(program)
        .current_dir(dir)
        .args(args)
        .status()
        .with_context(|| format!("{}: spawn", context_msg))?;
    if !status.success() {
        anyhow::bail!("{}: exited with {}", context_msg, status);
    }
    Ok(())
}

/// Describes the OS + architecture of an artifact match.
pub(crate) struct OsArtifact {
    pub url: String,
    pub sha256: String,
    pub os: String,
    pub arch: String,
}

/// Find all Archive artifacts for the given crate whose target or path
/// matches `os_needle` (e.g. "linux", "darwin", "windows").
///
/// Returns a vec of `OsArtifact` with the URL, SHA256, and inferred
/// os/arch strings extracted from the target triple.
pub(crate) fn find_artifacts_by_os(
    ctx: &Context,
    crate_name: &str,
    os_needle: &str,
) -> Vec<OsArtifact> {
    ctx.artifacts
        .by_kind_and_crate(ArtifactKind::Archive, crate_name)
        .into_iter()
        .filter(|a| {
            a.target
                .as_deref()
                .map(|t| t.to_ascii_lowercase().contains(os_needle))
                .unwrap_or(false)
                || a.path
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .contains(os_needle)
        })
        .map(|a| {
            let url = a
                .metadata
                .get("url")
                .cloned()
                .unwrap_or_else(|| a.path.to_string_lossy().into_owned());
            let sha256 = a
                .metadata
                .get("sha256")
                .cloned()
                .unwrap_or_default();
            let target = a.target.as_deref().unwrap_or("");
            let arch = if target.contains("aarch64") || target.contains("arm64") {
                "arm64".to_string()
            } else if target.contains("x86_64") || target.contains("amd64") {
                "amd64".to_string()
            } else {
                "unknown".to_string()
            };
            let os = if target.contains("linux") {
                "linux".to_string()
            } else if target.contains("darwin") || target.contains("apple") {
                "darwin".to_string()
            } else if target.contains("windows") {
                "windows".to_string()
            } else {
                os_needle.to_string()
            };
            OsArtifact {
                url,
                sha256,
                os,
                arch,
            }
        })
        .collect()
}

/// Find all Archive artifacts for the given crate across all platforms.
///
/// Returns a vec of `OsArtifact` with the URL, SHA256, and inferred
/// os/arch strings extracted from the target triple.
pub(crate) fn find_all_platform_artifacts(ctx: &Context, crate_name: &str) -> Vec<OsArtifact> {
    ctx.artifacts
        .by_kind_and_crate(ArtifactKind::Archive, crate_name)
        .into_iter()
        .map(|a| {
            let url = a
                .metadata
                .get("url")
                .cloned()
                .unwrap_or_else(|| a.path.to_string_lossy().into_owned());
            let sha256 = a
                .metadata
                .get("sha256")
                .cloned()
                .unwrap_or_default();
            let target = a.target.as_deref().unwrap_or("");
            let arch = if target.contains("aarch64") || target.contains("arm64") {
                "arm64".to_string()
            } else if target.contains("x86_64") || target.contains("amd64") {
                "amd64".to_string()
            } else {
                "unknown".to_string()
            };
            let os = if target.contains("linux") {
                "linux".to_string()
            } else if target.contains("darwin") || target.contains("apple") {
                "darwin".to_string()
            } else if target.contains("windows") {
                "windows".to_string()
            } else {
                "unknown".to_string()
            };
            OsArtifact {
                url,
                sha256,
                os,
                arch,
            }
        })
        .collect()
}

/// Find a Windows Archive artifact for the given crate and return `(url, sha256)`.
///
/// Returns `None` when no matching artifact exists.
pub(crate) fn find_windows_artifact(ctx: &Context, crate_name: &str) -> Option<(String, String)> {
    let artifact = ctx
        .artifacts
        .by_kind_and_crate(ArtifactKind::Archive, crate_name)
        .into_iter()
        .find(|a| {
            a.target
                .as_deref()
                .map(|t| t.contains("windows") || t.contains("pc-windows"))
                .unwrap_or(false)
                || a.path
                    .to_string_lossy()
                    .to_ascii_lowercase()
                    .contains("windows")
        })?;

    let url = artifact
        .metadata
        .get("url")
        .cloned()
        .unwrap_or_else(|| artifact.path.to_string_lossy().into_owned());
    let hash = artifact
        .metadata
        .get("sha256")
        .cloned()
        .unwrap_or_default();
    Some((url, hash))
}

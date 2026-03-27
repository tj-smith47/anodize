use anodize_core::context::Context;
use anyhow::{Context as _, Result};
use std::process::Command;

// ---------------------------------------------------------------------------
// generate_manifest
// ---------------------------------------------------------------------------

/// Generate a WinGet YAML manifest string.
///
/// Produces a singleton-style manifest with the minimum required fields for
/// winget-pkgs submission.
#[allow(clippy::too_many_arguments)]
pub fn generate_manifest(
    package_id: &str,
    name: &str,
    version: &str,
    description: &str,
    license: &str,
    publisher: &str,
    publisher_url: &str,
    url: &str,
    hash: &str,
) -> String {
    let mut yaml = String::new();
    yaml.push_str(&format!("PackageIdentifier: {}\n", package_id));
    yaml.push_str(&format!("PackageVersion: {}\n", version));
    yaml.push_str(&format!("PackageName: {}\n", name));
    yaml.push_str(&format!("Publisher: {}\n", publisher));
    if !publisher_url.is_empty() {
        yaml.push_str(&format!("PublisherUrl: {}\n", publisher_url));
    }
    yaml.push_str(&format!("License: {}\n", license));
    yaml.push_str(&format!("ShortDescription: {}\n", description));
    yaml.push_str("Installers:\n");
    yaml.push_str("  - Architecture: x64\n");
    yaml.push_str(&format!("    InstallerUrl: {}\n", url));
    yaml.push_str(&format!("    InstallerSha256: {}\n", hash));
    yaml.push_str("    InstallerType: zip\n");
    yaml.push_str("ManifestType: singleton\nManifestVersion: 1.6.0\n");

    yaml
}

// ---------------------------------------------------------------------------
// publish_to_winget
// ---------------------------------------------------------------------------

pub fn publish_to_winget(ctx: &Context, crate_name: &str) -> Result<()> {
    let crate_cfg = ctx
        .config
        .crates
        .iter()
        .find(|c| c.name == crate_name)
        .ok_or_else(|| anyhow::anyhow!("winget: crate '{}' not found in config", crate_name))?;

    let publish = crate_cfg
        .publish
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("winget: no publish config for '{}'", crate_name))?;

    let winget_cfg = publish
        .winget
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("winget: no winget config for '{}'", crate_name))?;

    let manifests_repo = winget_cfg.manifests_repo.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "winget: no manifests_repo config for '{}'",
            crate_name
        )
    })?;

    let package_id = winget_cfg.package_identifier.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "winget: no package_identifier config for '{}'",
            crate_name
        )
    })?;

    if ctx.is_dry_run() {
        eprintln!(
            "[publish] (dry-run) would submit WinGet manifest for '{}' (pkg={}) to {}/{}",
            crate_name, package_id, manifests_repo.owner, manifests_repo.name
        );
        return Ok(());
    }

    // Resolve version.
    let version = ctx
        .template_vars()
        .get("Version")
        .cloned()
        .unwrap_or_default();

    let description = winget_cfg
        .description
        .clone()
        .unwrap_or_else(|| crate_name.to_string());
    let license = winget_cfg
        .license
        .clone()
        .unwrap_or_else(|| "MIT".to_string());
    let publisher_name = winget_cfg
        .publisher
        .clone()
        .unwrap_or_else(|| manifests_repo.owner.clone());
    let publisher_url = winget_cfg.publisher_url.clone().unwrap_or_default();

    // Find the windows Archive artifact.
    let windows_artifact = ctx
        .artifacts
        .by_kind_and_crate(anodize_core::artifact::ArtifactKind::Archive, crate_name)
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
        });

    let (url, hash) = if let Some(art) = windows_artifact {
        let url = art
            .metadata
            .get("url")
            .cloned()
            .unwrap_or_else(|| art.path.to_string_lossy().into_owned());
        let hash = art.metadata.get("sha256").cloned().unwrap_or_default();
        (url, hash)
    } else {
        eprintln!(
            "[publish] winget: no windows artifact found for '{}', using placeholder URL",
            crate_name
        );
        (
            format!(
                "https://github.com/{0}/{1}/releases/download/v{2}/{1}-{2}-windows-amd64.zip",
                manifests_repo.owner, crate_name, version
            ),
            String::new(),
        )
    };

    let manifest = generate_manifest(
        package_id,
        crate_name,
        &version,
        &description,
        &license,
        &publisher_name,
        &publisher_url,
        &url,
        &hash,
    );

    // Clone the winget-pkgs fork, write manifest, commit, push, submit PR.
    let token = ctx
        .options
        .token
        .clone()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok());

    let clone_url = if let Some(ref tok) = token {
        format!(
            "https://{}@github.com/{}/{}.git",
            tok, manifests_repo.owner, manifests_repo.name
        )
    } else {
        format!(
            "https://github.com/{}/{}.git",
            manifests_repo.owner, manifests_repo.name
        )
    };

    let tmp_dir = tempfile::tempdir().context("winget: create temp dir")?;
    let repo_path = tmp_dir.path();

    run_cmd(
        "git",
        &[
            "clone",
            "--depth=1",
            &clone_url,
            &repo_path.to_string_lossy(),
        ],
        "winget: git clone",
    )?;

    // Build the manifest path: manifests/<first_char>/<Publisher>/<PackageName>/<version>/
    let first_char = package_id
        .chars()
        .next()
        .unwrap_or('_')
        .to_ascii_lowercase();
    let manifest_dir = repo_path
        .join("manifests")
        .join(first_char.to_string())
        .join(package_id.replace('.', "/"))
        .join(&version);
    std::fs::create_dir_all(&manifest_dir)
        .with_context(|| format!("winget: create manifest dir {}", manifest_dir.display()))?;

    let manifest_file = manifest_dir.join(format!("{}.yaml", package_id));
    std::fs::write(&manifest_file, &manifest)
        .with_context(|| format!("winget: write manifest {}", manifest_file.display()))?;

    eprintln!(
        "[publish] wrote WinGet manifest: {}",
        manifest_file.display()
    );

    let branch_name = format!("{}-{}", package_id, version);
    run_cmd_in(
        repo_path,
        "git",
        &["checkout", "-b", &branch_name],
        "winget: git checkout",
    )?;
    run_cmd_in(
        repo_path,
        "git",
        &["add", "."],
        "winget: git add",
    )?;
    run_cmd_in(
        repo_path,
        "git",
        &[
            "commit",
            "-m",
            &format!("New version: {} version {}", package_id, version),
        ],
        "winget: git commit",
    )?;
    run_cmd_in(
        repo_path,
        "git",
        &["push", "-u", "origin", &branch_name],
        "winget: git push",
    )?;

    eprintln!(
        "[publish] WinGet manifest pushed to {}/{} branch '{}'",
        manifests_repo.owner, manifests_repo.name, branch_name
    );

    // Submit PR via GitHub CLI (gh) if available.
    let pr_result = Command::new("gh")
        .current_dir(repo_path)
        .args([
            "pr",
            "create",
            "--repo",
            "microsoft/winget-pkgs",
            "--title",
            &format!("New version: {} version {}", package_id, version),
            "--body",
            &format!(
                "## Package\n- **Package**: {}\n- **Version**: {}\n\nAutomatically submitted by anodize.",
                package_id, version
            ),
            "--head",
            &format!("{}:{}", manifests_repo.owner, branch_name),
        ])
        .status();

    match pr_result {
        Ok(status) if status.success() => {
            eprintln!(
                "[publish] WinGet PR submitted for {} version {}",
                package_id, version
            );
        }
        Ok(status) => {
            eprintln!(
                "[publish] winget: gh pr create exited with {} — you may need to create the PR manually",
                status
            );
        }
        Err(e) => {
            eprintln!(
                "[publish] winget: could not run gh to create PR: {} — you may need to create the PR manually",
                e
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn run_cmd(program: &str, args: &[&str], context_msg: &str) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("{}: spawn", context_msg))?;
    if !status.success() {
        anyhow::bail!("{}: exited with {}", context_msg, status);
    }
    Ok(())
}

fn run_cmd_in(
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // generate_manifest tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_manifest_basic() {
        let manifest = generate_manifest(
            "Org.MyTool",
            "mytool",
            "1.0.0",
            "A great tool",
            "MIT",
            "My Org",
            "https://example.com",
            "https://example.com/mytool-1.0.0-windows-amd64.zip",
            "deadbeef1234567890abcdef",
        );

        assert!(manifest.contains("PackageIdentifier: Org.MyTool"));
        assert!(manifest.contains("PackageVersion: 1.0.0"));
        assert!(manifest.contains("PackageName: mytool"));
        assert!(manifest.contains("Publisher: My Org"));
        assert!(manifest.contains("PublisherUrl: https://example.com"));
        assert!(manifest.contains("License: MIT"));
        assert!(manifest.contains("ShortDescription: A great tool"));
        assert!(manifest.contains("Installers:"));
        assert!(manifest.contains("  - Architecture: x64"));
        assert!(manifest.contains("    InstallerUrl: https://example.com/mytool-1.0.0-windows-amd64.zip"));
        assert!(manifest.contains("    InstallerSha256: deadbeef1234567890abcdef"));
        assert!(manifest.contains("    InstallerType: zip"));
        assert!(manifest.contains("ManifestType: singleton"));
        assert!(manifest.contains("ManifestVersion: 1.6.0"));
    }

    #[test]
    fn test_generate_manifest_no_publisher_url() {
        let manifest = generate_manifest(
            "Org.Tool",
            "tool",
            "2.0.0",
            "A tool",
            "Apache-2.0",
            "My Org",
            "",
            "https://example.com/tool.zip",
            "hash",
        );

        assert!(!manifest.contains("PublisherUrl:"));
        assert!(manifest.contains("Publisher: My Org"));
    }

    #[test]
    fn test_generate_manifest_complete_structure() {
        let manifest = generate_manifest(
            "TjSmith.Anodize",
            "anodize",
            "3.2.1",
            "Release automation for Rust projects",
            "Apache-2.0",
            "TJ Smith",
            "https://github.com/tj-smith47",
            "https://github.com/tj-smith47/anodize/releases/download/v3.2.1/anodize-3.2.1-windows-amd64.zip",
            "aabbccdd11223344",
        );

        // Verify the manifest is well-formed YAML-like text
        let lines: Vec<&str> = manifest.lines().collect();

        // Should start with PackageIdentifier
        assert!(lines[0].starts_with("PackageIdentifier:"));
        // Should end with ManifestVersion
        assert!(lines.last().unwrap().starts_with("ManifestVersion:"));

        // Every line should be non-empty
        for line in &lines {
            assert!(!line.is_empty(), "manifest should not have empty lines");
        }
    }

    #[test]
    fn test_generate_manifest_installer_section() {
        let manifest = generate_manifest(
            "Org.App",
            "app",
            "1.5.0",
            "An app",
            "MIT",
            "Publisher",
            "https://example.com",
            "https://example.com/app-1.5.0.zip",
            "sha256hash",
        );

        // The Installers section should have proper YAML list format
        assert!(manifest.contains("Installers:\n  - Architecture: x64"));

        // InstallerUrl, InstallerSha256, InstallerType should be indented under the list item
        assert!(manifest.contains("    InstallerUrl:"));
        assert!(manifest.contains("    InstallerSha256:"));
        assert!(manifest.contains("    InstallerType: zip"));
    }

    // -----------------------------------------------------------------------
    // publish_to_winget dry-run tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_publish_to_winget_dry_run() {
        use anodize_core::config::{
            Config, CrateConfig, PublishConfig, WingetConfig, WingetManifestsRepoConfig,
        };
        use anodize_core::context::{Context, ContextOptions};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                winget: Some(WingetConfig {
                    manifests_repo: Some(WingetManifestsRepoConfig {
                        owner: "myorg".to_string(),
                        name: "winget-pkgs".to_string(),
                    }),
                    package_identifier: Some("MyOrg.MyTool".to_string()),
                    description: Some("A great tool".to_string()),
                    publisher: Some("My Org".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        );

        // dry-run should succeed without any network/command calls
        assert!(publish_to_winget(&ctx, "mytool").is_ok());
    }

    #[test]
    fn test_publish_to_winget_missing_config() {
        use anodize_core::config::{Config, CrateConfig, PublishConfig};
        use anodize_core::context::{Context, ContextOptions};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig::default()),
            ..Default::default()
        }];

        let ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        );

        // Should fail because there's no winget config
        assert!(publish_to_winget(&ctx, "mytool").is_err());
    }

    #[test]
    fn test_publish_to_winget_missing_package_identifier() {
        use anodize_core::config::{
            Config, CrateConfig, PublishConfig, WingetConfig, WingetManifestsRepoConfig,
        };
        use anodize_core::context::{Context, ContextOptions};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                winget: Some(WingetConfig {
                    manifests_repo: Some(WingetManifestsRepoConfig {
                        owner: "myorg".to_string(),
                        name: "winget-pkgs".to_string(),
                    }),
                    package_identifier: None, // Missing
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        );

        // Should fail because package_identifier is missing
        assert!(publish_to_winget(&ctx, "mytool").is_err());
    }

    #[test]
    fn test_publish_to_winget_missing_manifests_repo() {
        use anodize_core::config::{Config, CrateConfig, PublishConfig, WingetConfig};
        use anodize_core::context::{Context, ContextOptions};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                winget: Some(WingetConfig {
                    manifests_repo: None, // Missing
                    package_identifier: Some("Org.Tool".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        );

        // Should fail because manifests_repo is missing
        assert!(publish_to_winget(&ctx, "mytool").is_err());
    }
}

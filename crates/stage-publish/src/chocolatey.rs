use anodize_core::context::Context;
use anyhow::{Context as _, Result};
use std::process::Command;

// ---------------------------------------------------------------------------
// generate_nuspec
// ---------------------------------------------------------------------------

/// Generate a Chocolatey `.nuspec` XML manifest string.
#[allow(clippy::too_many_arguments)]
pub fn generate_nuspec(
    name: &str,
    version: &str,
    description: &str,
    license: &str,
    authors: &str,
    project_url: &str,
    icon_url: &str,
    tags: &[String],
) -> String {
    let tags_str = if tags.is_empty() {
        name.to_string()
    } else {
        tags.join(" ")
    };

    // Escape XML special characters in user-provided strings.
    let esc = |s: &str| -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    };

    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    xml.push_str("<package xmlns=\"http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd\">\n");
    xml.push_str("  <metadata>\n");
    xml.push_str(&format!("    <id>{}</id>\n", esc(name)));
    xml.push_str(&format!("    <version>{}</version>\n", esc(version)));
    xml.push_str(&format!("    <title>{}</title>\n", esc(name)));
    xml.push_str(&format!("    <authors>{}</authors>\n", esc(authors)));
    xml.push_str(&format!(
        "    <description>{}</description>\n",
        esc(description)
    ));
    xml.push_str(&format!(
        "    <projectUrl>{}</projectUrl>\n",
        esc(project_url)
    ));
    if !icon_url.is_empty() {
        xml.push_str(&format!("    <iconUrl>{}</iconUrl>\n", esc(icon_url)));
    }
    xml.push_str(&format!(
        "    <licenseUrl>https://opensource.org/licenses/{}</licenseUrl>\n",
        esc(license)
    ));
    xml.push_str(&format!("    <tags>{}</tags>\n", esc(&tags_str)));
    xml.push_str("  </metadata>\n");
    xml.push_str("  <files>\n");
    xml.push_str(
        "    <file src=\"tools\\**\" target=\"tools\" />\n",
    );
    xml.push_str("  </files>\n");
    xml.push_str("</package>\n");

    xml
}

// ---------------------------------------------------------------------------
// generate_install_script
// ---------------------------------------------------------------------------

/// Generate a `chocolateyInstall.ps1` PowerShell script string.
///
/// The `_version` parameter is accepted for API symmetry with other manifest
/// generators and may be used in future for version-specific install logic.
pub fn generate_install_script(name: &str, _version: &str, url: &str, hash: &str) -> String {
    let mut ps = String::new();
    ps.push_str("$ErrorActionPreference = 'Stop'\n\n");
    ps.push_str("$packageArgs = @{\n");
    ps.push_str(&format!("  packageName    = '{}'\n", name));
    ps.push_str(&format!("  url64bit       = '{}'\n", url));
    ps.push_str(&format!("  checksum64     = '{}'\n", hash));
    ps.push_str("  checksumType64 = 'sha256'\n");
    ps.push_str(
        "  unzipLocation  = \"$(Split-Path -Parent $MyInvocation.MyCommand.Definition)\"\n",
    );
    ps.push_str("}\n\n");
    ps.push_str("Install-ChocolateyZipPackage @packageArgs\n");

    ps
}

// ---------------------------------------------------------------------------
// publish_to_chocolatey
// ---------------------------------------------------------------------------

pub fn publish_to_chocolatey(ctx: &Context, crate_name: &str) -> Result<()> {
    let crate_cfg = ctx
        .config
        .crates
        .iter()
        .find(|c| c.name == crate_name)
        .ok_or_else(|| {
            anyhow::anyhow!("chocolatey: crate '{}' not found in config", crate_name)
        })?;

    let publish = crate_cfg
        .publish
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("chocolatey: no publish config for '{}'", crate_name))?;

    let choco_cfg = publish
        .chocolatey
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("chocolatey: no chocolatey config for '{}'", crate_name))?;

    let source_repo = choco_cfg.source_repo.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "chocolatey: no source_repo config for '{}'",
            crate_name
        )
    })?;

    if ctx.is_dry_run() {
        eprintln!(
            "[publish] (dry-run) would push Chocolatey package for '{}' to {}/{}",
            crate_name, source_repo.owner, source_repo.name
        );
        return Ok(());
    }

    // Resolve version.
    let version = ctx
        .template_vars()
        .get("Version")
        .cloned()
        .unwrap_or_default();

    let description = choco_cfg
        .description
        .clone()
        .unwrap_or_else(|| crate_name.to_string());
    let license = choco_cfg
        .license
        .clone()
        .unwrap_or_else(|| "MIT".to_string());
    let authors = choco_cfg
        .authors
        .clone()
        .unwrap_or_else(|| crate_name.to_string());
    let project_url = choco_cfg
        .project_url
        .clone()
        .unwrap_or_else(|| format!("https://github.com/{}/{}", source_repo.owner, source_repo.name));
    let icon_url = choco_cfg.icon_url.clone().unwrap_or_default();
    let tags = choco_cfg.tags.clone().unwrap_or_default();

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
            "[publish] chocolatey: no windows artifact found for '{}', using placeholder URL",
            crate_name
        );
        (
            format!(
                "https://github.com/{0}/{1}/releases/download/v{2}/{1}-{2}-windows-amd64.zip",
                source_repo.owner, crate_name, version
            ),
            String::new(),
        )
    };

    let nuspec = generate_nuspec(
        crate_name,
        &version,
        &description,
        &license,
        &authors,
        &project_url,
        &icon_url,
        &tags,
    );
    let install_script = generate_install_script(crate_name, &version, &url, &hash);

    // Create temp directory, write files, run choco pack + push.
    let tmp_dir = tempfile::tempdir().context("chocolatey: create temp dir")?;
    let pkg_dir = tmp_dir.path();

    let nuspec_path = pkg_dir.join(format!("{}.nuspec", crate_name));
    std::fs::write(&nuspec_path, &nuspec)
        .with_context(|| format!("chocolatey: write nuspec {}", nuspec_path.display()))?;

    let tools_dir = pkg_dir.join("tools");
    std::fs::create_dir_all(&tools_dir).context("chocolatey: create tools dir")?;

    let install_path = tools_dir.join("chocolateyInstall.ps1");
    std::fs::write(&install_path, &install_script)
        .with_context(|| format!("chocolatey: write install script {}", install_path.display()))?;

    eprintln!(
        "[publish] wrote Chocolatey nuspec: {}",
        nuspec_path.display()
    );
    eprintln!(
        "[publish] wrote Chocolatey install script: {}",
        install_path.display()
    );

    // choco pack
    run_cmd_in(
        pkg_dir,
        "choco",
        &["pack", &nuspec_path.to_string_lossy()],
        "chocolatey: choco pack",
    )?;

    // choco push
    let nupkg = pkg_dir.join(format!("{}.{}.nupkg", crate_name, version));
    run_cmd_in(
        pkg_dir,
        "choco",
        &["push", &nupkg.to_string_lossy(), "--source", "https://push.chocolatey.org/"],
        "chocolatey: choco push",
    )?;

    eprintln!(
        "[publish] Chocolatey package pushed for '{}'",
        crate_name
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
    // generate_nuspec tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_nuspec_basic() {
        let nuspec = generate_nuspec(
            "mytool",
            "1.0.0",
            "A great tool",
            "MIT",
            "Test Author",
            "https://github.com/org/mytool",
            "https://example.com/icon.png",
            &["cli".to_string(), "tool".to_string()],
        );

        assert!(nuspec.contains("<?xml version=\"1.0\""));
        assert!(nuspec.contains("<id>mytool</id>"));
        assert!(nuspec.contains("<version>1.0.0</version>"));
        assert!(nuspec.contains("<title>mytool</title>"));
        assert!(nuspec.contains("<authors>Test Author</authors>"));
        assert!(nuspec.contains("<description>A great tool</description>"));
        assert!(nuspec.contains("<projectUrl>https://github.com/org/mytool</projectUrl>"));
        assert!(nuspec.contains("<iconUrl>https://example.com/icon.png</iconUrl>"));
        assert!(nuspec.contains("<tags>cli tool</tags>"));
        assert!(nuspec.contains("<file src=\"tools\\**\" target=\"tools\" />"));
    }

    #[test]
    fn test_generate_nuspec_no_icon() {
        let nuspec = generate_nuspec(
            "mytool",
            "1.0.0",
            "A tool",
            "MIT",
            "Author",
            "https://example.com",
            "",
            &[],
        );

        assert!(!nuspec.contains("<iconUrl>"));
    }

    #[test]
    fn test_generate_nuspec_empty_tags_uses_name() {
        let nuspec = generate_nuspec(
            "mytool",
            "1.0.0",
            "A tool",
            "MIT",
            "Author",
            "https://example.com",
            "",
            &[],
        );

        assert!(nuspec.contains("<tags>mytool</tags>"));
    }

    #[test]
    fn test_generate_nuspec_xml_escaping() {
        let nuspec = generate_nuspec(
            "my-tool",
            "1.0.0",
            "A tool for <things> & \"stuff\"",
            "MIT",
            "Author",
            "https://example.com",
            "",
            &[],
        );

        assert!(nuspec.contains("&lt;things&gt;"));
        assert!(nuspec.contains("&amp;"));
        assert!(nuspec.contains("&quot;stuff&quot;"));
        // Should still be valid XML structure
        assert!(nuspec.contains("<?xml version=\"1.0\""));
        assert!(nuspec.contains("</package>"));
    }

    #[test]
    fn test_generate_nuspec_has_license_url() {
        let nuspec = generate_nuspec(
            "tool",
            "2.0.0",
            "desc",
            "Apache-2.0",
            "Author",
            "https://example.com",
            "",
            &[],
        );

        assert!(nuspec.contains("<licenseUrl>https://opensource.org/licenses/Apache-2.0</licenseUrl>"));
    }

    #[test]
    fn test_generate_nuspec_complete_xml_structure() {
        let nuspec = generate_nuspec(
            "release-tool",
            "3.2.1",
            "Release automation",
            "MIT",
            "Jane Doe",
            "https://github.com/org/release-tool",
            "https://example.com/icon.png",
            &["release".to_string(), "automation".to_string(), "ci".to_string()],
        );

        // Verify the XML starts and ends correctly
        assert!(nuspec.starts_with("<?xml version=\"1.0\" encoding=\"utf-8\"?>"));
        assert!(nuspec.ends_with("</package>\n"));

        // Verify metadata section
        assert!(nuspec.contains("<metadata>"));
        assert!(nuspec.contains("</metadata>"));

        // Verify files section
        assert!(nuspec.contains("<files>"));
        assert!(nuspec.contains("</files>"));
    }

    // -----------------------------------------------------------------------
    // generate_install_script tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_install_script_basic() {
        let script = generate_install_script(
            "mytool",
            "1.0.0",
            "https://example.com/mytool-1.0.0-windows-amd64.zip",
            "deadbeef",
        );

        assert!(script.contains("$ErrorActionPreference = 'Stop'"));
        assert!(script.contains("packageName    = 'mytool'"));
        assert!(script.contains("url64bit       = 'https://example.com/mytool-1.0.0-windows-amd64.zip'"));
        assert!(script.contains("checksum64     = 'deadbeef'"));
        assert!(script.contains("checksumType64 = 'sha256'"));
        assert!(script.contains("Install-ChocolateyZipPackage @packageArgs"));
    }

    #[test]
    fn test_generate_install_script_has_unzip_location() {
        let script = generate_install_script(
            "tool",
            "2.0.0",
            "https://example.com/tool.zip",
            "abc",
        );

        assert!(script.contains("unzipLocation"));
        assert!(script.contains("Split-Path"));
    }

    #[test]
    fn test_generate_install_script_structure() {
        let script = generate_install_script(
            "my-app",
            "0.5.0",
            "https://example.com/my-app.zip",
            "hash123",
        );

        // Verify the script has the expected structure
        let lines: Vec<&str> = script.lines().collect();
        assert_eq!(lines[0], "$ErrorActionPreference = 'Stop'");
        // There should be a blank line after ErrorActionPreference
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "$packageArgs = @{");
        // Script should end with the Install command
        assert!(script.trim_end().ends_with("Install-ChocolateyZipPackage @packageArgs"));
    }

    // -----------------------------------------------------------------------
    // publish_to_chocolatey dry-run tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_publish_to_chocolatey_dry_run() {
        use anodize_core::config::{
            ChocolateyConfig, ChocolateyRepoConfig, Config, CrateConfig, PublishConfig,
        };
        use anodize_core::context::{Context, ContextOptions};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                chocolatey: Some(ChocolateyConfig {
                    source_repo: Some(ChocolateyRepoConfig {
                        owner: "myorg".to_string(),
                        name: "mytool".to_string(),
                    }),
                    description: Some("A great tool".to_string()),
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
        assert!(publish_to_chocolatey(&ctx, "mytool").is_ok());
    }

    #[test]
    fn test_publish_to_chocolatey_missing_config() {
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

        // Should fail because there's no chocolatey config
        assert!(publish_to_chocolatey(&ctx, "mytool").is_err());
    }

    #[test]
    fn test_publish_to_chocolatey_missing_source_repo() {
        use anodize_core::config::{ChocolateyConfig, Config, CrateConfig, PublishConfig};
        use anodize_core::context::{Context, ContextOptions};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                chocolatey: Some(ChocolateyConfig {
                    source_repo: None, // Missing
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

        // Should fail because source_repo is missing
        assert!(publish_to_chocolatey(&ctx, "mytool").is_err());
    }
}

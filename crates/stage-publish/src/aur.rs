use anodize_core::context::Context;
use anyhow::{Context as _, Result};

use crate::util::{find_artifacts_by_os, run_cmd_in};

// ---------------------------------------------------------------------------
// PkgbuildParams
// ---------------------------------------------------------------------------

/// Parameters for generating an Arch Linux PKGBUILD file.
pub struct PkgbuildParams<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub pkgrel: u32,
    pub description: &'a str,
    pub url: &'a str,
    pub license: &'a str,
    pub maintainers: &'a [String],
    pub depends: &'a [String],
    pub optdepends: &'a [String],
    pub conflicts: &'a [String],
    pub provides: &'a [String],
    pub replaces: &'a [String],
    pub backup: &'a [String],
    /// `(arch, url, sha256)` tuples — e.g. `("x86_64", url, hash)`.
    pub sources: &'a [(String, String, String)],
    pub binary_name: &'a str,
}

// ---------------------------------------------------------------------------
// generate_pkgbuild
// ---------------------------------------------------------------------------

/// Generate an Arch Linux PKGBUILD file string.
pub fn generate_pkgbuild(params: &PkgbuildParams<'_>) -> String {
    let mut out = String::new();

    // Maintainer comments.
    for m in params.maintainers {
        out.push_str(&format!("# Maintainer: {}\n", m));
    }
    if !params.maintainers.is_empty() {
        out.push('\n');
    }

    out.push_str(&format!("pkgname={}\n", params.name));
    out.push_str(&format!("pkgver={}\n", params.version));
    out.push_str(&format!("pkgrel={}\n", params.pkgrel));
    out.push_str(&format!("pkgdesc=\"{}\"\n", params.description));

    // Collect unique architectures from sources.
    let arches: Vec<&str> = {
        let mut a: Vec<&str> = params.sources.iter().map(|(arch, _, _)| arch.as_str()).collect();
        a.sort();
        a.dedup();
        a
    };
    out.push_str(&format!(
        "arch=({})\n",
        arches
            .iter()
            .map(|a| format!("'{}'", a))
            .collect::<Vec<_>>()
            .join(" ")
    ));

    out.push_str(&format!("url=\"{}\"\n", params.url));
    out.push_str(&format!("license=('{}')\n", params.license));

    // depends
    if !params.depends.is_empty() {
        out.push_str(&format!(
            "depends=({})\n",
            params
                .depends
                .iter()
                .map(|d| format!("'{}'", d))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    } else {
        out.push_str("depends=()\n");
    }

    // optdepends
    if !params.optdepends.is_empty() {
        out.push_str(&format!(
            "optdepends=({})\n",
            params
                .optdepends
                .iter()
                .map(|d| format!("'{}'", d))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }

    // conflicts
    if !params.conflicts.is_empty() {
        out.push_str(&format!(
            "conflicts=({})\n",
            params
                .conflicts
                .iter()
                .map(|c| format!("'{}'", c))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }

    // provides
    if !params.provides.is_empty() {
        out.push_str(&format!(
            "provides=({})\n",
            params
                .provides
                .iter()
                .map(|p| format!("'{}'", p))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }

    // replaces
    if !params.replaces.is_empty() {
        out.push_str(&format!(
            "replaces=({})\n",
            params
                .replaces
                .iter()
                .map(|r| format!("'{}'", r))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }

    // backup
    if !params.backup.is_empty() {
        out.push_str(&format!(
            "backup=({})\n",
            params
                .backup
                .iter()
                .map(|b| format!("'{}'", b))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }

    // Per-architecture source and sha256sums.
    for (arch, url, hash) in params.sources {
        out.push_str(&format!("source_{}=(\"{}\")\n", arch, url));
        out.push_str(&format!("sha256sums_{}=('{}')\n", arch, hash));
    }

    // package() function.
    out.push_str(&format!(
        "\npackage() {{\n    install -Dm755 \"$srcdir/{}\" \"$pkgdir/usr/bin/{}\"\n}}\n",
        params.binary_name, params.binary_name
    ));

    out
}

// ---------------------------------------------------------------------------
// publish_to_aur
// ---------------------------------------------------------------------------

pub fn publish_to_aur(ctx: &Context, crate_name: &str) -> Result<()> {
    let crate_cfg = ctx
        .config
        .crates
        .iter()
        .find(|c| c.name == crate_name)
        .ok_or_else(|| anyhow::anyhow!("aur: crate '{}' not found in config", crate_name))?;

    let publish = crate_cfg
        .publish
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("aur: no publish config for '{}'", crate_name))?;

    let aur_cfg = publish
        .aur
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("aur: no aur config for '{}'", crate_name))?;

    let git_url = aur_cfg.git_url.as_ref().ok_or_else(|| {
        anyhow::anyhow!("aur: no git_url config for '{}'", crate_name)
    })?;

    if ctx.is_dry_run() {
        eprintln!(
            "[publish] (dry-run) would push AUR PKGBUILD for '{}' to {}",
            crate_name, git_url
        );
        return Ok(());
    }

    // Resolve version.
    let version = ctx
        .template_vars()
        .get("Version")
        .cloned()
        .unwrap_or_default();

    let package_name = aur_cfg
        .package_name
        .clone()
        .unwrap_or_else(|| crate_name.to_string());
    let description = aur_cfg
        .description
        .clone()
        .unwrap_or_else(|| crate_name.to_string());
    let license = aur_cfg
        .license
        .clone()
        .unwrap_or_else(|| "MIT".to_string());
    let url = aur_cfg.url.clone().unwrap_or_else(|| {
        format!("https://github.com/{}", crate_name)
    });
    let maintainers = aur_cfg.maintainers.clone().unwrap_or_default();
    let depends = aur_cfg.depends.clone().unwrap_or_default();
    let optdepends = aur_cfg.optdepends.clone().unwrap_or_default();
    let conflicts = aur_cfg.conflicts.clone().unwrap_or_default();
    let provides = aur_cfg.provides.clone().unwrap_or_default();
    let replaces = aur_cfg.replaces.clone().unwrap_or_default();
    let backup = aur_cfg.backup.clone().unwrap_or_default();

    // Find Linux artifacts for the AUR package.
    let linux_artifacts = find_artifacts_by_os(ctx, crate_name, "linux");

    let sources: Vec<(String, String, String)> = if linux_artifacts.is_empty() {
        eprintln!(
            "[publish] aur: no linux artifacts found for '{}', using placeholder URLs",
            crate_name
        );
        vec![
            (
                "x86_64".to_string(),
                format!(
                    "https://github.com/{0}/releases/download/v{1}/{0}-{1}-linux-amd64.tar.gz",
                    crate_name, version
                ),
                String::new(),
            ),
        ]
    } else {
        linux_artifacts
            .iter()
            .map(|a| {
                let pkgbuild_arch = if a.arch == "arm64" {
                    "aarch64".to_string()
                } else {
                    "x86_64".to_string()
                };
                (pkgbuild_arch, a.url.clone(), a.sha256.clone())
            })
            .collect()
    };

    let pkgbuild = generate_pkgbuild(&PkgbuildParams {
        name: &package_name,
        version: &version,
        pkgrel: 1,
        description: &description,
        url: &url,
        license: &license,
        maintainers: &maintainers,
        depends: &depends,
        optdepends: &optdepends,
        conflicts: &conflicts,
        provides: &provides,
        replaces: &replaces,
        backup: &backup,
        sources: &sources,
        binary_name: crate_name,
    });

    // Clone AUR repo, write PKGBUILD, commit, push.
    let tmp_dir = tempfile::tempdir().context("aur: create temp dir")?;
    let repo_path = tmp_dir.path();

    let status = std::process::Command::new("git")
        .args(["clone", git_url, &repo_path.to_string_lossy()])
        .status()
        .context("aur: git clone: spawn")?;
    if !status.success() {
        anyhow::bail!("aur: git clone: exited with {}", status);
    }

    let pkgbuild_path = repo_path.join("PKGBUILD");
    std::fs::write(&pkgbuild_path, &pkgbuild)
        .with_context(|| format!("aur: write PKGBUILD {}", pkgbuild_path.display()))?;

    eprintln!(
        "[publish] wrote AUR PKGBUILD: {}",
        pkgbuild_path.display()
    );

    // Generate .SRCINFO using makepkg.
    let srcinfo_result = std::process::Command::new("makepkg")
        .current_dir(repo_path)
        .args(["--printsrcinfo"])
        .output()
        .context("aur: makepkg --printsrcinfo")?;

    if srcinfo_result.status.success() {
        let srcinfo_path = repo_path.join(".SRCINFO");
        std::fs::write(&srcinfo_path, &srcinfo_result.stdout)
            .with_context(|| format!("aur: write .SRCINFO {}", srcinfo_path.display()))?;
        eprintln!(
            "[publish] wrote AUR .SRCINFO: {}",
            srcinfo_path.display()
        );
    } else {
        eprintln!(
            "[publish] aur: makepkg --printsrcinfo failed (may not be available); skipping .SRCINFO generation"
        );
    }

    run_cmd_in(repo_path, "git", &["add", "."], "aur: git add")?;
    run_cmd_in(
        repo_path,
        "git",
        &[
            "commit",
            "-m",
            &format!("Update to version {}", version),
        ],
        "aur: git commit",
    )?;
    run_cmd_in(repo_path, "git", &["push"], "aur: git push")?;

    eprintln!(
        "[publish] AUR package '{}' pushed to {}",
        package_name, git_url
    );

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
    // generate_pkgbuild tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_generate_pkgbuild_basic() {
        let pkgbuild = generate_pkgbuild(&PkgbuildParams {
            name: "mytool",
            version: "1.0.0",
            pkgrel: 1,
            description: "A great tool",
            url: "https://github.com/org/mytool",
            license: "MIT",
            maintainers: &["Jane Doe <jane@example.com>".to_string()],
            depends: &[],
            optdepends: &[],
            conflicts: &[],
            provides: &[],
            replaces: &[],
            backup: &[],
            sources: &[(
                "x86_64".to_string(),
                "https://example.com/mytool-1.0.0-linux-amd64.tar.gz".to_string(),
                "deadbeef1234".to_string(),
            )],
            binary_name: "mytool",
        });

        assert!(pkgbuild.contains("# Maintainer: Jane Doe <jane@example.com>"));
        assert!(pkgbuild.contains("pkgname=mytool"));
        assert!(pkgbuild.contains("pkgver=1.0.0"));
        assert!(pkgbuild.contains("pkgrel=1"));
        assert!(pkgbuild.contains("pkgdesc=\"A great tool\""));
        assert!(pkgbuild.contains("arch=('x86_64')"));
        assert!(pkgbuild.contains("url=\"https://github.com/org/mytool\""));
        assert!(pkgbuild.contains("license=('MIT')"));
        assert!(pkgbuild.contains("depends=()"));
        assert!(pkgbuild.contains(
            "source_x86_64=(\"https://example.com/mytool-1.0.0-linux-amd64.tar.gz\")"
        ));
        assert!(pkgbuild.contains("sha256sums_x86_64=('deadbeef1234')"));
        assert!(pkgbuild.contains("package()"));
        assert!(pkgbuild.contains("install -Dm755 \"$srcdir/mytool\" \"$pkgdir/usr/bin/mytool\""));
    }

    #[test]
    fn test_generate_pkgbuild_multi_arch() {
        let pkgbuild = generate_pkgbuild(&PkgbuildParams {
            name: "mytool",
            version: "2.0.0",
            pkgrel: 1,
            description: "Multi-arch tool",
            url: "https://github.com/org/mytool",
            license: "Apache-2.0",
            maintainers: &[],
            depends: &[],
            optdepends: &[],
            conflicts: &[],
            provides: &[],
            replaces: &[],
            backup: &[],
            sources: &[
                (
                    "x86_64".to_string(),
                    "https://example.com/mytool-2.0.0-linux-amd64.tar.gz".to_string(),
                    "hash_amd64".to_string(),
                ),
                (
                    "aarch64".to_string(),
                    "https://example.com/mytool-2.0.0-linux-arm64.tar.gz".to_string(),
                    "hash_arm64".to_string(),
                ),
            ],
            binary_name: "mytool",
        });

        assert!(pkgbuild.contains("arch=('aarch64' 'x86_64')"));
        assert!(pkgbuild.contains("source_x86_64="));
        assert!(pkgbuild.contains("source_aarch64="));
        assert!(pkgbuild.contains("sha256sums_x86_64=('hash_amd64')"));
        assert!(pkgbuild.contains("sha256sums_aarch64=('hash_arm64')"));
    }

    #[test]
    fn test_generate_pkgbuild_with_depends() {
        let pkgbuild = generate_pkgbuild(&PkgbuildParams {
            name: "mytool",
            version: "1.0.0",
            pkgrel: 1,
            description: "A tool",
            url: "https://example.com",
            license: "MIT",
            maintainers: &[],
            depends: &["glibc".to_string(), "openssl".to_string()],
            optdepends: &["git: for VCS support".to_string()],
            conflicts: &["mytool-git".to_string()],
            provides: &["mytool".to_string()],
            replaces: &["old-mytool".to_string()],
            backup: &["etc/mytool/config.toml".to_string()],
            sources: &[(
                "x86_64".to_string(),
                "https://example.com/mytool.tar.gz".to_string(),
                "hash".to_string(),
            )],
            binary_name: "mytool",
        });

        assert!(pkgbuild.contains("depends=('glibc' 'openssl')"));
        assert!(pkgbuild.contains("optdepends=('git: for VCS support')"));
        assert!(pkgbuild.contains("conflicts=('mytool-git')"));
        assert!(pkgbuild.contains("provides=('mytool')"));
        assert!(pkgbuild.contains("replaces=('old-mytool')"));
        assert!(pkgbuild.contains("backup=('etc/mytool/config.toml')"));
    }

    #[test]
    fn test_generate_pkgbuild_no_maintainers() {
        let pkgbuild = generate_pkgbuild(&PkgbuildParams {
            name: "tool",
            version: "1.0.0",
            pkgrel: 1,
            description: "A tool",
            url: "https://example.com",
            license: "MIT",
            maintainers: &[],
            depends: &[],
            optdepends: &[],
            conflicts: &[],
            provides: &[],
            replaces: &[],
            backup: &[],
            sources: &[(
                "x86_64".to_string(),
                "https://example.com/tool.tar.gz".to_string(),
                "hash".to_string(),
            )],
            binary_name: "tool",
        });

        assert!(!pkgbuild.contains("# Maintainer:"));
        assert!(pkgbuild.starts_with("pkgname="));
    }

    #[test]
    fn test_generate_pkgbuild_complete_structure() {
        let pkgbuild = generate_pkgbuild(&PkgbuildParams {
            name: "anodize",
            version: "3.2.1",
            pkgrel: 1,
            description: "Release automation for Rust projects",
            url: "https://github.com/tj-smith47/anodize",
            license: "Apache-2.0",
            maintainers: &["TJ Smith <tj@example.com>".to_string()],
            depends: &[],
            optdepends: &[],
            conflicts: &[],
            provides: &[],
            replaces: &[],
            backup: &[],
            sources: &[
                (
                    "x86_64".to_string(),
                    "https://github.com/tj-smith47/anodize/releases/download/v3.2.1/anodize-3.2.1-linux-amd64.tar.gz".to_string(),
                    "aabbccdd".to_string(),
                ),
                (
                    "aarch64".to_string(),
                    "https://github.com/tj-smith47/anodize/releases/download/v3.2.1/anodize-3.2.1-linux-arm64.tar.gz".to_string(),
                    "eeff0011".to_string(),
                ),
            ],
            binary_name: "anodize",
        });

        // Starts with maintainer comment
        assert!(pkgbuild.starts_with("# Maintainer: TJ Smith <tj@example.com>"));

        // Contains required fields
        assert!(pkgbuild.contains("pkgname=anodize"));
        assert!(pkgbuild.contains("pkgver=3.2.1"));
        assert!(pkgbuild.contains("arch=('aarch64' 'x86_64')"));

        // Contains package() function
        assert!(pkgbuild.contains("package() {"));
        assert!(pkgbuild.contains("install -Dm755"));

        // Ends with closing brace
        assert!(pkgbuild.trim_end().ends_with('}'));
    }

    // -----------------------------------------------------------------------
    // publish_to_aur dry-run tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_publish_to_aur_dry_run() {
        use anodize_core::config::{AurConfig, Config, CrateConfig, PublishConfig};
        use anodize_core::context::{Context, ContextOptions};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                aur: Some(AurConfig {
                    git_url: Some(
                        "ssh://aur@aur.archlinux.org/mytool.git".to_string(),
                    ),
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

        assert!(publish_to_aur(&ctx, "mytool").is_ok());
    }

    #[test]
    fn test_publish_to_aur_missing_config() {
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

        assert!(publish_to_aur(&ctx, "mytool").is_err());
    }

    #[test]
    fn test_publish_to_aur_missing_git_url() {
        use anodize_core::config::{AurConfig, Config, CrateConfig, PublishConfig};
        use anodize_core::context::{Context, ContextOptions};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                aur: Some(AurConfig {
                    git_url: None, // Missing
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

        assert!(publish_to_aur(&ctx, "mytool").is_err());
    }
}

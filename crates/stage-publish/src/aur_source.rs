use anodize_core::context::Context;
use anodize_core::log::StageLogger;
use anyhow::{Context as _, Result};

use crate::util;

/// Publish AUR source packages for a crate.
///
/// Unlike the binary AUR publisher (which generates `-bin` packages with
/// prebuilt binaries), this generates source-only PKGBUILDs that build from
/// source using `cargo build`. The package name does NOT have a `-bin` suffix.
pub fn publish_to_aur_source(ctx: &mut Context, crate_name: &str, log: &StageLogger) -> Result<()> {
    let crate_cfg = ctx
        .config
        .crates
        .iter()
        .find(|c| c.name == crate_name)
        .ok_or_else(|| anyhow::anyhow!("aur_source: crate '{}' not found", crate_name))?;
    let publish_cfg = crate_cfg
        .publish
        .as_ref()
        .and_then(|p| p.aur_source.as_ref())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "aur_source: no aur_source config for crate '{}'",
                crate_name
            )
        })?;

    // Check disable
    if let Some(ref d) = publish_cfg.disable {
        if d.is_disabled(|tmpl| ctx.render_template(tmpl)) {
            log.status(&format!(
                "aur_source: skipping disabled config for crate '{}'",
                crate_name
            ));
            return Ok(());
        }
    }

    // Check skip_upload
    if let Some(ref skip) = publish_cfg.skip_upload {
        let should_skip = skip.is_disabled(|tmpl| ctx.render_template(tmpl));
        if should_skip {
            log.status(&format!(
                "aur_source: skipping upload for crate '{}' (skip_upload=true)",
                crate_name
            ));
            return Ok(());
        }
    }

    let version = ctx
        .template_vars()
        .get("Version")
        .cloned()
        .unwrap_or_else(|| "0.0.0".to_string())
        .replace('-', "_");

    // Package name: no -bin suffix (this is a source package)
    let pkg_name = publish_cfg
        .name
        .clone()
        .unwrap_or_else(|| crate_name.to_string());

    let description = publish_cfg
        .description
        .as_deref()
        .unwrap_or(crate_name);
    let homepage = publish_cfg
        .homepage
        .as_deref()
        .unwrap_or("");
    let license = publish_cfg
        .license
        .as_deref()
        .unwrap_or("MIT");

    let pkgrel: u32 = publish_cfg
        .rel
        .as_deref()
        .and_then(|r| r.parse().ok())
        .unwrap_or(1);

    // Source URL — use url_template or default release URL
    let tag = ctx
        .template_vars()
        .get("Tag")
        .cloned()
        .unwrap_or_default();

    let source_url = if let Some(ref tmpl) = publish_cfg.url_template {
        ctx.render_template(tmpl)
            .with_context(|| "aur_source: render url_template")?
    } else {
        // Default: GitHub release source tarball.
        // Extract owner from GitURL, supporting both HTTPS and SSH formats:
        //   https://github.com/owner/repo  -> owner at split('/')[3]
        //   git@github.com:owner/repo.git  -> owner before '/'
        let git_url = ctx
            .template_vars()
            .get("GitURL")
            .cloned()
            .unwrap_or_default();
        let owner = if git_url.contains("://") {
            // HTTPS-style URL
            git_url.split('/').nth(3).unwrap_or("").to_string()
        } else if git_url.contains(':') {
            // SSH-style URL (git@host:owner/repo)
            git_url
                .split(':')
                .nth(1)
                .unwrap_or("")
                .split('/')
                .next()
                .unwrap_or("")
                .to_string()
        } else {
            String::new()
        };
        let project = ctx
            .template_vars()
            .get("ProjectName")
            .cloned()
            .unwrap_or_default();
        if owner.is_empty() {
            log.warn("aur_source: could not extract owner from GitURL; set url_template explicitly");
        }
        format!(
            "https://github.com/{owner}/{project}/archive/refs/tags/{tag}.tar.gz",
            owner = owner,
            project = project,
            tag = tag,
        )
    };

    // Source URL has been computed above — used in PKGBUILD generation

    let maintainers = publish_cfg.maintainers.clone().unwrap_or_default();
    let contributors = publish_cfg.contributors.clone().unwrap_or_default();
    let depends = publish_cfg.depends.clone().unwrap_or_default();
    let optdepends = publish_cfg.optdepends.clone().unwrap_or_default();
    let conflicts = publish_cfg
        .conflicts
        .clone()
        .unwrap_or_else(|| vec![format!("{}-bin", pkg_name)]);
    let provides = publish_cfg
        .provides
        .clone()
        .unwrap_or_else(|| vec![pkg_name.clone()]);
    let backup = publish_cfg.backup.clone().unwrap_or_default();
    let makedepends = publish_cfg
        .makedepends
        .clone()
        .unwrap_or_else(|| vec!["rust".to_string(), "cargo".to_string()]);

    // Generate the source PKGBUILD using a custom template
    let pkgbuild = generate_source_pkgbuild(
        &pkg_name,
        &version,
        pkgrel,
        description,
        homepage,
        license,
        &maintainers,
        &contributors,
        &depends,
        &makedepends,
        &optdepends,
        &conflicts,
        &provides,
        &backup,
        &source_url,
        publish_cfg.prepare.as_deref(),
        publish_cfg.build.as_deref(),
        publish_cfg.package.as_deref(),
        crate_name,
    );

    // Generate .SRCINFO
    let srcinfo = generate_source_srcinfo(
        &pkg_name,
        &version,
        pkgrel,
        description,
        homepage,
        license,
        &depends,
        &makedepends,
        &optdepends,
        &conflicts,
        &provides,
        &source_url,
    );

    if ctx.is_dry_run() {
        log.status(&format!(
            "(dry-run) would publish AUR source package '{}'",
            pkg_name
        ));
        log.verbose(&format!("PKGBUILD:\n{}", pkgbuild));
        return Ok(());
    }

    // Write files to dist
    let dist = ctx.config.dist.clone();
    let aur_dir = dist.join("aur_source").join(&pkg_name);
    std::fs::create_dir_all(&aur_dir)
        .with_context(|| format!("aur_source: create dir {}", aur_dir.display()))?;

    std::fs::write(aur_dir.join("PKGBUILD"), &pkgbuild)
        .with_context(|| "aur_source: write PKGBUILD")?;
    std::fs::write(aur_dir.join(".SRCINFO"), &srcinfo)
        .with_context(|| "aur_source: write .SRCINFO")?;

    // Register artifacts
    let pkgbuild_path = aur_dir.join("PKGBUILD");
    let srcinfo_path = aur_dir.join(".SRCINFO");

    ctx.artifacts.add(anodize_core::artifact::Artifact {
        kind: anodize_core::artifact::ArtifactKind::SourcePkgBuild,
        name: "PKGBUILD".to_string(),
        path: pkgbuild_path,
        target: None,
        crate_name: crate_name.to_string(),
        metadata: {
            let mut m = std::collections::HashMap::new();
            m.insert("id".to_string(), pkg_name.clone());
            m.insert("format".to_string(), "aur_source".to_string());
            m
        },
        size: None,
    });

    ctx.artifacts.add(anodize_core::artifact::Artifact {
        kind: anodize_core::artifact::ArtifactKind::SourceSrcInfo,
        name: ".SRCINFO".to_string(),
        path: srcinfo_path,
        target: None,
        crate_name: crate_name.to_string(),
        metadata: {
            let mut m = std::collections::HashMap::new();
            m.insert("id".to_string(), pkg_name.clone());
            m
        },
        size: None,
    });

    // Push to AUR git repo if configured
    if let Some(ref git_url) = publish_cfg.git_url {
        let tmp_dir = tempfile::tempdir().context("aur_source: create temp dir")?;
        let repo_path = tmp_dir.path();

        if publish_cfg.private_key.is_some() || publish_cfg.git_ssh_command.is_some() {
            util::clone_repo_ssh(
                git_url,
                publish_cfg.private_key.as_deref(),
                publish_cfg.git_ssh_command.as_deref(),
                repo_path,
                "aur_source",
                log,
            )?;
        } else {
            util::clone_repo_with_auth(git_url, None, repo_path, "aur_source", log)?;
        }

        // Determine output directory
        let output_dir = if let Some(ref dir) = publish_cfg.directory {
            let rendered_dir = ctx.render_template(dir).unwrap_or_else(|_| dir.clone());
            let d = repo_path.join(&rendered_dir);
            std::fs::create_dir_all(&d)?;
            d
        } else {
            repo_path.to_path_buf()
        };

        std::fs::copy(aur_dir.join("PKGBUILD"), output_dir.join("PKGBUILD"))?;
        std::fs::copy(aur_dir.join(".SRCINFO"), output_dir.join(".SRCINFO"))?;

        let commit_msg = crate::homebrew::render_commit_msg(
            publish_cfg.commit_msg_template.as_deref(),
            &pkg_name,
            &version,
            "package",
        );
        let commit_opts = util::resolve_commit_opts(
            publish_cfg.commit_author.as_ref(),
            None,
            None,
        );
        util::commit_and_push_with_opts(
            repo_path,
            &["."],
            &commit_msg,
            None,
            "aur_source",
            &commit_opts,
        )?;

        log.status(&format!(
            "aur_source: package '{}' pushed to {}",
            pkg_name, git_url
        ));
    }

    log.status(&format!("aur_source: published '{}'", pkg_name));
    Ok(())
}

/// Generate a .SRCINFO file for a source AUR package.
fn generate_source_srcinfo(
    name: &str,
    version: &str,
    pkgrel: u32,
    description: &str,
    homepage: &str,
    license: &str,
    depends: &[String],
    makedepends: &[String],
    optdepends: &[String],
    conflicts: &[String],
    provides: &[String],
    source_url: &str,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!("pkgbase = {}", name));
    lines.push(format!("\tpkgdesc = {}", description));
    lines.push(format!("\tpkgver = {}", version));
    lines.push(format!("\tpkgrel = {}", pkgrel));
    if !homepage.is_empty() {
        lines.push(format!("\turl = {}", homepage));
    }
    lines.push("\tarch = x86_64".to_string());
    lines.push("\tarch = aarch64".to_string());
    lines.push(format!("\tlicense = {}", license));
    for d in makedepends {
        lines.push(format!("\tmakedepends = {}", d));
    }
    for d in depends {
        lines.push(format!("\tdepends = {}", d));
    }
    for d in optdepends {
        lines.push(format!("\toptdepends = {}", d));
    }
    for c in conflicts {
        lines.push(format!("\tconflicts = {}", c));
    }
    for p in provides {
        lines.push(format!("\tprovides = {}", p));
    }
    lines.push(format!("\tsource = {}", source_url));
    lines.push("\tsha256sums = SKIP".to_string());
    lines.push(String::new());
    lines.push(format!("pkgname = {}", name));
    lines.join("\n")
}

/// Generate a source-only PKGBUILD that builds from source using cargo.
fn generate_source_pkgbuild(
    name: &str,
    version: &str,
    pkgrel: u32,
    description: &str,
    homepage: &str,
    license: &str,
    maintainers: &[String],
    contributors: &[String],
    depends: &[String],
    makedepends: &[String],
    optdepends: &[String],
    conflicts: &[String],
    provides: &[String],
    backup: &[String],
    source_url: &str,
    prepare: Option<&str>,
    build: Option<&str>,
    package: Option<&str>,
    binary_name: &str,
) -> String {
    let mut lines = Vec::new();

    // Header comments
    for m in maintainers {
        lines.push(format!("# Maintainer: {}", m));
    }
    for c in contributors {
        lines.push(format!("# Contributor: {}", c));
    }
    if !maintainers.is_empty() || !contributors.is_empty() {
        lines.push(String::new());
    }

    lines.push(format!("pkgname='{}'", name));
    lines.push(format!("pkgver='{}'", version));
    lines.push(format!("pkgrel={}", pkgrel));
    lines.push(format!("pkgdesc=\"{}\"", description));
    lines.push("arch=('x86_64' 'aarch64')".to_string());
    if !homepage.is_empty() {
        lines.push(format!("url='{}'", homepage));
    }
    lines.push(format!("license=('{}')", license));

    if !depends.is_empty() {
        let d: Vec<String> = depends.iter().map(|s| format!("'{}'", s)).collect();
        lines.push(format!("depends=({})", d.join(" ")));
    }
    if !makedepends.is_empty() {
        let d: Vec<String> = makedepends.iter().map(|s| format!("'{}'", s)).collect();
        lines.push(format!("makedepends=({})", d.join(" ")));
    }
    if !optdepends.is_empty() {
        let d: Vec<String> = optdepends.iter().map(|s| format!("'{}'", s)).collect();
        lines.push(format!("optdepends=({})", d.join(" ")));
    }
    if !conflicts.is_empty() {
        let d: Vec<String> = conflicts.iter().map(|s| format!("'{}'", s)).collect();
        lines.push(format!("conflicts=({})", d.join(" ")));
    }
    if !provides.is_empty() {
        let d: Vec<String> = provides.iter().map(|s| format!("'{}'", s)).collect();
        lines.push(format!("provides=({})", d.join(" ")));
    }
    if !backup.is_empty() {
        let d: Vec<String> = backup.iter().map(|s| format!("'{}'", s)).collect();
        lines.push(format!("backup=({})", d.join(" ")));
    }

    lines.push(format!("source=(\"{}\")", source_url));
    lines.push("sha256sums=('SKIP')".to_string());

    lines.push(String::new());

    // prepare() function
    if let Some(prep) = prepare {
        lines.push("prepare() {".to_string());
        for line in prep.lines() {
            lines.push(format!("  {}", line));
        }
        lines.push("}".to_string());
        lines.push(String::new());
    }

    // build() function
    lines.push("build() {".to_string());
    if let Some(b) = build {
        for line in b.lines() {
            lines.push(format!("  {}", line));
        }
    } else {
        lines.push(format!(
            "  cd \"$srcdir/{}-$pkgver\"",
            binary_name
        ));
        lines.push("  cargo build --release --locked".to_string());
    }
    lines.push("}".to_string());
    lines.push(String::new());

    // package() function
    lines.push("package() {".to_string());
    if let Some(pkg) = package {
        for line in pkg.lines() {
            lines.push(format!("  {}", line));
        }
    } else {
        lines.push(format!(
            "  cd \"$srcdir/{}-$pkgver\"",
            binary_name
        ));
        lines.push(format!(
            "  install -Dm755 \"target/release/{}\" \"$pkgdir/usr/bin/{}\"",
            binary_name, binary_name
        ));
    }
    lines.push("}".to_string());

    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_source_pkgbuild() {
        let pkgbuild = generate_source_pkgbuild(
            "myapp",
            "1.0.0",
            1,
            "A test application",
            "https://example.com",
            "MIT",
            &["Test <test@example.com>".to_string()],
            &[],
            &["openssl".to_string()],
            &["rust".to_string(), "cargo".to_string()],
            &[],
            &["myapp-bin".to_string()],
            &["myapp".to_string()],
            &[],
            "https://github.com/user/myapp/archive/refs/tags/v1.0.0.tar.gz",
            None,
            None,
            None,
            "myapp",
        );

        assert!(pkgbuild.contains("pkgname='myapp'"));
        assert!(pkgbuild.contains("pkgver='1.0.0'"));
        assert!(pkgbuild.contains("pkgrel=1"));
        assert!(pkgbuild.contains("arch=('x86_64' 'aarch64')"));
        assert!(pkgbuild.contains("makedepends=('rust' 'cargo')"));
        assert!(pkgbuild.contains("conflicts=('myapp-bin')"));
        assert!(pkgbuild.contains("cargo build --release --locked"));
        assert!(pkgbuild.contains("install -Dm755"));
        assert!(pkgbuild.contains("# Maintainer: Test <test@example.com>"));
    }

    #[test]
    fn test_generate_source_pkgbuild_custom_build() {
        let pkgbuild = generate_source_pkgbuild(
            "myapp",
            "1.0.0",
            1,
            "Test",
            "",
            "MIT",
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            &[],
            "https://example.com/source.tar.gz",
            Some("cd myapp\npatch -p1 < fix.patch"),
            Some("make"),
            Some("make install DESTDIR=\"$pkgdir\""),
            "myapp",
        );

        assert!(pkgbuild.contains("prepare() {"));
        assert!(pkgbuild.contains("patch -p1 < fix.patch"));
        assert!(pkgbuild.contains("make\n}"));
        assert!(pkgbuild.contains("make install DESTDIR=\"$pkgdir\""));
    }

    #[test]
    fn test_aur_source_config_parsing() {
        use anodize_core::config::Config;

        let yaml = r#"
project_name: test
crates:
  - name: myapp
    path: "."
    tag_template: "v{{ .Version }}"
    publish:
      aur_source:
        name: myapp
        description: "My application"
        license: MIT
        makedepends:
          - rust
          - cargo
        depends:
          - openssl
        git_url: "ssh://aur@aur.archlinux.org/myapp.git"
"#;
        let config: Config = serde_yaml_ng::from_str(yaml).unwrap();
        let aur_src = config.crates[0]
            .publish
            .as_ref()
            .unwrap()
            .aur_source
            .as_ref()
            .unwrap();
        assert_eq!(aur_src.name.as_deref(), Some("myapp"));
        assert_eq!(aur_src.makedepends.as_ref().unwrap(), &["rust", "cargo"]);
        assert_eq!(aur_src.depends.as_ref().unwrap(), &["openssl"]);
    }
}

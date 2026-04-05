use anodize_core::context::Context;
use anodize_core::log::StageLogger;
use anyhow::{bail, Context as _, Result};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Generate a package.json object for an NPM binary wrapper package.
///
/// The generated package includes a `scripts.postinstall` entry that invokes
/// `node postinstall.js` to download the correct platform binary at install
/// time.
pub fn generate_package_json(
    name: &str,
    version: &str,
    description: Option<&str>,
    license: Option<&str>,
    author: Option<&str>,
    access: Option<&str>,
    extra: Option<&HashMap<String, serde_json::Value>>,
) -> serde_json::Value {
    let mut pkg = serde_json::json!({
        "name": name,
        "version": version,
        "scripts": {
            "postinstall": "node postinstall.js"
        }
    });

    let obj = pkg.as_object_mut().unwrap();

    if let Some(desc) = description {
        obj.insert(
            "description".to_string(),
            serde_json::Value::String(desc.to_string()),
        );
    }

    if let Some(lic) = license {
        obj.insert(
            "license".to_string(),
            serde_json::Value::String(lic.to_string()),
        );
    }

    if let Some(auth) = author {
        obj.insert(
            "author".to_string(),
            serde_json::Value::String(auth.to_string()),
        );
    }

    if let Some(acc) = access {
        obj.insert(
            "publishConfig".to_string(),
            serde_json::json!({ "access": acc }),
        );
    }

    // Merge extra fields into the root of the package.json object.
    if let Some(extra_fields) = extra {
        for (key, value) in extra_fields {
            obj.insert(key.clone(), value.clone());
        }
    }

    pkg
}

/// Generate a postinstall shell script that downloads the correct binary.
///
/// The script uses `uname -s` and `uname -m` to detect OS and architecture,
/// then downloads the appropriate binary from `download_url_base`.
pub fn generate_postinstall_script(download_url_base: &str) -> String {
    // Ensure the base URL ends with a slash for clean concatenation.
    let base = if download_url_base.ends_with('/') {
        download_url_base.to_string()
    } else {
        format!("{}/", download_url_base)
    };

    format!(
        r#"#!/bin/sh
set -e

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)  OS="linux" ;;
    Darwin) OS="darwin" ;;
    MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
    *)
        echo "Unsupported OS: $OS" >&2
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)  ARCH="amd64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    armv7l)        ARCH="armv7" ;;
    i386|i686)     ARCH="386" ;;
    *)
        echo "Unsupported architecture: $ARCH" >&2
        exit 1
        ;;
esac

URL="{base}${{OS}}_${{ARCH}}"

echo "Downloading binary from $URL ..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o bin "$URL"
elif command -v wget >/dev/null 2>&1; then
    wget -qO bin "$URL"
else
    echo "Error: curl or wget is required" >&2
    exit 1
fi

chmod +x bin
"#,
        base = base,
    )
}

// ---------------------------------------------------------------------------
// publish_to_npm
// ---------------------------------------------------------------------------

/// Publish NPM binary wrapper packages.
///
/// This is a top-level publisher: it reads from `ctx.config.npms` rather than
/// from per-crate publish configs.  Each entry generates a package.json with a
/// postinstall script that downloads the correct platform binary.
pub fn publish_to_npm(ctx: &Context, log: &StageLogger) -> Result<()> {
    let entries = match ctx.config.npms {
        Some(ref v) if !v.is_empty() => v,
        _ => return Ok(()),
    };

    for entry in entries {
        // Check disable flag.
        if let Some(ref d) = entry.disable {
            if d.is_disabled(|tmpl| ctx.render_template(tmpl)) {
                log.status("npm: entry disabled, skipping");
                continue;
            }
        }

        // Evaluate if_condition: render as a template and skip if result is
        // "false" or empty.
        if let Some(ref cond) = entry.if_condition {
            let rendered = ctx
                .render_template(cond)
                .with_context(|| format!("npm: failed to render if condition '{}'", cond))?;
            let trimmed = rendered.trim();
            if trimmed.is_empty() || trimmed == "false" {
                log.status("npm: if condition evaluated to false, skipping");
                continue;
            }
        }

        // Name is required — bail before dry-run so config errors surface even
        // in dry-run mode.
        let name_raw = match entry.name.as_deref() {
            Some(n) if !n.is_empty() => n,
            _ => bail!("npm: 'name' is required but not set"),
        };

        // Template-render all user-facing string fields.
        let name = ctx
            .render_template(name_raw)
            .with_context(|| format!("npm: failed to render name '{}'", name_raw))?;

        let description = match entry.description.as_deref() {
            Some(d) => Some(
                ctx.render_template(d)
                    .with_context(|| format!("npm: failed to render description '{}'", d))?,
            ),
            None => None,
        };

        let homepage = match entry.homepage.as_deref() {
            Some(h) => Some(
                ctx.render_template(h)
                    .with_context(|| format!("npm: failed to render homepage '{}'", h))?,
            ),
            None => None,
        };

        let author = match entry.author.as_deref() {
            Some(a) => Some(
                ctx.render_template(a)
                    .with_context(|| format!("npm: failed to render author '{}'", a))?,
            ),
            None => None,
        };

        let repository = match entry.repository.as_deref() {
            Some(r) => Some(
                ctx.render_template(r)
                    .with_context(|| format!("npm: failed to render repository '{}'", r))?,
            ),
            None => None,
        };

        let bugs = match entry.bugs.as_deref() {
            Some(b) => Some(
                ctx.render_template(b)
                    .with_context(|| format!("npm: failed to render bugs '{}'", b))?,
            ),
            None => None,
        };

        let url_template = match entry.url_template.as_deref() {
            Some(u) => Some(
                ctx.render_template(u)
                    .with_context(|| format!("npm: failed to render url_template '{}'", u))?,
            ),
            None => None,
        };

        // Resolve version from template vars.
        let version = ctx
            .template_vars()
            .get("Version")
            .cloned()
            .unwrap_or_else(|| "0.0.0".to_string());

        let access = entry.access.as_deref();
        let tag = entry.tag.as_deref().unwrap_or("latest");
        let format = entry.format.as_deref().unwrap_or("tgz");

        // --- Dry-run logging ---
        if ctx.is_dry_run() {
            log.status(&format!(
                "(dry-run) would publish NPM package '{}' version '{}'",
                name, version
            ));
            log.status(&format!("(dry-run) access: {:?}", access));
            log.status(&format!("(dry-run) tag: {}", tag));
            log.status(&format!("(dry-run) format: {}", format));
            if let Some(ref ids) = entry.ids {
                log.status(&format!("(dry-run) build ID filter: {:?}", ids));
            }
            if let Some(ref ut) = url_template {
                log.status(&format!("(dry-run) url_template: {}", ut));
            }
            if let Some(ref desc) = description {
                log.status(&format!("(dry-run) description: {}", desc));
            }
            if let Some(ref hp) = homepage {
                log.status(&format!("(dry-run) homepage: {}", hp));
            }
            if let Some(ref auth) = author {
                log.status(&format!("(dry-run) author: {}", auth));
            }
            if let Some(ref repo) = repository {
                log.status(&format!("(dry-run) repository: {}", repo));
            }
            if let Some(ref b) = bugs {
                log.status(&format!("(dry-run) bugs: {}", b));
            }
            if let Some(ref ef) = entry.extra_files {
                log.status(&format!("(dry-run) extra_files: {} entries", ef.len()));
            }
            if let Some(ref extra) = entry.extra {
                log.status(&format!("(dry-run) extra package.json fields: {:?}", extra));
            }
            continue;
        }

        // --- Live mode ---
        // Generate package.json.
        let pkg = generate_package_json(
            &name,
            &version,
            description.as_deref(),
            entry.license.as_deref(),
            author.as_deref(),
            access,
            entry.extra.as_ref(),
        );

        // Generate postinstall script.
        let download_base = url_template.as_deref().unwrap_or("");
        let postinstall = generate_postinstall_script(download_base);

        // Create a temp directory, write package.json and postinstall.js, run
        // `npm publish`.
        let tmp_dir = tempfile::tempdir()
            .context("npm: failed to create temporary directory")?;

        let pkg_json_path = tmp_dir.path().join("package.json");
        let pkg_json_str = serde_json::to_string_pretty(&pkg)
            .context("npm: failed to serialize package.json")?;
        std::fs::write(&pkg_json_path, &pkg_json_str)
            .context("npm: failed to write package.json")?;

        let postinstall_path = tmp_dir.path().join("postinstall.js");
        // Wrap the shell script in a Node.js child_process.execSync call so it
        // works cross-platform via `node postinstall.js`.
        let postinstall_js = format!(
            "const {{ execSync }} = require('child_process');\n\
             execSync(`{}`, {{ stdio: 'inherit' }});\n",
            postinstall.replace('\\', "\\\\").replace('`', "\\`")
        );
        std::fs::write(&postinstall_path, &postinstall_js)
            .context("npm: failed to write postinstall.js")?;

        // Build npm publish command.
        let mut cmd = std::process::Command::new("npm");
        cmd.arg("publish");
        cmd.arg("--tag");
        cmd.arg(tag);
        if let Some(acc) = access {
            cmd.arg("--access");
            cmd.arg(acc);
        }
        cmd.current_dir(tmp_dir.path());

        log.status(&format!("npm: publishing '{}' v{} ...", name, version));

        let output = cmd
            .output()
            .context("npm: failed to run 'npm publish'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "npm: 'npm publish' failed for '{}' v{}: {}",
                name,
                version,
                stderr.trim()
            );
        }

        log.status(&format!("npm: published '{}' v{}", name, version));
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
    use anodize_core::config::{Config, NpmConfig, StringOrBool};
    use anodize_core::context::{Context, ContextOptions};

    fn dry_run_ctx(config: Config) -> Context {
        Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_npm_skips_when_no_config() {
        let config = Config::default();
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_skips_when_empty_vec() {
        let mut config = Config::default();
        config.npms = Some(vec![]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_skips_when_disabled() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            disable: Some(StringOrBool::Bool(true)),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_skips_when_disabled_string_true() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            disable: Some(StringOrBool::String("true".to_string())),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_requires_name() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: None,
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        let err = publish_to_npm(&ctx, &log).unwrap_err();
        assert!(
            err.to_string().contains("'name' is required"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_npm_requires_name_nonempty() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: Some(String::new()),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        let err = publish_to_npm(&ctx, &log).unwrap_err();
        assert!(
            err.to_string().contains("'name' is required"),
            "unexpected error: {}",
            err
        );
    }

    #[test]
    fn test_npm_package_json_generation() {
        let pkg = generate_package_json(
            "@myorg/mypackage",
            "1.0.0",
            Some("My CLI tool"),
            Some("MIT"),
            Some("Jane Doe"),
            Some("public"),
            None,
        );
        assert_eq!(pkg["name"], "@myorg/mypackage");
        assert_eq!(pkg["version"], "1.0.0");
        assert_eq!(pkg["description"], "My CLI tool");
        assert_eq!(pkg["license"], "MIT");
        assert_eq!(pkg["author"], "Jane Doe");
        assert_eq!(pkg["publishConfig"]["access"], "public");
        assert!(pkg["scripts"]["postinstall"].is_string());
        assert_eq!(pkg["scripts"]["postinstall"], "node postinstall.js");
    }

    #[test]
    fn test_npm_package_json_minimal() {
        let pkg = generate_package_json(
            "simple-pkg",
            "0.1.0",
            None,
            None,
            None,
            None,
            None,
        );
        assert_eq!(pkg["name"], "simple-pkg");
        assert_eq!(pkg["version"], "0.1.0");
        assert!(pkg["scripts"]["postinstall"].is_string());
        // Optional fields should be absent.
        assert!(pkg.get("description").is_none());
        assert!(pkg.get("license").is_none());
        assert!(pkg.get("author").is_none());
        assert!(pkg.get("publishConfig").is_none());
    }

    #[test]
    fn test_npm_package_json_with_extra() {
        let mut extra = HashMap::new();
        extra.insert(
            "bin".to_string(),
            serde_json::json!({"mytool": "./bin/mytool"}),
        );
        extra.insert(
            "keywords".to_string(),
            serde_json::json!(["cli", "tool"]),
        );
        let pkg = generate_package_json(
            "@myorg/mypackage",
            "1.0.0",
            None,
            None,
            None,
            None,
            Some(&extra),
        );
        assert_eq!(pkg["name"], "@myorg/mypackage");
        assert_eq!(pkg["bin"]["mytool"], "./bin/mytool");
        assert_eq!(pkg["keywords"][0], "cli");
        assert_eq!(pkg["keywords"][1], "tool");
    }

    #[test]
    fn test_npm_postinstall_script_generation() {
        let script =
            generate_postinstall_script("https://github.com/owner/repo/releases/download/v1.0.0/");
        assert!(script.contains("https://github.com/owner/repo/releases/download/v1.0.0/"));
        assert!(script.contains("uname -s")); // OS detection
        assert!(script.contains("uname -m")); // Arch detection
        assert!(script.contains("curl"));
        assert!(script.contains("wget"));
        assert!(script.contains("chmod +x"));
    }

    #[test]
    fn test_npm_postinstall_script_adds_trailing_slash() {
        let script =
            generate_postinstall_script("https://github.com/owner/repo/releases/download/v1.0.0");
        // Should have added a trailing slash.
        assert!(script.contains("https://github.com/owner/repo/releases/download/v1.0.0/"));
    }

    #[test]
    fn test_npm_postinstall_script_no_double_slash() {
        let script =
            generate_postinstall_script("https://github.com/owner/repo/releases/download/v1.0.0/");
        // Should NOT have a double trailing slash.
        assert!(!script.contains("v1.0.0//"));
    }

    #[test]
    fn test_npm_dry_run() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: Some("@myorg/mypackage".to_string()),
            access: Some("public".to_string()),
            tag: Some("latest".to_string()),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_dry_run_with_all_fields() {
        let mut extra = HashMap::new();
        extra.insert(
            "bin".to_string(),
            serde_json::json!({"mytool": "./bin/mytool"}),
        );

        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: Some("@myorg/mypackage".to_string()),
            description: Some("My CLI tool".to_string()),
            homepage: Some("https://example.com".to_string()),
            author: Some("Jane Doe".to_string()),
            repository: Some("https://github.com/myorg/mypackage".to_string()),
            bugs: Some("https://github.com/myorg/mypackage/issues".to_string()),
            access: Some("public".to_string()),
            tag: Some("latest".to_string()),
            format: Some("tgz".to_string()),
            ids: Some(vec!["build1".to_string()]),
            url_template: Some("https://github.com/myorg/mypackage/releases/download/v1.0.0/".to_string()),
            extra: Some(extra),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_if_condition_skips() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: Some("@myorg/mypackage".to_string()),
            if_condition: Some("false".to_string()),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        // Should be skipped (no error) because "false" is falsy.
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_if_condition_empty_skips() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: Some("@myorg/mypackage".to_string()),
            if_condition: Some("".to_string()),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        // Empty string is falsy — should skip.
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_if_condition_true_proceeds() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: Some("@myorg/mypackage".to_string()),
            if_condition: Some("true".to_string()),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        // "true" is truthy — should proceed (dry-run will log).
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_multiple_entries() {
        let mut config = Config::default();
        config.npms = Some(vec![
            NpmConfig {
                name: Some("@myorg/pkg1".to_string()),
                ..Default::default()
            },
            NpmConfig {
                name: Some("@myorg/pkg2".to_string()),
                disable: Some(StringOrBool::Bool(true)),
                ..Default::default()
            },
        ]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        // First entry proceeds, second is skipped — both ok.
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_default_version_when_not_set() {
        // When Version is not in template vars, should use "0.0.0".
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: Some("@myorg/mypackage".to_string()),
            ..Default::default()
        }]);
        let ctx = dry_run_ctx(config);
        let log = ctx.logger("npm");
        // Should succeed in dry-run with fallback version.
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }

    #[test]
    fn test_npm_uses_version_from_template_vars() {
        let mut config = Config::default();
        config.npms = Some(vec![NpmConfig {
            name: Some("@myorg/mypackage".to_string()),
            ..Default::default()
        }]);
        let mut ctx = dry_run_ctx(config);
        ctx.template_vars_mut().set("Version", "2.5.0");
        let log = ctx.logger("npm");
        // Should succeed — version comes from template vars.
        assert!(publish_to_npm(&ctx, &log).is_ok());
    }
}

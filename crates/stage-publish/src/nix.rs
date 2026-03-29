use anodize_core::context::Context;
use anodize_core::log::StageLogger;
use anyhow::{Context as _, Result};

use crate::util;

// ---------------------------------------------------------------------------
// Nix expression template
// ---------------------------------------------------------------------------

const NIX_TEMPLATE: &str = r#"{ lib
, stdenvNoCC
, fetchurl
{% if needs_unzip %}, unzip
{% endif %}{% if needs_make_wrapper %}, makeWrapper
{% endif %}, installShellFiles
{% for dep in dep_args %}, {{ dep }}
{% endfor %}}:

let
  selectSystem = attrs: attrs.${stdenvNoCC.hostPlatform.system} or (throw "Unsupported system: ${stdenvNoCC.hostPlatform.system}");
  urlMap = {
{% for key, archive in archives %}    {{ key }} = "{{ archive.url }}";
{% endfor %}  };
  shaMap = {
{% for key, archive in archives %}    {{ key }} = "{{ archive.sha }}";
{% endfor %}  };
in
stdenvNoCC.mkDerivation rec {
  pname = "{{ name }}";
  version = "{{ version }}";

  src = fetchurl {
    url = selectSystem urlMap;
    sha256 = selectSystem shaMap;
  };

  sourceRoot = ".";

  nativeBuildInputs = [
    installShellFiles
{% if needs_make_wrapper %}    makeWrapper
{% endif %}{% if needs_unzip %}    unzip
{% endif %}  ];

  installPhase = ''
{% for line in install_lines %}    {{ line }}
{% endfor %}  '';
{% if has_post_install %}
  postInstall = ''
{% for line in post_install_lines %}    {{ line }}
{% endfor %}  '';
{% endif %}
  meta = with lib; {
{% if description %}    description = "{{ description }}";
{% endif %}{% if homepage %}    homepage = "{{ homepage }}";
{% endif %}{% if license %}    license = licenses.{{ license }};
{% endif %}    sourceProvenance = with sourceTypes; [ binaryNativeCode ];
    platforms = [ {% for p in platforms %}"{{ p }}" {% endfor %}];
  };
}
"#;

// ---------------------------------------------------------------------------
// NixParams
// ---------------------------------------------------------------------------

/// Parameters for generating a Nix expression.
pub struct NixParams<'a> {
    pub name: &'a str,
    pub version: &'a str,
    pub description: &'a str,
    pub homepage: &'a str,
    pub license: &'a str,
    /// Per-platform archives: `(nix_system, url, sha256)`.
    pub archives: &'a [(String, String, String)],
    /// Install commands. If empty, auto-generates `cp` for each binary.
    pub install_lines: &'a [String],
    /// Post-install commands.
    pub post_install_lines: &'a [String],
    /// Whether any archive is a .zip (need unzip dep).
    pub needs_unzip: bool,
    /// Whether dependencies are configured (need makeWrapper).
    pub needs_make_wrapper: bool,
    /// Dependency package names to add as function arguments in the derivation.
    pub dep_args: &'a [String],
}

// ---------------------------------------------------------------------------
// generate_nix_expression
// ---------------------------------------------------------------------------

/// Generate a Nix derivation expression string.
pub fn generate_nix_expression(params: &NixParams<'_>) -> String {
    let mut tera = tera::Tera::default();
    tera.add_raw_template("nix", NIX_TEMPLATE)
        .expect("nix: parse template");
    tera.autoescape_on(vec![]);

    let mut ctx = tera::Context::new();
    ctx.insert("name", params.name);
    ctx.insert("version", params.version);
    ctx.insert("description", params.description);
    ctx.insert("homepage", params.homepage);
    ctx.insert("license", params.license);
    ctx.insert("needs_unzip", &params.needs_unzip);
    ctx.insert("needs_make_wrapper", &params.needs_make_wrapper);
    ctx.insert("dep_args", &params.dep_args);

    // Archives map
    #[derive(serde::Serialize)]
    struct ArchiveEntry {
        url: String,
        sha: String,
    }
    let archives: std::collections::BTreeMap<String, ArchiveEntry> = params
        .archives
        .iter()
        .map(|(system, url, sha)| {
            (
                system.clone(),
                ArchiveEntry {
                    url: url.clone(),
                    sha: sha.clone(),
                },
            )
        })
        .collect();
    ctx.insert("archives", &archives);

    // Platforms list
    let platforms: Vec<&str> = params.archives.iter().map(|(s, _, _)| s.as_str()).collect();
    ctx.insert("platforms", &platforms);

    ctx.insert("install_lines", &params.install_lines);
    ctx.insert("has_post_install", &!params.post_install_lines.is_empty());
    ctx.insert("post_install_lines", &params.post_install_lines);

    tera.render("nix", &ctx)
        .expect("nix: render expression")
}

// ---------------------------------------------------------------------------
// License validation
// ---------------------------------------------------------------------------

/// Known valid Nix license identifiers from `lib.licenses`.
const VALID_NIX_LICENSES: &[&str] = &[
    "mit",
    "asl20",
    "bsd2",
    "bsd3",
    "gpl2Only",
    "gpl3Only",
    "lgpl21Only",
    "lgpl3Only",
    "mpl20",
    "isc",
    "unlicense",
    "artistic2",
    "cc0",
    "wtfpl",
    "zlib",
    "publicDomain",
    "unfree",
];

/// Validate that a license identifier is a known Nix license.
/// Returns `Ok(())` if valid, or `Err` with a descriptive message.
pub fn validate_nix_license(license: &str) -> Result<()> {
    if VALID_NIX_LICENSES.contains(&license) {
        Ok(())
    } else {
        anyhow::bail!(
            "nix: unknown license identifier '{}'. Valid values: {}",
            license,
            VALID_NIX_LICENSES.join(", ")
        )
    }
}

// ---------------------------------------------------------------------------
// Nix system mapping
// ---------------------------------------------------------------------------

/// Map canonical (os, arch) to Nix system string.
fn nix_system(os: &str, arch: &str) -> Option<String> {
    let nix_arch = match arch {
        "amd64" | "x86_64" => "x86_64",
        "arm64" | "aarch64" => "aarch64",
        "386" | "i686" => "i686",
        "arm" | "armv7l" => "armv7l",
        _ => return None,
    };
    let nix_os = match os {
        "linux" => "linux",
        "darwin" | "macos" => "darwin",
        _ => return None,
    };
    Some(format!("{}-{}", nix_arch, nix_os))
}

// ---------------------------------------------------------------------------
// publish_to_nix
// ---------------------------------------------------------------------------

pub fn publish_to_nix(ctx: &Context, crate_name: &str, log: &StageLogger) -> Result<()> {
    let (_crate_cfg, publish) = crate::util::get_publish_config(ctx, crate_name, "nix")?;

    let nix_cfg = publish
        .nix
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("nix: no nix config for '{}'", crate_name))?;

    // Check skip_upload before doing any work.
    if crate::homebrew::should_skip_upload(nix_cfg.skip_upload.as_deref(), ctx) {
        log.status(&format!(
            "nix: skipping upload for '{}' (skip_upload={})",
            crate_name,
            nix_cfg.skip_upload.as_deref().unwrap_or("")
        ));
        return Ok(());
    }

    // Resolve repository config.
    let (repo_owner, repo_name) = crate::util::resolve_repo_owner_name(
        nix_cfg.repository.as_ref(),
        None,
        None,
    )
    .ok_or_else(|| {
        anyhow::anyhow!("nix: no repository config for '{}'", crate_name)
    })?;

    let name = nix_cfg.name.as_deref().unwrap_or(crate_name);

    if ctx.is_dry_run() {
        log.status(&format!(
            "(dry-run) would publish Nix expression for '{}' to {}/{}",
            crate_name, repo_owner, repo_name
        ));
        return Ok(());
    }

    let version = ctx.version();
    let description = nix_cfg.description.as_deref().unwrap_or("");
    let homepage = nix_cfg.homepage.as_deref().unwrap_or("");
    let license = nix_cfg.license.as_deref().unwrap_or("mit");

    // Validate license identifier against known Nix licenses.
    validate_nix_license(license)?;

    // Find artifacts for Linux and Darwin platforms, applying IDs filter.
    let ids_filter = nix_cfg.ids.as_deref();
    let all_artifacts = util::find_all_platform_artifacts_filtered(ctx, crate_name, ids_filter);

    let url_template = nix_cfg.url_template.as_deref();

    let archives: Vec<(String, String, String)> = all_artifacts
        .iter()
        .filter_map(|a| {
            let system = nix_system(&a.os, &a.arch)?;
            let download_url = if let Some(tmpl) = url_template {
                util::render_url_template(tmpl, crate_name, &version, &a.arch, &a.os)
            } else {
                a.url.clone()
            };
            Some((system, download_url, a.sha256.clone()))
        })
        .collect();

    if archives.is_empty() {
        anyhow::bail!("nix: no Linux/Darwin archive artifacts found for '{}'", crate_name);
    }

    // Check if any archive is a zip (needs unzip dep)
    let needs_unzip = all_artifacts
        .iter()
        .any(|a| a.url.ends_with(".zip"));

    // Check if dependencies are configured (needs makeWrapper)
    let deps = nix_cfg.dependencies.as_deref().unwrap_or(&[]);
    let needs_make_wrapper = !deps.is_empty();

    // Collect unique dependency package names for the derivation function arguments.
    let dep_args: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        deps.iter()
            .filter(|d| seen.insert(d.name.clone()))
            .map(|d| d.name.clone())
            .collect()
    };

    // Build install lines
    let install_lines: Vec<String> = if let Some(ref custom_install) = nix_cfg.install {
        let mut lines: Vec<String> = custom_install.lines().map(|l| l.to_string()).collect();
        if let Some(ref extra) = nix_cfg.extra_install {
            lines.extend(extra.lines().map(|l| l.to_string()));
        }
        lines
    } else {
        let mut lines = vec!["mkdir -p $out/bin".to_string()];
        lines.push(format!("cp -vr ./{name} $out/bin/{name}"));
        if let Some(ref extra) = nix_cfg.extra_install {
            lines.extend(extra.lines().map(|l| l.to_string()));
        }
        // Generate wrapProgram invocations from dependencies with OS filtering.
        if needs_make_wrapper {
            // Partition deps by OS for conditional wrapping.
            let all_os_deps: Vec<&str> = deps
                .iter()
                .filter(|d| d.os.is_none())
                .map(|d| d.name.as_str())
                .collect();
            let darwin_deps: Vec<&str> = deps
                .iter()
                .filter(|d| d.os.as_deref() == Some("darwin"))
                .map(|d| d.name.as_str())
                .collect();
            let linux_deps: Vec<&str> = deps
                .iter()
                .filter(|d| d.os.as_deref() == Some("linux"))
                .map(|d| d.name.as_str())
                .collect();

            let mut prefix_parts: Vec<String> = Vec::new();
            if !all_os_deps.is_empty() {
                let bins: Vec<String> = all_os_deps
                    .iter()
                    .map(|d| format!("${{lib.getBin {}}}/bin", d))
                    .collect();
                prefix_parts.push(bins.join(":"));
            }
            if !darwin_deps.is_empty() {
                let bins: Vec<String> = darwin_deps
                    .iter()
                    .map(|d| format!("${{lib.getBin {}}}/bin", d))
                    .collect();
                prefix_parts.push(format!(
                    "''${{lib.optionalString stdenvNoCC.isDarwin \"{}\"}}",
                    bins.join(":")
                ));
            }
            if !linux_deps.is_empty() {
                let bins: Vec<String> = linux_deps
                    .iter()
                    .map(|d| format!("${{lib.getBin {}}}/bin", d))
                    .collect();
                prefix_parts.push(format!(
                    "''${{lib.optionalString stdenvNoCC.isLinux \"{}\"}}",
                    bins.join(":")
                ));
            }

            if !prefix_parts.is_empty() {
                lines.push(format!(
                    "wrapProgram $out/bin/{} --prefix PATH : {}",
                    name,
                    prefix_parts.join(":")
                ));
            }
        }
        lines
    };

    let post_install_lines: Vec<String> = nix_cfg
        .post_install
        .as_ref()
        .map(|s| s.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default();

    let nix_expr = generate_nix_expression(&NixParams {
        name,
        version: &version,
        description,
        homepage,
        license,
        archives: &archives,
        install_lines: &install_lines,
        post_install_lines: &post_install_lines,
        needs_unzip,
        needs_make_wrapper,
        dep_args: &dep_args,
    });

    // Optionally format with alejandra or nixfmt
    // (only if the formatter binary is available)

    // Clone repo, write nix expression, commit, push.
    let token = util::resolve_repo_token(ctx, nix_cfg.repository.as_ref(), None);
    let repo_url = format!("https://github.com/{}/{}.git", repo_owner, repo_name);

    let tmp_dir = tempfile::tempdir().context("nix: create temp dir")?;
    let repo_path = tmp_dir.path();
    util::clone_repo_with_auth(&repo_url, token.as_deref(), repo_path, "nix", log)?;

    // Write nix file at configured path or default
    let nix_path = nix_cfg
        .path
        .as_deref()
        .map(|p| p.to_string())
        .unwrap_or_else(|| format!("pkgs/{}/default.nix", name));
    let nix_file = repo_path.join(&nix_path);

    if let Some(parent) = nix_file.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("nix: create dir {}", parent.display()))?;
    }

    std::fs::write(&nix_file, &nix_expr)
        .with_context(|| format!("nix: write {}", nix_file.display()))?;

    // Run formatter if configured
    if let Some(ref formatter) = nix_cfg.formatter {
        let nix_file_str = nix_file.to_string_lossy();
        match formatter.as_str() {
            "alejandra" | "nixfmt" => {
                if let Ok(output) = std::process::Command::new(formatter)
                    .arg(&*nix_file_str)
                    .output()
                {
                    if !output.status.success() {
                        log.warn(&format!("nix: {} formatting failed", formatter));
                    }
                } else {
                    log.warn(&format!("nix: {} not available, skipping format", formatter));
                }
            }
            _ => {
                log.warn(&format!("nix: unknown formatter '{}', skipping", formatter));
            }
        }
    }

    log.status(&format!("wrote Nix expression: {}", nix_file.display()));

    let commit_msg = crate::homebrew::render_commit_msg(
        nix_cfg.commit_msg_template.as_deref(),
        name,
        &version,
        "package",
    );
    let commit_opts = util::resolve_commit_opts(
        nix_cfg.commit_author.as_ref(),
        None,
        None,
    );
    let branch = util::resolve_branch(nix_cfg.repository.as_ref());
    util::commit_and_push_with_opts(
        repo_path,
        &[&nix_path],
        &commit_msg,
        branch,
        "nix",
        &commit_opts,
    )?;

    log.status(&format!(
        "Nix expression pushed to {}/{} for '{}'",
        repo_owner, repo_name, crate_name
    ));

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nix_system_mapping() {
        assert_eq!(nix_system("linux", "amd64"), Some("x86_64-linux".to_string()));
        assert_eq!(nix_system("linux", "arm64"), Some("aarch64-linux".to_string()));
        assert_eq!(nix_system("darwin", "amd64"), Some("x86_64-darwin".to_string()));
        assert_eq!(nix_system("darwin", "arm64"), Some("aarch64-darwin".to_string()));
        assert_eq!(nix_system("linux", "386"), Some("i686-linux".to_string()));
        assert_eq!(nix_system("windows", "amd64"), None);
    }

    #[test]
    fn test_generate_nix_expression_basic() {
        let archives = vec![
            ("x86_64-linux".to_string(), "https://example.com/tool-linux-amd64.tar.gz".to_string(), "abc123".to_string()),
            ("aarch64-darwin".to_string(), "https://example.com/tool-darwin-arm64.tar.gz".to_string(), "def456".to_string()),
        ];
        let install_lines = vec![
            "mkdir -p $out/bin".to_string(),
            "cp -vr ./mytool $out/bin/mytool".to_string(),
        ];

        let expr = generate_nix_expression(&NixParams {
            name: "mytool",
            version: "1.0.0",
            description: "A great tool",
            homepage: "https://example.com",
            license: "mit",
            archives: &archives,
            install_lines: &install_lines,
            post_install_lines: &[],
            needs_unzip: false,
            needs_make_wrapper: false,
            dep_args: &[],
        });

        assert!(expr.contains("pname = \"mytool\""));
        assert!(expr.contains("version = \"1.0.0\""));
        assert!(expr.contains("description = \"A great tool\""));
        assert!(expr.contains("homepage = \"https://example.com\""));
        assert!(expr.contains("licenses.mit"));
        assert!(expr.contains("x86_64-linux"));
        assert!(expr.contains("aarch64-darwin"));
        assert!(expr.contains("abc123"));
        assert!(expr.contains("def456"));
        assert!(expr.contains("mkdir -p $out/bin"));
    }

    #[test]
    fn test_generate_nix_expression_with_unzip() {
        let archives = vec![
            ("x86_64-linux".to_string(), "https://example.com/tool.zip".to_string(), "abc".to_string()),
        ];
        let install = vec!["mkdir -p $out/bin".to_string()];

        let expr = generate_nix_expression(&NixParams {
            name: "mytool",
            version: "1.0.0",
            description: "",
            homepage: "",
            license: "mit",
            archives: &archives,
            install_lines: &install,
            post_install_lines: &[],
            needs_unzip: true,
            needs_make_wrapper: false,
            dep_args: &[],
        });

        assert!(expr.contains(", unzip"));
    }

    #[test]
    fn test_generate_nix_expression_with_post_install() {
        let archives = vec![
            ("x86_64-linux".to_string(), "https://example.com/tool.tar.gz".to_string(), "abc".to_string()),
        ];
        let install = vec!["mkdir -p $out/bin".to_string()];
        let post = vec!["installShellCompletion --bash comp.bash".to_string()];

        let expr = generate_nix_expression(&NixParams {
            name: "mytool",
            version: "1.0.0",
            description: "",
            homepage: "",
            license: "mit",
            archives: &archives,
            install_lines: &install,
            post_install_lines: &post,
            needs_unzip: false,
            needs_make_wrapper: false,
            dep_args: &[],
        });

        assert!(expr.contains("postInstall"));
        assert!(expr.contains("installShellCompletion"));
    }

    #[test]
    fn test_publish_to_nix_dry_run() {
        use anodize_core::config::{Config, CrateConfig, NixConfig, PublishConfig, RepositoryConfig};
        use anodize_core::context::{Context, ContextOptions};
        use anodize_core::log::{StageLogger, Verbosity};

        let config = Config {
            crates: vec![CrateConfig {
                name: "mytool".to_string(),
                path: ".".to_string(),
                tag_template: "v{{ .Version }}".to_string(),
                publish: Some(PublishConfig {
                    nix: Some(NixConfig {
                        repository: Some(RepositoryConfig {
                            owner: Some("myorg".to_string()),
                            name: Some("nixpkgs-overlay".to_string()),
                            ..Default::default()
                        }),
                        description: Some("My tool".to_string()),
                        license: Some("mit".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        let ctx = Context::new(config, ContextOptions { dry_run: true, ..Default::default() });
        let log = StageLogger::new("publish", Verbosity::Normal);
        assert!(publish_to_nix(&ctx, "mytool", &log).is_ok());
    }
}

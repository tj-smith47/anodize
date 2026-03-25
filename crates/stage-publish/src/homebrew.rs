use anodize_core::context::Context;
use anyhow::{Context as _, Result};
use std::process::Command;

// ---------------------------------------------------------------------------
// generate_formula
// ---------------------------------------------------------------------------

/// Generate a Homebrew Ruby formula string.
///
/// `archives` is a slice of `(platform_tag, url, sha256)` tuples.
/// When there is a single archive entry (no `on_` OS block needed) the formula
/// uses a flat `url`/`sha256` pair; otherwise it emits an `on_macos`/`on_linux`
/// block per entry.
pub fn generate_formula(
    name: &str,
    version: &str,
    archives: &[(&str, &str, &str)],
    description: &str,
    license: &str,
    install: &str,
    test: &str,
) -> String {
    // Ruby class name: capitalise first letter, replace hyphens.
    let class_name: String = {
        let mut chars = name.replace('-', "_");
        // PascalCase each segment
        chars = chars
            .split('_')
            .map(|seg| {
                let mut c = seg.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join("");
        chars
    };

    let mut f = String::new();
    f.push_str(&format!("class {} < Formula\n", class_name));
    f.push_str(&format!("  desc \"{}\"\n", description));
    f.push_str(&format!("  homepage \"https://github.com/{}\"\n", name));
    f.push_str(&format!("  license \"{}\"\n", license));
    f.push_str(&format!("  version \"{}\"\n", version));
    f.push('\n');

    match archives {
        [] => {}
        [(_, url, sha256)] => {
            f.push_str(&format!("  url \"{}\"\n", url));
            f.push_str(&format!("  sha256 \"{}\"\n", sha256));
        }
        entries => {
            for (platform, url, sha256) in entries {
                let os_block = if platform.contains("darwin") || platform.contains("macos") {
                    "on_macos"
                } else if platform.contains("linux") {
                    "on_linux"
                } else {
                    // Unknown platform — emit a commented entry.
                    f.push_str(&format!(
                        "  # platform: {}\n  url \"{}\"\n  sha256 \"{}\"\n",
                        platform, url, sha256
                    ));
                    continue;
                };

                let arch_block = if platform.contains("arm64") || platform.contains("aarch64") {
                    Some("on_arm")
                } else if platform.contains("amd64") || platform.contains("x86_64") {
                    Some("on_intel")
                } else {
                    None
                };

                if let Some(arch) = arch_block {
                    f.push_str(&format!("  {} do\n", os_block));
                    f.push_str(&format!("    {} do\n", arch));
                    f.push_str(&format!("      url \"{}\"\n", url));
                    f.push_str(&format!("      sha256 \"{}\"\n", sha256));
                    f.push_str("    end\n");
                    f.push_str("  end\n");
                } else {
                    f.push_str(&format!("  {} do\n", os_block));
                    f.push_str(&format!("    url \"{}\"\n", url));
                    f.push_str(&format!("    sha256 \"{}\"\n", sha256));
                    f.push_str("  end\n");
                }
            }
        }
    }

    f.push('\n');
    f.push_str("  def install\n");
    for line in install.lines() {
        f.push_str(&format!("    {}\n", line));
    }
    f.push_str("  end\n");
    f.push('\n');
    f.push_str("  test do\n");
    for line in test.lines() {
        f.push_str(&format!("    {}\n", line));
    }
    f.push_str("  end\n");
    f.push_str("end\n");

    f
}

// ---------------------------------------------------------------------------
// publish_to_homebrew
// ---------------------------------------------------------------------------

pub fn publish_to_homebrew(ctx: &Context, crate_name: &str) -> Result<()> {
    let crate_cfg = ctx
        .config
        .crates
        .iter()
        .find(|c| c.name == crate_name)
        .ok_or_else(|| anyhow::anyhow!("homebrew: crate '{}' not found in config", crate_name))?;

    let publish = crate_cfg
        .publish
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("homebrew: no publish config for '{}'", crate_name))?;

    let hb_cfg = publish
        .homebrew
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("homebrew: no homebrew config for '{}'", crate_name))?;

    let tap = hb_cfg
        .tap
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("homebrew: no tap config for '{}'", crate_name))?;

    if ctx.is_dry_run() {
        eprintln!(
            "[publish] (dry-run) would update Homebrew tap {}/{} for '{}'",
            tap.owner, tap.name, crate_name
        );
        return Ok(());
    }

    // Resolve version from template vars.
    let version = ctx
        .template_vars()
        .get("Version")
        .cloned()
        .unwrap_or_default();

    let description = hb_cfg
        .description
        .clone()
        .unwrap_or_else(|| crate_name.to_string());
    let license = hb_cfg
        .license
        .clone()
        .unwrap_or_else(|| "MIT".to_string());
    let install = hb_cfg
        .install
        .clone()
        .unwrap_or_else(|| format!("bin.install \"{}\"", crate_name));
    let test_block = hb_cfg
        .test
        .clone()
        .unwrap_or_else(|| format!("system \"#{{bin}}/{}\", \"--version\"", crate_name));

    // Collect Archive artifacts for this crate to build the formula entries.
    let archives: Vec<(&str, &str, &str)> = ctx
        .artifacts
        .by_kind_and_crate(
            anodize_core::artifact::ArtifactKind::Archive,
            crate_name,
        )
        .iter()
        .filter_map(|a| {
            let url = a.metadata.get("url")?.as_str();
            let sha256 = a.metadata.get("sha256")?.as_str();
            let target = a.target.as_deref().unwrap_or("");
            Some((target, url, sha256))
        })
        .collect();

    let formula = generate_formula(
        crate_name,
        &version,
        &archives,
        &description,
        &license,
        &install,
        &test_block,
    );

    // Clone tap repo, write formula, commit, push.
    let tap_repo = format!("https://github.com/{}/{}.git", tap.owner, tap.name);
    let tmp_dir = tempfile::tempdir().context("homebrew: create temp dir")?;
    let repo_path = tmp_dir.path();

    // Determine the token for git auth.
    let token = ctx.options.token.clone()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok());

    let clone_url = if let Some(ref tok) = token {
        format!(
            "https://{}@github.com/{}/{}.git",
            tok, tap.owner, tap.name
        )
    } else {
        tap_repo.clone()
    };

    run_cmd("git", &["clone", "--depth=1", &clone_url, &repo_path.to_string_lossy()], "homebrew: git clone")?;

    // Determine formula folder.
    let folder = hb_cfg.folder.clone().unwrap_or_else(|| "Formula".to_string());
    let formula_dir = repo_path.join(&folder);
    std::fs::create_dir_all(&formula_dir)
        .with_context(|| format!("homebrew: create formula dir {}", formula_dir.display()))?;

    let formula_path = formula_dir.join(format!("{}.rb", crate_name));
    std::fs::write(&formula_path, &formula)
        .with_context(|| format!("homebrew: write formula {}", formula_path.display()))?;

    eprintln!("[publish] wrote Homebrew formula: {}", formula_path.display());

    // git add + commit + push
    run_cmd_in(
        repo_path,
        "git",
        &["add", &formula_path.to_string_lossy()],
        "homebrew: git add",
    )?;
    run_cmd_in(
        repo_path,
        "git",
        &[
            "commit",
            "-m",
            &format!("chore: update {} formula to {}", crate_name, version),
        ],
        "homebrew: git commit",
    )?;
    run_cmd_in(repo_path, "git", &["push"], "homebrew: git push")?;

    eprintln!(
        "[publish] Homebrew tap {}/{} updated for '{}'",
        tap.owner, tap.name, crate_name
    );

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

fn run_cmd_in(dir: &std::path::Path, program: &str, args: &[&str], context_msg: &str) -> Result<()> {
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
mod tests {
    use super::*;

    #[test]
    fn test_generate_formula() {
        let formula = generate_formula(
            "cfgd",
            "1.0.0",
            &[("darwin-amd64", "https://example.com/cfgd-1.0.0-darwin-amd64.tar.gz", "sha256abc")],
            "Declarative config management",
            "MIT",
            "bin.install \"cfgd\"",
            "system \"#{bin}/cfgd\", \"--version\"",
        );
        assert!(formula.contains("class Cfgd < Formula"));
        assert!(formula.contains("version \"1.0.0\""));
        assert!(formula.contains("sha256abc"));
        assert!(formula.contains("bin.install"));
    }

    #[test]
    fn test_generate_formula_multiple_archives() {
        let formula = generate_formula(
            "my-tool",
            "2.0.0",
            &[
                ("darwin-amd64", "https://example.com/my-tool-2.0.0-darwin-amd64.tar.gz", "abc123"),
                ("linux-amd64", "https://example.com/my-tool-2.0.0-linux-amd64.tar.gz", "def456"),
            ],
            "A tool",
            "Apache-2.0",
            "bin.install \"my-tool\"",
            "system \"#{bin}/my-tool\", \"--version\"",
        );
        assert!(formula.contains("class MyTool < Formula"));
        assert!(formula.contains("on_macos"));
        assert!(formula.contains("on_linux"));
        assert!(formula.contains("abc123"));
        assert!(formula.contains("def456"));
    }

    #[test]
    fn test_generate_formula_class_name_hyphen() {
        let formula = generate_formula(
            "cfgd-core",
            "1.0.0",
            &[],
            "desc",
            "MIT",
            "bin.install \"cfgd-core\"",
            "system \"#{bin}/cfgd-core\", \"--version\"",
        );
        assert!(formula.contains("class CfgdCore < Formula"));
    }
}

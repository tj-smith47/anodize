use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context as _, Result};

use anodize_core::artifact::{Artifact, ArtifactKind};
use anodize_core::context::Context;
use anodize_core::stage::Stage;
use anodize_core::target::map_target;

// ---------------------------------------------------------------------------
// platform_to_arch
// ---------------------------------------------------------------------------

/// Extract the architecture component from a Docker platform string.
/// e.g. "linux/amd64" → "amd64", "linux/arm64" → "arm64"
pub fn platform_to_arch(platform: &str) -> &str {
    platform
        .rfind('/')
        .map(|idx| &platform[idx + 1..])
        .unwrap_or(platform)
}

// ---------------------------------------------------------------------------
// build_docker_command
// ---------------------------------------------------------------------------

/// Construct the `docker buildx build` command arguments.
///
/// * `staging_dir` – path to the directory that acts as the Docker build
///   context (already contains the Dockerfile and binaries).
/// * `platforms` – Docker platform strings, e.g. `["linux/amd64", "linux/arm64"]`.
/// * `tags` – fully-qualified image tags.
/// * `dry_run` – when `true`, uses `--load` (single-platform, local) instead
///   of `--push`.
pub fn build_docker_command(
    staging_dir: &str,
    platforms: &[&str],
    tags: &[&str],
    dry_run: bool,
) -> Vec<String> {
    let mut cmd: Vec<String> = vec![
        "docker".to_string(),
        "buildx".to_string(),
        "build".to_string(),
    ];

    // --platform=linux/amd64,linux/arm64
    let platform_str = platforms.join(",");
    cmd.push(format!("--platform={platform_str}"));

    // --tag <tag> for each image tag
    for tag in tags {
        cmd.push("--tag".to_string());
        cmd.push(tag.to_string());
    }

    // --push or --load
    if dry_run {
        cmd.push("--load".to_string());
    } else {
        cmd.push("--push".to_string());
    }

    // Build context directory (positional, last argument)
    cmd.push(staging_dir.to_string());

    cmd
}

// ---------------------------------------------------------------------------
// DockerStage
// ---------------------------------------------------------------------------

pub struct DockerStage;

impl Stage for DockerStage {
    fn name(&self) -> &str {
        "docker"
    }

    fn run(&self, ctx: &mut Context) -> Result<()> {
        let selected = ctx.options.selected_crates.clone();
        let dry_run = ctx.options.dry_run;
        let dist = ctx.config.dist.clone();

        // Collect crates that have docker config
        let crates: Vec<_> = ctx
            .config
            .crates
            .iter()
            .filter(|c| selected.is_empty() || selected.contains(&c.name))
            .filter(|c| c.docker.is_some())
            .cloned()
            .collect();

        if crates.is_empty() {
            return Ok(());
        }

        let mut new_artifacts: Vec<Artifact> = Vec::new();

        for krate in &crates {
            let docker_configs = krate.docker.as_ref().unwrap();

            for (idx, docker_cfg) in docker_configs.iter().enumerate() {
                // Determine platforms (default: linux/amd64 + linux/arm64)
                let platforms: Vec<String> = docker_cfg
                    .platforms
                    .clone()
                    .unwrap_or_else(|| {
                        vec![
                            "linux/amd64".to_string(),
                            "linux/arm64".to_string(),
                        ]
                    });

                // Build the staging directory path
                let staging_dir: PathBuf =
                    dist.join("docker").join(&krate.name).join(idx.to_string());

                if !dry_run {
                    fs::create_dir_all(&staging_dir).with_context(|| {
                        format!(
                            "docker: create staging dir {}",
                            staging_dir.display()
                        )
                    })?;
                }

                // ------------------------------------------------------------------
                // Stage binaries per platform/arch
                // ------------------------------------------------------------------
                for platform in &platforms {
                    let arch = platform_to_arch(platform);

                    let binaries_dir = staging_dir.join("binaries").join(arch);
                    if !dry_run {
                        fs::create_dir_all(&binaries_dir).with_context(|| {
                            format!(
                                "docker: create binaries dir {}",
                                binaries_dir.display()
                            )
                        })?;
                    }

                    // Determine which binary names this docker config cares about
                    let binary_filter = docker_cfg.binaries.as_ref();

                    // Find Binary artifacts whose target maps to this arch
                    let matching_binaries: Vec<_> = ctx
                        .artifacts
                        .by_kind_and_crate(ArtifactKind::Binary, &krate.name)
                        .into_iter()
                        .filter(|b| {
                            // Check the arch of the artifact's target triple matches
                            let artifact_arch = b
                                .target
                                .as_deref()
                                .map(|t| map_target(t).1)
                                .unwrap_or_default();
                            if artifact_arch != arch {
                                return false;
                            }
                            // Apply optional binary name filter
                            match binary_filter {
                                None => true,
                                Some(names) => {
                                    let bin_name = b
                                        .metadata
                                        .get("binary")
                                        .map(|s| s.as_str())
                                        .unwrap_or("");
                                    names.iter().any(|n| n == bin_name)
                                }
                            }
                        })
                        .collect();

                    for bin_artifact in matching_binaries {
                        let bin_name = bin_artifact
                            .metadata
                            .get("binary")
                            .map(|s| s.as_str())
                            .unwrap_or_else(|| {
                                bin_artifact
                                    .path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("binary")
                            });

                        let dest = binaries_dir.join(bin_name);

                        if dry_run {
                            eprintln!(
                                "[docker] (dry-run) would copy {} → {}",
                                bin_artifact.path.display(),
                                dest.display()
                            );
                        } else {
                            eprintln!(
                                "[docker] staging binary {} → {}",
                                bin_artifact.path.display(),
                                dest.display()
                            );
                            fs::copy(&bin_artifact.path, &dest).with_context(|| {
                                format!(
                                    "docker: copy binary {} to {}",
                                    bin_artifact.path.display(),
                                    dest.display()
                                )
                            })?;
                        }
                    }
                }

                // ------------------------------------------------------------------
                // Copy Dockerfile
                // ------------------------------------------------------------------
                let dockerfile_src = PathBuf::from(&docker_cfg.dockerfile);
                let dockerfile_dest = staging_dir.join("Dockerfile");

                if dry_run {
                    eprintln!(
                        "[docker] (dry-run) would copy Dockerfile {} → {}",
                        dockerfile_src.display(),
                        dockerfile_dest.display()
                    );
                } else {
                    eprintln!(
                        "[docker] copying Dockerfile {} → {}",
                        dockerfile_src.display(),
                        dockerfile_dest.display()
                    );
                    fs::copy(&dockerfile_src, &dockerfile_dest).with_context(|| {
                        format!(
                            "docker: copy Dockerfile from {} to {}",
                            dockerfile_src.display(),
                            dockerfile_dest.display()
                        )
                    })?;
                }

                // ------------------------------------------------------------------
                // Render image tag templates
                // ------------------------------------------------------------------
                let mut rendered_tags: Vec<String> = Vec::new();
                for tmpl in &docker_cfg.image_templates {
                    let tag = ctx.render_template(tmpl).with_context(|| {
                        format!(
                            "docker: render image_template '{}' for crate {}",
                            tmpl, krate.name
                        )
                    })?;
                    rendered_tags.push(tag);
                }

                // ------------------------------------------------------------------
                // Build and run the docker buildx command
                // ------------------------------------------------------------------
                let platform_refs: Vec<&str> =
                    platforms.iter().map(|s| s.as_str()).collect();
                let tag_refs: Vec<&str> =
                    rendered_tags.iter().map(|s| s.as_str()).collect();
                let staging_str = staging_dir.to_string_lossy().into_owned();

                let cmd_args = build_docker_command(
                    &staging_str,
                    &platform_refs,
                    &tag_refs,
                    dry_run,
                );

                if dry_run {
                    eprintln!("[docker] (dry-run) would run: {}", cmd_args.join(" "));
                } else {
                    eprintln!("[docker] running: {}", cmd_args.join(" "));

                    let status = Command::new(&cmd_args[0])
                        .args(&cmd_args[1..])
                        .status()
                        .with_context(|| {
                            format!(
                                "docker: execute buildx for crate {} index {}",
                                krate.name, idx
                            )
                        })?;

                    if !status.success() {
                        anyhow::bail!(
                            "docker buildx failed for crate {} index {}: exit code {:?}",
                            krate.name,
                            idx,
                            status.code()
                        );
                    }
                }

                // ------------------------------------------------------------------
                // Register DockerImage artifacts
                // ------------------------------------------------------------------
                for tag in &rendered_tags {
                    let mut meta = HashMap::new();
                    meta.insert("tag".to_string(), tag.clone());
                    meta.insert("platforms".to_string(), platforms.join(","));

                    new_artifacts.push(Artifact {
                        kind: ArtifactKind::DockerImage,
                        path: staging_dir.clone(),
                        target: None,
                        crate_name: krate.name.clone(),
                        metadata: meta,
                    });
                }
            }
        }

        for artifact in new_artifacts {
            ctx.artifacts.add(artifact);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_platform_to_arch() {
        assert_eq!(platform_to_arch("linux/amd64"), "amd64");
        assert_eq!(platform_to_arch("linux/arm64"), "arm64");
    }

    #[test]
    fn test_build_docker_command() {
        let cmd = build_docker_command(
            "/tmp/staging",
            &["linux/amd64", "linux/arm64"],
            &["ghcr.io/owner/app:v1.0.0", "ghcr.io/owner/app:latest"],
            false, // not dry-run, so push
        );
        assert!(cmd.contains(&"buildx".to_string()));
        assert!(cmd.contains(&"build".to_string()));
        assert!(cmd.contains(&"--platform=linux/amd64,linux/arm64".to_string()));
        assert!(cmd.contains(&"--push".to_string()));
        assert!(cmd.contains(&"--tag".to_string()));
    }

    #[test]
    fn test_build_docker_command_dry_run() {
        let cmd = build_docker_command(
            "/tmp/staging",
            &["linux/amd64"],
            &["ghcr.io/owner/app:v1.0.0"],
            true, // dry-run, so --load instead of --push
        );
        assert!(cmd.contains(&"--load".to_string()));
        assert!(!cmd.contains(&"--push".to_string()));
    }

    #[test]
    fn test_stage_skips_without_docker_config() {
        use anodize_core::config::Config;
        use anodize_core::context::{Context, ContextOptions};

        let config = Config::default();
        let mut ctx = Context::new(config, ContextOptions::default());
        let stage = DockerStage;
        assert!(stage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_platform_to_arch_no_slash() {
        // Fallback: no slash in string returns the whole string
        assert_eq!(platform_to_arch("amd64"), "amd64");
    }

    #[test]
    fn test_build_docker_command_structure() {
        let cmd = build_docker_command(
            "/tmp/ctx",
            &["linux/amd64"],
            &["my-image:latest"],
            false,
        );
        assert_eq!(cmd[0], "docker");
        assert_eq!(cmd[1], "buildx");
        assert_eq!(cmd[2], "build");
        // staging dir is the last argument
        assert_eq!(cmd.last().unwrap(), "/tmp/ctx");
    }

    #[test]
    fn test_build_docker_command_multiple_tags() {
        let cmd = build_docker_command(
            "/tmp/ctx",
            &["linux/amd64", "linux/arm64"],
            &["repo/img:v1.0.0", "repo/img:latest"],
            false,
        );
        // Both tags should appear after --tag flags
        let tag_positions: Vec<usize> = cmd
            .iter()
            .enumerate()
            .filter_map(|(i, t)| if t == "--tag" { Some(i) } else { None })
            .collect();
        assert_eq!(tag_positions.len(), 2);
        assert_eq!(cmd[tag_positions[0] + 1], "repo/img:v1.0.0");
        assert_eq!(cmd[tag_positions[1] + 1], "repo/img:latest");
    }

    #[test]
    fn test_docker_stage_dry_run_registers_artifacts() {
        use anodize_core::config::{Config, CrateConfig, DockerConfig};
        use anodize_core::context::{Context, ContextOptions};
        use anodize_core::artifact::{Artifact, ArtifactKind};

        let tmp = TempDir::new().unwrap();

        // Create fake binaries so the stage has something to pick up
        let amd64_bin = tmp.path().join("myapp-amd64");
        let arm64_bin = tmp.path().join("myapp-arm64");
        fs::write(&amd64_bin, b"fake amd64 binary").unwrap();
        fs::write(&arm64_bin, b"fake arm64 binary").unwrap();

        // Create a fake Dockerfile (not needed in dry-run, but still)
        let dockerfile = tmp.path().join("Dockerfile");
        fs::write(&dockerfile, b"FROM scratch\nCOPY . /\n").unwrap();

        let docker_cfg = DockerConfig {
            image_templates: vec![
                "ghcr.io/owner/myapp:{{ .Tag }}".to_string(),
                "ghcr.io/owner/myapp:latest".to_string(),
            ],
            dockerfile: dockerfile.to_string_lossy().into_owned(),
            platforms: Some(vec![
                "linux/amd64".to_string(),
                "linux/arm64".to_string(),
            ]),
            binaries: None,
            build_flag_templates: None,
        };

        let crate_cfg = CrateConfig {
            name: "myapp".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            docker: Some(vec![docker_cfg]),
            ..Default::default()
        };

        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.dist = tmp.path().join("dist");
        config.crates = vec![crate_cfg];

        let mut ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        );
        ctx.template_vars_mut().set("Version", "1.0.0");
        ctx.template_vars_mut().set("Tag", "v1.0.0");

        // Register binary artifacts
        let mut meta_amd64 = HashMap::new();
        meta_amd64.insert("binary".to_string(), "myapp".to_string());
        ctx.artifacts.add(Artifact {
            kind: ArtifactKind::Binary,
            path: amd64_bin.clone(),
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            crate_name: "myapp".to_string(),
            metadata: meta_amd64,
        });

        let mut meta_arm64 = HashMap::new();
        meta_arm64.insert("binary".to_string(), "myapp".to_string());
        ctx.artifacts.add(Artifact {
            kind: ArtifactKind::Binary,
            path: arm64_bin.clone(),
            target: Some("aarch64-unknown-linux-gnu".to_string()),
            crate_name: "myapp".to_string(),
            metadata: meta_arm64,
        });

        let stage = DockerStage;
        stage.run(&mut ctx).unwrap();

        // Should have registered 2 DockerImage artifacts (one per rendered tag)
        let docker_images = ctx.artifacts.by_kind(ArtifactKind::DockerImage);
        assert_eq!(docker_images.len(), 2);

        let tags: Vec<&str> = docker_images
            .iter()
            .map(|a| a.metadata.get("tag").unwrap().as_str())
            .collect();
        assert!(tags.contains(&"ghcr.io/owner/myapp:v1.0.0"));
        assert!(tags.contains(&"ghcr.io/owner/myapp:latest"));
    }
}

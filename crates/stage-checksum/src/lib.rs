use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{bail, Context as _, Result};
use sha2::{Digest, Sha256, Sha512};

use anodize_core::artifact::{Artifact, ArtifactKind};
use anodize_core::context::Context;
use anodize_core::stage::Stage;

// ---------------------------------------------------------------------------
// Hash helpers
// ---------------------------------------------------------------------------

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("sha256: open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)
            .with_context(|| format!("sha256: read {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn sha512_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)
        .with_context(|| format!("sha512: open {}", path.display()))?;
    let mut hasher = Sha512::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)
            .with_context(|| format!("sha512: read {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn hash_file(path: &Path, algorithm: &str) -> Result<String> {
    match algorithm {
        "sha256" => sha256_file(path),
        "sha512" => sha512_file(path),
        _ => bail!("unsupported checksum algorithm: {}", algorithm),
    }
}

pub fn format_checksum_line(hash: &str, filename: &str) -> String {
    format!("{}  {}", hash, filename)
}

// ---------------------------------------------------------------------------
// ChecksumStage
// ---------------------------------------------------------------------------

pub struct ChecksumStage;

impl Stage for ChecksumStage {
    fn name(&self) -> &str {
        "checksum"
    }

    fn run(&self, ctx: &mut Context) -> Result<()> {
        if ctx.is_dry_run() {
            eprintln!("[checksum] (dry-run) skipping checksum generation");
            return Ok(());
        }

        let selected = ctx.options.selected_crates.clone();
        let dist = ctx.config.dist.clone();

        // Global checksum algorithm/name-template defaults
        let global_algorithm = ctx
            .config
            .defaults
            .as_ref()
            .and_then(|d| d.checksum.as_ref())
            .and_then(|c| c.algorithm.clone())
            .unwrap_or_else(|| "sha256".to_string());
        let global_name_template = ctx
            .config
            .defaults
            .as_ref()
            .and_then(|d| d.checksum.as_ref())
            .and_then(|c| c.name_template.clone());

        // Collect crate configs up-front to avoid borrow conflicts
        let crates: Vec<_> = ctx
            .config
            .crates
            .iter()
            .filter(|c| selected.is_empty() || selected.contains(&c.name))
            .cloned()
            .collect();

        let mut new_artifacts: Vec<Artifact> = Vec::new();

        for crate_cfg in &crates {
            let crate_name = &crate_cfg.name;

            // Per-crate overrides
            let algorithm = crate_cfg
                .checksum
                .as_ref()
                .and_then(|c| c.algorithm.clone())
                .unwrap_or_else(|| global_algorithm.clone());

            let name_template = crate_cfg
                .checksum
                .as_ref()
                .and_then(|c| c.name_template.clone())
                .or_else(|| global_name_template.clone());

            // Gather Archive and LinuxPackage artifacts for this crate
            let mut source_artifacts: Vec<Artifact> = Vec::new();
            source_artifacts.extend(
                ctx.artifacts
                    .by_kind_and_crate(ArtifactKind::Archive, crate_name)
                    .into_iter()
                    .cloned(),
            );
            source_artifacts.extend(
                ctx.artifacts
                    .by_kind_and_crate(ArtifactKind::LinuxPackage, crate_name)
                    .into_iter()
                    .cloned(),
            );

            if source_artifacts.is_empty() {
                eprintln!("[checksum] no Archive/LinuxPackage artifacts for crate {crate_name}, skipping");
                continue;
            }

            // Extension for individual sidecar files
            let ext = &algorithm; // e.g. "sha256" or "sha512"

            let mut combined_lines: Vec<String> = Vec::new();

            for artifact in &source_artifacts {
                let hash = hash_file(&artifact.path, &algorithm)
                    .with_context(|| {
                        format!(
                            "checksum: hashing {} for crate {crate_name}",
                            artifact.path.display()
                        )
                    })?;

                let filename = artifact
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");

                // Build the sidecar path next to the artifact: e.g. myapp.tar.gz.sha256
                let sidecar_path = artifact.path.parent().unwrap_or(Path::new("."))
                    .join(format!("{}.{}", filename, ext));

                let line = format_checksum_line(&hash, filename);
                let mut sidecar_file = File::create(&sidecar_path)
                    .with_context(|| format!("checksum: create sidecar {}", sidecar_path.display()))?;
                writeln!(sidecar_file, "{}", line)
                    .with_context(|| format!("checksum: write sidecar {}", sidecar_path.display()))?;

                eprintln!(
                    "[checksum] {} -> {} ({})",
                    artifact.path.display(),
                    sidecar_path.display(),
                    algorithm
                );

                combined_lines.push(line);

                // Register sidecar as a Checksum artifact
                new_artifacts.push(Artifact {
                    kind: ArtifactKind::Checksum,
                    path: sidecar_path,
                    target: artifact.target.clone(),
                    crate_name: crate_name.clone(),
                    metadata: {
                        let mut m = HashMap::new();
                        m.insert("algorithm".to_string(), algorithm.clone());
                        m.insert("source".to_string(), artifact.path.to_string_lossy().into_owned());
                        m
                    },
                });
            }

            // Write combined checksums file
            let combined_filename = if let Some(tmpl) = &name_template {
                ctx.render_template(tmpl)
                    .with_context(|| format!("checksum: render name_template for {crate_name}"))?
            } else {
                format!("{}_checksums.{}", crate_name, ext)
            };

            let combined_path = dist.join(&combined_filename);
            std::fs::create_dir_all(&dist)
                .with_context(|| format!("checksum: create dist dir {}", dist.display()))?;

            let mut combined_file = File::create(&combined_path)
                .with_context(|| format!("checksum: create combined file {}", combined_path.display()))?;
            for line in &combined_lines {
                writeln!(combined_file, "{}", line)
                    .with_context(|| format!("checksum: write combined file {}", combined_path.display()))?;
            }

            eprintln!("[checksum] combined checksums -> {}", combined_path.display());

            new_artifacts.push(Artifact {
                kind: ArtifactKind::Checksum,
                path: combined_path,
                target: None,
                crate_name: crate_name.clone(),
                metadata: {
                    let mut m = HashMap::new();
                    m.insert("algorithm".to_string(), algorithm.clone());
                    m.insert("combined".to_string(), "true".to_string());
                    m
                },
            });
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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_sha256_file() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("test.txt");
        fs::write(&f, b"hello world").unwrap();
        let hash = sha256_file(&f).unwrap();
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_sha512_file() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("test.txt");
        fs::write(&f, b"hello world").unwrap();
        let hash = sha512_file(&f).unwrap();
        assert!(hash.starts_with("309ecc489c12d6eb4cc40f50c902f2b4"));
    }

    #[test]
    fn test_format_checksum_line() {
        let line = format_checksum_line("abcdef1234", "myfile.tar.gz");
        assert_eq!(line, "abcdef1234  myfile.tar.gz");
    }

    #[test]
    fn test_hash_file_dispatches() {
        let tmp = TempDir::new().unwrap();
        let f = tmp.path().join("test.txt");
        fs::write(&f, b"hello world").unwrap();
        let h256 = hash_file(&f, "sha256").unwrap();
        let h512 = hash_file(&f, "sha512").unwrap();
        assert_ne!(h256, h512);
        assert_eq!(h256.len(), 64); // SHA256 hex length
        assert_eq!(h512.len(), 128); // SHA512 hex length
    }

    #[test]
    fn test_checksum_stage_run() {
        use anodize_core::config::{Config, CrateConfig};
        use anodize_core::context::{Context, ContextOptions};

        let tmp = TempDir::new().unwrap();
        let dist = tmp.path().join("dist");
        fs::create_dir_all(&dist).unwrap();

        // Create a fake archive file
        let archive_path = dist.join("myapp-1.0.0-linux-amd64.tar.gz");
        fs::write(&archive_path, b"fake archive content").unwrap();

        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.dist = dist.clone();
        config.crates = vec![CrateConfig {
            name: "myapp".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            ..Default::default()
        }];

        let mut ctx = Context::new(config, ContextOptions::default());
        ctx.template_vars_mut().set("Version", "1.0.0");

        // Register an Archive artifact
        ctx.artifacts.add(Artifact {
            kind: ArtifactKind::Archive,
            path: archive_path.clone(),
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            crate_name: "myapp".to_string(),
            metadata: Default::default(),
        });

        let stage = ChecksumStage;
        stage.run(&mut ctx).unwrap();

        // Should have registered Checksum artifacts (sidecar + combined)
        let checksums = ctx.artifacts.by_kind(ArtifactKind::Checksum);
        assert_eq!(checksums.len(), 2);

        // Sidecar file should exist next to the archive
        let sidecar = dist.join("myapp-1.0.0-linux-amd64.tar.gz.sha256");
        assert!(sidecar.exists(), "sidecar file should exist");
        let sidecar_content = fs::read_to_string(&sidecar).unwrap();
        assert!(sidecar_content.contains("  myapp-1.0.0-linux-amd64.tar.gz"));

        // Combined file should exist in dist
        let combined = dist.join("myapp_checksums.sha256");
        assert!(combined.exists(), "combined checksums file should exist");
        let combined_content = fs::read_to_string(&combined).unwrap();
        assert!(combined_content.contains("  myapp-1.0.0-linux-amd64.tar.gz"));
    }

    #[test]
    fn test_checksum_stage_dry_run() {
        use anodize_core::config::{Config, CrateConfig};
        use anodize_core::context::{Context, ContextOptions};

        let tmp = TempDir::new().unwrap();
        let dist = tmp.path().join("dist");
        fs::create_dir_all(&dist).unwrap();

        let archive_path = dist.join("myapp.tar.gz");
        fs::write(&archive_path, b"fake").unwrap();

        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.dist = dist.clone();
        config.crates = vec![CrateConfig {
            name: "myapp".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            ..Default::default()
        }];

        let mut ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        );

        ctx.artifacts.add(Artifact {
            kind: ArtifactKind::Archive,
            path: archive_path.clone(),
            target: None,
            crate_name: "myapp".to_string(),
            metadata: Default::default(),
        });

        let stage = ChecksumStage;
        stage.run(&mut ctx).unwrap();

        // In dry-run, no Checksum artifacts are registered
        let checksums = ctx.artifacts.by_kind(ArtifactKind::Checksum);
        assert!(checksums.is_empty());
    }

    #[test]
    fn test_checksum_stage_sha512() {
        use anodize_core::config::{ChecksumConfig, Config, CrateConfig};
        use anodize_core::context::{Context, ContextOptions};

        let tmp = TempDir::new().unwrap();
        let dist = tmp.path().join("dist");
        fs::create_dir_all(&dist).unwrap();

        let archive_path = dist.join("myapp.tar.gz");
        fs::write(&archive_path, b"content").unwrap();

        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.dist = dist.clone();
        config.crates = vec![CrateConfig {
            name: "myapp".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            checksum: Some(ChecksumConfig {
                algorithm: Some("sha512".to_string()),
                name_template: None,
            }),
            ..Default::default()
        }];

        let mut ctx = Context::new(config, ContextOptions::default());

        ctx.artifacts.add(Artifact {
            kind: ArtifactKind::Archive,
            path: archive_path.clone(),
            target: None,
            crate_name: "myapp".to_string(),
            metadata: Default::default(),
        });

        let stage = ChecksumStage;
        stage.run(&mut ctx).unwrap();

        let sidecar = dist.join("myapp.tar.gz.sha512");
        assert!(sidecar.exists(), "sha512 sidecar should exist");
        let content = fs::read_to_string(&sidecar).unwrap();
        // SHA512 hex is 128 chars
        let hash_part = content.split_whitespace().next().unwrap_or("");
        assert_eq!(hash_part.len(), 128);

        let combined = dist.join("myapp_checksums.sha512");
        assert!(combined.exists());
    }

    #[test]
    fn test_checksum_stage_no_artifacts_skips() {
        use anodize_core::config::{Config, CrateConfig};
        use anodize_core::context::{Context, ContextOptions};

        let tmp = TempDir::new().unwrap();
        let dist = tmp.path().join("dist");

        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.dist = dist.clone();
        config.crates = vec![CrateConfig {
            name: "myapp".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            ..Default::default()
        }];

        let mut ctx = Context::new(config, ContextOptions::default());
        // No artifacts registered at all

        let stage = ChecksumStage;
        stage.run(&mut ctx).unwrap();

        let checksums = ctx.artifacts.by_kind(ArtifactKind::Checksum);
        assert!(checksums.is_empty());
    }
}

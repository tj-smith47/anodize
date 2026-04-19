use std::path::Path;

use anyhow::{Context as _, Result};

use anodizer_core::log::StageLogger;

/// Synchronize the `[package].version` field in a crate's Cargo.toml to the
/// given version string.  Skips writing if the version already matches.
/// In dry-run mode, logs what would happen without modifying the file.
pub fn sync_version(
    crate_path: &str,
    version: &str,
    dry_run: bool,
    log: &StageLogger,
) -> Result<()> {
    let cargo_toml_path = Path::new(crate_path).join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("failed to read {}", cargo_toml_path.display()))?;

    let mut doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse {}", cargo_toml_path.display()))?;

    // Read current version
    let current_version = doc
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if current_version == version {
        log.verbose(&format!(
            "version-sync: {} already at version {}",
            crate_path, version
        ));
        return Ok(());
    }

    if dry_run {
        log.status(&format!(
            "(dry-run) version-sync: would update {} from {} to {}",
            cargo_toml_path.display(),
            current_version,
            version
        ));
        return Ok(());
    }

    // Update the version
    doc["package"]["version"] = toml_edit::value(version);

    std::fs::write(&cargo_toml_path, doc.to_string())
        .with_context(|| format!("failed to write {}", cargo_toml_path.display()))?;

    log.status(&format!(
        "version-sync: updated {} from {} to {}",
        cargo_toml_path.display(),
        current_version,
        version
    ));

    Ok(())
}

/// Read the current `[package].version` from a crate's Cargo.toml.
pub fn read_cargo_version(crate_path: &str) -> Result<String> {
    let cargo_toml_path = Path::new(crate_path).join("Cargo.toml");
    let content = std::fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("failed to read {}", cargo_toml_path.display()))?;
    let doc = content
        .parse::<toml_edit::DocumentMut>()
        .with_context(|| format!("failed to parse {}", cargo_toml_path.display()))?;
    Ok(doc
        .get("package")
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string())
}

/// Recursively find all Cargo.toml files, excluding root and target dirs.
fn find_cargo_tomls(dir: &Path, root_toml: &Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default();
            if name == "target" || name == ".git" || name == "dist" {
                continue;
            }
            find_cargo_tomls(&path, root_toml, out);
        } else if path.file_name().map(|n| n == "Cargo.toml").unwrap_or(false) && path != root_toml
        {
            out.push(path);
        }
    }
}

/// Update intra-workspace dependency version specs for a given crate name.
///
/// Scans all Cargo.toml files under `workspace_root` for `[dependencies]`,
/// `[dev-dependencies]`, and `[build-dependencies]` entries that reference
/// `crate_name` with a `path` key (workspace-local deps). Updates their
/// `version` field to match the new version.
///
/// Returns a list of modified file paths (for staging).
pub fn sync_workspace_deps(
    workspace_root: &str,
    crate_name: &str,
    version: &str,
    dry_run: bool,
    log: &StageLogger,
) -> Result<Vec<String>> {
    let mut modified = Vec::new();
    let root = Path::new(workspace_root);

    // Find all Cargo.toml files under workspace root
    let mut cargo_tomls = Vec::new();
    find_cargo_tomls(root, &root.join("Cargo.toml"), &mut cargo_tomls);

    for path in &cargo_tomls {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let mut doc = match content.parse::<toml_edit::DocumentMut>() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let mut changed = false;
        for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
            let needs_update = doc
                .get(section)
                .and_then(|d| d.as_table())
                .and_then(|deps| deps.get(crate_name))
                .map(|dep| {
                    let has_path = dep.get("path").is_some();
                    let cur_ver = dep.get("version").and_then(|v| v.as_str());
                    has_path && cur_ver.is_some_and(|v| v != version)
                })
                .unwrap_or(false);

            if needs_update {
                let dep = &mut doc[section][crate_name];
                if let Some(tbl) = dep.as_inline_table_mut() {
                    tbl.insert("version", version.into());
                } else if let Some(tbl) = dep.as_table_mut() {
                    tbl.insert("version", toml_edit::Item::Value(version.into()));
                }
                changed = true;
            }
        }

        if changed {
            let path_str = path.to_string_lossy().to_string();
            if dry_run {
                log.status(&format!(
                    "(dry-run) version-sync: would update {} dep in {}",
                    crate_name,
                    path.display()
                ));
            } else {
                std::fs::write(path, doc.to_string())
                    .with_context(|| format!("failed to write {}", path.display()))?;
                log.status(&format!(
                    "version-sync: updated {} dep in {}",
                    crate_name,
                    path.display()
                ));
            }
            modified.push(path_str);
        }
    }

    Ok(modified)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anodizer_core::log::Verbosity;

    fn test_logger() -> StageLogger {
        StageLogger::new("build", Verbosity::Normal)
    }

    #[test]
    fn test_sync_version_updates_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        std::fs::write(
            &cargo_toml,
            r#"[package]
name = "my-crate"
version = "0.1.0"
edition = "2024"
"#,
        )
        .unwrap();

        sync_version(tmp.path().to_str().unwrap(), "1.2.3", false, &test_logger()).unwrap();

        let updated = std::fs::read_to_string(&cargo_toml).unwrap();
        let doc = updated.parse::<toml_edit::DocumentMut>().unwrap();
        assert_eq!(doc["package"]["version"].as_str().unwrap(), "1.2.3");
    }

    #[test]
    fn test_sync_version_skips_when_already_correct() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        let original = r#"[package]
name = "my-crate"
version = "1.2.3"
edition = "2024"
"#;
        std::fs::write(&cargo_toml, original).unwrap();

        sync_version(tmp.path().to_str().unwrap(), "1.2.3", false, &test_logger()).unwrap();

        // File should be unchanged
        let content = std::fs::read_to_string(&cargo_toml).unwrap();
        assert_eq!(content, original);
    }

    #[test]
    fn test_sync_version_dry_run_does_not_modify() {
        let tmp = tempfile::tempdir().unwrap();
        let cargo_toml = tmp.path().join("Cargo.toml");
        let original = r#"[package]
name = "my-crate"
version = "0.1.0"
edition = "2024"
"#;
        std::fs::write(&cargo_toml, original).unwrap();

        sync_version(tmp.path().to_str().unwrap(), "2.0.0", true, &test_logger()).unwrap();

        // File should be unchanged in dry-run mode
        let content = std::fs::read_to_string(&cargo_toml).unwrap();
        assert_eq!(content, original);
    }

    #[test]
    fn test_sync_version_missing_cargo_toml_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let result = sync_version(tmp.path().to_str().unwrap(), "1.0.0", false, &test_logger());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("failed to read"),
            "error should mention read failure, got: {err}"
        );
    }
}

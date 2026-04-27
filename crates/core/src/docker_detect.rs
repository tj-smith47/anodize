//! `docker` invocations needed by the CLI's `check` command.
//!
//! Centralised here so all `Command::new("docker")` shell-outs live inside
//! the module-boundaries allow-list (`.claude/rules/module-boundaries.md`).

use std::process::Command;

/// Run `docker buildx version` and return whether buildx is installed and
/// reachable. Used by `anodizer check` to surface a warning when docker
/// itself is present but buildx (the multi-platform builder) is missing.
pub fn buildx_available() -> bool {
    Command::new("docker")
        .args(["buildx", "version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

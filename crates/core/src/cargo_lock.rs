//! `cargo` invocations needed by the CLI's `tag` command.
//!
//! Centralised here so all `Command::new("cargo")` shell-outs live inside
//! the module-boundaries allow-list (`.claude/rules/module-boundaries.md`).
//! `tag.rs` previously called `Command::new("cargo")` from inside the CLI
//! crate — that was outside the allow-list and counted as a boundary
//! violation.

use std::path::Path;
use std::process::Command;

/// Run `cargo update --workspace`, optionally inside `dir`.
///
/// Returns `true` when the update succeeded, `false` otherwise. Callers
/// that care about the failure should check the return; the legacy
/// behaviour was to log a "cargo update failed; Cargo.lock may be stale"
/// warning and continue.
pub fn cargo_update_workspace(dir: Option<&Path>) -> bool {
    let mut cmd = Command::new("cargo");
    cmd.args(["update", "--workspace"]);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

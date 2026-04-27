//! Generic external-tool detection for the CLI's `healthcheck` command.
//!
//! Centralised here so the `Command::new(<tool>)` probe shell-outs live
//! inside the module-boundaries allow-list
//! (`.claude/rules/module-boundaries.md`). The CLI used to do these probes
//! inline; that put `Command::new` outside the allow-list and counted as a
//! boundary violation.

use std::process::Command;

/// Probe `<name> --version` and return whether the tool ran successfully.
/// stdout/stderr are silenced so a missing tool doesn't pollute the log.
pub fn tool_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run `<name> --version` and return the first stdout line trimmed.
/// Returns `None` if the tool is missing or exits non-zero.
pub fn tool_version(name: &str) -> Option<String> {
    let output = Command::new(name).arg("--version").output().ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Some(stdout.lines().next().unwrap_or("").trim().to_string())
    } else {
        None
    }
}

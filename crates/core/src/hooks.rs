use crate::config::HookEntry;
use crate::log::StageLogger;
use anyhow::{Context as _, Result};
use std::process::Command;

/// Execute a list of shell hook commands.
/// In dry-run mode, log but do not execute.
/// Supports both simple string hooks and structured hooks with cmd/dir/env/output.
pub fn run_hooks(
    hooks: &[HookEntry],
    label: &str,
    dry_run: bool,
    log: &StageLogger,
) -> Result<()> {
    for hook in hooks {
        let (cmd, dir, env, output_flag) = match hook {
            HookEntry::Simple(s) => (s.as_str(), None, None, None),
            HookEntry::Structured(h) => (
                h.cmd.as_str(),
                h.dir.as_deref(),
                h.env.as_ref(),
                h.output,
            ),
        };
        if dry_run {
            log.status(&format!("[dry-run] {} hook: {}", label, cmd));
        } else {
            log.status(&format!("running {} hook: {}", label, cmd));
            let mut command = Command::new("sh");
            command.arg("-c").arg(cmd);
            if let Some(d) = dir {
                command.current_dir(d);
            }
            if let Some(envs) = env {
                for (k, v) in envs {
                    command.env(k, v);
                }
            }
            let output = command
                .output()
                .with_context(|| format!("failed to spawn {} hook: {}", label, cmd))?;

            // When output flag is true, print the hook's stdout to the log
            if output_flag == Some(true) {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.is_empty() {
                    log.status(&format!("[hook output] {}", stdout.trim()));
                }
            }

            log.check_output(output, &format!("{} hook: {}", label, cmd))?;
        }
    }
    Ok(())
}

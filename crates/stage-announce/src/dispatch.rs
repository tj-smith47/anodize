use anodizer_core::context::Context;
use anyhow::Result;

/// Log and optionally execute a provider send action, respecting dry-run mode.
pub(crate) fn dispatch(
    ctx: &Context,
    provider: &str,
    log_line: &str,
    send: impl FnOnce() -> Result<()>,
) -> Result<()> {
    let log = ctx.logger("announce");
    if ctx.is_dry_run() {
        log.status(&format!("(dry-run) {provider}: {log_line}"));
    } else {
        log.status(&format!("{provider}: {log_line}"));
        send()?;
    }
    Ok(())
}

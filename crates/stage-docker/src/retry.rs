use std::time::Duration;

use anyhow::{Context as _, Result};

use anodizer_core::config::DockerRetryConfig;

// ---------------------------------------------------------------------------
// parse_duration_string
// ---------------------------------------------------------------------------

/// Parse a human-readable duration string into a [`Duration`].
///
/// Supported suffixes: `ms` (milliseconds), `s` (seconds), `m` (minutes).
/// Examples: `"500ms"`, `"1s"`, `"30s"`, `"2m"`.
///
/// Returns an error if the string is empty, has an unknown suffix, or contains
/// a non-numeric prefix.
pub fn parse_duration_string(s: &str) -> Result<Duration> {
    let s = s.trim();
    if s.is_empty() {
        anyhow::bail!("empty duration string");
    }

    if let Some(n) = s.strip_suffix("ms") {
        let millis: u64 = n
            .parse()
            .with_context(|| format!("invalid milliseconds in duration '{s}'"))?;
        Ok(Duration::from_millis(millis))
    } else if let Some(n) = s.strip_suffix('m') {
        let mins: u64 = n
            .parse()
            .with_context(|| format!("invalid minutes in duration '{s}'"))?;
        Ok(Duration::from_secs(mins * 60))
    } else if let Some(n) = s.strip_suffix('s') {
        let secs: u64 = n
            .parse()
            .with_context(|| format!("invalid seconds in duration '{s}'"))?;
        Ok(Duration::from_secs(secs))
    } else if let Ok(secs) = s.parse::<u64>() {
        // Bare number without suffix — treat as seconds (GoReleaser compat)
        Ok(Duration::from_secs(secs))
    } else {
        anyhow::bail!(
            "unknown duration suffix in '{s}'; expected ms, s, or m (e.g. '500ms', '1s', '2m')"
        );
    }
}

/// Resolve retry parameters from an optional [`DockerRetryConfig`].
///
/// Returns `(attempts, base_delay, max_delay)` with sensible defaults:
/// - attempts defaults to 10 (matching GoReleaser's default)
/// - delay defaults to 10s
/// - max_delay defaults to 5m (caps exponential backoff at a reasonable ceiling)
pub fn resolve_retry_params(
    retry: &Option<DockerRetryConfig>,
) -> Result<(u32, Duration, Option<Duration>)> {
    // Default max_delay of 5 minutes prevents exponential backoff from growing
    // to unreasonably long waits (e.g. 42 minutes at attempt 9 with 10s base).
    let default_max_delay = Some(Duration::from_secs(300));

    match retry {
        None => Ok((10, Duration::from_secs(10), default_max_delay)),
        Some(cfg) => {
            let attempts = cfg.attempts.unwrap_or(10);
            let base_delay = match &cfg.delay {
                Some(d) => parse_duration_string(d)?,
                None => Duration::from_secs(10),
            };
            let max_delay = match &cfg.max_delay {
                Some(d) => Some(parse_duration_string(d)?),
                None => default_max_delay,
            };
            Ok((attempts, base_delay, max_delay))
        }
    }
}

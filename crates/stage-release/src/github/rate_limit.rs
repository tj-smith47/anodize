//! Proactive GitHub API rate-limit checking.
//!
//! Before every PATCH/POST/PUT we hit `/rate_limit`; if the remaining quota
//! sits at or below `threshold` we sleep until reset. Mirrors GoReleaser's
//! `internal/client/github.go::checkRateLimit` (post-commit `60028b1`,
//! which made the wait iterative + ctx-cancellable). The Go version uses
//! `time.After(sleep)` inside `select { case <-ctx.Done() ... }`; the Rust
//! analog is `tokio::select!` against `tokio::signal::ctrl_c()` so a Ctrl-C
//! during a long rate-limit wait surfaces immediately rather than after the
//! reset window expires (which can be tens of minutes).
//!
//! Note: `check_github_search_rate_limit` was deleted alongside the Search
//! API author-lookup removal (commit 17315a5 / parity item P3). Re-introduce
//! it only if a future feature actually queries `/search/users` — otherwise
//! `#[allow(dead_code)]` would creep back in violation of
//! `.claude/rules/anti-patterns.md`.

use crate::release_log;

/// Proactively check the GitHub core rate limit before issuing a request.
///
/// If `remaining > threshold` returns immediately. Otherwise sleeps until the
/// reset epoch (plus a 1-second buffer), or until Ctrl-C interrupts the wait —
/// whichever is sooner.
///
/// Failures (transport, non-success response, malformed JSON) silently
/// degrade to "continue and hope for the best", matching the upstream
/// behaviour where `rateLimitChecker` logs and returns without aborting the
/// outer release flow.
pub(crate) async fn check_github_rate_limit(client: &reqwest::Client, token: &str, threshold: u64) {
    let url = "https://api.github.com/rate_limit";
    let resp = match client
        .get(url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", anodizer_core::http::USER_AGENT)
        .send()
        .await
    {
        Ok(r) => r,
        Err(_) => return, // Can't check — continue and hope for the best
    };

    if !resp.status().is_success() {
        return;
    }

    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => return,
    };

    let remaining = body
        .pointer("/resources/core/remaining")
        .and_then(|v| v.as_u64())
        .unwrap_or(u64::MAX);
    let reset_epoch = body
        .pointer("/resources/core/reset")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    if remaining > threshold {
        return;
    }

    // Sleep until reset + small buffer.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let sleep_secs = if reset_epoch > now {
        reset_epoch - now + 1
    } else {
        5 // Minimum 5 seconds if reset is in the past
    };
    release_log().status(&format!(
        "rate limit almost reached ({remaining} remaining), sleeping for {sleep_secs}s..."
    ));

    // Mirrors GoReleaser commit 60028b1 — use a single `select`-based wait
    // so Ctrl-C aborts the sleep instead of stalling the whole release for
    // up to an hour. We race the timer against `tokio::signal::ctrl_c()`;
    // there is no project-wide CancellationToken yet, so Ctrl-C is the
    // cancellation channel.
    let sleep = tokio::time::sleep(std::time::Duration::from_secs(sleep_secs));
    tokio::pin!(sleep);
    tokio::select! {
        _ = &mut sleep => {}
        _ = tokio::signal::ctrl_c() => {
            release_log().warn(
                "rate-limit wait interrupted by Ctrl-C; release will likely fail \
                 on the next API call",
            );
        }
    }
}

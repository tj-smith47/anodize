use std::ops::ControlFlow;
use std::process::Command;

use anyhow::{Context as _, Result};

use anodizer_core::PublisherGroup;
use anodizer_core::artifact::ArtifactKind;
use anodizer_core::context::Context;
use anodizer_core::retry::retry_sync;
use anodizer_core::stage::Stage;

use crate::command::{
    is_retriable_snap_push, resolve_effective_channels, snapcraft_upload_command,
};

// ---------------------------------------------------------------------------
// SnapcraftPublishStage — uploads previously built .snap artifacts
// ---------------------------------------------------------------------------

pub struct SnapcraftPublishStage;

impl Stage for SnapcraftPublishStage {
    fn name(&self) -> &str {
        "snapcraft-publish"
    }

    fn run(&self, ctx: &mut Context) -> Result<()> {
        let log = ctx.logger("snapcraft-publish");
        if ctx.skip_in_snapshot(&log, "snapcraft-publish") {
            return Ok(());
        }

        // Submitter-gate check: SnapcraftPublishStage is a Submitter-group
        // surface (irreversible snap-store upload — once a revision is
        // pushed there is no programmatic rollback). When the trait-based
        // dispatch in PublishStage flagged a required Assets/Manager
        // publisher failure, skip the snapcraft upload to avoid the
        // "released to one half-broken surface" failure mode.
        let gate_submitter = ctx.options.gate_submitter.unwrap_or(true);
        if gate_submitter
            && let Some(report) = ctx.publish_report()
            && (report.any_failed(PublisherGroup::Assets, true)
                || report.any_failed(PublisherGroup::Manager, true))
        {
            log.status("snapcraft-publish skipped via submitter-gate");
            return Ok(());
        }

        let selected = ctx.options.selected_crates.clone();
        let dry_run = ctx.options.dry_run;
        // Q8.1 — wrap snapcraft upload in retry. Mirrors GR upstream
        // commit eb944f9 (`isRetriableSnapPush`): 5xx Store responses
        // (500/502/503/504) are transient, every other failure is fatal.
        let retry_policy = ctx.retry_policy();

        // Collect crates that have snapcraft config with publish: true
        let crates: Vec<_> = ctx
            .config
            .crates
            .iter()
            .filter(|c| selected.is_empty() || selected.contains(&c.name))
            .filter(|c| c.snapcrafts.is_some())
            .cloned()
            .collect();

        if crates.is_empty() {
            return Ok(());
        }

        // Collect all snap artifacts that were built
        let snap_artifacts: Vec<_> = ctx
            .artifacts
            .by_kind(ArtifactKind::Snap)
            .into_iter()
            .cloned()
            .collect();

        if snap_artifacts.is_empty() {
            return Ok(());
        }

        for krate in &crates {
            let Some(snap_configs) = krate.snapcrafts.as_ref() else {
                continue;
            };

            for snap_cfg in snap_configs {
                // Only publish configs that opt in
                if !snap_cfg.publish.unwrap_or(false) {
                    continue;
                }
                // Skip configs marked skip:
                if let Some(ref d) = snap_cfg.skip {
                    let off = d
                        .try_evaluates_to_true(|tmpl| ctx.render_template(tmpl))
                        .with_context(|| {
                            format!(
                                "snapcraft: render publish.skip template for crate {}",
                                krate.name
                            )
                        })?;
                    if off {
                        continue;
                    }
                }

                // Find snap artifacts for this crate (optionally filtered by id)
                let matching: Vec<_> = snap_artifacts
                    .iter()
                    .filter(|a| a.crate_name == krate.name)
                    .filter(|a| {
                        if let Some(ref filter_id) = snap_cfg.id {
                            a.metadata
                                .get("id")
                                .map(|id| id == filter_id)
                                .unwrap_or(false)
                        } else {
                            true
                        }
                    })
                    .collect();

                for artifact in &matching {
                    let snap_path = artifact.path.to_string_lossy();

                    // GoReleaser renders each channel template through the
                    // template engine, filtering out empty results.
                    let rendered_channels: Option<Vec<String>> =
                        snap_cfg.channel_templates.as_ref().map(|templates| {
                            templates
                                .iter()
                                .filter_map(|tmpl| {
                                    ctx.render_template(tmpl).ok().filter(|s| !s.is_empty())
                                })
                                .collect()
                        });
                    // GoReleaser also renders grade through the template engine
                    let rendered_grade: Option<String> = snap_cfg
                        .grade
                        .as_deref()
                        .map(|g| ctx.render_template(g).unwrap_or_else(|_| g.to_string()));
                    let effective_channels = resolve_effective_channels(
                        rendered_channels.as_deref(),
                        rendered_grade.as_deref(),
                    );
                    let upload_args =
                        snapcraft_upload_command(&snap_path, effective_channels.as_deref());

                    if dry_run {
                        log.status(&format!("(dry-run) would run: {}", upload_args.join(" "),));
                        continue;
                    }

                    let max_attempts = retry_policy.max_attempts.max(1);
                    retry_sync(&retry_policy, |attempt| {
                        if attempt > 1 {
                            log.warn(&format!(
                                "snapcraft upload attempt {}/{} failed (5xx), retrying…",
                                attempt - 1,
                                max_attempts,
                            ));
                        }
                        log.status(&format!("running: {}", upload_args.join(" ")));
                        let upload_output = match Command::new(&upload_args[0])
                            .args(&upload_args[1..])
                            .output()
                        {
                            Ok(o) => o,
                            Err(e) => {
                                // Spawning snapcraft itself failed (binary missing,
                                // permission denied) — not a transient Store error.
                                return Err(ControlFlow::Break(anyhow::Error::from(e).context(
                                    format!(
                                        "execute snapcraft upload for crate {} snap {}",
                                        krate.name, snap_path
                                    ),
                                )));
                            }
                        };

                        if upload_output.status.success() {
                            return Ok(());
                        }

                        // Review-pending responses from the Snap Store should be
                        // warnings, not fatal errors — the snap was uploaded
                        // successfully but needs human review.
                        const REVIEW_PENDING_STRINGS: &[&str] = &[
                            "Waiting for previous upload",
                            "A human will soon review your snap",
                            "(NEEDS REVIEW)",
                        ];
                        let stderr = String::from_utf8_lossy(&upload_output.stderr);
                        let stdout = String::from_utf8_lossy(&upload_output.stdout);
                        let combined = format!("{}{}", stdout, stderr);
                        if REVIEW_PENDING_STRINGS.iter().any(|s| combined.contains(s)) {
                            log.warn(&format!("snap upload pending review: {}", combined.trim()));
                            return Ok(());
                        }

                        // Materialize the failure as an anyhow::Error via
                        // `log.check_output`, which preserves stderr/stdout for
                        // operators reading the log.
                        let err = match log.check_output(upload_output, "snapcraft upload") {
                            Ok(_) => return Ok(()),
                            Err(e) => e,
                        };
                        if is_retriable_snap_push(&combined) {
                            Err(ControlFlow::Continue(err))
                        } else {
                            // Auth failures, malformed snap, quota errors, etc.
                            // fast-fail without burning retry budget.
                            Err(ControlFlow::Break(err))
                        }
                    })?;
                }
            }
        }

        Ok(())
    }
}

use std::ops::ControlFlow;
use std::process::Command;

use anyhow::{Context as _, Result};

use anodizer_core::artifact::ArtifactKind;
use anodizer_core::context::Context;
use anodizer_core::retry::retry_sync;
use anodizer_core::stage::Stage;
use anodizer_core::{
    PublishEvidence, PublishReport, PublisherGroup, PublisherOutcome, PublisherResult, SkipReason,
};

use crate::command::{
    is_retriable_snap_push, resolve_effective_channels, snapcraft_upload_command,
};

// ---------------------------------------------------------------------------
// SnapcraftPublishStage — uploads previously built .snap artifacts
// ---------------------------------------------------------------------------
//
// `SnapcraftPublishStage` is the load-bearing snapcraft runner. Following
// the Task 15 (commit 026c854) BlobStage pattern, the stage writes its own
// `PublisherResult` directly into `ctx.publish_report` so the Submitter gate
// (and any downstream consumers, e.g. announce-gating, `--rollback-only
// --from-run`) observes outcomes uniformly. A parallel trait-based
// `SnapcraftPublisher` registration would fire `snapcraft upload` a second
// time per release — see
// `.claude/audits/2026-05-15-release-resilience-review.md` finding C3 and
// the doc comment on `stage-publish::registry::configured_publishers`.

pub struct SnapcraftPublishStage;

impl Stage for SnapcraftPublishStage {
    fn name(&self) -> &str {
        "snapcraft-publish"
    }

    fn run(&self, ctx: &mut Context) -> Result<()> {
        let log = ctx.logger("snapcraft-publish");
        if ctx.skip_in_snapshot(&log, "snapcraft-publish") {
            record_snapcraft_result(ctx, None, PublisherOutcome::Skipped(SkipReason::Snapshot));
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
            record_snapcraft_result(
                ctx,
                None,
                PublisherOutcome::Skipped(SkipReason::SubmitterGated),
            );
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
            // Mirrors BlobStage: nothing attempted, nothing to record.
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
            // Mirrors BlobStage: nothing attempted, nothing to record.
            return Ok(());
        }

        // Capture the resolved per-target snapshot BEFORE we start uploading
        // so a mid-stream failure still leaves the operator a manual
        // channel-management pointer for each snap we attempted to push.
        // The snapshot also feeds `PublishEvidence::extra.snapcraft_targets`
        // on success so `--rollback-only --from-run` consumers can decode
        // the recorded shape.
        let targets = collect_snapcraft_targets(ctx);

        // Track whether any real upload (non-dry-run) was attempted so we
        // can mirror BlobStage's "no work, no record" contract.
        let mut attempted_upload = false;
        let exec_result: Result<()> = (|| -> Result<()> {
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

                        attempted_upload = true;
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
                                    return Err(ControlFlow::Break(
                                        anyhow::Error::from(e).context(format!(
                                            "execute snapcraft upload for crate {} snap {}",
                                            krate.name, snap_path
                                        )),
                                    ));
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
                                log.warn(&format!(
                                    "snap upload pending review: {}",
                                    combined.trim()
                                ));
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
        })();

        if !attempted_upload {
            // Either every config was `publish: false`, or every snap
            // entry was disabled via `skip:`, or every run was dry-run.
            // Mirror BlobStage: nothing attempted, nothing to record.
            // Surface any catastrophic error (e.g. failed skip-template
            // render) from the closure as a stage error — these aren't
            // upload outcomes and don't go through PublisherResult.
            return exec_result;
        }

        let outcome = match &exec_result {
            Ok(()) => PublisherOutcome::Succeeded,
            Err(e) => PublisherOutcome::Failed(format!("{e:#}")),
        };
        let evidence = matches!(outcome, PublisherOutcome::Succeeded)
            .then(|| build_snapcraft_evidence(&targets));
        record_snapcraft_result(ctx, evidence, outcome);
        // Per-target upload errors are reported via PublisherResult; they
        // must NOT bail the pipeline because announce-gating and the
        // Submitter gate downstream depend on this stage returning Ok(()).
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PublisherResult recording — mirrors `stage-blob::run::record_blob_result`
// ---------------------------------------------------------------------------

/// Build the `PublishEvidence` recorded on a successful snapcraft run.
///
/// `primary_ref` points at the first uploaded package's snapcraft.io
/// listing; `extra.snapcraft_targets` carries the full per-target
/// snapshot used by `--rollback-only --from-run` to surface the
/// (package, channel) tuples an operator needs to address manually.
fn build_snapcraft_evidence(targets: &[SnapcraftTarget]) -> PublishEvidence {
    let mut evidence = PublishEvidence::new("snapcraft");
    if let Some(first) = targets.first() {
        evidence.primary_ref = Some(format!("https://snapcraft.io/{}", first.package_name));
    }
    evidence.extra = serde_json::json!({ "snapcraft_targets": targets });
    evidence
}

/// Append a `PublisherResult` for the snapcraft stage to
/// `ctx.publish_report`. Initializes the report when `None` (covers
/// `--publish` runs where the regular `PublishStage` was skipped).
/// Snapcraft is a Submitter-group publisher with `required = false`,
/// matching the trait-based classification before Bundle B2.
pub(crate) fn record_snapcraft_result(
    ctx: &mut Context,
    evidence: Option<PublishEvidence>,
    outcome: PublisherOutcome,
) {
    if ctx.publish_report.is_none() {
        ctx.publish_report = Some(PublishReport::default());
    }
    let report = ctx
        .publish_report
        .as_mut()
        .expect("publish_report initialized above");
    report.results.push(PublisherResult {
        name: "snapcraft".to_string(),
        group: PublisherGroup::Submitter,
        required: false,
        outcome,
        evidence,
    });
}

// ---------------------------------------------------------------------------
// SnapcraftTarget — per-target snapshot recorded in PublishEvidence::extra
// ---------------------------------------------------------------------------

/// Serialized shape of a recorded snapcraft publish. One entry per
/// `(crate, snapcraft config)` tuple whose `publish: true` opt-in
/// matched the [`SnapcraftPublishStage`] iteration order.
///
/// `package_name` is the resolved Snap Store package name (defaults to
/// the crate name when `snapcrafts[].name` is not overridden);
/// `channel` is the rendered channel template (or `None` when the
/// publish path falls back to the `grade`-derived default).
///
/// The serde shape is wire-stable: it is the value carried in
/// `PublishEvidence::extra.snapcraft_targets` and consumed by
/// `--rollback-only --from-run` to surface per-target channel-management
/// pointers. Byte-shape changes here are breaking for replay consumers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct SnapcraftTarget {
    /// The crate this publish covered.
    pub(crate) crate_name: String,
    /// Snap Store package name — defaults to the crate name when
    /// `snapcrafts[].name` is not set.
    pub(crate) package_name: String,
    /// First rendered channel template, or `None` when the publish
    /// path falls back to the `grade`-derived default.
    pub(crate) channel: Option<String>,
    /// Reserved for future use — snapcraft prints the revision number
    /// on upload but the existing publish stage does not capture
    /// stdout, so this stays `None` until we wire that capture.
    pub(crate) revision: Option<String>,
}

/// Walk `ctx.config.crates[].snapcrafts[]` and build one
/// [`SnapcraftTarget`] per opted-in snap config. Mirrors the publish
/// stage's filters: `selected_crates` gate, `publish: true` opt-in.
/// Skipped configs (`skip: true`) are excluded here too so the recorded
/// evidence matches what actually shipped.
pub(crate) fn collect_snapcraft_targets(ctx: &Context) -> Vec<SnapcraftTarget> {
    let selected = &ctx.options.selected_crates;
    let mut out: Vec<SnapcraftTarget> = Vec::new();
    for krate in &ctx.config.crates {
        if !selected.is_empty() && !selected.contains(&krate.name) {
            continue;
        }
        let Some(snap_configs) = krate.snapcrafts.as_ref() else {
            continue;
        };
        for snap_cfg in snap_configs {
            if !snap_cfg.publish.unwrap_or(false) {
                continue;
            }
            if let Some(ref d) = snap_cfg.skip {
                let off = d
                    .try_evaluates_to_true(|tmpl| ctx.render_template(tmpl))
                    .unwrap_or(false);
                if off {
                    continue;
                }
            }
            let package_name = snap_cfg.name.clone().unwrap_or_else(|| krate.name.clone());
            // GoReleaser parity: `channel_templates` is a Vec rendered
            // through the template engine. Capture the first non-empty
            // rendering — operators reading the warn line only need one
            // channel pointer to find the listing page.
            let channel = snap_cfg.channel_templates.as_ref().and_then(|tmpls| {
                tmpls
                    .iter()
                    .filter_map(|t| ctx.render_template(t).ok().filter(|s| !s.is_empty()))
                    .next()
            });
            out.push(SnapcraftTarget {
                crate_name: krate.name.clone(),
                package_name,
                channel,
                revision: None,
            });
        }
    }
    out
}

/// Decode the `snapcraft_targets` array from [`PublishEvidence::extra`].
///
/// Returns an empty Vec on any of: missing key, wrong shape, empty
/// array. Used by `--rollback-only --from-run` consumers and the
/// wire-stability tests below.
#[cfg(test)]
pub(crate) fn decode_snapcraft_targets(extra: &serde_json::Value) -> Vec<SnapcraftTarget> {
    extra
        .get("snapcraft_targets")
        .and_then(|v| serde_json::from_value::<Vec<SnapcraftTarget>>(v.clone()).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod publish_stage_tests {
    use super::*;
    use anodizer_core::config::{CrateConfig, SnapcraftConfig};
    use anodizer_core::test_helpers::TestContextBuilder;

    fn snap_crate(name: &str, package_name: Option<&str>, channel: Option<&str>) -> CrateConfig {
        CrateConfig {
            name: name.to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            snapcrafts: Some(vec![SnapcraftConfig {
                name: package_name.map(|s| s.to_string()),
                publish: Some(true),
                channel_templates: channel.map(|c| vec![c.to_string()]),
                ..Default::default()
            }]),
            ..Default::default()
        }
    }

    // ---------------------------------------------------------------
    // SnapcraftTarget wire-shape coverage (preserves replay contract)
    // ---------------------------------------------------------------

    #[test]
    fn snapcraft_target_extra_roundtrips() {
        let original = vec![
            SnapcraftTarget {
                crate_name: "demo".into(),
                package_name: "demo".into(),
                channel: Some("stable".into()),
                revision: None,
            },
            SnapcraftTarget {
                crate_name: "widget".into(),
                package_name: "widget-snap".into(),
                channel: None,
                revision: None,
            },
        ];
        let extra = serde_json::json!({ "snapcraft_targets": original.clone() });
        let decoded = decode_snapcraft_targets(&extra);
        assert_eq!(decoded, original);
    }

    #[test]
    fn build_snapcraft_evidence_pins_success_wire_shape() {
        // Success-path evidence is what `--rollback-only --from-run`
        // and any replay consumer reads back. Pin the three load-bearing
        // fields: publisher name, primary_ref pointing at the first
        // package's snapcraft.io listing, and the full per-target
        // snapshot in extra.snapcraft_targets.
        let targets = vec![
            SnapcraftTarget {
                crate_name: "demo".into(),
                package_name: "demo-snap".into(),
                channel: Some("stable".into()),
                revision: None,
            },
            SnapcraftTarget {
                crate_name: "widget".into(),
                package_name: "widget".into(),
                channel: None,
                revision: None,
            },
        ];
        let evidence = build_snapcraft_evidence(&targets);
        assert_eq!(evidence.publisher, "snapcraft");
        assert_eq!(
            evidence.primary_ref.as_deref(),
            Some("https://snapcraft.io/demo-snap")
        );
        let decoded = decode_snapcraft_targets(&evidence.extra);
        assert_eq!(decoded, targets);
    }

    #[test]
    fn build_snapcraft_evidence_handles_empty_targets() {
        // Edge case: success path with no resolved targets — should
        // still produce a well-formed evidence stub with no
        // primary_ref but an empty snapcraft_targets array.
        let evidence = build_snapcraft_evidence(&[]);
        assert_eq!(evidence.publisher, "snapcraft");
        assert!(evidence.primary_ref.is_none());
        assert_eq!(decode_snapcraft_targets(&evidence.extra), Vec::new());
    }

    #[test]
    fn snapcraft_target_extra_carries_no_secret_material() {
        // Defense-in-depth: serialize a target and assert no field
        // names that could leak SNAPCRAFT_LOGIN / token / auth material
        // are present.
        let t = SnapcraftTarget {
            crate_name: "demo".into(),
            package_name: "demo".into(),
            channel: Some("stable".into()),
            revision: None,
        };
        let s = serde_json::to_string(&t).expect("serialize");
        assert!(!s.contains("\"token\":"), "{s}");
        assert!(!s.contains("\"login\":"), "{s}");
        assert!(!s.contains("\"password\":"), "{s}");
        assert!(!s.contains("\"auth\":"), "{s}");
        assert!(!s.contains("\"api_key\":"), "{s}");
        assert!(!s.contains("\"snapcraft_login\":"), "{s}");
    }

    #[test]
    fn snapcraft_collect_targets_resolves_package_name_override() {
        let ctx = TestContextBuilder::new()
            .crates(vec![snap_crate("demo", Some("demo-snap"), Some("stable"))])
            .build();
        let targets = collect_snapcraft_targets(&ctx);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].crate_name, "demo");
        assert_eq!(targets[0].package_name, "demo-snap");
        assert_eq!(targets[0].channel.as_deref(), Some("stable"));
    }

    #[test]
    fn snapcraft_collect_targets_defaults_to_crate_name() {
        let ctx = TestContextBuilder::new()
            .crates(vec![snap_crate("demo", None, None)])
            .build();
        let targets = collect_snapcraft_targets(&ctx);
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].package_name, "demo");
        assert_eq!(targets[0].channel, None);
    }

    #[test]
    fn snapcraft_collect_targets_skips_non_publish_configs() {
        // A snapcrafts entry with `publish: false` (or unset) must NOT
        // surface as an evidence target — the publish path also skips
        // it, and recording a target we never pushed would mislead
        // operators reading any replay consumer.
        let krate = CrateConfig {
            name: "demo".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            snapcrafts: Some(vec![SnapcraftConfig {
                name: Some("demo".to_string()),
                publish: Some(false),
                ..Default::default()
            }]),
            ..Default::default()
        };
        let ctx = TestContextBuilder::new().crates(vec![krate]).build();
        let targets = collect_snapcraft_targets(&ctx);
        assert!(targets.is_empty(), "publish:false should be filtered out");
    }

    // ---------------------------------------------------------------
    // PublisherResult recording behavior
    // ---------------------------------------------------------------

    #[test]
    fn snapshot_mode_records_skipped_snapshot() {
        // Snapshot skip-gate fires BEFORE any other check — assert the
        // stage records `Skipped(Snapshot)` so the Submitter gate /
        // announce-gating / replay consumers all see a uniform entry.
        let ctx_builder = TestContextBuilder::new()
            .crates(vec![snap_crate("demo", None, Some("stable"))])
            .snapshot(true);
        let mut ctx = ctx_builder.build();
        let stage = SnapcraftPublishStage;
        stage.run(&mut ctx).expect("snapshot path returns Ok");

        let report = ctx.publish_report().expect("report initialized");
        let snap_results: Vec<&PublisherResult> = report
            .results
            .iter()
            .filter(|r| r.name == "snapcraft")
            .collect();
        assert_eq!(
            snap_results.len(),
            1,
            "exactly one snapcraft entry recorded"
        );
        let r = snap_results[0];
        assert_eq!(r.group, PublisherGroup::Submitter);
        assert!(!r.required);
        assert_eq!(r.outcome, PublisherOutcome::Skipped(SkipReason::Snapshot));
        assert!(r.evidence.is_none(), "snapshot skip records no evidence");
    }

    #[test]
    fn submitter_gate_records_skipped_gated() {
        // Pre-seed the publish report with a required Assets failure so
        // the Submitter-gate path fires. Assert the stage records
        // `Skipped(SubmitterGated)` (mirrors blob's contract — the gate
        // is observable in the report, not just silent).
        let mut ctx = TestContextBuilder::new()
            .crates(vec![snap_crate("demo", None, Some("stable"))])
            .build();
        // Seed a required Assets failure to trip the gate.
        let mut report = PublishReport::default();
        report.results.push(PublisherResult {
            name: "blob".to_string(),
            group: PublisherGroup::Assets,
            required: true,
            outcome: PublisherOutcome::Failed("simulated upload failure".to_string()),
            evidence: None,
        });
        ctx.publish_report = Some(report);

        let stage = SnapcraftPublishStage;
        stage.run(&mut ctx).expect("gate path returns Ok");

        let snap_results: Vec<&PublisherResult> = ctx
            .publish_report()
            .expect("report initialized")
            .results
            .iter()
            .filter(|r| r.name == "snapcraft")
            .collect();
        assert_eq!(snap_results.len(), 1);
        let r = snap_results[0];
        assert_eq!(r.group, PublisherGroup::Submitter);
        assert_eq!(
            r.outcome,
            PublisherOutcome::Skipped(SkipReason::SubmitterGated)
        );
        assert!(r.evidence.is_none(), "gated skip records no evidence");
    }

    #[test]
    fn no_configured_crates_records_nothing() {
        // BlobStage parity: when there is no work to attempt, do NOT
        // append a PublisherResult — the slot stays clean so downstream
        // consumers can distinguish "configured-and-skipped" from
        // "never asked to run".
        let mut ctx = TestContextBuilder::new().build();
        let stage = SnapcraftPublishStage;
        stage.run(&mut ctx).expect("no-crates path returns Ok");
        assert!(
            ctx.publish_report().is_none()
                || !ctx
                    .publish_report()
                    .unwrap()
                    .results
                    .iter()
                    .any(|r| r.name == "snapcraft"),
            "no snapcraft entry should be recorded when no crates are configured"
        );
    }

    #[test]
    fn dry_run_with_publishable_config_records_nothing() {
        // Mirrors BlobStage's dry-run contract: we log what WOULD run,
        // but no PublisherResult lands because no upload was attempted.
        use anodizer_core::artifact::{Artifact, ArtifactKind};
        use anodizer_core::context::ContextOptions;
        use std::collections::HashMap;
        use std::path::PathBuf;

        let crate_cfg = snap_crate("demo", Some("demo"), Some("edge"));
        let config = anodizer_core::config::Config {
            project_name: "demo".to_string(),
            crates: vec![crate_cfg],
            ..Default::default()
        };
        let mut ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        );
        ctx.template_vars_mut().set("Version", "1.0.0");
        ctx.artifacts.add(Artifact {
            kind: ArtifactKind::Snap,
            name: String::new(),
            path: PathBuf::from("/tmp/dist/demo_1.0.0_amd64.snap"),
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            crate_name: "demo".to_string(),
            metadata: HashMap::new(),
            size: None,
        });

        let stage = SnapcraftPublishStage;
        stage.run(&mut ctx).expect("dry-run returns Ok");
        let recorded_snap = ctx
            .publish_report()
            .map(|r| r.results.iter().any(|r| r.name == "snapcraft"))
            .unwrap_or(false);
        assert!(
            !recorded_snap,
            "dry-run path must NOT record a snapcraft PublisherResult"
        );
    }
}

//! Run summary JSON writer + end-of-pipeline status table.
//!
//! Emits a stable JSON document describing every publisher outcome,
//! gate flags, and the runtime/compile-time non-determinism allowlist.
//! Consumed by CI (parse `summary.json` to gate further jobs) and by
//! operators (status table prints to stderr so non-CI runs see the
//! per-publisher result at a glance).
//!
//! The schema is `deny_unknown_fields` so a downstream consumer
//! fails loudly if a future anodize version adds a field they don't
//! understand — preferable to silent shape drift.

use anodizer_core::context::Context;
use anodizer_core::publish_evidence::PublishEvidence;
use anodizer_core::publish_report::{PublisherGroup, PublisherOutcome, SkipReason};
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunSummary {
    pub schema_version: u32,
    pub anodize_version: String,
    pub tag: String,
    pub submitter_gated: bool,
    pub announce_gated: bool,
    pub results: Vec<RunSummaryResult>,
    pub determinism_allowlist: DeterminismAllowlist,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunSummaryResult {
    pub name: String,
    pub group: PublisherGroup,
    pub required: bool,
    /// Kebab-case status string per the spec's "Status Set".
    pub status: String,
    pub evidence: Option<PublishEvidence>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct DeterminismAllowlist {
    pub compile_time: Vec<DeterminismAllowlistEntry>,
    pub runtime: Vec<DeterminismAllowlistEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeterminismAllowlistEntry {
    pub artifact: String,
    pub reason: String,
}

impl RunSummary {
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    /// Build a `RunSummary` from `Context`. Pulls per-publisher results
    /// from `ctx.publish_report`, the runtime allowlist from
    /// `ctx.options.runtime_nondeterministic_allowlist`, and the tag
    /// from the `Tag` template var (which the pipeline sets in
    /// `Context::apply_git_info` so it's stable across stages).
    pub fn from_context(ctx: &Context) -> Self {
        let report = ctx.publish_report.as_ref();
        let results = report
            .map(|r| {
                r.results
                    .iter()
                    .map(|res| RunSummaryResult {
                        name: res.name.clone(),
                        group: res.group,
                        required: res.required,
                        status: outcome_to_status_string(&res.outcome),
                        evidence: res.evidence.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let tag = ctx.template_vars().get("Tag").cloned().unwrap_or_default();

        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            anodize_version: env!("CARGO_PKG_VERSION").to_string(),
            tag,
            submitter_gated: report.is_some_and(|r| r.submitter_gated),
            announce_gated: report.is_some_and(|r| r.announce_gated),
            results,
            determinism_allowlist: DeterminismAllowlist {
                // compile_time is populated by the determinism foundation
                // task; runtime flows in via ContextOptions today.
                compile_time: Vec::new(),
                runtime: ctx
                    .options
                    .runtime_nondeterministic_allowlist
                    .iter()
                    .map(|(name, reason)| DeterminismAllowlistEntry {
                        artifact: name.clone(),
                        reason: reason.clone(),
                    })
                    .collect(),
            },
        }
    }
}

/// Map a `PublisherOutcome` to the kebab-case status string defined
/// by the spec's "Status Set" enumeration. The `announce-gated`
/// status is intentionally not produced here because it is a
/// top-level flag on the report rather than a per-publisher outcome.
fn outcome_to_status_string(outcome: &PublisherOutcome) -> String {
    match outcome {
        PublisherOutcome::Succeeded => "succeeded".into(),
        PublisherOutcome::Skipped(reason) => match reason {
            SkipReason::SubmitterGated => "skipped-submitter-gated".into(),
            SkipReason::NotConfigured => "skipped-not-configured".into(),
            SkipReason::Snapshot => "skipped-snapshot".into(),
            SkipReason::DryRun => "skipped-dry-run".into(),
        },
        PublisherOutcome::Failed(_) => "failed".into(),
        PublisherOutcome::RolledBack => "rolled-back".into(),
        PublisherOutcome::RollbackFailed(_) => "rollback-failed".into(),
        PublisherOutcome::RollbackSkippedNoScope => "rollback-skipped-no-scope".into(),
        PublisherOutcome::PendingModeration => "pending-moderation".into(),
        PublisherOutcome::PendingValidation => "pending-validation".into(),
        PublisherOutcome::PublishedNoRollback => "published-no-rollback".into(),
    }
}

/// Write the run summary JSON to the given path. Creates parent
/// directories if missing. Pretty-prints so operators reading the
/// file directly do not have to pipe through `jq`.
pub fn write_summary_json(summary: &RunSummary, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "create parent directory {} for summary.json",
                parent.display()
            )
        })?;
    }
    let text = serde_json::to_string_pretty(summary).context("serialize run summary")?;
    fs::write(path, text).with_context(|| format!("write run summary to {}", path.display()))?;
    Ok(())
}

/// Pretty-print a per-publisher status table to the supplied writer
/// (typically `stderr`).
///
/// Output shape (operator-facing):
///
/// ```text
/// Publisher status:
///   name              group     required  status
///   github-release    Assets    true      succeeded
///   homebrew          Manager   false     failed
///   cargo             Submitter true      skipped-submitter-gated
///
/// Run flags: submitter_gated=false announce_gated=true
/// ```
pub fn print_status_table(
    summary: &RunSummary,
    out: &mut dyn std::io::Write,
) -> std::io::Result<()> {
    writeln!(out, "Publisher status:")?;
    writeln!(
        out,
        "  {:<20} {:<10} {:<10} status",
        "name", "group", "required",
    )?;
    for r in &summary.results {
        writeln!(
            out,
            "  {:<20} {:<10} {:<10} {}",
            r.name,
            format!("{:?}", r.group),
            r.required,
            r.status,
        )?;
    }
    writeln!(out)?;
    writeln!(
        out,
        "Run flags: submitter_gated={} announce_gated={}",
        summary.submitter_gated, summary.announce_gated,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anodizer_core::publish_evidence::PublishEvidence;
    use anodizer_core::publish_report::{PublishReport, PublisherResult};

    fn populated_summary() -> RunSummary {
        RunSummary {
            schema_version: RunSummary::CURRENT_SCHEMA_VERSION,
            anodize_version: "0.0.0-test".to_string(),
            tag: "v1.2.3".to_string(),
            submitter_gated: true,
            announce_gated: false,
            results: vec![
                RunSummaryResult {
                    name: "github-release".to_string(),
                    group: PublisherGroup::Assets,
                    required: true,
                    status: "succeeded".to_string(),
                    evidence: Some(PublishEvidence {
                        schema_version: 1,
                        publisher: "github-release".to_string(),
                        primary_ref: Some("https://example.com/r/v1".to_string()),
                        artifact_paths: vec![],
                        nondeterministic: None,
                        extra: serde_json::Value::Null,
                    }),
                },
                RunSummaryResult {
                    name: "cargo".to_string(),
                    group: PublisherGroup::Submitter,
                    required: true,
                    status: "skipped-submitter-gated".to_string(),
                    evidence: None,
                },
            ],
            determinism_allowlist: DeterminismAllowlist {
                compile_time: vec![],
                runtime: vec![DeterminismAllowlistEntry {
                    artifact: "anodizer.tar.gz".to_string(),
                    reason: "embedded build date".to_string(),
                }],
            },
        }
    }

    #[test]
    fn run_summary_schema_v1_roundtrips_through_json() {
        let s = populated_summary();
        let text = serde_json::to_string(&s).expect("serialize");
        let back: RunSummary = serde_json::from_str(&text).expect("deserialize");
        assert_eq!(back, s);
    }

    #[test]
    fn run_summary_rejects_unknown_fields() {
        let bad = r#"{
            "schema_version": 1,
            "anodize_version": "0.0.0-test",
            "tag": "v0.0.0",
            "submitter_gated": false,
            "announce_gated": false,
            "results": [],
            "determinism_allowlist": {"compile_time": [], "runtime": []},
            "future_field": "should reject"
        }"#;
        let parsed: std::result::Result<RunSummary, _> = serde_json::from_str(bad);
        assert!(parsed.is_err(), "unknown fields must be denied");
    }

    #[test]
    fn outcome_to_status_string_for_each_variant() {
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::Succeeded),
            "succeeded"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::Skipped(SkipReason::SubmitterGated)),
            "skipped-submitter-gated"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::Skipped(SkipReason::NotConfigured)),
            "skipped-not-configured"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::Skipped(SkipReason::Snapshot)),
            "skipped-snapshot"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::Skipped(SkipReason::DryRun)),
            "skipped-dry-run"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::Failed("boom".into())),
            "failed"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::RolledBack),
            "rolled-back"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::RollbackFailed("oops".into())),
            "rollback-failed"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::RollbackSkippedNoScope),
            "rollback-skipped-no-scope"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::PendingModeration),
            "pending-moderation"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::PendingValidation),
            "pending-validation"
        );
        assert_eq!(
            outcome_to_status_string(&PublisherOutcome::PublishedNoRollback),
            "published-no-rollback"
        );
    }

    #[test]
    fn from_context_captures_runtime_allowlist() {
        let mut ctx = anodizer_core::context::Context::test_fixture();
        ctx.options.runtime_nondeterministic_allowlist = vec![
            (
                "anodizer.tar.gz".to_string(),
                "embedded build date".to_string(),
            ),
            ("anodizer.deb".to_string(), "dpkg timestamp".to_string()),
        ];
        let s = RunSummary::from_context(&ctx);
        assert_eq!(s.determinism_allowlist.runtime.len(), 2);
        assert_eq!(
            s.determinism_allowlist.runtime[0].artifact,
            "anodizer.tar.gz"
        );
        assert_eq!(
            s.determinism_allowlist.runtime[0].reason,
            "embedded build date"
        );
        assert_eq!(s.determinism_allowlist.runtime[1].artifact, "anodizer.deb");
        assert!(s.determinism_allowlist.compile_time.is_empty());
    }

    #[test]
    fn from_context_pulls_tag_and_gate_flags() {
        let mut ctx = anodizer_core::context::Context::test_fixture();
        let report = PublishReport {
            submitter_gated: true,
            announce_gated: true,
            results: vec![PublisherResult {
                name: "homebrew".to_string(),
                group: PublisherGroup::Manager,
                required: false,
                outcome: PublisherOutcome::Failed("boom".to_string()),
                evidence: None,
            }],
        };
        ctx.publish_report = Some(report);
        let s = RunSummary::from_context(&ctx);
        assert_eq!(s.tag, "v0.0.0-test");
        assert!(s.submitter_gated);
        assert!(s.announce_gated);
        assert_eq!(s.results.len(), 1);
        assert_eq!(s.results[0].status, "failed");
        assert_eq!(s.results[0].name, "homebrew");
    }

    #[test]
    fn from_context_without_publish_report_yields_empty_results_and_default_flags() {
        let ctx = anodizer_core::context::Context::test_fixture();
        let s = RunSummary::from_context(&ctx);
        assert!(s.results.is_empty());
        assert!(!s.submitter_gated);
        assert!(!s.announce_gated);
    }

    #[test]
    fn write_summary_json_creates_parent_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let nested = tmp.path().join("a").join("b").join("summary.json");
        let s = populated_summary();
        write_summary_json(&s, &nested).expect("write");
        assert!(nested.exists());
        let back: RunSummary =
            serde_json::from_str(&fs::read_to_string(&nested).expect("read")).expect("parse");
        assert_eq!(back, s);
    }

    #[test]
    fn write_summary_json_overwrites_existing_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("summary.json");
        fs::write(&path, "stale").expect("seed");
        let s = populated_summary();
        write_summary_json(&s, &path).expect("write");
        let back: RunSummary =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(back, s);
    }

    #[test]
    fn print_status_table_renders_human_readable() {
        let s = populated_summary();
        let mut buf: Vec<u8> = Vec::new();
        print_status_table(&s, &mut buf).expect("print");
        let text = String::from_utf8(buf).expect("utf8");
        assert!(text.contains("Publisher status:"), "header missing: {text}");
        assert!(text.contains("succeeded"), "succeeded row missing: {text}");
        assert!(
            text.contains("skipped-submitter-gated"),
            "submitter-gated row missing: {text}"
        );
        assert!(
            text.contains("Run flags: submitter_gated=true announce_gated=false"),
            "run-flags line missing: {text}"
        );
    }

    #[test]
    fn summary_anodize_version_is_cargo_pkg_version() {
        let ctx = anodizer_core::context::Context::test_fixture();
        let s = RunSummary::from_context(&ctx);
        assert_eq!(s.anodize_version, env!("CARGO_PKG_VERSION"));
    }
}

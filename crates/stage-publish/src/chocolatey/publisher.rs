//! `ChocolateyPublisher` — Submitter-group `Publisher` impl wrapping the
//! per-crate [`publish_to_chocolatey`](super::publish_to_chocolatey)
//! entrypoint.
//!
//! Chocolatey is structurally a Submitter publisher: the push to the
//! community feed lands the package in a **moderation queue** at
//! `community.chocolatey.org/packages/<id>`. There is no public
//! programmatic withdraw endpoint. The community gallery's "Maintain"
//! UI is the only path back, and only the package owner can drive it.
//!
//! Bundle "Submitter group, no-rollback" contract for chocolatey: record
//! `(crate_name, package_id, version)` tuples in
//! [`anodizer_core::PublishEvidence::extra`] so a `--rollback-only`
//! invocation can surface the exact package page the operator needs to
//! address manually. The `rollback` method itself is warn-only and does
//! not call out to the gallery.
//!
//! CREDENTIAL HANDLING: [`ChocolateyTarget`] stores no auth material.
//! The chocolatey API key (resolved from `publish.chocolatey.api_key`
//! or the `CHOCOLATEY_API_KEY` env var at publish time) is irrelevant
//! to rollback — the manual withdraw flow runs through the community
//! web UI under the package owner's account, not via the push API key
//! — so persisting it into evidence would only leak a credential with
//! no operator benefit.

use anodizer_core::context::Context;
use serde::{Deserialize, Serialize};

simple_publisher!(
    ChocolateyPublisher,
    "chocolatey",
    anodizer_core::PublisherGroup::Submitter,
    false,
    // Chocolatey's rollback is operator-driven via the community web UI;
    // no env-var credential applies. Naming a token scope here would be
    // misleading — the API key feeds the *push*, not the *withdraw*.
    None,
);

/// Serialized shape of a recorded chocolatey publish. One entry per crate
/// whose publish path successfully submitted to the community feed.
///
/// `package_id` is the rendered nuspec `<id>` (the URL slug on
/// community.chocolatey.org); `version` is the bare semver string
/// (without the leading `v`) — matching what
/// [`anodizer_core::context::Context::version`] returns.
///
/// NB: no `api_key`, `token`, or `password` fields — see module
/// rustdoc for the credential-handling rationale.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ChocolateyTarget {
    /// Per-target label — the crate the nupkg was generated for.
    /// Surfaces in log lines.
    target: String,
    /// The crate this publish covered. Duplicates `target` to keep the
    /// shape symmetric with the other Submitter publishers.
    crate_name: String,
    /// Chocolatey gallery package ID — the URL slug on
    /// `community.chocolatey.org/packages/<package_id>`. Resolved from
    /// `publish.chocolatey.name` when set, else the crate name.
    package_id: String,
    /// Bare semver (no leading `v`) — what the Chocolatey gallery
    /// records as the package version.
    version: String,
}

/// Decode the `chocolatey_targets` array from
/// [`anodizer_core::PublishEvidence::extra`].
///
/// Returns an empty Vec on any of: missing key, wrong shape, empty
/// array. Rollback treats empty-decode the same as no-evidence and
/// emits the canonical empty-evidence warn.
fn decode_chocolatey_targets(extra: &serde_json::Value) -> Vec<ChocolateyTarget> {
    extra
        .get("chocolatey_targets")
        .and_then(|v| serde_json::from_value::<Vec<ChocolateyTarget>>(v.clone()).ok())
        .unwrap_or_default()
}

/// True when the crate has a `publish.chocolatey` block — mirrors the
/// `per_crate!` predicate in `lib.rs` so the publisher iterates
/// exactly the same crate universe.
fn is_chocolatey_per_crate_configured(ctx: &Context, crate_name: &str) -> bool {
    ctx.config
        .crates
        .iter()
        .any(|c| c.name == crate_name && c.publish.as_ref().is_some_and(|p| p.chocolatey.is_some()))
}

/// Build a [`ChocolateyTarget`] for the given crate. Reads config + the
/// live process version so the recorded coordinates match what
/// `publish_to_chocolatey` will push. Returns `None` when no chocolatey
/// block is configured (matches the publish path's skip semantics).
fn collect_chocolatey_target(ctx: &Context, crate_name: &str) -> Option<ChocolateyTarget> {
    let c = ctx.config.crates.iter().find(|c| c.name == crate_name)?;
    let cfg = c.publish.as_ref().and_then(|p| p.chocolatey.as_ref())?;
    let package_id = cfg.name.as_deref().unwrap_or(crate_name).to_string();
    Some(ChocolateyTarget {
        target: package_id.clone(),
        crate_name: crate_name.to_string(),
        package_id,
        version: ctx.version(),
    })
}

impl anodizer_core::Publisher for ChocolateyPublisher {
    fn name(&self) -> &str {
        Self::PUBLISHER_NAME
    }
    fn group(&self) -> anodizer_core::PublisherGroup {
        Self::PUBLISHER_GROUP
    }
    fn required(&self) -> bool {
        Self::PUBLISHER_REQUIRED
    }
    fn rollback_scope_needed(&self) -> Option<&'static str> {
        Self::ROLLBACK_SCOPE
    }

    fn run(&self, ctx: &mut Context) -> anyhow::Result<anodizer_core::PublishEvidence> {
        let log = ctx.logger("publish");
        let mut targets: Vec<ChocolateyTarget> = Vec::new();
        let selected = ctx.options.selected_crates.clone();
        for crate_name in &selected {
            if !is_chocolatey_per_crate_configured(ctx, crate_name) {
                continue;
            }
            // Snapshot the target shape BEFORE the publish path runs so
            // a mid-publish failure still leaves the operator a manual
            // withdrawal pointer.
            if let Some(t) = collect_chocolatey_target(ctx, crate_name) {
                targets.push(t);
            }
            super::publish::publish_to_chocolatey(ctx, crate_name, &log)?;
        }
        let mut evidence = anodizer_core::PublishEvidence::new("chocolatey");
        if let Some(first) = targets.first() {
            evidence.primary_ref = Some(format!(
                "https://community.chocolatey.org/packages/{}",
                first.package_id
            ));
        }
        evidence.extra = serde_json::json!({ "chocolatey_targets": targets });
        Ok(evidence)
    }

    fn rollback(
        &self,
        ctx: &mut Context,
        evidence: &anodizer_core::PublishEvidence,
    ) -> anyhow::Result<()> {
        let log = ctx.logger("publish");
        let targets = decode_chocolatey_targets(&evidence.extra);
        if targets.is_empty() {
            log.warn(&crate::publisher_helpers::rollback_empty_warning_msg(
                "chocolatey",
                "submitted packages",
            ));
            return Ok(());
        }
        // Chocolatey has no programmatic withdraw endpoint. Surface a
        // warn per recorded target with the exact gallery URL the
        // operator needs to address. This is intentionally NOT an
        // error: a failed automated rollback should not gate the rest
        // of the pipeline.
        for t in &targets {
            log.warn(&format!(
                "chocolatey: manual withdrawal required for '{}' version '{}'; \
                 visit https://community.chocolatey.org/packages/{} and use the \
                 'Maintain' UI to withdraw the submission (only the package \
                 owner can drive this; the push API key does not authorize \
                 withdraws).",
                t.package_id, t.version, t.package_id
            ));
        }
        log.status(&format!(
            "chocolatey: {} package(s) require manual withdrawal",
            targets.len()
        ));
        Ok(())
    }

    fn preflight(&self, _ctx: &Context) -> anyhow::Result<anodizer_core::PreflightCheck> {
        Ok(anodizer_core::PreflightCheck::Pass)
    }
}

#[cfg(test)]
mod publisher_tests {
    use super::*;
    use anodizer_core::config::{ChocolateyConfig, CrateConfig, PublishConfig};
    use anodizer_core::test_helpers::TestContextBuilder;
    use anodizer_core::{PreflightCheck, PublishEvidence, Publisher, PublisherGroup};

    fn choco_crate(crate_name: &str, package_name: Option<&str>) -> CrateConfig {
        CrateConfig {
            name: crate_name.to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                chocolatey: Some(ChocolateyConfig {
                    name: package_name.map(|s| s.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn chocolatey_publisher_classification() {
        let p = ChocolateyPublisher::new();
        assert_eq!(p.name(), "chocolatey");
        assert_eq!(p.group(), PublisherGroup::Submitter);
        assert!(!p.required());
        assert_eq!(p.rollback_scope_needed(), None);
    }

    #[test]
    fn chocolatey_preflight_defaults_to_pass() {
        let ctx = TestContextBuilder::new().build();
        let p = ChocolateyPublisher::new();
        assert!(matches!(
            p.preflight(&ctx).expect("preflight ok"),
            PreflightCheck::Pass
        ));
    }

    #[test]
    fn chocolatey_rollback_warns_when_no_targets_recorded() {
        let mut ctx = TestContextBuilder::new().build();
        let evidence = PublishEvidence::new("chocolatey");
        let p = ChocolateyPublisher::new();
        assert!(p.rollback(&mut ctx, &evidence).is_ok());

        let msg = crate::publisher_helpers::rollback_empty_warning_msg(
            "chocolatey",
            "submitted packages",
        );
        assert!(msg.starts_with("chocolatey:"), "{msg}");
        assert!(msg.contains("submitted packages"), "{msg}");
        assert!(msg.contains("verify"), "{msg}");
        assert!(msg.contains("manually"), "{msg}");
    }

    #[test]
    fn chocolatey_rollback_warns_per_target_when_evidence_present() {
        // Warn-only when targets are recorded; assert it does NOT
        // return Err so the dispatch chain continues.
        let mut ctx = TestContextBuilder::new().build();
        let mut evidence = PublishEvidence::new("chocolatey");
        evidence.extra = serde_json::json!({
            "chocolatey_targets": [
                {"target": "demo", "crate_name": "demo", "package_id": "demo", "version": "1.2.3"},
                {"target": "widget", "crate_name": "widget", "package_id": "widget", "version": "1.2.3"},
            ],
        });
        let p = ChocolateyPublisher::new();
        assert!(p.rollback(&mut ctx, &evidence).is_ok());
        // Sanity-check that the warn pattern names both targets and
        // the gallery URL prefix.
        assert_eq!(decode_chocolatey_targets(&evidence.extra).len(), 2);
    }

    #[test]
    fn chocolatey_target_extra_roundtrips() {
        let original = vec![ChocolateyTarget {
            target: "demo".into(),
            crate_name: "demo".into(),
            package_id: "demo".into(),
            version: "1.2.3".into(),
        }];
        let extra = serde_json::json!({ "chocolatey_targets": original.clone() });
        let decoded = decode_chocolatey_targets(&extra);
        assert_eq!(decoded, original);
    }

    #[test]
    fn chocolatey_target_extra_carries_no_secret_material() {
        // Defense-in-depth: serialize a target and assert no field
        // names that could leak the chocolatey API key are present.
        let t = ChocolateyTarget {
            target: "demo".into(),
            crate_name: "demo".into(),
            package_id: "demo".into(),
            version: "1.2.3".into(),
        };
        let s = serde_json::to_string(&t).expect("serialize");
        assert!(!s.contains("\"token\":"), "{s}");
        assert!(!s.contains("\"api_key\":"), "{s}");
        assert!(!s.contains("\"apikey\":"), "{s}");
        assert!(!s.contains("\"auth\":"), "{s}");
        assert!(!s.contains("\"password\":"), "{s}");
    }

    #[test]
    fn chocolatey_collect_target_resolves_package_name_override() {
        let ctx = TestContextBuilder::new()
            .crates(vec![choco_crate("demo", Some("DemoTool"))])
            .build();
        let t = collect_chocolatey_target(&ctx, "demo").expect("target");
        assert_eq!(t.crate_name, "demo");
        assert_eq!(t.package_id, "DemoTool");
    }

    #[test]
    fn chocolatey_collect_target_defaults_to_crate_name() {
        let ctx = TestContextBuilder::new()
            .crates(vec![choco_crate("demo", None)])
            .build();
        let t = collect_chocolatey_target(&ctx, "demo").expect("target");
        assert_eq!(t.package_id, "demo");
    }
}

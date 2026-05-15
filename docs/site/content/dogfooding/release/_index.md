+++
title = "Release pipeline"
description = "Release-pipeline config keys: release.*, changelog.*, announce.*, blobs[], publishers[]."
weight = 30
template = "section.html"
+++

# Release pipeline

The keys that drive the release itself: GitHub/GitLab/Gitea release surface,
changelog generation, announcers, cloud uploads, and custom publishers.

## Release and changelog

| Key | Status | Notes |
|---|---|---|
| `release.github` | ✅ Verified | [anodizer releases](https://github.com/tj-smith47/anodizer/releases). Header/footer/draft/prerelease/make_latest all exercised |
| `release.metadata` | ✅ Verified | [v0.1.1 metadata.json](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/metadata.json) · [artifacts.json](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/artifacts.json) |
| `release.name_template` / `tag_template` | ✅ Verified | [cfgd `.anodizer.yaml`](https://github.com/tj-smith47/cfgd/blob/master/.anodizer.yaml) (`tag_template: "core-v{{ Version }}"` / `"v{{ Version }}"` / `"operator-v{{ Version }}"` / `"csi-v{{ Version }}"`) |
| `release.header` / `footer` | ✅ Verified | [cfgd v0.3.5 release body](https://github.com/tj-smith47/cfgd/releases/tag/v0.3.5) (`What's new` header + `Released with anodizer` footer) |
| `changelog.groups` | ✅ Verified | "Features" / "Bug Fixes" / "Others" sections in the [v0.1.1 release body](https://github.com/tj-smith47/anodizer/releases/tag/v0.1.1) |
| `changelog.filters.include` / `exclude` | ✅ Verified | [anodizer `.anodizer.yaml`](https://github.com/tj-smith47/anodizer/blob/master/.anodizer.yaml) (`changelog.filters.include` / `exclude` patterns) |
| `changelog.use: git` | ✅ Verified | [`crates/stage-changelog/src/lib.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-changelog/src/lib.rs) (`use: git` branch) |
| `changelog.use: github-native` | ✅ Verified | [`crates/stage-changelog/src/lib.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-changelog/src/lib.rs) (`use: github-native` branch) |
| `changelog.use: github` | ✅ Verified | [`crates/stage-changelog/src/lib.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-changelog/src/lib.rs) (`use: github` branch) |
| `changelog.use: gitlab` / `gitea` | ✅ Verified | [`crates/stage-changelog/src/lib.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-changelog/src/lib.rs) (`gitlab` / `gitea` branches) |
| `changelog.use: ai` | 🤝 Help wanted | anthropic / openai / ollama implemented; no live release uses it |
| `release.gitlab` | 🤝 Help wanted | We dogfood on GitHub only |
| `release.gitea` | 🤝 Help wanted | We dogfood on GitHub only |
| `milestones[]` | ✅ Verified | [`crates/core/src/config/milestone.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/core/src/config/milestone.rs) |

## Release resilience

These features shipped 2026-05-14 in response to the anodize **v0.2.0 cascade
failure** ([Run 25754442852](https://github.com/tj-smith47/anodizer/actions/runs/25754442852)
and four siblings on 2026-05-12, all failing in the publish stage). They form
three-group publisher dispatch (Assets, Manager, Submitter), a Submitter gate
that aborts the Submitter group when required Assets or Manager publishers
fail, opt-in rollback per-publisher, and a `--rollback-only --from-run=<id>`
replay path. Several behaviors have unit/integration test coverage today
(rows marked `✅ Verified (tests)` below); rows that need a live v0.2.x+ tag
to exercise the codepath stay `🤝 Help wanted`.

| Key | Status | Notes |
|---|---|---|
| Three-group Submitter gate (default-on) | ✅ Verified (tests) | [`crates/stage-publish/src/dispatch.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/dispatch.rs) — Assets / Manager / Submitter groups wired and gate verified via `dispatch::tests::submitter_gate_skips_submitter_when_required_manager_fails`; first v0.2.x release confirms end-to-end |
| `--no-gate-submitter` override | ✅ Verified (tests) | [`crates/stage-publish/src/dispatch.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/dispatch.rs) (`dispatch::tests::no_gate_submitter_runs_submitter_anyway`) + CLI parse (`crates/cli/src/main.rs::tests::release_parses_no_gate_submitter_flag`); awaits a live release that flips the override |
| `--rollback=best-effort` | ✅ Verified (tests) | [`crates/stage-publish/src/rollback.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/rollback.rs) — per-publisher rollback path verified via `preflight::tests::preflight_bails_when_required_publisher_missing_scope_and_rollback_best_effort` + CLI parse (`crates/cli/src/main.rs::tests::release_parses_rollback_best_effort`); no live release has rolled back yet |
| `--rollback-only --from-run=<id>` replay | ✅ Verified (tests) | [`crates/stage-publish/src/rollback_only.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/rollback_only.rs) — idempotency + read/dispatch covered by `rollback_only::tests::rollback_only_reads_report_and_dispatches`, `rollback_only_second_invocation_is_noop_for_already_rolled_back_entries`, plus path-traversal guard at the binary surface (`crates/cli/tests/integration.rs::release_from_run_rejects_path_traversal_at_binary_surface`, `release_rollback_only_invokes_replay_from_disk`) and `crates/stage-publish/tests/run_report_persistence.rs::publish_stage_writes_report_and_rollback_only_can_read_it` |
| `--fail-fast` | ✅ Verified | [anodize `.anodizer.yaml`](https://github.com/tj-smith47/anodizer/blob/master/.anodizer.yaml) plus [release command wiring](https://github.com/tj-smith47/anodizer/blob/master/crates/cli/src/commands/release/mod.rs) (`fail_fast` opts) - pre-resilience-work flag, exercised in v0.1.x runs |
| `--summary-json=<path>` audit-trail | ✅ Verified (tests) | [`crates/stage-publish/src/run_summary.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/run_summary.rs) — JSON schema v1 round-trip + writer covered by `run_summary::tests::run_summary_schema_v1_roundtrips_through_json`, `run_summary_rejects_unknown_fields`, `write_summary_json_creates_parent_dir`; CLI parse at `crates/cli/src/main.rs::tests::release_parses_summary_json`; no v0.2.x release has emitted one yet |
| `announce.gate_on` config (default `required_publishers`) | ✅ Verified (tests) | [`crates/stage-announce/src/run.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-announce/src/run.rs) — gate evaluation covered by `run::tests::announce_skips_when_gate_required_and_required_failure`, `announce_skips_when_gate_all_and_any_failure`, `announce_gate_serializes_as_snake_case`; no post-merge release has gated an announce on publisher health |
| Preflight rollback-scope checks | ✅ Verified (tests) | [`crates/stage-publish/src/preflight.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/preflight.rs) — warn / strict-block / best-effort-bail paths covered by `preflight::tests::preflight_warns_on_missing_rollback_scope`, `preflight_blocks_on_missing_rollback_scope_when_strict`, `preflight_bails_when_required_publisher_missing_scope_and_rollback_best_effort`; no live release has tripped them |
| AnnounceStage emit-summary-on-skip | ✅ Verified (tests) | [`crates/stage-announce/src/run.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-announce/src/run.rs) — emit-on-gate-skip + emit-when-stage-not-called covered by `run::tests::emit_summary_writes_when_gate_would_fire`, `emit_summary_writes_when_announce_stage_was_not_called`, `emit_summary_writes_summary_when_path_set`, plus integration test `crates/cli/tests/integration.rs::test_release_skip_announce_still_writes_summary_json`; no v0.2.x release has skipped an announce yet |
| BlobStage writes to `ctx.publish_report` | ✅ Verified (tests) | [`crates/stage-blob/src/run.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-blob/src/run.rs) — publish-report append + `BlobConfig.required` gating covered by `tests::blob_stage_appends_succeeded_to_publish_report`, `blob_stage_appends_failed_to_publish_report`, `blob_stage_initializes_publish_report_when_none`, `record_blob_result_required_false_by_default`, `record_blob_result_failed_required_blob_trips_assets_required_gate`; awaits a release with cloud blob credentials configured |

## Build determinism

Byte-stability contract plus a `check determinism` harness, an operator
`--allow-nondeterministic <name>=<reason>` escape, and a release-body
"Non-deterministic exemptions:" block that lists any waived artifacts. Merged
2026-05-14; rows fill in as v0.2.x+ releases exercise each surface.

| Key | Status | Notes |
|---|---|---|
| `anodize check determinism --runs=N` harness | 🤝 Help wanted | [`crates/cli/src/commands/check/determinism.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/cli/src/commands/check/determinism.rs) - N-run harness wired; not yet invoked from a tagged release run |
| `anodize check config` (post-restructure) | 🤝 Help wanted | [`crates/cli/src/commands/check/config.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/cli/src/commands/check/config.rs) - post-restructure config validator; no release has exercised the new surface yet |
| `--allow-nondeterministic <name>=<reason>` | 🤝 Help wanted | Operator escape parsed and threaded through the build stage; no live release has waived an artifact yet |
| "Non-deterministic exemptions:" block in release body | 🤝 Help wanted | [`crates/stage-release/src/release_body.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-release/src/release_body.rs) - emitter wired; release body fragment unused until an exemption ships |

## Announcers

13 channels implemented. Two are exercised by live cfgd releases; the
others have full test coverage but no live secrets configured.

| Key | Status | Notes |
|---|---|---|
| `announce.webhook` | ✅ Verified | [cfgd `.anodizer.yaml`](https://github.com/tj-smith47/cfgd/blob/master/.anodizer.yaml) (`announce.webhook.endpoint_url: https://tj.jarvispro.io/webhooks/anodizer`) |
| `announce.smtp` | ✅ Verified | [cfgd `.anodizer.yaml`](https://github.com/tj-smith47/cfgd/blob/master/.anodizer.yaml) (`announce.smtp.host: smtp.gmail.com`) |
| `announce.discord` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.slack` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.telegram` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.teams` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.mattermost` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.reddit` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.twitter` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.mastodon` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.bluesky` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.linkedin` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.opencollective` | 🤝 Help wanted | No live workflow has the secrets |
| `announce.discourse` | 🤝 Help wanted | No live workflow has the secrets |

## Blob and artifactory uploads

| Key | Status | Notes |
|---|---|---|
| `blobs[]` (S3 / GCS / Azure) | 🤝 Help wanted | `object_store` SDK wired. No release configures cloud credentials |
| `artifactories[]` | 🤝 Help wanted | Target, mode, TLS, headers wired; no live deployment |
| `uploads[]` | 🤝 Help wanted | Generic HTTP upload wired; no live deployment |
| `furies[]` | 🤝 Help wanted | Implemented; no live credentials |
| `cloudsmiths[]` | 🤝 Help wanted | Wired in [cfgd's config](https://github.com/tj-smith47/cfgd/blob/master/.anodizer.yaml) with a live `CLOUDSMITH_TOKEN`; uploads currently fail at HTTP layer so no package has landed in the `jarvispro/cfgd` repo. Awaiting endpoint debug |

## Custom publishers

| Key | Status | Notes |
|---|---|---|
| `publishers[]` | ✅ Verified | [`crates/cli/src/commands/publisher.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/cli/src/commands/publisher.rs) (custom command per artifact) |

## MCP registry

Publishes an MCP server manifest to `https://registry.modelcontextprotocol.io`.

Implementation is feature-complete with unit-test coverage of every branch
(auth providers, retry policy, dry-run, repository inference). Dogfooding is
**held**: anodizer's own `.anodizer.yaml` declares `packages[0].registry_type: oci`
with `identifier: ghcr.io/tj-smith47/anodizer`, but the project ships binary
archives and does not yet have a `dockers:` block. Publishing this manifest
today would point MCP clients at a 404, so the `mcp:` block is marked
`skip: true` until anodizer ships an OCI image (via a `dockers:` block) or
the package is pivoted to a registry type the project actually distributes.

| Key | Status | Notes |
|---|---|---|
| `mcp.name` | 🤝 Help wanted | Wired in [anodizer `.anodizer.yaml`](https://github.com/tj-smith47/anodizer/blob/master/.anodizer.yaml); blocked on `dockers:` block / first live publish |
| `mcp.packages[]` | 🤝 Help wanted | Wired in [anodizer `.anodizer.yaml`](https://github.com/tj-smith47/anodizer/blob/master/.anodizer.yaml) (`packages[].registry_type: oci`); blocked on `dockers:` block / first live publish |
| `mcp.auth.type: none` | 🤝 Help wanted | [`crates/stage-publish/src/mcp/auth.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/auth.rs) (None branch) — unit-tested; blocked on `dockers:` block before dogfood publish |
| `mcp.auth.type: github` | 🤝 Help wanted | [`crates/stage-publish/src/mcp/auth.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/auth.rs) (PAT exchange branch) — unit-tested; blocked on `dockers:` block before dogfood publish |
| `mcp.auth.type: github-oidc` | 🤝 Help wanted | [`crates/stage-publish/src/mcp/auth.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/auth.rs) (OIDC id-token branch); blocked on `dockers:` block before dogfood publish |
| `mcp.repository` | 🤝 Help wanted | [`crates/stage-publish/src/mcp/manifest.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/manifest.rs) — unit-tested; blocked on `dockers:` block before dogfood publish |
| `mcp.skip` (tera, accepts `disable:` alias) | 🤝 Help wanted | [`crates/stage-publish/src/mcp/mod.rs`](https://github.com/tj-smith47/anodizer/blob/master/crates/stage-publish/src/mcp/mod.rs) — unit-tested; blocked on `dockers:` block before dogfood publish |

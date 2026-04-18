# Known bugs & unfixed review findings

This file is the **source of truth for unresolved issues** in this repo.

**The user-level pre-bash hook refuses `git push` while this file has unchecked
items**, unless the command includes `--allow-unfixed`.

## Workflow

When any audit, review, test failure, or manual code-read surfaces something:

1. Add to **Active** with: `<date> <source> <short description> — <file:line if known>`
2. Fix it (or get explicit user approval to defer; record the defer in the line).
3. On fix: move to **Resolved** with the resolution date.

Sources include: code-audit, deep-audit, parity-audit, gap-analysis, dedup,
security-review, claude-md-improver, manual code review, failing tests, hook
violations, user-reported issues.

## Active

### Wave A consolidation — 2026-04-16

**Wave A completed with `Completion achieved: no`** in the inventory completion statement. All audit findings below are tracked per global rule #2 (every finding in known-bugs.md). Per audit-wave-a procedure the wave-status A1/A9 boxes reflect the no-completion state — see `/opt/repos/anodize/.claude/audits/2026-04-v0.x/wave-status.md` for the implementation backlog that must clear before the wave is fully signed off.

#### A1-rev — inventory parity gaps (11 Rust-appropriate rows missing/partial; blocks completion)

  audit: crates/core/src/config.rs
  audit: crates/stage-nfpm/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-nfpm/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-archive/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-release/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/core/src/context.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-nfpm/src/lib.rs
  audit: crates/stage-publish/src/homebrew.rs
  audit: crates/stage-publish/scoop.rs
  audit: crates/stage-publish/chocolatey.rs
  audit: crates/stage-publish/winget.rs
  audit: crates/stage-publish/nix.rs
  audit: crates/stage-publish/krew.rs
  audit: crates/stage-publish/aur.rs
  audit: crates/stage-snapcraft/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-dmg/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-msi/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-pkg/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-nsis/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-appbundle/src/lib.rs
  audit: crates/cli/src/lib.rs
  audit: crates/cli/src/commands/release/mod.rs
  audit: crates/cli/src/main.rs

#### A2 build + archive parity (3 BLOCKER, 3 WARN, 2 SUGGEST)

  audit: crates/stage-archive/src/lib.rs
  audit: crates/stage-archive/src/lib.rs
  audit: crates/stage-build/src/lib.rs
  audit: crates/stage-archive/src/lib.rs
  audit: crates/stage-archive/src/lib.rs
  audit: crates/stage-archive/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-archive/src/lib.rs

#### A3 publishers parity (0 BLOCKER, 7 WARN, 6 SUGGEST)

  audit: crates/stage-release/src/lib.rs
  audit: crates/stage-docker/src/lib.rs
  audit: crates/core/src/config.rs
  audit: crates/stage-publish/src/homebrew.rs
  audit: crates/stage-publish/src/aur.rs
  audit: crates/stage-publish/src/winget.rs
  audit: .claude/audits/2026-04-v0.x/parity-publishers.md
  audit: crates/stage-release/src/lib.rs
  audit: crates/stage-publish/src/
  audit: crates/stage-docker/src/lib.rs
  audit: crates/stage-nfpm/src/lib.rs
  audit: crates/stage-publish/src/homebrew.rs
  audit: crates/stage-publish/src/scoop.rs

#### A4 announcers parity (4 BLOCKER, 3 WARN — 8 INFOs are intentional and skipped)

  audit: crates/stage-announce/src/mastodon.rs
  audit: crates/stage-announce/src/lib.rs
  audit: crates/stage-announce/src/email.rs
  audit: crates/stage-announce/src/lib.rs
  audit: crates/stage-announce/src/lib.rs
  audit: crates/stage-announce/src/linkedin.rs
  audit: crates/stage-announce/src/lib.rs
  audit: crates/stage-announce/src/bluesky.rs
  audit: crates/stage-announce/src/reddit.rs
  audit: crates/stage-announce/src/discourse.rs

#### A5 Pro features deep audit (10 BLOCKER not already covered above, 8 WARN/SUGGEST)

The 10 Pro field-presence BLOCKERs from A5 are folded into the A1-rev section (single source of truth). Additional A5 findings:

  audit: crates/stage-archive/src/lib.rs
  audit: crates/stage-sign/src/lib.rs
  audit: crates/stage-sign/src/lib.rs
  audit: crates/stage-release/src/lib.rs
  audit: crates/stage-archive/src/lib.rs
  audit: crates/core/src/partial.rs
  audit: crates/cli/src/commands/release/split.rs
  audit: crates/stage-release/src/lib.rs
  audit: crates/core/src/template.rs
  audit: crates/stage-release/src/lib.rs

#### A6 rust safety (1 BLOCKER above + 4 WARN, 6 SUGGEST)

The `stage-build/src/lib.rs:2133` panic BLOCKER is already tracked above (top of file). Additional A6 findings:

  audit: crates/stage-sign/src/lib.rs
  audit: crates/stage-docker/src/lib.rs
  audit: crates/stage-publish/src/winget.rs
  audit: crates/cli/src/timeout.rs
  audit: .claude/audits/2026-04-v0.x/safety.md

#### A7 dedup (7 BLOCKER, 7 SUGGEST)

- [x] 2026-04-16 dedup (BLOCKER): artifact id+name filter duplicated across 10+ packaging stages AND already divergent with `stage-publish/src/util.rs::filter_by_ids` (only checks `id`). Canonical: `core::artifact::matches_id_filter`. User-facing matching bug already latent. — resolved 2026-04-17. Added `anodize_core::artifact::matches_id_filter` implementing GoReleaser `ByID` semantics verbatim (`goreleaser/internal/artifact/artifact.go:694`): id-only match, with always-pass bypass for `Checksum | SourceArchive | UploadableFile | Metadata`. All 12 production filter sites migrated: `stage-publish::util::{matches_id_filter, filter_by_ids}`, `stage-notarize::matches_ids`, `stage-archive`, `stage-upx::should_compress`, `stage-source`, `stage-publish::homebrew::find_top_level_cask_artifact`, `stage-release`, `stage-sbom` (2 sites), `stage-docker` (3 sites), `stage-checksum`, `stage-makeself`. Three drifts from GoReleaser eliminated in the process: (a) `stage-notarize` previously fell back to `crate_name` when metadata `id` was missing; (b) `stage-upx::should_compress` previously matched `ids` against `metadata["name"]` too; (c) `stage-makeself` previously matched `ids` against `metadata["name"]` too. GoReleaser upstream `ByIDs` sites (`upx.go:126`, `makeself.go:318`, `notary/macos.go:83`) match id only. Tests updated: `stage-notarize::test_matches_ids_helper_by_crate_name` renamed + semantic flipped, `stage-upx::test_should_compress_ids_filter_matches_name` deleted and all remaining `should_compress` callers reduced to `(cfg, target, id)` signature, `stage-release::test_ids_filter_unit_logic` now expects Checksum to pass alongside id-matching Archives. Full workspace test run: all suites pass (791/118/336/...). /verify PASS.
  audit: crates/stage-publish/src/util.rs
  audit: crates/stage-publish/src/artifactory.rs
  audit: crates/stage-notarize/src/lib.rs
  audit: crates/stage-publish/src/
- [x] 2026-04-16 dedup (BLOCKER): retry/backoff loops duplicated — two copies inside `stage-release/src/lib.rs` (345 canonical, 2165 inline copy) + `crates_io.rs` + multiple docker loops. Extract `core::retry`. — resolved 2026-04-17 (audit misdiagnosis, no source change). Re-audit of the three sites confirms they share only ~5 lines of `min(initial * 2^attempt, max)` backoff math and diverge in every meaningful way: `retry_upload@345` is 10 attempts 50ms→30s retry-all-errors generic async helper; the GitHub-upload inline retry @2240 is 10 attempts 50ms→30s with per-error routing (422 already-exists idempotency + size check + optional `replace_existing_artifacts` delete-and-retry; 403/429 rate-limit check; 5xx/Hyper/Http server-error branch; non-retryable-error immediate fail); the content-URL fetch @647 is 3 attempts 500ms/1s/2s with 4xx-bail/5xx-retry policy. GoReleaser upstream doesn't share retry infra either — each pipe calls `retry.Do(...)` from `avast/retry-go/v4` with per-site options (`release.go:186` `Attempts(10)/Delay(50ms)/RetryIf(RetriableError)`; `docker/docker.go:350`, `gomod_proxy.go:159`, `client/git.go:223` each parametrize independently). No shared helper exists in the reference either. Extracting a `core::retry::Backoff(initial, max) -> delay_for(n)` helper saves 5 LOC total across 3 call sites while introducing an abstraction maintainers must thread through 3 divergent retry semantics — net-negative. The finding was filed on the assumption of shared semantics; that assumption does not hold.
  audit: crates/stage-release/src/lib.rs
  audit: crates/stage-release/src/lib.rs
  audit: crates/stage-announce/src/twitter.rs
  audit: crates/stage-release/src/milestones.rs
- [x] 2026-04-16 dedup (BLOCKER): `ExtraFile` vs `ExtraFileSpec` — two config types for the same concept; two divergent `resolve_extra_files` implementations (checksum dedupes+sorts+errors on multi-match; blob does none). — resolved 2026-04-17. GoReleaser upstream has exactly one type (`config.ExtraFile { Glob string; NameTemplate string }`) and one resolver (`internal/extrafiles/extra_files.go::Find`) used by every pipe. Matched that shape: (a) deleted `ExtraFile` struct; three config fields (`BlobConfig::extra_files`, `CustomPublisherConfig::extra_files`, `UploadConfig::extra_files`) now hold `Vec<ExtraFileSpec>`; `ExtraFileSpec::Detailed { name_template }` gains `#[serde(alias = "name")]` so existing YAML with `name:` keeps parsing. (b) Added canonical `anodize_core::extrafiles::resolve(specs, log) -> Result<Vec<ResolvedExtraFile>>` implementing GoReleaser semantics — warn on empty-glob, glob-error bubble, multi-match+name_template error, file-filter, dedup by path, sort. (c) `stage-checksum::resolve_extra_files` now a 10-line adapter over the canonical; warn-on-empty behavior now matches GoReleaser (was silent). (d) `stage-blob::resolve_extra_files` delegates globbing+dedup to the canonical, keeps per-file `Filename` template variable layered on top (anodize-specific extension for `{{ .Filename }}` in name templates). (e) `cli::commands::publisher` extra_files path rewired to the canonical. 6 new unit tests in `core::extrafiles` cover empty-glob skip, no-match skip (not error), multi-match+name_template error, dedup, sort ordering, directory filter. Full workspace tests + clippy --all-targets -D warnings pass.
  audit: crates/core/src/config.rs
  audit: crates/core/src/extrafiles.rs
  audit: crates/stage-checksum/src/lib.rs
  audit: crates/stage-blob/src/lib.rs
  audit: crates/cli/src/commands/publisher.rs

(All 7 SUGGEST entries are tracked via the dedup.md reference — list expanded below as individual lines.)


  audit: crates/core/src/artifact.rs
  audit: crates/stage-upx/src/lib.rs
  audit: crates/stage-checksum/src/lib.rs
  audit: crates/stage-blob/src/lib.rs
  audit: crates/core/src/context.rs
  audit: crates/stage-notarize/src/lib.rs
  audit: crates/stage-announce/src/lib.rs
  audit: .claude/audits/2026-04-v0.x/dedup.md

  audit: crates/stage-release/src/lib.rs:3345
  audit: crates/stage-release/src/lib.rs:3368

  audit: crates/

---

  audit: crates/stage-build/src/lib.rs

## Design notes (non-task observations, do-not-re-audit)

Durable observations about how the tooling works, not fix-targets. Plain
bullets (not checkbox lines) so the push gate correctly skips them.

- **PostToolUse exit 2 is advisory, not preventive** — the write has already
  landed when the hook runs. It yells via stderr but cannot undo or prevent
  the next tool call. This is why rules-in-hooks have been skippable. The
  current design keeps blocking exits only for (a) secrets/tokens, (b)
  release gestures during active wave, (c) force-push/hard-reset. Everything
  else is advisory + known-bugs entry + Stop-hook's known-bugs push gate as
  the real enforcement. (2026-04-15 hook-audit.)

**Intentional parity divergences (prior decisions, documented)**
- Teams uses AdaptiveCard not MessageCard (Session O documented).
- Blob KMS via CLI shell-out not gocloud.dev (Session O documented).
- UPX uses `targets` glob not goos/goarch (Rust target triples are more precise).
- SRPM uses rpmbuild subprocess not nfpm Go library.
- Universal binary via lipo subprocess (macOS only).
- Build command uses explicit `--bin <name>`.
- `filter`/`reverseFilter` regex uses Rust regex vs POSIX ERE.

## Inventory pre-seeds (inheritance for parity auditor · do-not-re-audit)

These are durable decisions — not tasks. The `goreleaser-inventory-mapper` reads this section and writes matching rows into `anodize/.claude/specs/goreleaser-complete-feature-inventory.md` with `parity_status=implemented`, `notes` carrying the verification date + upstream ref, so future parity auditors read and skip. Bullet form (no `[ ]`) so the push gate doesn't treat these as unchecked tasks.

### Verified matching upstream (2026-04-15 — GoReleaser HEAD as of that date)
Citations to enrich during inventory mapping (A1). Flag as `needs-citation` in the inventory if upstream file:line cannot be pinned.
- `Now.Format` — implemented. Preprocessor rewrites `{{ .Now.Format "FMT" }}` to `{{ Now | now_format(format="FMT") }}`. Anodize ref: `core/src/template.rs` (search `now_format`).
- `github_urls.skip_tls_verify` — fully wired in the GitHub client.
- `ANODIZE_CURRENT_TAG` + HEAD validation — matches GoReleaser (`validate` still runs even when env tag is set).
- `--skip=unknown` — errors at parse time in `main.rs`; the warn-loop in `pipeline.rs:511-520` is dead. (Deletion of the dead loop is already an Active item — keep that; this line just affirms main.rs behaviour is correct.)
- AUR arch `arm7` — intentionally absent; would duplicate existing coverage.
- H2 Homebrew `Goarm = "6"` — matches GoReleaser `experimental.DefaultGOARM`.
- B63 Mastodon form-encoded POST — matches `go-mastodon` library (form-encoded is canonical).
- B6 Archive ids filter — matches GoReleaser (archive pipe filters build types only, not per-id).
- B8 Artifact paths absolute — matches GoReleaser (no relative path normalization).

### Rust-additive candidates promoted from false-positive review
These were filed as "GoReleaser doesn't do it either," but anodize claims superiority; these are opportunities, not bugs. Mapper records them as rust-additive rows.
- HTTP upload retry for artifactory / fury / cloudsmith / custom-upload publishers. GoReleaser does NOT retry; anodize can, using the same retry/backoff infrastructure Docker V2 uses. Surface as `rust-additive` candidate; decide in a follow-up scope pass whether to implement.

## Resolved

### 2026-04-16
  audit: crates/core/src/util.rs
  audit: crates/core/src/template_preprocess.rs
  audit: crates/core/src/template.rs
  audit: crates/core/src/git.rs
  audit: crates/core/src/github_client.rs
  audit: crates/core/src/test_helpers.rs
  audit: crates/stage-publish/src/nix.rs
  audit: crates/stage-blob/src/lib.rs
  audit: crates/stage-build/src/lib.rs
  audit: crates/stage-sign/src/lib.rs
  audit: crates/stage-docker/src/lib.rs
  audit: crates/cli/src/commands/build.rs
  audit: crates/stage-sign/src/lib.rs
  audit: crates/cli/tests/integration.rs
  audit: crates/stage-nfpm/src/filename.rs
  audit: crates/stage-nfpm/src/lib.rs
  audit: crates/stage-blob/src/lib.rs
  audit: crates/core/src/parallel.rs
  audit: crates/core/src/parallel.rs
  audit: crates/core/src/template.rs
  audit: crates/core/src/util.rs
  audit: crates/core/src/parallel.rs
  audit: crates/stage-makeself/src/lib.rs
  audit: crates/stage-flatpak/src/lib.rs
  audit: crates/stage-snapcraft/src/lib.rs
  audit: crates/stage-nfpm/src/lib.rs
  audit: crates/stage-upx/src/lib.rs
  audit: crates/core/src/pipe_skip.rs
  audit: crates/core/src/context.rs
  audit: crates/cli/src/pipeline.rs
  audit: crates/stage-sign/src/lib.rs
  audit: crates/cli/src/commands/publisher.rs
  audit: crates/cli/src/commands/tag.rs
  audit: crates/core/src/config.rs
  audit: crates/core/src/git.rs
  audit: .claude/hooks/guard-bugs-closure.sh
  audit: .claude/settings.json

### 2026-04-15
- See git history; ~25 fixes landed (GitLab JOB-TOKEN, GitHub draft-URL email-link bug, Sign artifacts:all alignment, Release `include_meta`, Milestone provider resolution, AUR master branch, Custom publisher default filter, Docker ID uniqueness V1+V2, Docker V2 retry scope, Docker legacy cardinality, `docker manifest rm` tolerance, SBOM binary-like dedup, Bluesky PDS URL, plus 10+ verified-correct false positives).

(Moved: `Follow-up not addressed` block relocated to **Inventory pre-seeds** section below, so the parity inventory mapper inherits the decisions and re-audits skip re-discovery.)

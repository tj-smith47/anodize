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

_(no active items — all 2026-04-16 parity and hook-audit findings resolved; see Resolved below.)_

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
- [x] 2026-04-16 hook-audit: **142 `unwrap/expect` sites across 30 files → 0 in non-test lib code.** Eliminated via a mix of: (1) shared `anodize_core::util::static_regex(&str) -> Regex` helper for `LazyLock::new(…)` initializers (covers 20+ sites in `template_preprocess.rs`, `template.rs`, `git.rs`); (2) `let Some(X) = opt else { continue; }` for loop-scoped invariants (`krate.blobs`, `krate.snapcrafts`, `krate.pkgs`, `krate.nsis`, `krate.nfpm`, `krate.msis`, `krate.flatpaks`, `krate.app_bundles`, `krate.dmgs`); (3) `.context("…")?` propagation for genuinely-fallible paths (source stage config, stage-build job cmd, stage-blob KMS URL parsing, stage-release release_cfg); (4) `.unwrap_or_else(std::sync::PoisonError::into_inner)` for Mutex-lock poison recovery (template_preprocess regex cache, publisher parallel error collection, MockGitHubClient's test-helper state); (5) `.unwrap_or_else(|e| panic!(…))` / `.unwrap_or_else(|| panic!(…))` with programmer-bug diagnostics for infallible-by-construction Tera template parse/render (`stage-publish/src/nix.rs`, `chocolatey.rs`, `winget.rs`, `aur.rs`, `homebrew.rs`, `scoop.rs`, `krew.rs`), YAML serialize-by-Serialize-trait, `try_into` on fixed-size slices (`nix.rs` ELF parser — helper fns `read_u64/u32/u16`), `split_last()` after matching `len() >= 3` in `template.rs`. Thread-panic join-result sites (`sign.rs`, `docker.rs`) use `|_| panic!(…)` since `Box<dyn Any + Send>` doesn't impl Display. Doc-comment examples in `github_client.rs`, `test_helpers.rs` updated to use `?` instead of `.unwrap()`. 3176 tests pass, clippy clean, fmt clean. Remaining unwraps are confined to `#[cfg(test)]` / `#[cfg(all(test, feature = "test-helpers"))]` inline test modules where panicking on setup failure is the correct test-failure mode.
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
- [x] 2026-04-16 parity-audit: **`anodize build` now matches GoReleaser `BuildCmdPipeline` outputs end-to-end.** Verified already-present: before hooks, `dist/config.yaml` (effective config), `dist/metadata.json`, `dist/artifacts.json`, `reportsizes`. Added missing binary-only signing + macOS notarization: new `BinarySignStage` in `stage-sign` mirrors GoReleaser `sign.BinaryPipe` (runs only the `binary_signs` loop, not the generic `signs` loop — at build time the generic signs pipe would have the wrong semantics for `artifacts: all`). `build.rs` now wires `BinarySignStage` (after `UpxStage`, gated on `ctx.should_skip("sign")`) and `NotarizeStage` (gated on `ctx.should_skip("notarize")`). New integration test `test_e2e_build_command_matches_goreleaser_pipeline_outputs` runs `anodize build` end-to-end and asserts the before hook executed (touches a marker file), `dist/config.yaml` / `dist/metadata.json` / `dist/artifacts.json` exist with the expected content, and `report_sizes` emits a size line to stderr. 3176 tests pass.
  audit: crates/cli/src/commands/build.rs
  audit: crates/stage-sign/src/lib.rs
  audit: crates/cli/tests/integration.rs
- [x] 2026-04-16 parity-audit: **Per-packager `ConventionalFileName` (nfpm v2.44 parity).** New `stage-nfpm/src/filename.rs` module ports the `ConventionalFileName` + arch-translation logic for deb, rpm, apk, archlinux, ipk, and termux.deb from upstream nfpm v2.44.0. Distinct shapes now produced correctly: deb `{name}_{version~prerelease+meta-release}_{archToDebian}.deb`, rpm `{name}-{formatVersion}-{release|1}.{archToRPM}.rpm` (amd64→x86_64, arm64→aarch64, arm7→armv7hl, prerelease dashes replaced with underscores), apk `{name}_{pkgver}_{archToAlpine}.apk` (release gets auto `r` prefix, metadata gets auto `p` unless it starts with a VCS tag), archlinux `{name}-{version}{_prerelease}-{pkgrel|1}-{archToArchLinux}.pkg.tar.zst` through `validPkgName` char filtering, ipk same shape as deb with distinct archToIPK. `FileNameInfo::from_config` resolves `release`, `prerelease`, `version_metadata` from `NfpmConfig` and feeds into `conventional_filename(format, &info)` which returns `None` for unknown formats so callers fall back to the legacy `{pkg}_{ver}_{os}_{arch}{ext}` shape. Template-var setter at `stage-nfpm/src/lib.rs::run` switched from the hand-rolled deb-shaped string to the per-packager helper. 27 new unit tests cover every format and arch variant, plus the existing `test_conventional_filename_template_var` integration test was corrected to assert the upstream rpm shape (`myapp-5.0.0-1.x86_64.rpm` instead of the buggy `myapp_5.0.0_linux_amd64.rpm`).
  audit: crates/stage-nfpm/src/filename.rs
  audit: crates/stage-nfpm/src/lib.rs
- [x] 2026-04-16 parity-audit: **Blob now parallel across configs too, not just per-file.** `stage-blob` split into Phase 1 (serial — render templates per config, build ObjectStore, pre-render per-item `PutOptions` while holding `&mut ctx`) and Phase 2 (parallel across configs via `anodize_core::parallel::run_parallel_chunks` bounded by `ctx.options.parallelism`). New `upload_files_owned` takes fully-owned data so workers never touch `ctx`; the old `upload_files` / `UploadParams` wrapper is gone and tests now exercise `upload_files_owned` directly. Intra-config per-file tokio semaphore concurrency preserved. Per-config "uploading X -> Y" log lines stay in Phase 1 so announcement order remains deterministic. 68 blob tests still pass.
  audit: crates/stage-blob/src/lib.rs
  audit: crates/core/src/parallel.rs
- [x] 2026-04-16 code-audit: **Shared parallelism helper + template-var helpers extract duplicated staging code across 10 stages.** New `anodize_core::parallel::run_parallel_chunks<J, T, F>` (6 unit tests) wraps the `for chunk in jobs.chunks(n) { std::thread::scope(…) }` pattern with bounded concurrency, submission-order results, fail-fast per chunk, zero-parallelism clamp, and attributable panic reporting — consumed by `stage-upx`, `stage-makeself`, `stage-flatpak`, `stage-snapcraft`, `stage-nfpm`, `stage-blob`. New `anodize_core::template::clear_per_target_vars` + `PER_TARGET_VARS` centralises the `Os / Arch / Target / Arm / Arm64 / Amd64 / Mips / I386` end-of-stage cleanup that 5 stages (flatpak, snapcraft, nfpm, appbundle, dmg, pkg, nsis, makeself) were repeating in-line. New `anodize_core::util::collect_if_replace` wraps the `if cfg.replace.unwrap_or(false) { archives_to_remove.extend(collect_replace_archives(...)) }` boilerplate across 11 callsites in 7 stages (flatpak×2, snapcraft×2, appbundle×2, dmg×2, nsis×2, pkg×2, msi). Motivation: user flagged mid-refactor that each new parallelized stage was repeating the same loop skeleton — extracting turned 4 × 16-line inline copies into one 50-line helper + six 3-line call sites.
  audit: crates/core/src/parallel.rs
  audit: crates/core/src/template.rs
  audit: crates/core/src/util.rs
- [x] 2026-04-16 parity-audit: **nfpm / snapcraft / makeself / flatpak packaging loops now parallel via `ctx.parallelism`.** Shared helper `anodize_core::parallel::run_parallel_chunks<J, T, F>` (new module) preserves bounded concurrency, submission-order results, fail-fast per chunk, and attributable panic reporting with 6 unit tests (order preservation, bounded concurrency, first-error propagation, zero-parallelism clamp, empty jobs, panic-becomes-anyhow). Four stages refactored to Phase 1 / Phase 2 / Phase 3: **`stage-makeself`** (serial template render → parallel subprocess + file I/O → serial artifact register), **`stage-flatpak`** (serial work-dir staging + mod_timestamp pre-parse → parallel `flatpak-builder` + `flatpak build-bundle` → serial register), **`stage-snapcraft`** (serial prime-dir staging + templated_extra_files → parallel `snapcraft pack` → serial register), **`stage-nfpm`** (serial YAML write + mtime pre-parse → parallel `nfpm pkg --packager <format>` + reproducible-build mtime stamping → serial register). `stage-upx` (which inspired the pattern) migrated to the shared helper too, deleting its inline `for chunk in …chunks(n) { std::thread::scope(|s| …) }` duplication. Per-stage `Job` structs own all thread-portable data so workers never touch `ctx`.
  audit: crates/core/src/parallel.rs
  audit: crates/stage-makeself/src/lib.rs
  audit: crates/stage-flatpak/src/lib.rs
  audit: crates/stage-snapcraft/src/lib.rs
  audit: crates/stage-nfpm/src/lib.rs
  audit: crates/stage-upx/src/lib.rs
- [x] 2026-04-16 parity-audit: **`SkipMemento` pattern wired for sign / docker_signs / custom publisher skips.** New module `anodize_core::pipe_skip` introduces a thread-safe `SkipMemento` (Arc<Mutex<Vec<SkipEvent>>> under the hood) with dedup and snapshot/drain helpers. `Context` now owns a `skip_memento: SkipMemento` and exposes `ctx.remember_skip(stage, label, reason)`. Stages: `stage-sign` `process_sign_configs` (artifacts: none, if-condition skip, if-render-error) and `DockerSignStage` (artifacts: none, if-condition skip, if-render-error) record via the memento instead of bare `continue`. `run_publishers` in `cli/src/commands/publisher.rs` gained an `Option<&SkipMemento>` parameter; the release pipeline call site passes `Some(&ctx.skip_memento)` while standalone tests pass `None`. Pipeline runner drains at end-of-pipeline and prints a yellow "N intentional skips" block so operators can tell an intentionally-disabled sub-config apart from a misconfigured one. 11 new tests (5 on SkipMemento itself, 3 on stage-sign, 2 on publisher, 1 yaml).
  audit: crates/core/src/pipe_skip.rs
  audit: crates/core/src/context.rs
  audit: crates/cli/src/pipeline.rs
  audit: crates/stage-sign/src/lib.rs
  audit: crates/cli/src/commands/publisher.rs
- [x] 2026-04-16 parity-audit: **`tag_pre_hooks` / `tag_post_hooks` wired on the `tag` subcommand.** Added `tag_pre_hooks` and `tag_post_hooks: Option<Vec<HookEntry>>` to `TagConfig`; the `create_tag` closure in `commands/tag.rs` renders a minimal `TemplateVars` context (`Tag`, `PrefixedTag`, `Version`, `PreviousTag`), exports `ANODIZE_CURRENT_TAG` / `ANODIZE_PREVIOUS_TAG` into the process env, and invokes both hook lists via `anodize_core::hooks::run_hooks` (dry-run honoured). Also removed the stale `// TODO: Wire GitConfig (ignore_tags, ignore_tag_prefixes) into tag discovery` by extending `get_all_semver_tags` / `get_branch_semver_tags` to accept `Option<&GitConfig>` + `Option<&TemplateVars>` and applying the `ignore_tags` (glob) + `ignore_tag_prefixes` (starts_with) filters; callers in `find_previous_tag` now load `config.git` and pass it through. 5 new tests (2 YAML roundtrip on TagConfig, 3 git ignore_tags / ignore_tag_prefixes integration).
  audit: crates/cli/src/commands/tag.rs
  audit: crates/core/src/config.rs
  audit: crates/core/src/git.rs
- [x] 2026-04-16 hook-infra: **Lazy-write-off guard added.** `.claude/hooks/guard-bugs-closure.sh` (PreToolUse on Edit/Write). Detects `[ ]`→`[x]` closures in this file and blocks unless (a) session diff touched a path or backticked identifier named in the entry, (b) the entry contains an `audit: <path-or-identifier>` line the hook can resolve (grep against crates/ or filesystem existence), or (c) the entry contains an in-band `AUDITED: <reason>` override (reviewable in git blame). No env-var bypass — every skip must leave a git-blameable trail.
  audit: .claude/hooks/guard-bugs-closure.sh
  audit: .claude/settings.json

### 2026-04-15
- See git history; ~25 fixes landed (GitLab JOB-TOKEN, GitHub draft-URL email-link bug, Sign artifacts:all alignment, Release `include_meta`, Milestone provider resolution, AUR master branch, Custom publisher default filter, Docker ID uniqueness V1+V2, Docker V2 retry scope, Docker legacy cardinality, `docker manifest rm` tolerance, SBOM binary-like dedup, Bluesky PDS URL, plus 10+ verified-correct false positives).

(Moved: `Follow-up not addressed` block relocated to **Inventory pre-seeds** section below, so the parity inventory mapper inherits the decisions and re-audits skip re-discovery.)

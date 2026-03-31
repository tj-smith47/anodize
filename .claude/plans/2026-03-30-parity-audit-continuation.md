# Parity Audit Continuation Plan

## Status: Tests don't compile — struct field additions broke ~16 test constructors

All behavioral fixes are applied and the non-test code compiles cleanly (`cargo check` passes). The test code needs the new fields added to struct constructors.

## What's Done (compiles, not yet tested)

### Task 1: Behavioral bugs (22 fixes)
- `context.rs`: IsDraft wired from config, RuntimeGoos mapped to Go naming (darwin/amd64), Version preserves build metadata, default parallelism 4, FirstCommit template var
- `git.rs`: --dirty flag on describe, committer date (%cI), first_commit field, check_git_available(), git_status_porcelain(), git_head_short()
- `helpers.rs`: tag-at-HEAD is error (not warning), no-tag warns, git errors fail in non-snapshot, resolve_git_context returns Result
- `release.rs`: snapshot default template applied even without config, dirty error includes file list, snapshot doesn't auto-skip release stage, git availability check
- `pipeline.rs`: changelog before archive
- `template.rs`: validate_single_env_only()

### Task 2: Config type fixes
- `ReleaseConfig.skip_upload` → `Option<StringOrBool>` (supports "auto")
- `HooksConfig.pre` has `#[serde(alias = "hooks")]`
- `SnapshotConfig` uses `version_template` as primary serde name

### Task 3: Config field additions
- UPX: compress, lzma, brute (wired to CLI flags in stage-upx)
- ArchiveFileSpec::Detailed: strip_parent
- DockerManifestConfig: retry
- NfpmContent: packager, expand
- NfpmRpmConfig: scripts (NfpmRpmScripts), build_host
- NfpmDebConfig: scripts (NfpmDebScripts)
- NfpmApkConfig: scripts (NfpmApkScripts)
- NfpmSignatureConfig: key_name, type_
- SnapcraftLayout: bind_file, type_ (wired to YAML output)
- BuildConfig: mod_timestamp
- UniversalBinaryConfig: hooks, mod_timestamp

### Task 4: Artifact system
- Artifact struct has `name: String` field (set at add-time)
- Path normalization (backslash→forward slash) on add
- Duplicate name warning for uploadable artifacts
- `uploadable_kinds()` shared function

### Task 5: Release stage fixes
- Missing artifact files → hard error (not warning)
- IDs filter exempts Checksum/SourceArchive/Sbom/Metadata
- Signature and Certificate kinds added to upload list

### Batch 2-6 fixes applied
- Archive: zstd level 19→3
- Docker: default platforms empty (not forced buildx)
- Docker: build streams output via Stdio::inherit
- nFPM: template render errors propagated (not silently swallowed)
- Snapcraft: bind_file/type_ wired to YAML, --destructive-mode added, grade validation
- Sign: --detach-sig→--detach-sign (GPG fix), docker sign adds --yes
- Scoop: .exe only appended if not present, always array format
- AUR: conflicts/provides default to base name (without -bin)
- Homebrew: class name splits on spaces too
- Release: Signature/Certificate kinds in upload list

## What's NOT Done

### Test compilation (immediate)
These struct changes broke test constructors that construct the structs directly:
- `DockerManifestConfig` needs `retry: None` in ~5 test constructors
- `NfpmContent` needs `packager: None, expand: None` in ~4 tests
- `NfpmApkConfig` needs `scripts: None` in ~2 tests
- `NfpmSignatureConfig` needs `key_name: None, type_: None` in ~1 test
- `UniversalBinaryConfig` needs `hooks: None, mod_timestamp: None` in tests (stage-build)
- `SnapcraftLayout` needs `bind_file: None, type_: None` in tests
- `Artifact` needs `name: String::new()` in a few remaining test constructors
- Some files have DUPLICATE `retry: None` or `name: String::new()` from bad sed — need dedup

Fix approach: `cargo test --workspace 2>&1 | grep "error\[E006"` shows exact file:line for each. Fix manually, not with sed.

### Unfixed comparison findings (apply then review)
**Release stage:**
- append/prepend mode: should PATCH existing release, not POST new (crashes with 422)
- Draft search limited to 100 releases (no pagination)
- find_draft_by_name swallows API errors silently

**Sign stage:**
- Default `artifacts` filter should be `"none"` not `"checksum"` (GoReleaser is opt-in)
- Default `output` should be `true` not `false` (for both sign and docker_sign)

**Changelog:**
- Spurious "## Changes" heading when no groups configured

**Homebrew:**
- `skip_upload` not template-rendered (affects ALL publishers via `should_skip_upload`)
- Homepage fallback URL broken (uses crate name, not owner/repo)
- Dependencies not sorted

**Scoop:**
- Missing WrappedIn folder path in bin entries

**WinGet:**
- Missing WrappedIn folder in RelativeFilePath

**nFPM:**
- Default filename missing arch (causes overwrites for multi-target)
- No ConventionalFileName support
- Artifact path may not match what nfpm actually produces

**Docker:**
- Empty manifest images not skipped (crash on conditional templates)
- manifest rm errors fully swallowed (should only ignore "no such manifest")
- Manifest create/push has no retry logic

**Build:**
- Missing `aarch64-pc-windows-msvc` in DEFAULT_TARGETS
- Hardcoded `target/` path breaks with CARGO_TARGET_DIR

**Snapcraft:**
- Publish coupled to build (should be separate phase)

**Blob:**
- No per-file upload logging
- Parallelism uses global setting (may hit rate limits)

### Code reviews (after all fixes)
Run parallel code-reviewer agents per batch, fix all findings, re-review until clean.

### Final report (Task 13)
Summary table of all BUGs/GAPs fixed, TRACKED, INTENTIONAL.

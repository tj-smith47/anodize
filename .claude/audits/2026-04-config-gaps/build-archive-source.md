# Build / Archive / Source / Universal / UPX gap audit (re-graded)

Date: 2026-04-25
References: `/opt/repos/anodizer` HEAD, `/opt/repos/goreleaser` HEAD.

**Re-grading rule:** Go-toolchain-specific fields (goos/goarch/goarm/goamd64/gomod/ldflags/gcflags/asmflags) are NOT MISSING-FIELD in a Rust tool. Anodizer expresses platform filtering through Rust target triples and `targets:` glob.

## Summary
- **MISSING-FIELD: 0** (the original agent flagged UpxConfig.goos/goarch/goarm as MISSING; on re-grade these are GO-IDIOM-ONLY because anodizer's `targets:` glob serves the same purpose).
- **Defaults differing with user impact: 1**.
- **Code smells / behavioral findings: 18**.
- **Anodizer-only intentional fields: 9** (verified wired & justified).

## BuildConfig (config.rs:1037)

GoReleaser source: `pkg/config/config.go:483` `type Build struct`.
Anodizer consumers: `crates/stage-build/src/lib.rs`, `binstall.rs`, `crates/cli/src/commands/check.rs`.

- **Missing fields:** none (every GR Build field with a Rust analog is present).
- **GO-IDIOM-ONLY:** `Goos`, `Goarch`, `Goamd64`, `Go386`, `Goarm`, `Goarm64`, `Gomips`, `Goppc64`, `Goriscv64` (config.go:485-493); `Main`, `Builder`, `Buildmode`, `Ldflags`, `Tags`, `Asmflags`, `Gcflags`, `NoMainCheck`, `UnproxiedMain`, `UnproxiedDir`, `GoBinary`, `Tool`/`Command`.
- **Default divergence (user-visible):** `flags` typing — GR uses `FlagArray` (`[]string`), anodizer uses `Option<String>` and tokenizes by whitespace at stage-build/lib.rs:182-183 and :240. A flag with embedded spaces (e.g. `--config-toml='target.x.linker = "lld"'`) is mis-split. Workaround: shell-escape gymnastics. Fix: accept `Vec<String>` or use `shlex`.
- **Anodizer-only intentional:** `cross`/`cross_tool` (Rust cross-compile strategy), `features`/`no_default_features`, `copy_from`, `reproducible` (SOURCE_DATE_EPOCH), `env: HashMap<String, HashMap<String,String>>` (per-target env, GR is flat), `no_unique_dist_dir` (also exposed at CrateConfig level).
- **Code smells:**
  - stage-build/lib.rs:182-183, :240 — `flags.split_whitespace()` mis-splits quoted args.
  - stage-build/lib.rs:528 — universal binary `mod_timestamp` `unwrap_or_else(|_| ts.clone())` swallows template errors.
  - stage-build/lib.rs:548 — universal binary metadata-copy whitelist (`["dynamically_linked", "abi", "libc", "id"]`) is brittle; new metadata keys silently fail to propagate.
  - stage-build/lib.rs:1622 — `build.env.get(target.as_str())` exact match; users supplying glob targets silently get no env.
  - stage-build/lib.rs:1042 — reproducible-epoch warning uses `eprintln!`-style; bypasses tracing.

## UniversalBinaryConfig (config.rs:1014)

GoReleaser: `UniversalBinary` (config.go:591).

- **Missing fields:** none — full parity (ID, IDs, NameTemplate, Replace, Hooks, ModTimestamp).
- **Code smells:**
  - stage-build/lib.rs:528 — same `mod_timestamp` swallow bug.
  - stage-build/lib.rs:370-394 — parity comment claims fallback matches GR; GR falls through `unibin.ID = ProjectName`, anodizer falls back to *crate name*. Comment misleading.
  - stage-build/lib.rs:380-389 — metadata lookup checks both `id` and `binary` keys; GR only checks `id`. The `binary` fallback may be stale.
  - stage-build/lib.rs:565 — registers artifact with `name: String::new()`. Other artifact-registration sites set `name` to the filename.

## ArchiveConfig (config.rs:1288)

GoReleaser: `Archive` (config.go:615) + FormatOverride/File/FileInfo.

- **Missing fields:** none. Mapping: ID, IDs (with `builds` alias), BuildsInfo, NameTemplate, Formats, Format (deprecated), FormatOverrides, WrapInDirectory, StripBinaryDirectory, Files, Meta, AllowDifferentBinaryCount, Builds-deprecated alias — all present.
- **Default divergence (user-impacting):** **`archives: []` behavior** — anodizer treats `vec![]` as "no archives, skip stage" (config.rs:983). GoReleaser auto-injects one default `Archive{}` when `len(ctx.Config.Archives) == 0` (archive.go:57-59). **Users with `archives: []` (or omitting `archives:`) may produce no archives at all where GR would produce one default `tar.gz`.** Verify which behavior anodizer applies for the omit case vs the explicit-empty case.
- **Default divergence (round-trip only):** Archive `id` — GR persists `"default"` post-Default(); anodizer renders lazily, persists `None`. Runtime identical, YAML round-trip differs.
- **Behavioral mismatch:** FormatOverride match is exact `==` (stage-archive/lib.rs:774); GR uses `strings.HasPrefix(platform, override.Goos)` (archive.go:349). Currently moot because `map_target` normalizes; document or align.
- **Anodizer-only:** `binaries: Option<Vec<String>>` — filter by binary name. Wired stage-archive/lib.rs:1071.
- **Code smells:**
  - stage-archive/lib.rs:62 — `eprintln!`-style mtime warning bypasses StageLogger.
  - stage-archive/lib.rs:560-567 — default extra-file glob order differs from GR (anodizer: `["LICENSE*", "license*", "README*"]`, GR: `["license*", "LICENSE*"]`). Case-insensitive filesystems get different first-match.
  - stage-archive/lib.rs:838-845 — `defaults.archives` only carries `format` (singular) and `format_overrides`. Workspace users can't share `name_template`/`formats`/`wrap_in_directory`/`builds_info`.
  - stage-archive/lib.rs:1108 — empty-binaries-after-filter skip is silent; GR logs a warning.
  - stage-archive (general) — 1700 lines in one file; should split.

## SourceConfig (config.rs:4090)

GoReleaser: `Source` (config.go:1252).

- **Missing fields:** none — full parity (NameTemplate, Format, Enabled, PrefixTemplate, Files).
- **Code smells:**
  - stage-source/lib.rs:677 — `ctx.render_template(&entry.src).unwrap_or_else(|_| entry.src.clone())` swallows template errors. GR propagates. Use `?`.
  - stage-source/lib.rs:34 — `#[allow(clippy::too_many_arguments)]` on `create_source_archive`; refactor into `SourceArchiveOpts` struct.
  - stage-source/lib.rs:651 — dry-run skips glob validation; malformed globs in `source.files` silently succeed in dry-run, fail at real run.
  - **Type inconsistency:** `SourceFileInfo.mode: Option<u32>` (config.rs:4083) vs `ArchiveFileInfo.mode: Option<String>` (config.rs:1380). YAML-friendly form differs across two near-identical types.

## UpxConfig (config.rs:4516)

GoReleaser: `UPX` (config.go:601).

- **Missing fields:** none on re-grade. The original audit flagged `goos`/`goarch`/`goarm`/`goamd64` as MISSING; per re-grading rule these are GO-IDIOM-ONLY because anodizer's `targets:` glob serves the same filtering purpose. Document the divergence so users migrating from GoReleaser see the equivalence.
- **Anodizer-only:** `id` (log prefix), `args` (raw upx flags escape hatch), `required` (fail when upx not on PATH; GR silently skips), `targets` (Rust-equivalent of goos/goarch/goarm filters).
- **Code smells:**
  - stage-upx/lib.rs:181, :232 — missing-file metadata produces misleading "100%" compression ratio.
  - stage-upx/lib.rs:140 — `compress: "abc"` passes `-abc` to upx; should validate against `enum=1..9,best`.
  - stage-upx/lib.rs:108-109 — universal binary handling ordering not documented; if `replace: true`, the source binaries get UPX'd in addition to the universal; verify pipeline order.
  - stage-upx/lib.rs:117 — `id` metadata filter assumes builds always populate `id`; verify default-id artifacts match.

## CrateConfig (config.rs:898)

Anodizer-original; no GR analog (GR monorepo support is Pro-only via `MonorepoConfig`).

- **Code smells:**
  - config.rs:919 — custom deserializer for `archives: false | array`; verify error message is informative.
  - config.rs:927 — `pub docker: Option<Vec<DockerConfig>>` described as "legacy API" but sits alongside `docker_v2`. Audit whether `docker` is still wired; if replaced, mark with warn-on-use shim.

## DefaultArchiveConfig (config.rs:840)

Anodizer-original (workspace-wide archive defaults). Only exposes `format` (singular) + `format_overrides`. Should also carry `formats`/`name_template`/`wrap_in_directory`/`builds_info` for monorepo DRY. (Enhancement, not a parity gap.)

## ArchiveHooksConfig / BuildHooksConfig

Near-duplicate types differing only in canonical field-name spelling (`pre`/`post` vs `before`/`after`); each accepts the other via serde aliases. Justified for YAML round-trip fidelity but maintenance-heavy. `BuildHooksConfig` carries archive-stage hooks too despite the `Build` prefix — name lies about scope.

## Cross-cutting

- stage-build/lib.rs:2292-2296 — universal binary loop calls `build_universal_binary` once per UB config. Two configs with name_templates rendering to the same filename will silently overwrite.

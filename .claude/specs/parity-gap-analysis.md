# Anodize vs GoReleaser OSS — Parity Gap Analysis

**Date:** 2026-03-25
**Baseline:** Anodize v0.1.0 — 138 tests, 0 clippy warnings, ~8k LOC
**Compared against:** GoReleaser OSS v2.x (current stable)

This document covers every GoReleaser OSS feature and whether anodize has it, partially has it, or is missing it entirely. Pro-only features are noted but not counted as gaps.

---

## Executive Summary

| Category | Has | Partial | Missing | Notes |
|----------|-----|---------|---------|-------|
| Pipeline stages | 10/10 | — | 3 OSS stages | Missing: source, sbom, snapcraft |
| CLI commands | 5/8 | — | 3 | Missing: healthcheck, completion, jsonschema |
| CLI flags | 9/22 | — | 13 | Missing: --config, --timeout, --parallelism, etc. |
| Config schema | ~60% | ~15% | ~25% | Core structure present, many fields missing |
| Template engine | ~30% | — | ~70% | Variables only; no functions, conditionals, pipes |
| GitHub Action | Basic | — | Significant | Composite shell action vs full JS action |
| Package managers | 3/7 | — | 4 | Missing: AUR, Snapcraft, Chocolatey, Winget |
| Announce providers | 3/8+ | — | 5+ | Missing: Telegram, Teams, Mattermost, etc. |
| Documentation | Minimal | — | Most | README + 1 config doc vs full docs site |
| Test coverage | 138 | — | ~1000+ | No e2e, limited integration, no cross-platform |

**Bottom line:** Anodize has the right architecture and all core pipeline stages. The biggest gaps are: (1) template engine maturity, (2) CLI flag coverage, (3) config field completeness within existing stages, and (4) missing OSS features like source archives, SBOMs, and additional package managers.

---

## 1. Feature-by-Feature Comparison

### 1.1 Pipeline Stages

| Stage | GoReleaser OSS | Anodize | Gap |
|-------|---------------|---------|-----|
| Before hooks | `before.hooks` | `before.hooks` | **Parity** |
| Build | `builds[]` | `crates[].builds[]` | Partial — see 1.2 |
| Archive | `archives[]` | `crates[].archives[]` | Partial — see 1.3 |
| NFpm | `nfpms[]` | `crates[].nfpm[]` | Partial — see 1.4 |
| Checksum | `checksum` | `crates[].checksum` | Partial — see 1.5 |
| Changelog | `changelog` | `changelog` | Partial — see 1.6 |
| Release | `release` | `crates[].release` | Partial — see 1.7 |
| Publish (Homebrew) | `brews[]` | `publish.homebrew` | Partial — see 1.8 |
| Publish (Scoop) | `scoops[]` | `publish.scoop` | Partial — see 1.9 |
| Docker | `dockers[]` | `crates[].docker[]` | Partial — see 1.10 |
| Docker manifest | `docker_manifests[]` | — | **Missing** — anodize uses `docker buildx` directly for multi-arch, but lacks the manifest-list approach |
| Sign | `signs[]` | `sign` | Partial — see 1.11 |
| Docker sign | `docker_signs[]` | `docker_signs[]` | Partial — see 1.12 |
| Announce | `announce.*` | `announce.*` | Partial — see 1.13 |
| Source archive | `source` | — | **Missing** — creates source tarballs |
| SBOM | `sboms[]` | — | **Missing** — generates Software Bill of Materials |
| Snapcraft | `snapcrafts[]` | — | **Missing** — Linux snap packaging |
| UPX | `upx[]` | — | **Missing** — binary compression |
| Blob upload | `blobs[]` | — | **Missing** — S3/GCS/Azure upload |
| Custom publishers | `publishers[]` | — | **Missing** — generic HTTP/command publish |
| AUR | `aurs[]` | — | **Missing** — Arch User Repository |
| Krew | `krews[]` | — | **Missing** — kubectl plugin manifests |
| Chocolatey | `chocolateys[]` | — | **Missing** — Windows package manager |
| Winget | `wingets[]` | — | **Missing** — Windows Package Manager |
| Fury.io | `furies[]` | — | **Missing** — hosted apt/yum repo |
| Ko | `kos[]` | — | N/A — Go-specific, no Rust equivalent |
| Artifactory | `artifactories[]` | — | **Missing** — deprecated in GR, covered by publishers |
| Report sizes | `report_sizes` | — | **Missing** — artifact size table |
| Metadata | `metadata` | — | **Missing** — dist/metadata.json output |
| After hooks | `after.hooks` | `after.hooks` | **Parity** |

### 1.2 Build Stage Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Build ID | `id` | — | **Missing** — needed for archive/docker filtering |
| Binary name | `binary` | `binary` | Parity |
| Targets/matrix | `goos` × `goarch` × `goarm` with `ignore` | `targets[]` (explicit list) | Different approach, adequate for Rust |
| Build flags | `flags`, `ldflags`, `gcflags`, `asmflags`, `tags` | `flags`, `features`, `no_default_features` | Adapted for Rust — adequate |
| Per-target env | `env[]` (template-aware) | `env` map per target | Parity |
| Cross-compilation | Native Go cross-compile | `cross: auto/zigbuild/cross/cargo` | **Better than GR** — explicit strategy |
| Copy/alias binary | — | `copy_from` | **Better than GR** |
| Pre/post build hooks | `pre_hooks`, `post_hooks` (Pro) | — | N/A (Pro-only) |
| Build working dir | `dir` | — | **Missing** — defaults to crate path |
| Output path override | via `--output` CLI flag | — | **Missing** |
| Skip build | `skip: true` | — | **Missing** |
| Build mode | `buildmode` (e.g., c-shared, pie) | — | N/A — Go-specific |
| Target ignore list | `ignore: [{goos, goarch}]` | — | **Missing** — useful for excluding combos |
| Single-target mode | `--single-target` CLI flag | — | **Missing** |
| Per-target overrides | `overrides[]` | — | **Missing** — override env/flags per target |
| Parallelism | `--parallelism` flag | — | **Missing** — builds run sequentially |
| Timeout | `--timeout` flag | — | **Missing** |
| `mod_timestamp` | Sets binary mtime for reproducibility | — | **Missing** |

### 1.3 Archive Stage Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Archive ID | `id` | — | **Missing** |
| Name template | `name_template` | `name_template` | Parity |
| Format | `tar.gz`, `tar.xz`, `tar.zst`, `zip`, `gz`, `binary` | `tar.gz`, `zip` | **Missing** formats: `tar.xz`, `tar.zst`, `gz`, `binary` |
| Format overrides | `format_overrides[].goos` | `format_overrides[].os` | Parity (different key name, same concept) |
| Extra files | `files[]` (glob + src/dst) | `files[]` (string list) | **Partial** — no glob patterns, no src/dst mapping |
| Binary filter | `builds[]` (by build ID) | `binaries[]` | Similar concept |
| Wrap in directory | `wrap_in_directory` | — | **Missing** |
| Strip binary dir | `strip_binary_directory` | — | **Missing** |
| Builds info | `builds_info` (mtime, owner, group, mode) | — | **Missing** |
| Allow different binary count | `allow_different_binary_count` | — | **Missing** |

### 1.4 NFpm Stage Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Core fields | `package_name`, `formats`, `vendor`, `homepage`, `maintainer`, `description`, `license`, `bindir` | All present | Parity |
| Contents | `contents[].{src, dst, type, file_info, packager}` | `contents[].{src, dst}` | **Partial** — missing `type`, `file_info`, `packager` |
| Scripts | `scripts.{preinstall, postinstall, preremove, postremove}` | — | **Missing** |
| Dependencies | `dependencies[]` | `dependencies` map per format | Parity |
| Recommends/suggests | `recommends`, `suggests` | — | **Missing** |
| Conflicts/replaces/provides | `conflicts`, `replaces`, `provides` | — | **Missing** |
| RPM-specific | `rpm.{group, summary, compression, signature}` | — | **Missing** |
| Deb-specific | `deb.{fields, triggers, breaks, signature}` | — | **Missing** |
| APK-specific | `apk.signature` | — | **Missing** |
| File name template | `file_name_template` | `file_name_template` | Parity |
| Overrides | Per-format overrides | `overrides` (serde_json::Value) | Parity (flexible) |

### 1.5 Checksum Stage Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Name template | `name_template` | `name_template` | Parity |
| Algorithm | `sha256`, `sha512`, `sha1`, `crc32`, `md5`, `sha224`, `sha384`, `blake2b`, `blake2s` | `sha256`, `sha512` | **Missing** algorithms: sha1, crc32, md5, sha224, sha384, blake2b, blake2s |
| Extra files | `extra_files[]` | — | **Missing** |
| IDs filter | `ids[]` | — | **Missing** |
| Disable | `disable: true` | — | **Missing** |

### 1.6 Changelog Stage Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Sort | `sort: asc/desc` | `sort: asc/desc` | Parity |
| Filters exclude | `filters.exclude[]` | `filters.exclude[]` | Parity |
| Filters include | `filters.include[]` | — | **Missing** |
| Groups | `groups[].{title, regexp, order}` | `groups[].{title, regexp, order}` | Parity |
| Disable | `disable: true` | — | **Missing** |
| Use source | `use: git/github/github-native/gitlab` | git only | **Missing** — `github-native` is popular |
| Header/footer | `changelog.header`, `changelog.footer` | — | **Missing** |
| Abbrev | `abbrev` (hash length) | — | **Missing** |

### 1.7 Release Stage Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| GitHub | `github.{owner, name}` | `github.{owner, name}` | Parity |
| GitLab | `gitlab.{owner, name}` | — | **Missing** (Release 1 is GitHub-only by design) |
| Gitea | `gitea.{owner, name}` | — | **Missing** (by design) |
| Draft | `draft` | `draft` | Parity |
| Prerelease | `prerelease: auto/true/false` | `prerelease: auto/true/false` | Parity |
| Name template | `name_template` | `name_template` | Parity |
| Disable | `disable: true` | — | **Missing** |
| Make latest | `make_latest: true/false/auto` | — | **Missing** |
| Skip upload | `skip_upload` | — | **Missing** |
| Extra files | `extra_files[]` | — | **Missing** |
| Header/footer | `header`, `footer` (in release body) | — | **Missing** |
| Replace existing draft | `replace_existing_draft` | — | **Missing** |
| Replace existing artifacts | `replace_existing_artifacts` | — | **Missing** |
| Target commitish | `target_commitish` | — | **Missing** |
| IDs filter | `ids[]` | — | **Missing** |
| Infer owner/name from git remote | Auto-detected | — | **Missing** — always requires explicit config |

### 1.8 Homebrew Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Tap repo | `repository.{owner, name}` | `tap.{owner, name}` | Parity |
| Folder | `folder` | `folder` | Parity |
| Description | `description` | `description` | Parity |
| License | `license` | `license` | Parity |
| Install block | `install` | `install` | Parity |
| Test block | `test` | `test` | Parity |
| Repository branch | `repository.branch` | — | **Missing** |
| Repository token | `repository.token` | — | **Missing** (uses GITHUB_TOKEN) |
| URL template | `url_template` | — | **Missing** |
| Download strategy | `download_strategy` | — | **Missing** |
| Commit author | `commit_author.{name, email}` | — | **Missing** |
| Commit message template | `commit_msg_template` | — | **Missing** |
| Homepage | `homepage` | — | **Missing** |
| Dependencies | `dependencies[]` | — | **Missing** |
| Conflicts | `conflicts[]` | — | **Missing** |
| Caveats | `caveats` | — | **Missing** |
| Skip upload | `skip_upload` | — | **Missing** |
| IDs filter | `ids[]` | — | **Missing** |
| Name override | `name` | — | **Missing** — uses project name |
| Custom block | `custom_block` | — | **Missing** |

### 1.9 Scoop Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Bucket repo | `repository.{owner, name}` | `bucket.{owner, name}` | Parity |
| Description | `description` | `description` | Parity |
| License | `license` | `license` | Parity |
| Repository branch/token | `repository.{branch, token}` | — | **Missing** |
| URL template | `url_template` | — | **Missing** |
| Commit author | `commit_author.{name, email}` | — | **Missing** |
| Homepage | `homepage` | — | **Missing** |
| Skip upload | `skip_upload` | — | **Missing** |
| Persist dirs | `persist[]` | — | **Missing** |
| Pre/post install | `pre_install[]`, `post_install[]` | — | **Missing** |
| Dependencies | `depends[]` | — | **Missing** |
| Shortcuts | `shortcuts[]` | — | **Missing** |

### 1.10 Docker Stage Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Image templates | `image_templates[]` | `image_templates[]` | Parity |
| Dockerfile | `dockerfile` | `dockerfile` | Parity |
| Build flags | `build_flag_templates[]` | `build_flag_templates[]` | Parity |
| Binaries | `ids[]` (by build ID) | `binaries[]` | Similar |
| Platforms | N/A (single-platform per entry) | `platforms[]` | **Better** — anodize handles multi-arch natively |
| Docker manifests | `docker_manifests[]` (separate config) | — | Different approach — anodize uses buildx --platform directly |
| Skip push | `skip_push` | — | **Missing** |
| Use buildx/docker | `use: docker/buildx` | Always buildx | OK — buildx is the modern standard |
| Extra files | `extra_files[]` | — | **Missing** |
| Push flags | `push_flags[]` | — | **Missing** |
| ID | `id` | — | **Missing** |

### 1.11 Sign Stage Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Command | `cmd` | `cmd` | Parity |
| Args | `args[]` (templated) | `args[]` (templated) | Parity |
| Artifacts filter | `artifacts: none/all/checksum/source/archive/binary/package/sbom` | `artifacts: none/all/checksum` | **Partial** — missing: source, archive, binary, package, sbom |
| Multiple sign configs | `signs[]` (array) | `sign` (single) | **Missing** — only one sign config allowed |
| ID | `id` | — | **Missing** |
| IDs filter | `ids[]` | — | **Missing** |
| Signature template | `signature` | — | **Missing** |
| Stdin/stdin_file | `stdin`, `stdin_file` | — | **Missing** |
| Certificate | `certificate` | — | **Missing** |
| Env | `env[]` | — | **Missing** |
| Output flag | `output: bool` | — | **Missing** |

### 1.12 Docker Sign Details

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| Command | `cmd` | `cmd` | Parity |
| Args | `args[]` | `args[]` | Parity |
| Artifacts filter | `artifacts: none/all/manifests/images` | `artifacts: all` | **Partial** |
| Multiple configs | `docker_signs[]` | `docker_signs[]` | Parity |
| IDs filter, stdin, env | Various | — | **Missing** |

### 1.13 Announce Stage Details

| Provider | GoReleaser OSS | Anodize | Status |
|----------|---------------|---------|--------|
| Discord | Yes | Yes | Parity |
| Slack | Yes | Yes | Parity |
| Generic webhook | Yes | Yes | Parity |
| Telegram | Yes | — | **Missing** |
| Teams | Yes | — | **Missing** |
| Mattermost | Yes | — | **Missing** |
| Reddit | Yes | — | **Missing** |
| SMTP/Email | Yes | — | **Missing** |
| Twitter | Yes | — | **Missing** |
| Mastodon | Yes | — | **Missing** |
| LinkedIn | Yes | — | **Missing** |
| OpenCollective | Yes | — | **Missing** |

---

## 2. CLI Parity

### 2.1 Commands

| Command | GoReleaser | Anodize | Status |
|---------|-----------|---------|--------|
| `release` | Yes (OSS) | Yes | Parity |
| `build` | Yes (OSS) | Yes | Parity |
| `check` | Yes (OSS) | Yes | Parity |
| `init` | Yes (OSS) | Yes | Parity |
| `changelog` | Pro only | Yes (OSS!) | **Better** — free in anodize |
| `healthcheck` | Yes (OSS) | — | **Missing** — check command does env checks but they're warnings only; no dedicated healthcheck |
| `completion` | Yes (OSS) | — | **Missing** — shell completion generation |
| `jsonschema` | Yes (OSS) | — | **Missing** — JSON schema output for IDE support |
| `publish` | Pro only | — | N/A |
| `announce` | Pro only | — | N/A |
| `continue` | Pro only | — | N/A |

### 2.2 Global Flags

| Flag | GoReleaser | Anodize | Status |
|------|-----------|---------|--------|
| `--debug` | Yes | Yes | Parity |
| `--verbose` | Yes | Yes | Parity |
| `--help` | Yes (clap auto) | Yes (clap auto) | Parity |
| `--version` | Yes | Yes | Parity |

### 2.3 `release` Command Flags

| Flag | GoReleaser | Anodize | Status |
|------|-----------|---------|--------|
| `--config` / `-f` | Yes | — | **Missing** — always searches for config in CWD |
| `--snapshot` | Yes | `--snapshot` | Parity |
| `--dry-run` | — | `--dry-run` | **Better** — GoReleaser has no exact equivalent (closest is `--snapshot`) |
| `--skip` | Yes (many values) | `--skip` (stage names) | **Partial** — GR has finer-grained skip values |
| `--clean` | Yes | `--clean` | Parity |
| `--timeout` | Yes (default 30m) | — | **Missing** |
| `--parallelism` / `-p` | Yes (default: num CPUs) | — | **Missing** |
| `--auto-snapshot` | Yes | — | **Missing** — auto-snapshot if repo is dirty |
| `--single-target` | Yes | — | **Missing** — build only for current host |
| `--release-notes` | Yes | — | **Missing** — custom release notes from file |
| `--release-notes-tmpl` | Yes | — | **Missing** |
| `--release-header` / `--release-header-tmpl` | Yes | — | **Missing** |
| `--release-footer` / `--release-footer-tmpl` | Yes | — | **Missing** |
| `--token` | — | `--token` | **Better** — GR uses env var only |
| `--crate` | — | `--crate` | N/A — Rust-specific |
| `--all` | — | `--all` | N/A — Rust workspace-specific |
| `--force` | — | `--force` | N/A — Rust workspace-specific |
| `--deprecated` | Yes | — | N/A |

### 2.4 `build` Command Flags

| Flag | GoReleaser | Anodize | Status |
|------|-----------|---------|--------|
| `--config` / `-f` | Yes | — | **Missing** |
| `--id` | Yes (filter builds) | — | **Missing** |
| `--output` / `-o` | Yes | — | **Missing** |
| `--snapshot` | Yes | — | **Missing** — build command has no snapshot mode |
| `--timeout` | Yes | — | **Missing** |
| `--parallelism` | Yes | — | **Missing** |
| `--clean` | Yes | — | **Missing** |
| `--single-target` | Yes | — | **Missing** |
| `--crate` | — | `--crate` | N/A — Rust-specific |

### 2.5 `check` Command Flags

| Flag | GoReleaser | Anodize | Status |
|------|-----------|---------|--------|
| `--config` / `-f` | Yes | — | **Missing** |
| `--quiet` / `-q` | Yes | — | **Missing** |
| `--deprecated` | Yes | — | N/A |

### 2.6 `changelog` Command Flags

| Flag | GoReleaser (Pro) | Anodize | Status |
|------|-----------------|---------|--------|
| `--config` / `-f` | Yes | — | **Missing** |
| `--since` | Yes | — | **Missing** |
| `--crate` | — | `--crate` | N/A — Rust-specific |

---

## 3. Config Schema Parity

### 3.1 Top-Level Fields

| Field | GoReleaser | Anodize | Status |
|-------|-----------|---------|--------|
| `project_name` | Yes | Yes | Parity |
| `dist` | Yes | Yes | Parity |
| `version` | Yes (schema version) | — | **Missing** |
| `env` | Yes (global env vars) | — | **Missing** |
| `env_files` | Yes (.env file loading) | — | **Missing** |
| `report_sizes` | Yes | — | **Missing** |
| `metadata` | Yes (metadata.json) | — | **Missing** |
| `before` / `after` | Yes | Yes | Parity |
| `snapshot` | Yes | Yes | Parity |
| `git` | Yes (various git options) | — | **Missing** — GR has `git.tag_sort`, `git.prerelease_suffix`, etc. |

### 3.2 Structural Differences

GoReleaser uses **top-level arrays** for most stage configs (`builds[]`, `archives[]`, `nfpms[]`, `dockers[]`, etc.) with `id` fields for cross-referencing. Anodize uses a **per-crate** nesting model (`crates[].builds[]`, `crates[].archives[]`). This is intentionally different for Rust workspace support and is a valid design choice, not a gap.

However, the `id`-based cross-referencing system is missing in anodize, which limits flexibility (e.g., can't have an archive that includes binaries from multiple crates).

---

## 4. Template Engine Parity

### 4.1 Template Variables

| Variable | GoReleaser | Anodize | Status |
|----------|-----------|---------|--------|
| `{{ .ProjectName }}` | Yes | Yes | Parity |
| `{{ .Version }}` | Yes | Yes | Parity |
| `{{ .Tag }}` | Yes | Yes | Parity |
| `{{ .ShortCommit }}` | Yes | Yes | Parity |
| `{{ .FullCommit }}` | Yes | Yes | Parity |
| `{{ .Commit }}` | Yes (alias for FullCommit) | — | **Missing** alias |
| `{{ .Os }}` | Yes | Yes | Parity |
| `{{ .Arch }}` | Yes | Yes | Parity |
| `{{ .Arm }}` | Yes | — | N/A — Go ARM variants |
| `{{ .Amd64 }}` | Yes | — | N/A — Go AMD64 microarch |
| `{{ .Major }}` | Yes | Yes | Parity |
| `{{ .Minor }}` | Yes | Yes | Parity |
| `{{ .Patch }}` | Yes | Yes | Parity |
| `{{ .Prerelease }}` | Yes | Yes (in spec) | Parity |
| `{{ .RawVersion }}` | Yes | — | **Missing** — version without prefix |
| `{{ .Branch }}` | Yes | — | **Missing** |
| `{{ .PreviousTag }}` | Yes | — | **Missing** |
| `{{ .IsSnapshot }}` | Yes | Yes (in spec) | Parity |
| `{{ .IsDraft }}` | Yes | Yes (in spec) | Parity |
| `{{ .IsNightly }}` | Yes | — | N/A (Release 2) |
| `{{ .Env.VAR }}` | Yes | Yes | Parity |
| `{{ .Date }}` | Yes | Yes (in spec) | Parity |
| `{{ .Timestamp }}` | Yes | Yes (in spec) | Parity |
| `{{ .CommitDate }}` | Yes | — | **Missing** |
| `{{ .CommitTimestamp }}` | Yes | — | **Missing** |
| `{{ .GitURL }}` | Yes | — | **Missing** |
| `{{ .GitTreeState }}` | Yes (clean/dirty) | — | **Missing** |
| `{{ .IsGitDirty }}` | Yes | — | **Missing** |
| `{{ .Summary }}` | Yes (tag-commits-hash) | — | **Missing** |
| `{{ .Now }}` | Yes | — | **Missing** |
| `{{ .ReleaseURL }}` | Yes | Yes (in spec) | Parity |
| `{{ .Signature }}` | Yes | Yes | Parity |
| `{{ .Artifact }}` | Yes (in sign context) | Yes | Parity |
| `{{ .ArtifactName }}` | Yes | — | **Missing** |
| `{{ .ArtifactPath }}` | Yes | — | **Missing** |
| `{{ .Changelog }}` | Yes (full changelog text) | — | **Missing** |
| `{{ .Runtime.Goos }}` | Yes | — | N/A — Go-specific |
| `{{ .Runtime.Goarch }}` | Yes | — | N/A — Go-specific |
| `{{ .ConventionalFileName }}` | Yes (nfpm) | — | **Missing** |

### 4.2 Template Functions

| Function | GoReleaser | Anodize | Status |
|----------|-----------|---------|--------|
| `title` | Yes | — | **Missing** |
| `tolower` / `toLower` | Yes | — | **Missing** |
| `toupper` / `toUpper` | Yes | — | **Missing** |
| `trim` | Yes | — | **Missing** |
| `trimprefix` | Yes | — | **Missing** |
| `trimsuffix` | Yes | — | **Missing** |
| `replace` | Yes | — | **Missing** |
| `split` | Yes | — | **Missing** |
| `join` | Yes | — | **Missing** |
| `time` | Yes | — | **Missing** |
| `abs` | Yes | — | **Missing** |
| `dir` | Yes | — | **Missing** |
| `base` | Yes | — | **Missing** |
| `contains` | Yes | — | **Missing** |
| `hasPrefix` | Yes | — | **Missing** |
| `hasSuffix` | Yes | — | **Missing** |
| `filter` / `reverseFilter` | Yes | — | **Missing** |
| `map` | Yes | — | **Missing** |
| `envOrDefault` | Yes | — | **Missing** |
| `isEnvSet` | Yes | — | **Missing** |
| `incmajor` / `incminor` / `incpatch` | Yes | — | **Missing** |
| `mdv2escape` | Yes | — | **Missing** |

### 4.3 Template Syntax

| Feature | GoReleaser | Anodize | Status |
|---------|-----------|---------|--------|
| Variable substitution | `{{ .Var }}` | `{{ .Var }}` and `{{ Var }}` | **Better** — dual syntax |
| Nested access | `{{ .Env.VAR }}` | `{{ .Env.VAR }}` | Parity |
| Conditionals | `{{ if }}...{{ else }}...{{ end }}` | — | **Missing** |
| Range/loops | `{{ range }}...{{ end }}` | — | **Missing** |
| Pipe syntax | `{{ .Version | toupper }}` | — | **Missing** |
| With blocks | `{{ with }}...{{ end }}` | — | **Missing** |
| Comments | `{{/* comment */}}` | — | **Missing** |
| Default values | `{{ .Var | default "fallback" }}` | — | **Missing** |

---

## 5. GitHub Action Parity

| Feature | goreleaser-action | anodize action.yml | Status |
|---------|------------------|--------------------|--------|
| Action type | JavaScript (`node20`) | Composite (bash) | **Gap** — JS is faster, supports caching natively |
| Installation | Pre-built binary download (<5s) | `cargo install` (1-5 min) | **Critical gap** |
| Binary caching | `@actions/tool-cache` | None | **Critical gap** |
| Install-only mode | Yes | Yes | Parity |
| Version pinning | `~> v2` semver constraints | Exact version or `latest` | **Missing** semver ranges |
| Workdir | Yes | Yes | Parity |
| Args | Yes | Yes | Parity |
| Output: artifacts | JSON file path | — | **Missing** |
| Output: metadata | JSON file path | — | **Missing** |
| Grouped log output | `@actions/core` groups | — | **Missing** |
| Cross-platform | Auto-detects runner OS/arch | Requires Rust toolchain on runner | **Gap** |
| Distribution selection | `goreleaser` / `goreleaser-pro` | N/A | N/A (single distribution) |

---

## 6. Test Coverage Comparison

### GoReleaser
- **Unit tests:** Thousands, covering every stage, config variation, and edge case
- **Integration tests:** Real builds against test Go projects
- **E2E tests:** Full release pipeline with snapshot mode
- **Cross-platform:** CI matrix includes Linux, macOS, Windows
- **Test infrastructure:** Extensive test helpers, mock GitHub API, mock Docker
- **Coverage:** High coverage with explicit error path testing

### Anodize
- **Total tests:** 138
- **Unit tests:** Good coverage for core (config parsing, template engine, artifact registry, git, target mapping)
- **Stage tests:** Each stage has 5-16 tests (mostly unit-level with mocked contexts)
- **Integration tests:** 5 (CLI help, version, check, init)
- **E2E tests:** None
- **Cross-platform:** None (Linux only)

| Category | GoReleaser | Anodize | Gap |
|----------|-----------|---------|-----|
| Config parsing edge cases | Extensive | 6 tests | **Missing:** malformed YAML, unknown fields, type mismatches |
| Template engine | Extensive | 10 tests | **Missing:** conditionals, functions, pipes, error cases |
| Build stage | Comprehensive | 6 tests | **Missing:** real cargo builds, cross-compilation, error paths |
| Archive stage | Comprehensive | 6 tests | **Missing:** tar.xz, real file trees, large archives |
| Changelog | Comprehensive | 13 tests | Decent coverage |
| Release (GitHub API) | Mocked API tests | 6 tests | **Missing:** API error handling, upload retry |
| Publish (Homebrew) | Comprehensive | 4 tests | **Missing:** real formula generation edge cases |
| Docker | Comprehensive | 8 tests | **Missing:** real buildx invocations |
| E2E pipeline | Snapshot releases | None | **Missing** |
| Error paths | Extensive | Minimal | **Missing** |
| Cross-platform | Linux/macOS/Windows | Linux only | **Missing** |

---

## 7. Documentation Parity

### GoReleaser (goreleaser.com)
- Getting started guide
- Per-stage documentation pages (20+ pages)
- Full configuration reference with examples
- CI/CD integration guides (GitHub Actions, GitLab CI, Travis, Circle, Drone, etc.)
- Migration guide (from other tools)
- FAQ
- Blog posts
- Community resources
- JSON schema for IDE support
- Shell completion docs
- Deprecation notices

### Anodize
- README.md (basic overview, quick start, CLI reference, GitHub Actions example)
- docs/configuration.md (full config reference with examples)

| Section | GoReleaser | Anodize | Status |
|---------|-----------|---------|--------|
| Getting started | Yes | In README | **Partial** |
| Configuration reference | Yes (comprehensive) | Yes (comprehensive) | **Partial** — exists but less detailed |
| Per-stage documentation | Yes (20+ pages) | No | **Missing** |
| CI/CD integration guides | Yes (multiple platforms) | Basic GH Actions example | **Missing** |
| Migration guide | Yes | No | **Missing** |
| FAQ | Yes | No | **Missing** |
| JSON schema | Yes (+ IDE support) | No | **Missing** |
| Shell completions | Yes | No | **Missing** |
| Docs site | goreleaser.com | No site | **Missing** |

---

## 8. Priority Ranking

### P0 — Must fix before Release 1 (blocks usability)

1. **`--config` / `-f` flag** — Users need to specify non-default config paths
2. **Template conditionals (`if`/`else`)** — Many real-world templates use conditionals
3. **Template functions (`tolower`, `toupper`, `replace`, `trim`)** — Used in name templates constantly
4. **`--timeout` flag** — Long builds can hang CI without a timeout
5. **`make_latest` release field** — GitHub shows "latest" badge; users need to control it
6. **Changelog header/footer** — Very commonly used for custom release notes content
7. **`disable` field on checksum/changelog** — Users need to skip stages per-config, not just via CLI
8. **Auto-detect github owner/name from git remote** — Reduces boilerplate in config

### P1 — Important for parity (users will notice)

9. **Shell completion generation** (`completion` command)
10. **`--parallelism` flag** — Multi-target builds are slow without parallelism
11. **`--auto-snapshot` flag** — Common CI pattern: snapshot on dirty, release on clean
12. **`--single-target` flag** — Essential for local development builds
13. **Additional archive formats** (`tar.xz`, `tar.zst`, `binary`)
14. **Archive glob patterns in `files`** — `files: ["LICENSE*", "README*"]`
15. **Metadata output** (`dist/metadata.json`) — CI integration depends on this
16. **Report sizes** — Trivial to implement, high UX value
17. **Release header/footer** — Distinct from changelog header/footer
18. **Release `extra_files`** — Upload additional files as release assets
19. **`skip_push` on Docker** — Testing Docker builds without pushing
20. **Multiple sign configs** — Different signing for checksums vs. archives
21. **Custom publishers** (`publishers[]`) — Extensibility escape hatch
22. **NFpm scripts** — Pre/post install/remove scripts
23. **More checksum algorithms** — At minimum add `sha1`, `blake2b`

### P2 — Nice to have for full parity

24. **JSON schema output** (`jsonschema` command)
25. **`healthcheck` command** (currently embedded in `check`)
26. **Blob storage upload** (S3/GCS/Azure)
27. **AUR support**
28. **Source archive generation**
29. **SBOM generation**
30. **Chocolatey support**
31. **Winget support**
32. **Krew plugin manifests**
33. **Additional announce providers** (Telegram, Teams, etc.)
34. **Wrap in directory** (archive option)
35. **UPX binary compression**
36. **`env_files` / `.env` loading**
37. **Config schema versioning** (`version: 2`)
38. **Build `ignore` list** — Exclude specific target combos
39. **Build per-target overrides**

### P3 — Release 2 / low priority

40. GitLab/Gitea support
41. Snapcraft
42. Fury.io
43. Nightly builds
44. Config includes
45. Split/merge
46. DMG/MSI/PKG
47. Reproducible builds (`SOURCE_DATE_EPOCH`)
48. macOS Universal Binaries
49. Monorepo support

---

## 9. Anodize Advantages Over GoReleaser OSS

These are features where anodize is **better** than GoReleaser OSS:

1. **Rust workspace native support** — Per-crate release cadences, dependency-aware ordering, workspace change detection. GoReleaser has no equivalent.
2. **crates.io publishing** with index polling — First-class Rust package registry support.
3. **`--dry-run` mode** — Full pipeline with no side effects. GoReleaser lacks this (Pro has `--prepare`).
4. **`copy_from` binary aliasing** — Create multiple binaries from one compilation (e.g., `cfgd` → `kubectl-cfgd`).
5. **Cross-compilation strategy selection** — `auto/zigbuild/cross/cargo` is more explicit than Go's "just works" approach.
6. **Dual template syntax** — `{{ .Var }}` (GoReleaser compat) and `{{ Var }}` (native Tera-style).
7. **`changelog` as a free command** — Pro-only in GoReleaser.
8. **TOML config support** — GoReleaser only supports YAML.
9. **Smart config generation** — `anodize init` reads Cargo.toml workspace structure and generates per-crate configs with dependency ordering. GoReleaser's `init` generates a generic template.

---

## 10. Recommended Action Items

Based on this analysis, the next implementation sessions should focus on:

### Session 1: Template Engine + CLI Flags (P0)
- Add `if`/`else`/`end` conditionals
- Add core template functions: `tolower`, `toupper`, `title`, `replace`, `trim`, `trimprefix`, `trimsuffix`
- Add pipe syntax: `{{ .Version | toupper }}`
- Add `--config`/`-f` flag to all commands
- Add `--timeout` flag
- Add missing template variables: `Branch`, `PreviousTag`, `CommitDate`, `CommitTimestamp`, `RawVersion`, `IsGitDirty`

### Session 2: Config Completeness (P0-P1)
- Add `make_latest` to release config
- Add `header`/`footer` to changelog config
- Add `disable` field to checksum and changelog configs
- Add `extra_files` to release config
- Add `skip_push` to docker config
- Auto-detect github owner/name from git remote
- Add `report_sizes` top-level config
- Add metadata.json output

### Session 3: CLI Completeness + Package Managers (P1)
- Add `completion` command (clap has built-in support)
- Add `jsonschema` command
- Add `--parallelism`, `--auto-snapshot`, `--single-target` flags
- Add `healthcheck` command
- Add additional archive formats (`tar.xz`, `tar.zst`)
- Add glob support for archive `files`

### Session 4: Extensibility + Publishing (P1-P2)
- Add custom publishers (`publishers[]`)
- Add more checksum algorithms
- Add NFpm scripts
- Multiple sign configs
- AUR support
- Source archive generation

### Session 5: Test Coverage (P1)
- E2E test: `anodize release --snapshot` on a real Cargo project
- Error path tests for all stages
- Cross-platform CI (GitHub Actions matrix)
- Mock GitHub API integration tests

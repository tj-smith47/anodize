# Package-manager publishers gap audit (re-graded with disable/skip_upload framing)

Date: 2026-04-25
References: `/opt/repos/goreleaser/internal/pipe/{brew,cask,scoop,winget,chocolatey,aur,aursources,krew,nix}/`, `/opt/repos/goreleaser/pkg/config/config.go`, `/opt/repos/anodizer/crates/{core/src/config.rs,stage-publish/src/}`.

**Re-grading rule:** GoReleaser's per-publisher keying is canonical. Don't flag "missing disable" as a MISSING when GR doesn't have disable on that publisher; the right key per publisher is what GR uses. Authoritative matrix is captured separately in the conversation; below applies that matrix.

## GoReleaser canonical keying (from prior matrix work)

| Pattern | Structs |
|---|---|
| `disable` only | Blob, Changelog, Checksum, DockerDigest, DockerV2, Flatpak, Ko, MCPDetails, Makeself, Publisher, SBOM, Snapcraft |
| `skip_upload` only | Homebrew, HomebrewCask, Krew, Nix, Scoop, Winget |
| `skip_publish` only | Chocolatey |
| Both `disable` + `skip_upload` | AUR, AURSource, Release |

## Anodizer drift (after this session's reverted disable additions)

| Struct | GR uses | Anodizer has | Action (per Group 1 plan tasks) |
|---|---|---|---|
| AurConfig | disable + skip_upload | disable + skip_upload | ✓ matches |
| AurSourceConfig | disable + skip_upload | disable + skip_upload | ✓ matches |
| HomebrewConfig | skip_upload | skip_upload | ✓ matches |
| HomebrewCaskConfig (per-crate, anodizer-only) | n/a | NEITHER | Group 1.3: investigate, decide delete vs add skip_upload |
| TopLevelHomebrewCaskConfig | skip_upload | skip_upload | ✓ matches |
| ScoopConfig | skip_upload | skip_upload | ✓ matches |
| WingetConfig | skip_upload | skip_upload | ✓ matches |
| KrewConfig | skip_upload | disable + skip_upload | Group 1.2: REMOVE disable |
| NixConfig | skip_upload | skip_upload | ✓ matches |
| ChocolateyConfig | skip_publish | disable + skip_publish | Group 1.1: REMOVE disable |
| CratesPublishConfig (anodizer-only) | n/a | enabled: bool | Group 1.4 candidate (no template support) |
| CloudSmithConfig (anodizer-only) | n/a | skip | Group 1.4 candidate |

## Real findings to keep (per-publisher)

### HomebrewConfig (config.rs:2123)
- **Doc/code drift:** `commit_msg_template` doc (config.rs:2166) says default is `"chore: update {{ name }} formula to {{ version }}"`; actual code at homebrew.rs:1283 uses `"Brew formula update for {{ ProjectName }} version {{ Tag }}"`. Doc lies.
- **arm_variant default mystery:** GR brew.go:85-86 defaults `Goarm` to `experimental.DefaultGOARM()`; anodizer hardcodes `arm_variant` default to `"6"` at homebrew.rs:1460. May be wrong.
- **Two-source-of-truth:** `tap` and `repository` can both be set with no validator; one silently dropped.
- **Legacy fields:** `commit_author_name`/`commit_author_email` parallel to `commit_author` struct.
- **Anodizer-only:** `cask` (nests per-crate cask), `plist` (convenience; GR uses `Service`).

### HomebrewCaskConfig per-crate (config.rs:2249) — anodizer-only with no GR analog
- **Disable/skip semantics: NEITHER.** User has no way to skip just the cask without skipping the formula.
- **Smaller field set than TopLevelHomebrewCaskConfig:** missing repository, commit_author, directory, ids, structured url, generate_completions, app, alternative_names. Either delete or unify (Group 1.3).

### TopLevelHomebrewCaskConfig (config.rs:2302)
- **Anodizer-only:** `app`, `alternative_names`.
- **Default to verify:** GR cask.go:65-69 defaults `Directory` to `"Casks"` and warns if changed; anodizer renders the field but does not enforce/default.

### ScoopConfig (config.rs:2472)
- **Legacy:** `bucket` legacy parallel to `repository`.
- **Anodizer-only:** `use_artifact` ("archive"/"msi"/"nsis" selector); GR has no `use:` field.
- **Template fan-out:** GR scoop.go:147-152 calls `tp.ApplyAll` on Name/Description/Homepage/SkipUpload; verify anodizer scoop.rs covers all.

### WingetConfig (config.rs:2642)
- **Anodizer-only:** `manifests_repo` (legacy), `product_code`, `use_artifact`.
- **Validation gap (likely bug):** GR winget.go:37/151 enforces `package_identifier` regex; anodizer winget.rs has no equivalent — invalid IDs would push and be rejected by winget late.
- **Template fan-out:** GR winget.go:115-134 calls `tp.ApplyAll` on ~17 string fields; verify anodizer winget.rs covers all.
- **`ReleaseNotes` template:** GR winget.go:173-178 templates with extra `Changelog` field; anodizer must replicate.

### ChocolateyConfig (config.rs:2527)
- **Anodizer-only:** `project_repo` (GR derives from ProjectURL + git remote), `use_artifact`, `tags` accepts Vec<String> OR space-separated string.
- **Disable field added in 3a0af07 — Group 1.1 removes it.** GR uses `skip_publish` only.
- **Default `Goamd64="v1"` matches.** SourceRepo default matches.
- **Idempotency code:** chocolatey.rs:578-625 — best-in-class hash-match short-circuit. Pattern should be ported to crates_io.rs.
- **Group 2.2 fixes** are tracked separately (moderation-state, 403, no-windows-artifact placeholder).

### AurConfig (config.rs:2731)
- **Anodizer-only:** `replaces: Option<Vec<String>>` (useful upgrade-path field), `url` (legacy, redundant with `homepage`; verify dead code).
- **Defaults to verify in stage:**
  - GR aur.go:55-57 auto-suffixes `Name` with `-bin` if missing.
  - GR aur.go:58-63 defaults `Conflicts`/`Provides` to `[ProjectName]`.
  - GR aur.go:64-66 defaults `Rel = "1"`.
  - GR aur.go:67-69 defaults `Goamd64 = "v1"` ✓ at aur.rs:402.
- **arm_variant=7 hardcode mystery:** aur.rs:400 comment claims GR hardcodes Goarm to "7" for AUR, but GR aur.go:120 uses `artifact.ByGoarm(aur.Goarm)` against the field's default of empty. Either the hardcode is wrong or the comment is.

### AurSourceConfig (config.rs:5924) — REAL MISSING-FIELD
- **MISSING-FIELD: `goamd64`/`amd64_variant`** (config.go:173) — not exposed in anodizer's AurSourceConfig. RPM-style filter, RPM-specific (not Go-only here because the AUR build pipeline does care about the assembled binary's microarch). Add `pub amd64_variant: Option<String>`.
- **Code smell:** aur_source.rs:257 and aur_source.rs:299 duplicate the disable/skip_upload check across per-crate AND top-level publish paths. Extract.

### KrewConfig (config.rs:2785)
- **Anodizer-only:** `manifests_repo` / `upstream_repo` (both legacy — pick one), `disable` (Group 1.2 removes; misleading comment falsely claimed GR parity at config.rs:2814).
- **Defaults to verify:** Default commit_msg `"Krew manifest update for ..."`. Default `Goamd64="v1"`.
- **Validation gap (likely bug):** GR krew.go:86-91 errors out if `description` or `short_description` is empty; verify anodizer krew.rs replicates.

### NixConfig (config.rs:2870)
- **Defaults to verify:**
  - **Likely default-template bug:** GR nix.go:78-79 defaults commit_msg to `"{{ .ProjectName }}: {{ .PreviousTag }} -> {{ .Tag }}"`. Anodizer's default-template helper at homebrew.rs:1283-1286 has no "nix" branch — Nix likely falls into the catch-all `"package" => "Update to {{ Tag }}"` default, which DIFFERS from GR. Verify.
- **Validates License against `validLicenses`** — anodizer has `validate_nix_license` at nix.rs:655 ✓.
- **Code smell:** Reuses `homebrew::should_skip_upload` cross-module. Helper should live in `util.rs`.
- **No `disable` field** — silently drops `nix: { disable: true }` from user YAML. Confirm serde unknown-field behavior on PublishConfig (deny vs warn vs silent-drop).

### CratesPublishConfig (config.rs:2088) — anodizer-only
- **Uses `enabled: bool` instead of `disable: Option<StringOrBool>`.** Inconsistent with other publishers — no template-conditional skip support.
- **Crates_io error is FATAL** (lib.rs:106 `?` instead of `try_publish!`) — intentional per the comment, but document for users.
- **No idempotency:** cargo's own 409/422 is the only safety net. Recent commit 6f24986 indicates same-version drift was hit. Consider porting chocolatey hash-compare pattern.

### CloudSmithConfig (config.rs:5166) — anodizer-only
- **Uses `skip` only** — inconsistent with Artifactory (which has both `skip` AND `disable`). Pick one.

### Legacy `{owner, name}` structs
- TapConfig, BucketConfig, ChocolateyRepoConfig, WingetManifestsRepoConfig, KrewManifestsRepoConfig — all 4-field-shy of GR's `RepoRef` (token, branch, git, pull_request). Long-term: replace with `Option<RepositoryConfig>`.

## Cross-cutting

- **No per-publisher CLI skip flags.** GR supports `--skip=brew`, `--skip=scoop`, etc.; anodizer has only the global publish-stage skip (lib.rs:59). Selective re-publishing is impossible.
- **Top-level vs per-crate divergence:** GR exposes `brews: []`, `scoops: []`, `wingets: []`, `chocolateys: []`, `aurs: []`, `nix: []`, `krews: []` at top-level. Anodizer scopes everything except `homebrew_casks: []` and `aur_sources: []` to per-crate config. Multi-publisher-per-publisher (e.g., production tap + nightly tap) cannot be expressed.
- **Serde unknown-field behavior:** Confirm anodizer either denies or at minimum warns on unknown fields in PublishConfig — otherwise `{ disable: true }` written by users gets silently dropped on structs that don't have the field.

## Summary counts (re-graded)

- **Strict GR-field MISSING:** 1 (AurSourceConfig.amd64_variant).
- **Per-plan keying drift to fix in Group 1:** 2 (KrewConfig, ChocolateyConfig).
- **Per-plan structural gap in Group 1:** 1 (HomebrewCaskConfig per-crate, no skip mechanism).
- **Per-plan anodizer-only structs to align:** 3 (CratesPublishConfig, CloudSmithConfig, ArtifactoryConfig).
- **Default divergences with user impact:** ~5 (commit_msg defaults on Homebrew/Nix, arm_variant hardcode, etc.).
- **Validation gaps (likely bugs):** 2 (winget package_identifier regex, krew description-required).
- **Behavioral / code smells:** ~12.
- **Legacy struct cleanups:** 5.

# Docker / nFPM / Native installers gap audit (re-graded)

Date: 2026-04-25
References: `/opt/repos/goreleaser/internal/pipe/{docker,docker/v2,dockerdigest,nfpm,snapcraft,srpm,makeself,flatpak}/`, `/opt/repos/goreleaser/www/content/customization/package/{dmg,msi,pkg,nsis,app_bundles}.md` (Pro pipe public docs), `/opt/repos/anodizer/crates/{core/src/config.rs,stage-{docker,nfpm,snapcraft,dmg,msi,pkg,nsis,appbundle,flatpak,makeself,srpm}/src/lib.rs}`.

**Re-grading rule applied:** GO-IDIOM-ONLY fields (goos/goarch/goarm/goamd64) NOT counted as MISSING. The original audit flagged 8 such fields as MISSING; on re-grade these are GO-IDIOM-ONLY because anodizer expresses platform filtering through Rust target triples + per-config target filters.

## Summary
- **MISSING-FIELD: 7** (down from original 15 after re-grading 8 GO-IDIOM-ONLY items).
- **Type-variant gap: 1** (NfpmConfig.umask string-only when GR allows int).
- **Default divergence with user impact: 1** (DockerV2.SBOM off-by-default, GR on-by-default).
- **YAML key alias gaps: 5** (top-level `nfpms`/`dmg`/`msi`/`flatpak` + Makeself `filename`).
- **Code smells (production): 5**.

## Top-level YAML key drift

`/opt/repos/anodizer/crates/core/src/config.rs:935-949`

| GoReleaser key | Anodizer field | Drift |
|---|---|---|
| `nfpms` | `nfpm` (Vec) | Singularized; **no `nfpms` alias** |
| `dmg` | `dmgs` | Pluralized; **no `dmg` alias** |
| `msi` | `msis` | Pluralized; **no `msi` alias** |
| `flatpak` | `flatpaks` | Pluralized; **no `flatpak` alias** |
| `snapcrafts` / `pkgs` / `nsis` / `app_bundles` | match | OK |

**4 alias gaps.** A user copying their goreleaser.yaml `nfpms:` block sees configs silently dropped.

## DockerConfig (legacy `dockers`) — config.rs:2900

GoReleaser `Docker` (config.go:1099-1114).
- **Missing fields:** 0 (after re-grade — original flagged Goos/Goarch/Goarm/Goamd64 which are GO-IDIOM-ONLY).
- **Anodizer-only:** `binaries`, `templated_extra_files` (Pro feature; OK).
- **Default divergence:** **DockerRetryConfig doc says "default: 3"** (config.rs:2974) but actual default is **10** (stage-docker/lib.rs:326,328). Doc lies.

## DockerV2Config — config.rs:2997

GoReleaser `DockerV2` (config.go:1131-1146).
- **Missing fields:** 0.
- **Anodizer-only:** `skip_push: Option<StringOrBool>` (config.rs:3028) — GR's v2 pipe always pushes (`internal/pipe/docker/v2/docker.go:155`). Anodizer extension. Document or remove.
- **Default divergence (user-impacting):** **`SBOM` default** — GR sets `"true"` at `internal/pipe/docker/v2/docker.go:85-87`. Anodizer's `is_docker_v2_sbom_enabled()` (stage-docker/lib.rs:645-650) returns `false` for `None`. **Migrating users silently lose attestation.**
- **Defaults to verify in stage:** Platforms `["linux/amd64","linux/arm64"]`, Tags `["{{.Tag}}"]`, ID=ProjectName, Dockerfile="Dockerfile" (anodizer matches the last at stage-docker/lib.rs:1949-1951).

## DockerDigestConfig — config.rs:3043

GR `DockerDigest` (config.go:1149-1152). All fields present. `name_template` fallback `"digests.txt"` matches (digest.go:34-35 ↔ stage-docker/lib.rs:2838).

## DockerManifestConfig — config.rs:3056

GR `DockerManifest` (config.go:1119-1128). 8/8 fields. Anodizer-additive `disable` (config.rs:3077-3078).

## NfpmConfig — config.rs:3088

GR `NFPM` (config.go:683-711).
- **Missing fields:** 0 (after re-grade — original flagged GoAmd64 which is GO-IDIOM-ONLY).
- **Type-variant gap:** **`umask: Option<String>`** at config.rs:3146. GR is `fs.FileMode` with `oneof_type=string;integer`. User writing literal `umask: 0o002` (number) fails to deserialize. **Silent breakage on copy-paste from goreleaser.yaml.** Need `StringOrU32` deserializer.
- **Per-format coverage:** RPM 8/8, Deb 8/8 (+anodizer-only `arch_variant`), APK 2/2, Archlinux 3/3, IPK 7/7, Signature unified (anodizer-only `key_id` — verify wiring).
- **Pro-only Anodizer-additive:** `if_condition`, `templated_contents`, `templated_scripts` (Pro features) — OK.
- **Default divergence:** `Libdirs` — GR applies unconditionally (nfpm.go:59-67); anodizer applies only when libdirs block exists OR library artifacts present (stage-nfpm/lib.rs:443-446). Subtle gap.
- **Defaults verified:** ID="default" ✓, Bindir="/usr/bin" ✓.
- **Defaults to verify in stage:** PackageName=ProjectName, FileNameTemplate, Maintainer deprecation warning.
- **Code smells (production):**
  - **stage-nfpm/lib.rs:647** — `serde_yaml_ng::to_string(...).unwrap_or_else(|e| panic!(...))`. **Production `panic!`** outside test code. The anti-patterns hook regex misses `panic!` (only catches `unwrap`/`expect`). Convert to `Result<String>` and `?`-propagate.
  - stage-nfpm/lib.rs:551-557 — `.ok()?` on JSON→YAML round-trip silently drops `overrides:` entries that fail. Should `tracing::warn!` at minimum.

## NfpmContent vs NfpmContentConfig — DRY violation

Two parallel content structs with different field names and optionality:
- `NfpmContent` (config.rs:3215) — fields: `src, dst, content_type, file_info, packager, expand`.
- `NfpmContentConfig` (config.rs:5868) — fields: `source` (alias src), `destination` (alias dst), `type_`, `packager`. Used by SrpmConfig.

GoReleaser uses one (`NFPMContent` config.go:863). Unify.

## SnapcraftConfig — config.rs:3475

GR `Snapcraft` (config.go:1009-1035). 19/19 top-level + SnapcraftApp 31/31 + SnapcraftLayout 4/4 + SnapcraftExtraFiles 3/3.
- **Anodizer-only top-level fields:** `slots: Option<Vec<String>>` (config.rs:3509) — at top-level (GR has it only per-app at SnapcraftAppMetadata.Slots). Valid snap YAML. `replace`, `mod_timestamp` — anodizer-only.
- **Defaults verified:** Grade="stable" ✓, Confinement="strict" ✓, ChannelTemplates by-grade ✓.

## DmgConfig — config.rs:3681

GR Pro doc: `www/content/customization/package/dmg.md`. 10/10 documented Pro fields present.
- **Missing fields:** 0 on re-grade (original flagged `goamd64` which is GO-IDIOM-ONLY).

## MsiConfig — config.rs:3717

Pro doc: `www/content/customization/package/msi.md`.
- **Missing fields:** 0 on re-grade.
- **Anodizer-only:** `if_condition` (config.rs:3742) — IS in dmg.md but not in msi.md. Verify against pro source.
- **Anodizer-additive:** `hooks: BuildHooksConfig` (Pro v2.14+ feature — OK).
- **Defaults to verify:** `Name` default `"{{.ProjectName}}_{{.MsiArch}}"` (msi.md:21).

## PkgConfig — config.rs:3755

Pro doc: `www/content/customization/package/pkg.md`. All Pro fields present.
- **Anodizer-additive:** `extra_files`, `min_os_version`, `disable` (not in pkg.md).
- **Default verified:** `install_location` default `/usr/local/bin` ✓.

## NsisConfig — config.rs:3794

Pro doc: `www/content/customization/package/nsis.md`.
- **Missing fields:** 0 on re-grade.
- **Anodizer-additive:** `if_condition` (not in nsis.md).
- **Defaults to verify:** `Name` default `"{{.ProjectName}}_{{.Arch}}_setup"` (nsis.md:23-26).

## AppBundleConfig — config.rs:3828

Pro doc: `www/content/customization/package/app_bundles.md`. 9/9.
- **Anodizer-additive:** `replace`, `disable`.

## FlatpakConfig — config.rs:3866

GR `Flatpak` (config.go:1045-1069) + Pro flatpak.md. 9/9.
- **Anodizer-only:** `extra_files`, `replace`, `mod_timestamp` (not in GR).
- **Required-validation TODO:** GR errors when AppID/Runtime/RuntimeVersion/SDK are empty (flatpak.go:86-97); anodizer's are `Option<String>` — verify runtime validation exists.

## MakeselfConfig — config.rs:5705

GR `Makeself` (config.go:1527-1544). 16/16.
- **Field-name drift (alias gap):** GR yaml is `filename` (config.go:1529); anodizer renames to `name_template` (config.rs:5711). **No `#[serde(alias = "filename")]`.** Migrating user's `filename: foo` is silently dropped.
- **Defaults to verify:** ID="default", Filename template, Name="{{.ProjectName}}", Goos=["linux","darwin"].

## SrpmConfig — config.rs:5811 (BIGGEST gaps)

GR `SRPM` (config.go:872-892, embeds `NFPMRPM`).

- **Missing fields (LEGITIMATE — these are RPM-spec-specific, not Go-toolchain-specific):**
  1. **`Bins map[string]string`** (config.go:889) — GR feeds the `{{.Bins}}` template var; default `{ProjectName: "%{goipath}"}` (srpm.go:45-49). Without it, drop-in spec templates referencing `{{.Bins}}` are functionally broken.
  2. **`ImportPath string`** (config.go:878) — GR feeds `{{.ImportPath}}` (srpm.go:75).
  3. **`Prefixes []string`** (NFPMRPM, config.go:746) — Relocatable RPM prefixes.
  4. **`BuildHost string`** (NFPMRPM, config.go:748) — RPM BuildHost tag.
  5. **`Scripts.Pretrans` / `Scripts.Posttrans`** (NFPMRPM, config.go:745) — Cannot define pretrans/posttrans for SRPM.
  6. **`Prerelease`** — Cannot tag prerelease in SRPM filename.
  7. **`VersionMetadata`** — Cannot tag version metadata in SRPM filename.

- **DRY violation (related):** `SrpmSignatureConfig` (config.rs:5857) forks `NfpmSignatureConfig` (config.rs:3454). GR uses `NFPMRPMSignature` for srpm. Unify and gain missing fields free.

- **Defaults to verify:** PackageName=ProjectName, FileNameTemplate="{{.PackageName}}-{{.Version}}.src.rpm", source-archive mode 0o664, spec file mode 0o660.

## Code smells (production, summarized)

1. **Production `panic!` at stage-nfpm/lib.rs:647** (anti-patterns hook regex misses `panic!`).
2. **stage-nfpm/lib.rs:551-557** — silent `.ok()?` drops bad `overrides:` entries.
3. **DRY: NfpmContent vs NfpmContentConfig** (config.rs:3215 vs 5868).
4. **DRY: NfpmSignatureConfig vs SrpmSignatureConfig** (config.rs:3454 vs 5857).
5. **Doc bug: DockerRetryConfig.attempts comment "default: 3"** vs actual default 10.

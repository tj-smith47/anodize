# Session A — categorization (a/b/c/d)

Date: 2026-04-25
Source audits (all in this directory):
- `build-archive-source.md`
- `docker-nfpm-installers.md`
- `publishers-pkgmgr.md`
- `release-integrity_pass-a.md` (release + changelog + milestone)
- `release-integrity_pass-b.md` (checksum + sign + notarize + sbom)
- `infra-announcers_pass-a.md` (artifactory + dockerhub + upload + blob)
- `infra-announcers_pass-b.md` (announcers x13)

## Category legend

- **(a)** production bug fix — apply inline this session
- **(b)** config-schema change — hand off to Session B
- **(c)** publisher-behavior change — hand off to Session C
- **(d)** docs/comment fix — apply inline this session
- **(done)** already shipped earlier this session (per `PLAN.md` "Already done")

## Re-grading rule

Go-toolchain-specific fields (goos/goarch/goarm/goamd64/gomod/ldflags/gcflags) are NOT MISSING-FIELD findings. Anodizer expresses platform filtering through Rust target triples and `targets:` glob.

---

## Audit 1: build-archive-source.md

| # | Finding | Site | Bucket |
|---|---|---|---|
| 1 | `BuildConfig.flags: Option<String>` mis-splits quoted args | stage-build/lib.rs:182,240 | (b) |
| 2 | universal `mod_timestamp` swallows template errors | stage-build/lib.rs:528 | (a) |
| 3 | universal metadata-copy whitelist brittle | stage-build/lib.rs:548 | (c) |
| 4 | exact-match `build.env.get(target)` (no glob) | stage-build/lib.rs:1622 | (c) |
| 5 | `eprintln!` reproducible-epoch warning bypasses tracing | stage-build/lib.rs:1042 | (a) |
| 6 | universal id-fallback parity comment misleading | stage-build/lib.rs:370-394 | (d) |
| 7 | universal metadata both `id` + `binary` (GR id only) | stage-build/lib.rs:380-389 | (c) |
| 8 | universal `register_artifact name: String::new()` | stage-build/lib.rs:565 | (a) |
| 9 | `archives: []` skip vs GR auto-inject default | config.rs:983 | (c) |
| 10 | archive `id` round-trip differs (None vs `"default"`) | config.rs / stage-archive | (a) |
| 11 | `FormatOverride` exact `==` vs GR `HasPrefix` | stage-archive/lib.rs:774 | (c) |
| 12 | `eprintln!` mtime warning bypasses StageLogger | stage-archive/lib.rs:62 | (a) |
| 13 | default extra-file glob order differs from GR | stage-archive/lib.rs:560-567 | (c) |
| 14 | `defaults.archives` only carries `format`+overrides | stage-archive/lib.rs:838-845 | (b) |
| 15 | empty-binaries-after-filter silent skip (GR warns) | stage-archive/lib.rs:1108 | (a) |
| 16 | stage-archive 1700-line monolith | stage-archive/lib.rs | (a) |
| 17 | source template error swallow | stage-source/lib.rs:677 | (a) |
| 18 | `#[allow(too_many_arguments)]` source archive | stage-source/lib.rs:34 | (a) |
| 19 | dry-run skips glob validation | stage-source/lib.rs:651 | (a) |
| 20 | `SourceFileInfo.mode: u32` vs `ArchiveFileInfo.mode: String` | config.rs:4083 / 1380 | (b) |
| 21 | UPX missing-file → "100%" compression metadata | stage-upx/lib.rs:181,232 | (a) |
| 22 | UPX `compress: "abc"` no validation | stage-upx/lib.rs:140 | (a) |
| 23 | UPX universal-binary ordering undocumented | stage-upx/lib.rs:108-109 | (d) |
| 24 | UPX `id` metadata filter assumes default-id | stage-upx/lib.rs:117 | (a) |
| 25 | universal binary loop silent overwrite duplicate filename | stage-build/lib.rs:2292-2296 | (a) |
| 26 | `CrateConfig.docker` legacy alongside docker_v2 | config.rs:927 | (b) |

**Counts:** (a) 13 · (b) 4 · (c) 6 · (d) 3

---

## Audit 2: docker-nfpm-installers.md

| # | Finding | Site | Bucket |
|---|---|---|---|
| 1 | `nfpms`/`dmg`/`msi`/`flatpak` top-level alias gaps | config.rs:935-949 | (b) |
| 2 | DockerRetryConfig doc "default 3" vs actual 10 | config.rs:2974, stage-docker:326,328 | (d) |
| 3 | DockerV2 `skip_push` not in GR (anodizer-extra) | config.rs:3028 | (b) |
| 4 | DockerV2 SBOM default off (GR on) | stage-docker/lib.rs:645-650 | (c) |
| 5 | `NfpmConfig.umask` String-only (GR int\|string) | config.rs:3146 | (b) |
| 6 | nFPM Libdirs conditional vs GR unconditional | stage-nfpm/lib.rs:443-446 | (c) |
| 7 | Production `panic!` at stage-nfpm/lib.rs:647 | stage-nfpm/lib.rs:647 | (a) |
| 8 | nFPM overrides `.ok()?` silent drop | stage-nfpm/lib.rs:551-557 | (a) |
| 9 | NfpmContent vs NfpmContentConfig DRY | config.rs:3215 vs 5868 | (b) |
| 10 | NfpmSignatureConfig vs SrpmSignatureConfig DRY | config.rs:3454 vs 5857 | (b) |
| 11 | Snapcraft top-level `slots` not in GR | config.rs:3509 | (b) |
| 12 | DMG/MSI/NSIS `if_condition` Pro-doc presence inconsistent | dmg/msi/nsis | (d) |
| 13 | Flatpak required-fields validation gap | flatpak.rs | (a) |
| 14 | Makeself `filename` alias gap | config.rs:5711 | (b) |
| 15 | SrpmConfig 7 RPM-spec missing fields (Bins, ImportPath, Prefixes, BuildHost, Pretrans, Posttrans, Prerelease, VersionMetadata) | config.rs:5811 | (b) |

**Counts:** (a) 3 · (b) 7 · (c) 2 · (d) 2

---

## Audit 3: publishers-pkgmgr.md

| # | Finding | Site | Bucket |
|---|---|---|---|
| 1 | KrewConfig.disable removal | config.rs:2785 | (done) |
| 2 | ChocolateyConfig.disable removal | chocolatey.rs | (done) |
| 3 | Per-crate HomebrewCaskConfig skip_upload | config.rs:2249 | (done) |
| 4 | Homebrew commit_msg_template doc/code drift | config.rs:2166 vs homebrew.rs:1283 | (d) |
| 5 | Homebrew arm_variant default "6" hardcode | homebrew.rs:1460 | (c) |
| 6 | Homebrew `tap` + `repository` two-source-of-truth | config.rs | (a) |
| 7 | Homebrew legacy commit_author_{name,email} parallel | config.rs | (b) |
| 8 | TopLevelHomebrewCaskConfig `Directory="Casks"` enforcement | config.rs:2302 | (c) |
| 9 | Scoop legacy `bucket` parallel to `repository` | config.rs:2472 | (b) |
| 10 | Scoop template fan-out missing fields | scoop.rs | (a) |
| 11 | Winget `package_identifier` regex validation gap | winget.rs | (a) |
| 12 | Winget template fan-out missing fields | winget.rs | (a) |
| 13 | Winget ReleaseNotes Changelog template | winget.rs | (a) |
| 14 | Chocolatey `tags` accepts Vec<String> OR string | config.rs | (b) |
| 15 | Chocolatey idempotency port to crates_io | crates_io.rs | (c) |
| 16 | AurConfig `replaces` (anodizer-only, OK) | config.rs:2731 | — verified |
| 17 | AurConfig `url` legacy redundant with `homepage` | config.rs:2731 | (b) |
| 18 | AUR `Name` auto-suffix `-bin` if missing | aur.rs | (c) |
| 19 | AUR default Conflicts/Provides=[ProjectName] | aur.rs | (c) |
| 20 | AUR default Rel="1" | aur.rs | (c) |
| 21 | AUR arm_variant=7 hardcode mystery | aur.rs:400 | (a) |
| 22 | AurSourceConfig MISSING amd64_variant | config.rs:5924 | (b) |
| 23 | AurSource disable/skip_upload duplicated check | aur_source.rs:257,299 | (a) |
| 24 | KrewConfig manifests_repo/upstream_repo legacy | config.rs:2785 | (b) |
| 25 | KrewConfig description-required validation gap | krew.rs | (a) |
| 26 | NixConfig commit_msg default differs from GR | nix.rs | (a) |
| 27 | NixConfig reuses homebrew::should_skip_upload | nix.rs | (a) |
| 28 | NixConfig no `disable` field — silent serde drop | nix.rs | (a) |
| 29 | CratesPublishConfig enabled bool inconsistent | config.rs:2088 | (b) |
| 30 | CratesPublishConfig no idempotency | crates_io.rs | (c) |
| 31 | CloudSmithConfig `skip` only inconsistent | config.rs:5166 | (b) |
| 32 | TapConfig/BucketConfig/etc. legacy 4-field shy of RepoRef | config.rs | (b) |
| 33 | No per-publisher CLI skip flags | lib.rs:59 | (b) |
| 34 | Top-level `brews:`/`scoops:`/etc. multi-publisher pattern | config.rs | (b) |
| 35 | Serde unknown-field behavior unconfirmed | PublishConfig | (a) |

**Counts:** (a) 11 · (b) 11 · (c) 6 · (d) 1 · (done) 3

---

## Audit 4: release-integrity_pass-a.md (release + changelog + milestone)

### release pipe

| # | Finding | Site | Bucket |
|---|---|---|---|
| R1 | `name_template` default surface differs (`{{.Tag}}` vs `{{ Tag }}`) | stage-release/lib.rs:1230 | (d) |
| R2 | `release.github.owner`/`name` not template-rendered | config.rs:1676, stage-release/lib.rs:1093+ | (a) |
| R3 | `prerelease == "auto"` per-tag vs Default()-time global | stage-release/lib.rs:418-425 | (c) |
| R4 | No `ctx.ReleaseURL` template variable for downstream stages | stage-release | (a) |
| R5 | Body template structure: GR `{Header}\n\n{Body}` vs anodizer `{Header}\n{Body}` | stage-release/lib.rs:434-464 | (a) |
| R6 | `Checksums` map keys by absolute path (env-dependent) for unmarked artifacts | stage-release/lib.rs:1141-1156 | (a) |
| R7 | Empty-after-IDs-filter verbose-only (no warn) | stage-release/lib.rs:1317-1324 | (a) |
| R8 | `release.github`/`gitlab`/`gitea` per-crate (not global) — intentional, documented | config.rs:392-431 | — verified |
| R9 | `skip_upload: "yes"` falls through to false instead of error | config.rs:1574,1583 | (a) |
| R10 | `skip_upload` resolution duplicates `StringOrBool` parsing | stage-release/lib.rs:1240-1256 | (a) |
| R11 | `skip_upload` template-render error swallow | stage-release/lib.rs:1247 | (a) |
| R12 | `make_latest` template-render error swallow | stage-release/lib.rs:557 | (a) |
| R13 | `eprintln!` insecure-TLS warning bypasses StageLogger | stage-release/lib.rs:937 | (a) |
| R14 | stage-release/lib.rs 5732 lines — split | stage-release/lib.rs | (a) |

### changelog pipe

| # | Finding | Site | Bucket |
|---|---|---|---|
| C1 | Snapshot mode runs changelog (GR skips) | stage-changelog/lib.rs:842-843 | (c) |
| C2 | Default `Format` SCM-mode uses `{{ ShortSHA }}` (GR uses `.SHA` full) | stage-changelog/lib.rs:382 | (c) |
| C3 | `## Changelog` title escape-hatch when `title=""` (GR has none) | stage-changelog/lib.rs:398-403 | (c) |
| C4 | Changelog header/footer go to disk only, not release notes | stage-changelog/lib.rs:1138, 1156-1169 | (c) |
| C5 | No empty-after-render warn for header/footer/notes file load | cli/release/mod.rs + stage-changelog | (a) |
| C6 | github-native auth/repo check deferred to release stage | stage-changelog/lib.rs:904-918 | (a) |
| C7 | SCM changelogers always API-call when no previous tag (GR pre-empts) | stage-changelog/lib.rs:1034-1091 | (c) |
| C8 | `Authors`/`Logins` per-entry deduplication missing | stage-changelog/lib.rs:1130 | (b) |
| C9 | `--release-notes` path bypasses dist-write logic | stage-changelog/lib.rs:849-855, 875-884 | (a) |
| C10 | `Filters.{Include,Exclude}` halts on bad regex (GR continues) | stage-changelog/lib.rs:124-126,154-156 | (a) |
| C11 | `header`/`footer` String only (release uses ContentSource) — asymmetric | config.rs:4288-4290 | (b) |
| C12 | Snapshot-always-on should be opt-in | stage-changelog/lib.rs:842-843 | (b) |
| C13 | SCM-mode `ShortSHA` semantic-swap of `SHA` template var | stage-changelog/lib.rs:382 | (a) |
| C14 | Combined-vs-per-crate two-truth output build | stage-changelog/lib.rs:1138, 1140 | (a) |
| C15 | `fetch_git_commits` `unwrap_or_default()` swallows errors | stage-changelog/lib.rs:1186-1195 | (a) |

### milestone pipe

| # | Finding | Site | Bucket |
|---|---|---|---|
| M1 | `name_template` default surface differs (`{{.Tag}}` vs `{{ Tag }}`) | milestones.rs:28 | (d) |
| M2 | `Repo` resolution publish-time only (no Default()-time persistence) | milestones.rs:108-172 | (c) |
| M3 | Empty-after-render `name` skipped (GR doesn't) | milestones.rs:33-36 | (c) |
| M4 | Tokio runtime created per close call (3 runtimes per milestone) | milestones.rs:185, 313, 399 | (a) |
| M5 | `resolve_milestone_api_url` strip-then-append (brittle) | milestones.rs:279-298 | (a) |
| M6 | Empty repo + no-fail_on_error: anodizer permissive vs GR strict | milestones.rs:43-49 | (c) |
| M7 | `close==false` continue (GR halts) — anodizer correct, GR bug | milestones.rs:21-23 | — verified |
| M8 | "Milestone not found → success" silent (no verbose log) | milestones.rs:243-249, 358-361, 443-446 | (a) |
| M9 | Gitea PATCH includes `title` (round-trips title rename concern) | milestones.rs:457 | (a) |
| M10 | "Any release block" fallback iterates all crates twice | milestones.rs:147-161 | (a) |

**Counts (audit 4):** (a) 18 · (b) 3 · (c) 11 · (d) 3 · — verified 2

---

## Audit 5: release-integrity_pass-b.md (checksum + sign + notarize + sbom)

### checksum pipe

| # | Finding | Site | Bucket |
|---|---|---|---|
| K1 | `name_template` lazy default vs GR eager (round-trip differs) | stage-checksum/lib.rs:482-490 | (a) |
| K2 | Artifact-source kinds narrower than GR (missing Makeself/Flatpak/SourceRpm/Signature/Certificate/UploadableFile) | stage-checksum/lib.rs:261-272 | (c) |
| K3 | Extra-file synthetic kind label uses `Archive` (GR uses `UploadableFile`) | stage-checksum/lib.rs:296 | (a) |
| K4 | Algorithm not validated at load time | stage-checksum/lib.rs:108-125 | (a) |
| K5 | `split` + non-`{{ .ArtifactName }}` template silent overwrite | stage-checksum/lib.rs:400-419 | (a) |
| K6 | `unwrap_or("unknown")` for non-UTF8 filename | stage-checksum/lib.rs:374 | (a) |
| K7 | `extra_name_template` render error swallow | stage-checksum/lib.rs:390 | (a) |
| K8 | Extra-file checksummed once per crate in workspace runs | stage-checksum/lib.rs:289-303 | (a) |
| K9 | Checksum artifact `name: String::new()` | stage-checksum/lib.rs:443, 527 | (a) |

### sign pipe

| # | Finding | Site | Bucket |
|---|---|---|---|
| S1 | `binary_signs` reuses SignConfig; `artifacts` not enum-constrained | config.rs:93, 4441 | (b) |
| S2 | `signs.cmd` defaults gpg/git override (parity OK) | stage-sign/lib.rs:140-153, 414-418 | — verified |
| S3 | Default Args use Tera placeholders not shell `$artifact` | stage-sign/lib.rs:427-434 | (d) |
| S4 | Default Signature template lazy vs GR eager | stage-sign/lib.rs:99 | (a) |
| S5 | docker_signs default cmd `cosign` (parity) | stage-sign/lib.rs:977-981 | — verified |
| S6 | `docker_signs.artifacts` warn+fallback vs GR error | stage-sign/lib.rs:1028-1037 | (a) |
| S7 | `binary_signs` Signature template form differs (`${artifact}` vs `{{ .Artifact }}`) | stage-sign/lib.rs:26 | (d) |
| S8 | `docker_signs.artifacts` no upfront validation | stage-sign/lib.rs:1028-1037 | (a) |
| S9 | `signs.artifacts: none` records skip but downstream still sees entries | stage-sign/lib.rs:408-411 | (a) |
| S10 | `signs.ids` no-effect warning when artifacts=checksum/source | stage-sign | (a) |
| S11 | `signs.env` template render error swallow | stage-sign/lib.rs:712-718 | (a) |
| S12 | `signs.env` HashMap loses ordering; can't chain envs | config.rs:4456 | (b) |
| S13 | `sig_path.file_name() unwrap_or("")` | stage-sign/lib.rs:643 | (a) |
| S14 | `default_sign_cmd` shells out per config (GR caches) | stage-sign/lib.rs:140-153 | (a) |
| S15 | shell_vars iter promotes all into env | stage-sign/lib.rs:725-732 | (a) |
| S16 | SignStage + BinarySignStage may double-execute binary_signs | stage-sign/lib.rs:892-907 | (a) |
| S17 | stage-sign/lib.rs 3729 lines — split | stage-sign/lib.rs | (a) |

### notarize pipe

| # | Finding | Site | Bucket |
|---|---|---|---|
| N1 | Timeout default `"10m"` String vs GR Duration | stage-notarize/lib.rs:294, 511-515 | (b) |
| N2 | IDs default to project name (parity) | stage-notarize/lib.rs:301-307 | — verified |
| N3 | Hard-coded Apple timestamp URL | stage-notarize/lib.rs:351-352 | (a) |
| N4 | MacOSSign cert/password validated only at Run | stage-notarize/lib.rs:262-267 | (a) |
| N5 | No guard against `macos` + `macos_native` both populated | stage-notarize/lib.rs:200-225 | (a) |
| N6 | `is_disabled` swallows template render errors | stage-notarize/lib.rs:206-211 | (a) |
| N7 | `enabled` + top-level `disable` doubled surface | stage-notarize/lib.rs:248 | (b) |
| N8 | API key file not stat-checked | stage-notarize/lib.rs:280-292 | (a) |
| N9 | `macos_native.use_` no enum validation | config.rs:4012 | (b) |
| N10 | sign args list imperative push (refactor) | stage-notarize/lib.rs:343-358 | (a) |
| N11 | `--max-wait` only flag gated on `wait` | stage-notarize/lib.rs:390-407 | (a) |
| N12 | `"10m"` literal duplicated; lift to const | stage-notarize/lib.rs:294, 511, 512-515 | (a) |
| N13 | `sensitive_flags` allow-list too narrow | stage-notarize/lib.rs:104 | (a) |
| N14 | notarize artifact-source includes UploadableBinary (sign divergence) | stage-notarize/lib.rs:309-322 | (a) |
| N15 | `xcrun stapler` no `cfg!(target_os = "macos")` precondition | stage-notarize/lib.rs:722, 886 | (a) |

### sbom pipe

| # | Finding | Site | Bucket |
|---|---|---|---|
| B1 | env HashMap loses order (same as sign) | config.rs:4202 | (b) |
| B2 | `artifacts: binary` default documents template (parity) | stage-sbom/lib.rs:354-356 | — verified |
| B3 | `artifacts` warn+fallback vs GR error (typos silent) | stage-sbom/lib.rs:444-458 | (a) |
| B4 | Built-in Cargo.lock SBOM mode (anodizer-only, documented) | stage-sbom/lib.rs:339, 674-718 | — verified |
| B5 | Multi-document validation (parity) | stage-sbom/lib.rs:364-370 | — verified |
| B6 | Documents abs-path inside dist not handled like GR | stage-sbom/lib.rs:550-562, 625 | (a) |
| B7 | `env` ordering / template-chain (cross-cut) | config.rs:4202 | (b) |
| B8 | passthrough env vars list duplicated; lift to core | stage-sbom/lib.rs:596-605 | (a) |
| B9 | args render `unwrap_or_else` swallows template errors | stage-sbom/lib.rs:572-574 | (a) |
| B10 | binary-mode dedup loop hand-rolled | stage-sbom/lib.rs:411-439 | (a) |
| B11 | Sbom artifact `name: String::new()` | stage-sbom/lib.rs:638-649 | (a) |
| B12 | Built-in spdx vs cyclonedx by substring match | stage-sbom/lib.rs:684-692 | (a) |
| B13 | Raw stderr in error messages (no trim) | stage-sbom/lib.rs:619-620 | (a) |
| B14 | `clear_artifact_vars()` should live in Context | stage-sbom/lib.rs:662-668 | (a) |

**Counts (audit 5):** (a) 33 · (b) 6 · (c) 1 · (d) 2 · — verified 6

---

## Audit 6: infra-announcers_pass-a.md (artifactory + dockerhub + upload + blob)

### artifactory

| # | Finding | Site | Bucket |
|---|---|---|---|
| A1 | Password env-first cascade vs GR config-first | artifactory.rs:407-414 | (a) |
| A2 | Username env-fallback only when None (empty-string disables fallback) | artifactory.rs:400-405 | (a) |
| A3 | `checksum_header` default `X-Checksum-SHA256` parity | artifactory.rs:417-420 | — verified |
| A4 | `render_artifact_url` hand-rolled string-replace (no full template) | artifactory.rs:179-213 | (a) |
| A5 | name-append produces double-name when template uses ArtifactName | artifactory.rs:198-205 | (a) |
| A6 | JSON error parser ignores `errors[].status` | artifactory.rs:308-322 | (a) |
| A7 | 1119-line module mixes generic + specific helpers | artifactory.rs:1 | (a) |
| A8 | `artifact_kinds_for_mode("archive")` omits SRPM/SBOM/Snap/DiskImage/Installer/MSI/PKG | artifactory.rs:33-44 | (a) |
| A9 | Dry-run renders via ctx.render_template; live uses string-replace (drift) | artifactory.rs:436-497 | (a) |
| A10 | `basic_auth` only when both fields non-empty (silent anonymous upload) | artifactory.rs:266-269 | (a) |
| A11 | PEM empty-after-parse hard-error (GR soft-skips) | artifactory.rs:160-164 | (a) |
| A12 | `validate_upload_mode` case-sensitive (GR lowers) | artifactory.rs:367-368 | (a) |

### dockerhub

| # | Finding | Site | Bucket |
|---|---|---|---|
| D1 | `secret_name` default `DOCKER_PASSWORD` (parity) | dockerhub.rs:140 | — verified |
| D2 | `username` no `DOCKER_USERNAME` env fallback | dockerhub.rs:74-79 | (a) |
| D3 | `description` doesn't infer from metadata | dockerhub.rs:131-137 | (a) |
| D4 | `resp.text().unwrap_or_default()` swallows body-read error | dockerhub.rs:32-39 | (a) |
| D5 | Bare image name proceeds; GR doc warns | dockerhub.rs:178-183 | (a) |
| D6 | UTF-8 byte length vs grapheme for short-description warn | dockerhub.rs:90-95 | (a) |
| D7 | Fresh `Client::new()` per entry (no pool) | dockerhub.rs:119 | (a) |
| D8 | `secret_name` env not validated before dry-run skip | dockerhub.rs:74-79 | (a) |
| D9 | Bare image names warn but don't block live PATCH | dockerhub.rs:97-105 | (a) |
| D10 | `image.contains('/')` accepts multi-slash invalid path | dockerhub.rs:99 | (a) |
| D11 | No exactly-one validation for `from_url` vs `from_file` | dockerhub.rs:46 | (a) |

### upload (custom publisher)

| # | Finding | Site | Bucket |
|---|---|---|---|
| U1 | Field name `disable` vs GR `skip` — no alias | config.rs:5900 | (b) |
| U2 | Password env-first cascade (same as A1) | upload.rs:67-74 | (a) |
| U3 | `name` defaults to `"upload"` (GR errors) | upload.rs:30 | (a) |
| U4 | Empty `target` `bail!` (GR `Skip`) | upload.rs:37-39 | (a) |
| U5 | Cross-module `crate::artifactory::...` import | upload.rs:6 | (a) |
| U6 | Empty-Vec early-return repeated 3 sites | upload.rs:14-19 | (a) |
| U7 | Empty-artifacts log level inconsistent (verbose vs status) | upload.rs:97-103 | (a) |
| U8 | Render-target duplicates artifactory.rs with drift | upload.rs:135-165 | (a) |
| U9 | No cross-validate username+password pair | upload.rs:60-74 | (a) |
| U10 | No mTLS pair check | upload.rs | (a) |
| U11 | `validate_upload_mode` case-sensitive (same as A12) | upload.rs:33-34 | (a) |
| U12 | No `trusted_certificates` PEM-empty check | upload.rs | (a) |
| U13 | `password` doc doesn't note cascade-target role | config.rs:5870 | (d) |

### blob

| # | Finding | Site | Bucket |
|---|---|---|---|
| L1 | `Directory` default uses Tera not Go-template (round-trip surface) | stage-blob/lib.rs:747 | (d) |
| L2 | `ContentDisposition` default omitted (intentional, documented) | stage-blob/lib.rs:412-422 | — verified |
| L3 | `s3_force_path_style` applies when endpoint empty | stage-blob/lib.rs:296-298 | (a) |
| L4 | `release_uploadable_kinds()` excludes installers (DMG/MSI/PKG/NSIS) | crates/core/src/artifact.rs:403-418 | (a) |
| L5 | `include_meta` adds Metadata kind (parity) | stage-blob/lib.rs:492-494 | — verified |
| L6 | `encrypt_with_kms` shells out to aws/gcloud/az; no preflight check | stage-blob/lib.rs:90-251 | (a) |
| L7 | Three nearly-identical `Command::new` blocks for KMS providers | stage-blob/lib.rs:99-114, 151-166, 206-225 | (a) |
| L8 | `crate_name` filter couples blob to per-crate model | stage-blob/lib.rs:497-500 | (a) |
| L9 | `id` filter checks both `metadata["id"]` and `metadata["name"]` | stage-blob/lib.rs:506-516 | (a) |
| L10 | `format_remote_path` duplicates `Provider::display_name` | stage-blob/lib.rs:592-604, 42-48 | (a) |
| L11 | Tokio Runtime per blob job (N runtimes) | stage-blob/lib.rs:528-590 | (a) |
| L12 | S3 ACL set as default header (per-write more flexible) | stage-blob/lib.rs:317-346 | (a) |
| L13 | GCS ACL no enum validation | stage-blob/lib.rs:355-377 | (a) |
| L14 | BlobJob clones Vecs (could be Arc<Vec>) | stage-blob/lib.rs:660-665 | (a) |
| L15 | `provider` enum check after template render (Phase 1 partway) | stage-blob/lib.rs:720-725, 30-40 | (a) |
| L16 | S3 ACL whitelist comment is sole doc of choice | stage-blob/lib.rs:330-336 | (d) |
| L17 | No validation `kms_key:` URL scheme matches `provider:` | stage-blob | (a) |
| L18 | No validation `cache_control` directives RFC-7234 | stage-blob | (a) |
| L19 | content-disposition references unrelated artifact vars resolve to "" | stage-blob/lib.rs:412-422 | (a) |

**Counts (audit 6):** (a) 41 · (b) 1 · (c) 0 · (d) 3 · — verified 4

---

## Audit 7: infra-announcers_pass-b.md (announcers x13)

### discord
| # | Finding | Site | Bucket |
|---|---|---|---|
| AN1 | `author` default `anodizer` (brand divergence) | stage-announce/lib.rs:166 | (c) |
| AN2 | `icon_url` default omitted (intentional, no hosted avatar) | stage-announce/lib.rs:187 | — verified |
| AN3 | `require_rendered` error msg doesn't list env-var fallbacks | stage-announce/lib.rs:160 | (a) |
| AN4 | `author: { icon_url }` without `name` may be rejected | stage-announce/discord.rs:31-40 | (a) |
| AN5 | Color parser rejects negative ints | stage-announce/lib.rs:169-186 | (a) |

### slack
| AN6 | `username` default `anodizer` (brand divergence) | stage-announce/lib.rs:265 | (c) |
| AN7 | `icon_emoji`/`icon_url` not template-rendered (others are) | stage-announce/lib.rs:266-267 | (a) |
| AN8 | Defensive `as_object_mut().unwrap_or_else` unreachable | stage-announce/slack.rs:30-32 | (a) |
| AN9 | `serde_json::to_value(blocks)` happens even with no template vars | stage-announce/lib.rs:269-276 | (a) |

### teams
| AN10 | `icon_url` default omitted (intentional) | stage-announce/lib.rs:467 | — verified |
| AN11 | Adaptive Card v1.4 vs GR MessageCard format (intentional documented) | stage-announce/teams.rs:80-99 | — verified |
| AN12 | `themeColor` placement (outer envelope) may not render | stage-announce/teams.rs:97-99 | (a) |
| AN13 | Title default lazy (round-trip differs) | stage-announce/lib.rs:457-461 | (a) |

### twitter
| AN14 | Uses Twitter v2 API (GR uses deprecated v1) | stage-announce/twitter.rs:14 | — verified |
| AN15 | Hand-rolled OAuth1 — no test for param-ordering edges | stage-announce/twitter.rs:41-80 | (a) |
| AN16 | 4 sequential env reads → 4 distinct error messages | stage-announce/lib.rs:583-597 | (a) |

### mastodon
| AN17 | Bearer-token only (1 env vs GR 3) — intentional simplification | stage-announce/mastodon.rs:8-12 | — verified |
| AN18 | Empty `server` soft-skip with status log (GR silent) | stage-announce/lib.rs:619-624 | (a) |
| AN19 | No User-Agent header (others use anodizer USER_AGENT) | stage-announce/lib.rs:632 | (a) |

### bluesky
| AN20 | Milliseconds-precision timestamp vs GR seconds | stage-announce/bluesky.rs:51 | (a) |

### webhook
| AN21 | Authorization config-wins-over-env intentional documented | stage-announce/lib.rs:317-337 | — verified |
| AN22 | User-Agent `anodizer/x.y.z` vs GR `goreleaser` | stage-announce/lib.rs:340-341 | (c) |
| AN23 | `webhook_body` wrapper does nothing | stage-announce/webhook.rs:13-15 | (a) |
| AN24 | `Url::parse` accepts relative-with-base (GR `ParseRequestURI` stricter) | stage-announce/lib.rs:302-307 | (a) |

### smtp / email
| AN25 | YAML key renamed `smtp` → `email` with no alias | config.rs (AnnounceConfig.email) | (b) |
| AN26 | `port` default 587 vs GR 0 (errors) | stage-announce/lib.rs:814 | (c) |
| AN27 | Sendmail/msmtp fallback when host unset (anodizer-only) | stage-announce/lib.rs:827-832 | — verified |
| AN28 | Plain SMTP (port 25) without TLS would fail under STARTTLS branch | stage-announce/email.rs:57-71 | (a) |
| AN29 | `from` validated to contain `@` (GR doesn't pre-check) | stage-announce/lib.rs:754-759 | — verified |
| AN30 | Empty `to` validated (GR doesn't pre-check) | stage-announce/lib.rs:761-763 | — verified |

### mattermost
| AN31 | `username` default `anodizer` (brand divergence) | stage-announce/lib.rs:498 | (c) |
| AN32 | Reads `mattermost.color` (GR has cross-pipe bug) — anodizer correct | stage-announce/lib.rs:502-508 | — verified |
| AN33 | `text: ""` always emitted (matches GR) | stage-announce/mattermost.rs:35 | — verified |
| AN34 | Channel/username/icon template-rendered (GR doesn't) | stage-announce/lib.rs:495-500 | (c) |

### telegram
| AN35 | Tera vs Go-template default template syntax (user-doc footgun) | stage-announce/lib.rs:386 | (d) |
| AN36 | Unknown `parse_mode` warns (GR silent overwrite) — anodizer better | stage-announce/lib.rs:395-405 | — verified |
| AN37 | `chat_id` required (anodizer earlier than GR) — anodizer better | stage-announce/lib.rs:383 | — verified |

### reddit
| AN38 | `application_id`/`username`/`sub` `require_rendered` (GR doesn't pre-validate) | stage-announce/lib.rs:539-547 | — verified |
| AN39 | No subreddit-name format validation | stage-announce/reddit.rs | (a) |
| AN40 | No rate-limit headers surfaced | stage-announce/reddit.rs | (a) |

### linkedin
| AN41 | `/v2/userinfo` then fallback `/v2/me` (anodizer-original) | stage-announce/linkedin.rs:56-79 | — verified |
| AN42 | No JWT/non-empty after-render check on access token | stage-announce/lib.rs:691-694 | (a) |

### opencollective
| AN43 | Empty `slug` soft-skip with log (GR silent) | stage-announce/lib.rs:711-714 | (a) |
| AN44 | Two-step GraphQL flow (createUpdate, publishUpdate) — verify always-fires | stage-announce/opencollective.rs:14-46 | (a) |
| AN45 | No slug regex validation | stage-announce/opencollective.rs | (a) |
| AN46 | `Personal-Token` header — no token-format check | stage-announce/opencollective.rs | (a) |

### Cross-cutting
| AN47 | Brand defaults: `GoReleaser` → `anodizer` substitution (multi-provider) | stage-announce/lib.rs (multi) | (c) — single decision |
| AN48 | Skip-when-empty UX inconsistent (some log, some error, GR silent) | stage-announce/lib.rs (multi) | (c) |

**Counts (audit 7):** (a) 21 · (b) 1 · (c) 7 · (d) 1 · — verified 13

---

## Master totals

| Audit | (a) | (b) | (c) | (d) | (done) | verified |
|---|---|---|---|---|---|---|
| 1 build/archive/source | 13 | 4 | 6 | 3 | 0 | 0 |
| 2 docker/nfpm/installers | 3 | 7 | 2 | 2 | 0 | 0 |
| 3 publishers-pkgmgr | 11 | 11 | 6 | 1 | 3 | 1 |
| 4 release/changelog/milestone | 18 | 3 | 11 | 3 | 0 | 2 |
| 5 checksum/sign/notarize/sbom | 33 | 6 | 1 | 2 | 0 | 6 |
| 6 infra publishers | 41 | 1 | 0 | 3 | 0 | 4 |
| 7 announcers | 21 | 1 | 7 | 1 | 0 | 13 |
| **Total** | **140** | **33** | **33** | **15** | **3** | **26** |

Grand total findings: **250** across 5 audit areas (3 already-shipped, 26 verified-OK / no-op, 221 actionable).

## Next-step contracts

- `_session-b-inputs.md` — receives all 33 (b) entries.
- `_session-c-inputs.md` — receives all 33 (c) entries.
- (a) + (d) = **155** in-session items. Per discipline rules, each (a)/(d) requires explicit user "go" before applying. Group similar fixes (e.g. cross-cutting "render-error swallow") into batched approvals to reduce churn.

## Recommended approval batches (for STOP-and-ask presentation)

These are batching suggestions to keep the per-fix approval ceremony tractable. Each batch is a coherent theme; user can approve/deny per batch or per item.

1. **Logging hygiene** (3 items) — replace `eprintln!` with StageLogger/tracing across stage-build/archive/release.
2. **Template render-error swallows** (~12 items) — replace `unwrap_or_else(|_| literal)` patterns with `?` propagation in stage-release/changelog/checksum/sign/notarize/sbom/build.
3. **Artifact `name: String::new()`** (4 items) — populate `name` consistently in checksum/sbom/sign/build register-artifact sites.
4. **Validation gaps — fail at Default() time** (~10 items) — checksum algorithm, sign artifacts, notarize key file, blob provider enum, cargo-publish empty-after-render, etc.
5. **Doc/comment fixes** (15 items) — DockerRetryConfig "default 3" comment, parity-comment claims, default-syntax docs, etc. Bundleable into one diff.
6. **Cross-cutting cleanups** (~20 items) — case-insensitive `validate_upload_mode`, password env-cascade order reversal, shared HTTP-upload helper extraction, `release_uploadable_kinds()` add installer kinds.
7. **Stage-internal monolith splits** (3 items) — stage-archive (1700L), stage-release (5732L), stage-sign (3729L).
8. **Tokio runtime reuse** (2 items) — milestones, blob.
9. **Per-publisher production bugs** (~30 items) — homebrew, scoop, winget, krew, nix, aur, aursource, chocolatey, cratesio, dockerhub, artifactory, upload, blob, mastodon, twitter, mattermost, etc.
10. **stage-nfpm production `panic!`** (1 item, blocker-grade) — convert to `Result<>`.

Total batches: 10. Recommended sequencing: 10 → 1 → 5 → 4 → 3 → 2 → 6 → 9 → 7 → 8 (blockers + low-risk first; monoliths and runtimes last).



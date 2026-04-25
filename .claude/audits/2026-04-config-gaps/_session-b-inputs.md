# Session B inputs — config-schema changes (b) findings from Session A

Date: 2026-04-25
Source: `_categorization.md`. 33 (b) entries across 7 audit areas.

Session B mandate (per `/root/.claude/plans/anodizer-refactor-program.md` § "Dedicated Session B"): every schema change starts with discussion. Present current YAML shape vs proposed YAML shape, surface trade-offs, STOP and wait for user decision per item before any code changes.

These items are inputs into the Session B agenda. They do NOT mandate a fix in this session.

## Existing Session B agenda items (from plan.md § B5)

These are pre-categorized in the plan and confirmed by the new audits:

- `AurSourceConfig.amd64_variant` — confirmed missing (publishers-pkgmgr.md item 22).
- `NfpmConfig.umask` int|string deserializer — confirmed (docker-nfpm-installers.md item 5).
- `DockerV2.SBOM` default flip to on — covered in Session C3 (default flip is behavior, not schema; cross-link).
- Top-level alias gaps `nfpms`/`dmg`/`msi`/`flatpak` — confirmed (docker-nfpm-installers.md item 1).
- `Makeself.filename` alias — confirmed (docker-nfpm-installers.md item 14).
- SrpmConfig 7 RPM-spec fields — confirmed (docker-nfpm-installers.md item 15).
- NfpmContent vs NfpmContentConfig DRY — confirmed (docker-nfpm-installers.md item 9).
- NfpmSignatureConfig vs SrpmSignatureConfig DRY — confirmed (docker-nfpm-installers.md item 10).
- HomebrewConfig.commit_msg_template doc/code drift — covered in (d) (publishers-pkgmgr.md item 4).
- WingetConfig.package_identifier regex — covered in (a) (publishers-pkgmgr.md item 11).
- KrewConfig description-required — covered in (a) (publishers-pkgmgr.md item 25).

## New (b) items surfaced by Session A

### From audit 1 (build/archive/source)
- **B-new-1** `BuildConfig.flags` typing — change `Option<String>` → `Vec<String>` (or shlex). Source: build-archive-source.md item 1. Migration: existing `flags: "--locked --release"` strings must still deserialize; consider serde adapter that splits via shlex on string input but preserves Vec on array input.
- **B-new-2** `defaults.archives` field expansion — add `name_template`, `formats`, `wrap_in_directory`, `builds_info` to `DefaultArchiveConfig`. Source: build-archive-source.md item 14. Pre-Session B1 (workspace defaults) since defaults expansion is the same surface.
- **B-new-3** `SourceFileInfo.mode: Option<u32>` vs `ArchiveFileInfo.mode: Option<String>` — type unification across the two near-identical types. Source: build-archive-source.md item 20.
- **B-new-4** `CrateConfig.docker` legacy alongside `docker_v2` — decide deprecation/removal/warn-on-use. Source: build-archive-source.md item 26.

### From audit 2 (docker/nfpm/installers)
- **B-new-5** `DockerV2.skip_push` — anodizer-additive; decide keep+document or remove (GR has no equivalent on `dockers_v2:`). Source: docker-nfpm-installers.md item 3.
- **B-new-6** Snapcraft top-level `slots` field (anodizer-only at top-level) — decide keep+document or remove. Source: docker-nfpm-installers.md item 11.

### From audit 3 (publishers-pkgmgr)
- **B-new-7** Homebrew legacy `commit_author_name`/`commit_author_email` parallel to structured `commit_author` — deprecation/removal. Source: publishers-pkgmgr.md item 7.
- **B-new-8** Scoop legacy `bucket` parallel to `repository` — deprecation/removal. Source: publishers-pkgmgr.md item 9.
- **B-new-9** Chocolatey `tags` — already accepts Vec<String> OR space-separated string; decide whether to canonicalize. Source: publishers-pkgmgr.md item 14.
- **B-new-10** AurConfig `url` legacy redundant with `homepage` — deprecation/removal. Source: publishers-pkgmgr.md item 17.
- **B-new-11** KrewConfig `manifests_repo` / `upstream_repo` legacy — pick one, deprecate the other. Source: publishers-pkgmgr.md item 24.
- **B-new-12** `CratesPublishConfig` enabled→disable (covered in plan § B2/B3 already). Source: publishers-pkgmgr.md item 29.
- **B-new-13** CloudSmithConfig `skip` only inconsistent with peers — align. Source: publishers-pkgmgr.md item 31.
- **B-new-14** Legacy `{owner, name}` structs (TapConfig, BucketConfig, ChocolateyRepoConfig, WingetManifestsRepoConfig, KrewManifestsRepoConfig) → unify on `RepositoryConfig` with token/branch/git/pull_request fields. Source: publishers-pkgmgr.md item 32.
- **B-new-15** Add per-publisher CLI skip flags (`--skip=brew`, `--skip=scoop`, etc.). Source: publishers-pkgmgr.md item 33. (CLI surface, not pure schema, but reaches into config-driven cardinality.)
- **B-new-16** Top-level `brews:`/`scoops:`/`wingets:`/`chocolateys:`/`aurs:`/`nix:`/`krews:` (multi-publisher pattern parity with GR). Source: publishers-pkgmgr.md item 34. Inherits from B1 (workspace defaults).

### From audit 4 (release/changelog/milestone)
- **B-new-17** Changelog `header`/`footer` String → `ContentSource` (asymmetric with release block). Source: C11. Should fold with C12.
- **B-new-18** Changelog snapshot-always-on → opt-in `changelog.snapshot: bool`. Source: C12.
- **B-new-19** Per-entry `Authors` template field — currently `Logins` is global; expose per-entry `Authors` + `Logins`. Source: C8.

### From audit 5 (checksum/sign/notarize/sbom)
- **B-new-20** `SignConfig.binary_signs` — surface as separate type or constrain `artifacts: binary|none` via jsonschema enum. Source: S1.
- **B-new-21** `signs.env` HashMap → ordered Vec<String> ("KEY=VAL" entries) for ordering + chain support. Source: S12.
- **B-new-22** Notarize `timeout: String` → typed Duration (with serde). Source: N1.
- **B-new-23** Notarize top-level `disable` + per-cfg `enabled` doubled surface — consolidate. Source: N7.
- **B-new-24** `NotarizeConfig.macos_native.use_` enum constraint via jsonschema. Source: N9.
- **B-new-25** `SbomConfig.env` HashMap → ordered Vec<String> (same as S12). Source: B1/B7.

### From audit 6 (infra publishers)
- **B-new-26** `UploadConfig.disable` → `skip` rename (or alias) for parity with `ArtifactoryConfig.skip` and GR canonical. Source: U1.

### From audit 7 (announcers)
- **B-new-27** `AnnounceConfig.email` add `#[serde(alias = "smtp")]` for GR-migration compat. Source: AN25.

## Cross-cutting Session B work

- **Workspace defaults system (plan § B1)** — already on agenda; B-new-2 is a concrete dependent.
- **Disable/skip_upload/skip_publish keying** — already aligned (Group 1 done); 3 (done) entries above.
- **Schema validation discipline** — every new field added in Session B should land with a `schemars` enum or pattern when the value space is closed.

## Out of Session B (re-categorize)

The following are **(c) behavior** and belong to Session C even though they touch fields:

- DockerV2 SBOM default flip (already noted above).
- Brand-default substitutions (announcer `username`/`author`).
- AUR Default()-time `Name`/`Conflicts`/`Provides`/`Rel` substitution.

The following are **(a) bug** even though they look like validation:

- All `is_disabled` template-error swallows (cross-cutting; fix at the helper, not at every schema site).
- All `name: String::new()` artifact registrations.
- All `unwrap_or_else(|_| literal)` template fallbacks.

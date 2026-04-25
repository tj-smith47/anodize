# Session C inputs — publisher-behavior changes (c) findings from Session A

Date: 2026-04-25
Source: `_categorization.md`. 33 (c) entries across 7 audit areas.

Session C mandate (per `/root/.claude/plans/anodizer-refactor-program.md` § "Dedicated Session C"): per publisher, present current behavior vs GR behavior, propose refactor, STOP for user approval per publisher. Heaviest task in the program — may expand into multiple subtasks per publisher.

These items are inputs into the Session C agenda. They do NOT mandate a fix in this session.

## Existing Session C agenda items (from plan.md § C1)

`skip_upload` behavioral semantics — generate manifest on disk, skip only push (per publisher). Already in plan.

## Existing Session C3 agenda items (from plan.md)

Pre-categorized in plan and confirmed by new audits:
- `homebrew_cask` directory default `"Casks"` enforcement — confirmed (publishers-pkgmgr.md item 8).
- `nFPM Libdirs` apply unconditionally — confirmed (docker-nfpm-installers.md item 6).
- AUR `Name` auto-suffix `-bin` — confirmed (publishers-pkgmgr.md item 18).
- AUR default `Conflicts`/`Provides`=[ProjectName] — confirmed (publishers-pkgmgr.md item 19).
- AUR `Rel = "1"` default — confirmed (publishers-pkgmgr.md item 20).
- Source archive template error propagation — covered in (a) (build-archive-source.md item 17 — flagged as bug, applied this session).

## New (c) items surfaced by Session A

### From audit 1 (build/archive/source)
- **C-new-1** universal binary metadata-copy whitelist brittle — define a behavior contract for which artifact metadata keys propagate from per-arch builds to the universal binary. Source: build-archive-source.md item 3.
- **C-new-2** `build.env.get(target)` exact-match (no glob) — decide whether per-target env should glob-match or stay exact. Source: build-archive-source.md item 4.
- **C-new-3** universal binary metadata both `id` + `binary` keys (GR id only) — pick semantics. Source: build-archive-source.md item 7.
- **C-new-4** `archives: []` skip vs GR auto-inject default — pick semantics for omitted vs explicit-empty. Source: build-archive-source.md item 9.
- **C-new-5** `FormatOverride` exact `==` vs GR `HasPrefix` — pick semantics. Source: build-archive-source.md item 11.
- **C-new-6** Default extra-file glob order (anodizer LICENSE first, GR license first) — pick canonical order. Source: build-archive-source.md item 13.

### From audit 2 (docker/nfpm/installers)
- **C-new-7** DockerV2 SBOM default off → flip to on (matches GR). Source: docker-nfpm-installers.md item 4 (also in plan B5; behavior, not schema).
- **C-new-8** nFPM Libdirs unconditional application — already in plan §C3; cross-link.

### From audit 3 (publishers-pkgmgr)
- **C-new-9** Homebrew arm_variant default "6" hardcode — verify or align with GR's `experimental.DefaultGOARM`. Source: publishers-pkgmgr.md item 5.
- **C-new-10** TopLevelHomebrewCaskConfig `Directory="Casks"` — already in plan §C3.
- **C-new-11** Chocolatey idempotency port to crates_io — port hash-match short-circuit pattern. Source: publishers-pkgmgr.md item 15.
- **C-new-12** AUR `Name`/`Conflicts`/`Provides`/`Rel` defaults — already in plan §C3.
- **C-new-13** CratesPublishConfig idempotency (port chocolatey hash-compare). Source: publishers-pkgmgr.md item 30.

### From audit 4 (release/changelog/milestone)
- **C-new-14** `prerelease == "auto"` Default()-time global vs per-tag run-time. Source: R3.
- **C-new-15** Snapshot mode runs changelog (GR skips) — flip to opt-in. Source: C1.
- **C-new-16** Default `Format` SCM-mode `{{ ShortSHA }}` vs GR `.SHA` (full) — pick semantic. Source: C2.
- **C-new-17** `## Changelog` title escape-hatch (anodizer skips emission when title=""; GR has no opt-out) — keep or remove. Source: C3.
- **C-new-18** Changelog header/footer go to disk only — decide whether `--release-header`/`--release-footer` should also reach release notes (GR behavior). Source: C4.
- **C-new-19** SCM changelogers always-API-call when no previous tag — pre-empt to git fallback like GR. Source: C7.
- **C-new-20** Milestone `Repo` resolution Default()-time vs publish-time. Source: M2.
- **C-new-21** Milestone empty-after-render `name` skip (GR doesn't skip). Source: M3.
- **C-new-22** Milestone empty-repo + no-fail_on_error: anodizer permissive vs GR strict. Source: M6.

### From audit 5 (checksum/sign/notarize/sbom)
- **C-new-23** Checksum artifact-source kinds narrower than GR (missing Makeself/Flatpak/SourceRpm/Signature/Certificate/UploadableFile) — decide which kinds to include. Source: K2. Cross-link with `release_uploadable_kinds()` (audit 6 L4).

### From audit 7 (announcers)
- **C-new-24** Brand defaults: anodizer vs GoReleaser substitution across discord/slack/teams/mattermost — keep, document in CHANGELOG. Source: AN1, AN6, AN31, AN47.
- **C-new-25** Skip-when-empty UX inconsistency (some log status, some error, GR silent) — define UX policy. Source: AN48.
- **C-new-26** Webhook User-Agent `anodizer/x.y.z` vs GR `goreleaser` — keep or align. Source: AN22.
- **C-new-27** SMTP port default 587 vs GR 0 (errors) — keep permissive default or strict. Source: AN26.
- **C-new-28** Mattermost channel/username/icon template-rendered (GR doesn't) — keep or align. Source: AN34.

## Cross-cutting Session C work

- **Brand-default policy** — single decision affects discord, slack, teams, mattermost (and any future GR-default-username announcers). Resolve once; apply across.
- **Default lazy-vs-eager** — checksum/sign/notarize/sbom/release/milestone all defer defaults to runtime; GR persists into config struct at Default() time. Affects YAML round-trip and `--debug-config` introspection.
- **Skip-when-empty UX** — establish a single policy for what "config is empty / disabled / not applicable" emits (silent skip, status log, warn, error).

## Behavior contracts to write (artifacts of Session C, not pre-decisions)

For each Session C item above, the deliverable is a behavior contract: a one-page note in `crates/<crate>/docs/<feature>.md` (or in the rust doc-comment if short) that documents the chosen semantic and why it diverges from GR.

## Out of Session C (re-categorize)

Items below look like behavior but are bugs (handled this session):
- Universal binary `mod_timestamp` template error swallow — (a).
- `release_uploadable_kinds()` excludes installers — (a) bug.
- All `unwrap_or_else(|_| literal)` template fallbacks — (a).

Items below look like behavior but are schema (handled in Session B):
- `BuildConfig.flags` typing — (b).
- `defaults.archives` field expansion — (b).
- Top-level alias gaps — (b).

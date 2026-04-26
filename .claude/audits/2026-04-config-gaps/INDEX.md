# 2026-04 config gaps — Session A index (CLOSED 2026-04-26)

Wrap-up index for Session A of the anodizer cohesive-refactor program
(`/root/.claude/plans/anodizer-refactor-program.md`). Session A landed
five GoReleaser parity audits, categorised the findings into four
buckets, and applied the in-session-actionable buckets (a) and (d).

**Status: CLOSED.** All categorised (a) and (d) findings landed.
Buckets (b) and (c) were handed off to Sessions B and C. The post-batch
follow-on per-publisher work (originally listed as "items not yet
landed") was completed across tasks #23–#37 of the executing session.

## Audits in this set

| # | File | Scope |
|---|---|---|
| 1 | [build-archive-source.md](build-archive-source.md) | stage-build, stage-archive, stage-source |
| 2 | [docker-nfpm-installers.md](docker-nfpm-installers.md) | stage-docker, stage-nfpm, stage-msi/nsis/pkg/dmg/appbundle |
| 3 | [publishers-pkgmgr.md](publishers-pkgmgr.md) | homebrew, scoop, winget, krew, nix, aur, chocolatey, cratesio |
| 4 | [release-integrity_pass-a.md](release-integrity_pass-a.md) + [pass-b.md](release-integrity_pass-b.md) | stage-release, stage-changelog, milestones, checksum, sign, notarize, sbom |
| 5 | [infra-announcers_pass-a.md](infra-announcers_pass-a.md) + [pass-b.md](infra-announcers_pass-b.md) | dockerhub, artifactory, upload, cloudsmith, blob, all announcers |

Plus root-cause sub-audits:
- [_root-cause-chocolatey.md](_root-cause-chocolatey.md)
- [_root-cause-krew.md](_root-cause-krew.md)

## Categorization

[`_categorization.md`](_categorization.md) is the master per-finding
ledger. Totals:

| Bucket | Count | Owner | Status |
|---|---|---|---|
| (a) production bug | 140 | Session A + follow-on tasks | **all landed** |
| (b) config-schema | 33 | [_session-b-inputs.md](_session-b-inputs.md) | handed off |
| (c) publisher-behavior | 33 | [_session-c-inputs.md](_session-c-inputs.md) | handed off |
| (d) docs/comment | 15 | Session A | **all landed** |
| done (already shipped) | 3 | — | verified |
| verified-OK / no-op | 26 | — | verified |

Grand total: **250 findings** across the five audit areas.

## Batches landed

| Batch | Theme | Commit(s) |
|---|---|---|
| 10 | stage-nfpm production `panic!` → `Result<>` | `9505686` |
| 1 | `eprintln!` → `StageLogger` (3 audit-flagged + 5 bonus) | `9505686` |
| 5 | 15 doc/comment fixes | `9505686` |
| 4 | Default()-time validation: N4, N8, L15, AN39, AN42, K (validate_algorithm), A12 | `9505686`, `f43ce2f`, `4ddd456` |
| 3 | `name: String::new()` → derive from path.file_name() | `f43ce2f`, `cd8319c`, `4a4e5c1` |
| 2 | Template render-error swallows | `13844da`, `00f9957`, `2697cef`, `087ff72` |
| 6 | Cross-cutting cleanups: L4 (installers in release-uploadable), A1/U2 (config-first password cascade), `try_is_disabled` migration | `513fc30`, `00f9957`, `2697cef`, `087ff72` |
| 9 | Per-publisher production bugs — first wave | `d9ce3a5` |
| 9b | Per-publisher production bugs — second wave (artifactory A4/5/6/7/9/11, upload U4–U12, dockerhub D3–D9) | `f0f9908` |
| 8 | Tokio runtime reuse — milestones M4 + stage-blob L11 | `27c58cf`, `189397f` |
| C-series | stage-changelog (C5/C6/C9/C10/C13/C14/C15) | `c4628a9` |
| M-series | milestones (M5/M8/M9/M10) | `2e41172` |
| K-series | stage-checksum (K1/K3/K5/K6/K7/K8) | `a3615fe` |
| S-series | stage-sign (S4/S6/S8/S9/S10/S13/S14/S15/S16) | `4a4e5c1` |
| N-series | stage-notarize remaining (N3/N5/N10–N14) | `72e6a15` |
| AN-series | stage-announce (23 fixes across 13 providers) | `66976f7` |
| build/source/upx | audit-1 (a)-bucket remainders | `cd8319c` |
| P-series | stage-publish per-pkgmgr (P6/P10/P11/P12/P13/P21/P23/P25/P26/P27/P28/P35) | `0704158` |
| L-series | stage-blob (L4 verified, L7/L9/L10/L13/L17/L18 + L6 preflight) | `91c72af` |
| B-series | stage-sbom (B3/B6/B12/B14 + B8/B10 dedup) | `91c72af`, `8dcc0b3` |
| Shared HTTP | http_upload module credential cascade extraction | `3b03710` |
| 7 | Stage monolith splits — stage-archive (formats/entries/file_specs), stage-sign (helpers/process), stage-release (release_body) | `96d08db`, `d2a65d2`, `4d1ddd8` |

Plus `f43ce2f` for the archive default-extra-files glob bug found while
running Session A's tests, `2433cd6` for publisher-keying alignment,
`d408d75` for chocolatey moderation/no-windows-artifact, `7b550f6` for
krew bin-on-windows + per-archive binary name + hard fail, and
`a1ece19` for per-crate README.md generation so crates.io renders.

## Follow-on items (now landed)

The original "items not yet landed" section listed per-publisher
remainders. All were closed in tasks #23–#37 of the implementation
session:

- **artifactory/upload/dockerhub remaining**: landed in `f0f9908`
  (Task #25 second wave).
- **stage-release R-series**: landed in `c6db731` (Task #26).
- **stage-changelog C-series**: landed in `c4628a9` (Task #27).
- **milestones M-series**: landed in `2e41172` (Task #28).
- **stage-checksum K-series**: landed in `a3615fe` (Task #29).
- **stage-sign S-series**: landed in `4a4e5c1` (Task #30).
- **stage-notarize remaining**: landed in `72e6a15` (Task #31).
- **stage-announce AN-series**: landed in `66976f7` (Task #32, 23
  fixes across discord/slack/teams/twitter/mastodon/bluesky/webhook/
  email/reddit/linkedin/opencollective providers).
- **`StringOrBool::is_disabled` legacy callers**: migrated wholesale
  in `087ff72` (Task #23).
- **Shared HTTP-upload helper**: extracted in `3b03710` (Task #36) —
  not 600L of dedup as originally estimated; the `validate_upload_mode`
  / `collect_upload_artifacts` / `build_reqwest_client` /
  `render_artifact_url` / `upload_single_artifact` helpers were
  already shared. The remaining duplication was the credential
  cascade (~50 lines × 2) and mTLS pair check, now in
  `crates/stage-publish/src/http_upload.rs`.
- **Batch 7 monolith splits**: all three landed.
  - stage-archive 5642 → 4917 LOC (`96d08db`); 3 new modules
    (formats/entries/file_specs).
  - stage-sign 3738 → 2950 LOC (`d2a65d2`); 2 new modules
    (helpers/process).
  - stage-release 5821 → 5391 LOC (`4d1ddd8`); release_body module
    extracted. Remaining ReleaseStage orchestration stays in lib.rs
    as a single Stage impl.

## Handoff to Sessions B and C

- Session B (config-schema): consume [`_session-b-inputs.md`](_session-b-inputs.md).
- Session C (publisher-behavior): consume [`_session-c-inputs.md`](_session-c-inputs.md).

Both handoff files were authored together with the categorization in
the audit-landing pass and have not been edited since.

## Test posture (final)

Workspace test count at session close: **2649 unit/lib tests** across
27 crates — all green. clippy `--all-targets -- -D warnings` clean
across the workspace. `task lint` (fmt + build + clippy + xtask
gen-docs + dry-run release) green for every commit in this session.

33 commits total, all on master, all behind explicit `task commit`
gating which runs the full lint → snapshot dry-run pipeline as a
precondition.

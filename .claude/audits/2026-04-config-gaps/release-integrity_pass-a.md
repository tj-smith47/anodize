# release + changelog + milestone — parity audit (Pass A)

Audited: 2026-04-25
Re-graded with Go-toolchain rule applied.
References: `/opt/repos/anodizer` HEAD, `/opt/repos/goreleaser` HEAD.

Scope: `internal/pipe/release/{release,scm,body}.go`,
`internal/pipe/changelog/changelog.go`, `internal/pipe/milestone/milestone.go`
vs anodizer `crates/stage-release/src/{lib,gitea,gitlab}.rs`,
`crates/stage-changelog/src/lib.rs`,
`crates/cli/src/commands/release/milestones.rs`.

---

## release pipe

### Real MISSING fields (anodizer ReleaseConfig vs GoReleaser config.Release)

None at the per-release-block level. Every GoReleaser `Release` field has an
anodizer counterpart in `ReleaseConfig` (config.rs:1545):

| GR field | GR ref | anodizer field | anodizer ref |
|---|---|---|---|
| `GitHub` / `GitLab` / `Gitea` | config.go:646-648 | `github`/`gitlab`/`gitea` | config.rs:1547-1551 |
| `Draft`, `ReplaceExistingDraft`, `UseExistingDraft` | config.go:649-651 | `draft`, `replace_existing_draft`, `use_existing_draft` | config.rs:1553, 1577, 1596 |
| `TargetCommitish` | config.go:652 | `target_commitish` | config.rs:1590 |
| `Disable`, `SkipUpload` | config.go:653-654 | `disable`, `skip_upload` (StringOrBool) | config.rs:1583, 1574 |
| `Prerelease`, `MakeLatest` | config.go:655-656 | `prerelease`, `make_latest` | config.rs:1556, 1559 |
| `NameTemplate`, `IDs`, `ExtraFiles` | config.go:657-659 | `name_template`, `ids`, `extra_files` | config.rs:1561, 1588, 1567 |
| `DiscussionCategoryName` | config.go:660 | `discussion_category_name` | config.rs:1592 |
| `Header`, `Footer` | config.go:661-662 | `header`, `footer` (ContentSource) | config.rs:1563-1565 |
| `ReleaseNotesMode` (`mode`) | config.go:664 | `mode` | config.rs:1586 |
| `ReplaceExistingArtifacts`, `IncludeMeta` | config.go:665-666 | `replace_existing_artifacts`, `include_meta` | config.rs:1579, 1594 |
| `GitHubURLs.{API,Upload,Download,SkipTLSVerify}` | config.go:25-30 | `GitHubUrlsConfig` | config.rs:1707 |
| `GitLabURLs.{API,Download,SkipTLSVerify,UsePackageRegistry,UseJobToken}` | config.go:33-39 | `GitLabUrlsConfig` | config.rs:1722 |
| `GiteaURLs.{API,Download,SkipTLSVerify}` | config.go:42-46 | `GiteaUrlsConfig` | config.rs:1739 |

Anodizer-only fields (no parity gap, intentional supersets): `templated_extra_files`,
`tag` override (Pro feature given for free).

### Default divergences

- behavior: `name_template` default token form
  goreleaser: release.go:55-57 — defaults to `"{{.Tag}}"` (Go template)
  anodizer: stage-release/lib.rs:1230 — defaults to `"{{ Tag }}"` (tera)
  divergence: same render result; surface differs (no leading dot, spaces). User-facing in YAML is identical at the source level — not a runtime divergence — but may surprise users copying GR templates verbatim into a custom `name_template`.

- behavior: `Default()` injects the resolved `Repo` into config + applies `tmpl.ApplyAll` to `Owner`/`Name`
  goreleaser: scm.go:10-35 (GitHub), 37-62 (GitLab), 64-89 (Gitea) — at Default() time, missing `Name` is filled from `git.ExtractRepoFromConfig`; both `Owner` and `Name` are template-rendered (so users can write `owner: "{{ .Env.OWNER }}"`).
  anodizer: ScmRepoConfig (config.rs:1676) — `owner`/`name` are plain `String`, never template-rendered. The release stage (lib.rs:1093+) reads `release_cfg.github`/`gitlab`/`gitea` directly.
  divergence: anodizer does not template-render `release.github.owner` / `release.github.name` (and the GitLab/Gitea variants). Configs that set `owner: "{{ .Env.X }}"` will be sent literally to the API.

- behavior: pre-release auto-detection at Default()
  goreleaser: release.go:76-85 — at Default() time, `prerelease == "auto"` inspects `ctx.Semver.Prerelease` and sets `ctx.PreRelease`. `prerelease == "true"` sets it unconditionally.
  anodizer: stage-release/lib.rs:418-425 — `should_mark_prerelease` is per-tag, called once per crate at run time. Same semantics; resolution timing differs (no global ctx.PreRelease equivalent).

- behavior: setup of `ctx.ReleaseURL`
  goreleaser: scm.go:26-34 — Default() pre-computes the public release URL (`{download}/{owner}/{name}/releases/tag/{tag}`) and stashes it on `ctx.ReleaseURL`.
  anodizer: stage-release/lib.rs — no `ctx.release_url` template variable produced for downstream stages (e.g. announce). Searched: no equivalent set anywhere in stage-release.
  divergence: GR's `{{ .ReleaseURL }}` template variable is reachable from any later stage; anodizer has no such pre-published variable.

- behavior: artifact upload-types whitelist source
  goreleaser: release.go:160-167 — `artifact.ReleaseUploadableTypes()` is the canonical list; `IncludeMeta` appends `Metadata`.
  anodizer: stage-release/lib.rs:1292-1296 — calls `release_uploadable_kinds()` (see comment block 1283-1293). Same pattern, parity OK.

- behavior: body template structure
  goreleaser: body.go:13-16 — body template is `{{Header}}\n{{ReleaseNotes}}\n{{Footer}}\n`; header rendered to `out` (one extra `\n` after `Header`), footer prefixed with a `\n`.
  anodizer: stage-release/lib.rs:434-464 (`build_release_body`) — joins `[header, body, footer]` with single `"\n"` separators and trailing `"\n"`. Empty header drops cleanly via `parts.is_empty()`. GR's two-newline padding around `Header`+`Footer` (the `with`/explicit `"\n"` blocks) is collapsed.
  divergence: when both header AND changelog are present, GR emits `{Header}\n\n{ReleaseNotes}` (header line + blank line, because the template inserts `\n` after `with .Header`); anodizer emits `{Header}\n{Body}` (no blank line).

- behavior: `Checksums` template variable for >1 checksum artifact
  goreleaser: body.go:34-43 — key is `artifact.ExtraOr(*check, artifact.ExtraChecksumOf, "")` (empty string fallback).
  anodizer: stage-release/lib.rs:1141-1156 — key falls back to `artifact.path.to_string_lossy()` (full path) when `ChecksumOf` metadata is absent.
  divergence: with multiple unmarked checksum artifacts, anodizer keys the map by absolute filesystem paths (env-dependent, snapshot-unstable); GR keys by empty string (collisions overwrite).

### Validation gaps

- field: `mode` (`ReleaseNotesMode`)
  goreleaser: config.go:637-642 declares enum `keep-existing|append|prepend|replace`; jsonschema enforces it (config.go:664).
  anodizer: stage-release/lib.rs:575-594 (`resolve_release_mode`) DOES validate at runtime — parity OK.

- field: `IDs` filter — empty-after-filter behavior
  goreleaser: release.go:170-180 — silently uploads zero artifacts.
  anodizer: stage-release/lib.rs:1317-1324 — emits a verbose log line. Functional parity; verbose-only might be missed.

- field: `release.github` + `release.gitlab` + `release.gitea` mutual exclusion
  goreleaser: release.go:41-53 — `numOfReleases > 1` returns `ErrMultipleReleases` at Default(); validation is global (only one Release block ever).
  anodizer: config.rs:392-431 (`validate_release_backends`) — validates per-crate, not globally. Multiple crates with different backends are allowed (intentional, supports per-crate routing).
  divergence: scope differs by design; not a gap, but documented here so future Pass-B doesn't re-flag it.

- field: `extra_files` glob — non-existent file
  goreleaser: release.go:149-152 — checks `os.Stat`, returns wrapped fs.ErrNotExist if missing.
  anodizer: stage-release/lib.rs:484-523 — `glob` returns no entries if no match → bail "matched no files". Differs only in error wording; effectively equivalent.

- field: `target_commitish`
  goreleaser: config.go:652 — accepts a template, no validation; rendered by the SCM client.
  anodizer: stage-release/lib.rs:1263-1273 — template-rendered, no further validation. Parity.

- field: `disable`, `skip_upload`
  goreleaser: config.go:653-654 — `oneof_type=string;boolean` jsonschema; boolean values are coerced to "true"/"false" strings via `tmpl.New(ctx).Bool`.
  anodizer: config.rs:1574, 1583 — `StringOrBool` via `deserialize_string_or_bool_opt`. Parity, modulo unknown-string handling — `skip_upload: "yes"` falls through anodizer's `match` to `false`; GR's `tmpl.Bool` errors on non-bool/template strings.

### Code smells (anodizer-internal)

- stage-release/lib.rs:1230 — default name template `"{{ Tag }}"` is hard-coded; not derived from a shared `defaults` constant. New stages adding a name_template would re-state it.
- stage-release/lib.rs:1240-1256 — `skip_upload` resolution duplicates the `auto`/`true`/`1` parsing the `StringOrBool` type already exposes elsewhere (e.g. `is_disabled`). DRY: factor into `StringOrBool::is_truthy_or_auto(snapshot, render)`.
- stage-release/lib.rs:1247 — `unwrap_or_else(|_| s.as_str().to_string())` swallows template render errors silently for `skip_upload`; GR returns the error.
- stage-release/lib.rs:557 (`resolve_make_latest`) — same `unwrap_or_else(|_| tmpl.clone())` pattern; render errors silently fall through to literal-string match.
- stage-release/lib.rs:937 (`build_octocrab_client_insecure`) — `eprintln!` warning for TLS-verify-disabled bypasses the `StageLogger`.
- stage-release/lib.rs:5732 lines in one file — file is unreasonably large; mirrors the `stage-archive` smell flagged in the build-archive audit.

---

## changelog pipe

### Real MISSING fields (anodizer ChangelogConfig vs GoReleaser config.Changelog)

None.

| GR field | GR ref | anodizer field | anodizer ref |
|---|---|---|---|
| `Filters.{Include,Exclude}` | config.go:1156-1157 | `ChangelogFilters.{include,exclude}` | config.rs:4411-4416 |
| `Sort` | config.go:1163 | `sort` | config.rs:4282 |
| `Disable` | config.go:1164 | `disable` (StringOrBool) | config.rs:4294 |
| `Use` | config.go:1165 | `use_source` (`#[serde(rename = "use")]`) | config.rs:4300 |
| `Format` | config.go:1166 | `format` | config.rs:4308 |
| `Groups` | config.go:1167 | `groups` | config.rs:4286 |
| `Abbrev` | config.go:1168 | `abbrev` | config.rs:4302 |
| `ChangelogGroup.{Title,Regexp,Order}` | config.go:1172-1176 | `ChangelogGroup.{title,regexp,order}` | config.rs:4422-4426 |

Anodizer-only fields (intentional supersets, no parity gap): `header`, `footer`,
`paths`, `title`, `divider`, `ai`, nested `ChangelogGroup.groups`.

### Default divergences

- behavior: snapshot-mode skip
  goreleaser: changelog.go:46-48 — `Skip()` returns `true` when `ctx.Snapshot`; the changelog stage does not run for snapshots.
  anodizer: stage-changelog/lib.rs:842-843 — comment explicitly notes the divergence: "we intentionally generate it for testing/preview purposes." Always runs.
  divergence: GoReleaser produces no `dist/CHANGELOG.md` and no `ctx.ReleaseNotes` for snapshots; anodizer produces both.

- behavior: default `Format` template
  goreleaser: changelog.go:54-61 — when `Use == "" || "git"`: `"{{ .SHA }} {{ .Message }}"`; otherwise (github/gitlab/gitea/github-native): `"{{ .SHA }}: {{ .Message }} ({{ with .AuthorUsername }}@{{ . }}{{ else }}{{ .AuthorName }} <{{ .AuthorEmail }}>{{ end }})"`.
  anodizer: stage-changelog/lib.rs:377-388 — when `abbrev < 0`: `"{{ Message }}"`; for `git`: `"{{ SHA }} {{ Message }}"`; for github/gitlab/gitea: `"{{ ShortSHA }}: {{ Message }} ({% if Login %}@{{ Login }}{% else %}{{ AuthorName }} <{{ AuthorEmail }}>{% endif %})"`.
  divergence: anodizer's SCM-mode default uses `ShortSHA` (abbreviated); GR's default uses `.SHA` (full hash) — same template var name, different semantics. anodizer's `git`-mode default also uses `SHA` for the full hash by name (lib.rs:498-499 forces `SHA` to mean "abbreviated when abbrev set"), so a user template `{{ SHA }}` produces a SHORT hash in anodizer when `abbrev > 0` and a FULL hash in GR.

- behavior: default `Abbrev` value
  goreleaser: config.go:1168 — `Abbrev int` defaults to 0 (Go zero value); 0 means "no abbreviation" (full SHA).
  anodizer: config.rs:4302 — `abbrev: Option<i32>`, defaults to `None`; stage-changelog/lib.rs:940 unwraps to `0`. Parity.
  divergence: stage-changelog/lib.rs:482-494 — `abbrev == 0` keeps full hash for `SHA` rendering (parity with GR). But the SCM-mode `default_format` (lib.rs:382) uses `{{ ShortSHA }}` regardless of abbrev — see prior bullet. With `abbrev == 0` and SCM use, output differs from GR (anodizer renders full SHA; GR renders full `.SHA`).

- behavior: title heading "## Changelog" emission
  goreleaser: changelog.go:152-153 — always emits `## Changelog` as the first element (`title("Changelog", 2)`), regardless of whether groups are configured.
  anodizer: stage-changelog/lib.rs:398-403 — emits `"## {title}\n\n"` with title defaulting to `"Changelog"`. Parity, with one divergence: anodizer skips emission when `title == ""`; GR has no escape hatch.

- behavior: newline character for GitLab/Gitea
  goreleaser: changelog.go:128-135 (`newLineFor`) — switches on `ctx.TokenType` (the SCM provider for this run), not on `Use`.
  anodizer: stage-changelog/lib.rs:392-396 — switches on `scm_provider.unwrap_or(use_source)`. Caller passes `ctx.token_type.to_string()` (lib.rs:1125), parity.

- behavior: write `dist/CHANGELOG.md` even in dry-run
  goreleaser: changelog.go:109-111 — `os.WriteFile` unconditionally; no dry-run gate.
  anodizer: stage-changelog/lib.rs:1164-1169 — also unconditional for the git-changelog path. The `--release-notes` override path (lib.rs:875-884) DOES gate on `is_dry_run`, divergent from GR. Documented but inconsistent within anodizer itself.

- behavior: `Filters.Include` precedence
  goreleaser: changelog.go:310-322 — when `len(Include) > 0`, includes ARE the only source (exclude is dropped).
  anodizer: stage-changelog/lib.rs:1093-1099 — same precedence rule. Parity.

- behavior: `ChangelogGroup` empty-regexp (catch-all)
  goreleaser: changelog.go:167-175 — empty regexp purges remaining entries; subsequent groups stop processing (`break`).
  anodizer: stage-changelog/lib.rs:250-277 — same semantics. Parity.

- behavior: `dist/CHANGELOG.md` is the SAME file as the per-release notes source
  goreleaser: changelog.go:104-111 — assigns `ctx.ReleaseNotes = strings.Join(elements, "\n\n") + "\n"` (changelog + header + footer combined), THEN writes that exact string to `dist/CHANGELOG.md`. The release stage uses `ctx.ReleaseNotes` directly.
  anodizer: stage-changelog/lib.rs:1138, 1156-1169 — stores per-crate body in `ctx.changelogs[crate]` (without header/footer) and writes a SEPARATE combined string (with header/footer) to `dist/CHANGELOG.md`. The release stage reads `ctx.changelogs[crate]` (lib.rs:1122) — header/footer are added AGAIN by `build_release_body` from the release config.
  divergence: in GR the changelog header/footer are part of the release notes and uploaded; in anodizer the changelog header/footer go ONLY to the on-disk `dist/CHANGELOG.md`, never to the GitHub release. This is documented at lib.rs:1146-1149 ("These changelog header/footer values only affect the disk file"), but it diverges from GR's behavior where `Changelog.Header`/`Footer` (if set via `--release-header`/`--release-footer`) end up in the release body.

- behavior: `loadContent` for `release-header`/`release-footer`/`release-notes` files
  goreleaser: changelog.go:433-457 — supports both `Tmpl` (rendered) and `File` (raw) forms; warns if loaded content evaluates to empty.
  anodizer: cli/src/commands/release/mod.rs:136-178 + stage-changelog/lib.rs:849-885 — supports `--release-header`, `--release-header-tmpl`, `--release-footer`, `--release-footer-tmpl`, `--release-notes`, `--release-notes-tmpl`. The `_tmpl` flag stores raw content into `release.header`/`release.footer` (lib.rs:148-157, 169-177) so the release stage's existing `render_template` step handles rendering — parity-equivalent. No empty-after-render warning.

- behavior: GitHub-native changelog auth/repo extraction order
  goreleaser: changelog.go:391-410 (`newGithubChangeloger`) — creates a `ReleaseNotesGenerator` client, then `git.ExtractRepoFromConfig`, then `repo.CheckSCM`. Errors propagate.
  anodizer: stage-changelog/lib.rs:904-918 — `use: github-native` short-circuits the changelog stage, sets `ctx.github_native_changelog = true`, and stores empty-string per-crate. The actual release-notes generation is deferred to the release stage (lib.rs:831 `generate_release_notes`). The auth/repo check happens later inside the release POST. Order differs.

- behavior: `wrappingChangeloger` SCM fallback to git when no previous tag
  goreleaser: changelog.go:370-374 — when `Use` is `gitlab|gitea|github` AND `ctx.Git.PreviousTag == ""`, logs a warning and falls back to `gitChangeloger{}`.
  anodizer: stage-changelog/lib.rs:1034-1091 — for `github`/`gitlab`/`gitea`, fetches via API in all cases (no fallback). When previous tag is `None`, the API helper is still invoked. The strict-mode guard (lib.rs:1039, 1056, 1073) catches API errors to fall back to `fetch_git_commits`, but only when `ctx.strict_guard` allows it.
  divergence: GR pre-emptively avoids the API call when no previous tag exists; anodizer always tries first.

- behavior: `cleanupAuthors` deduplication for `Authors`/`Logins` template fields
  goreleaser: changelog.go:273-296 — dedupes by `cmp.Or(Username, Email, Name)`, exposes `{{ .Authors }}` and `{{ .Logins }}` per-entry.
  anodizer: stage-changelog/lib.rs — `Login` is a per-commit field; `Logins` (lib.rs:1130, 4 places) is a single comma-joined string per release, not per-entry. Co-author handling differs: anodizer extracts via `extract_co_authors` (lib.rs:96) but the `Authors` template field is not exposed.
  divergence: GR template `{{ range .Authors }}` per commit is unavailable; `Logins` is global instead of per-entry.

### Validation gaps

- field: `Sort`
  goreleaser: changelog.go:235-242 — `checkSortDirection` rejects anything other than `""`/`"asc"`/`"desc"`; ErrInvalidSortDirection.
  anodizer: stage-changelog/lib.rs:175-187 — same validation, error message differs only. Parity.

- field: `Use`
  goreleaser: changelog.go:366-378 — accepts `git|""|gitlab|gitea|github|github-native`; default falls through to `gitChangeloger`.
  anodizer: stage-changelog/lib.rs:921-928 — same set; explicit error otherwise. Parity.

- field: `Groups[].Regexp`
  goreleaser: changelog.go:177-180 — invalid regex → wrapped error `"failed to group into %q: %w"`.
  anodizer: stage-changelog/lib.rs:237-239 — same hard error, message differs. Parity.

- field: `Filters.{Include,Exclude}`
  goreleaser: changelog.go:315, 324 — invalid regex → returns the error AND the original `entries` (continues with original list). Anodizer (lib.rs:124-126, 154-156) bails on the first invalid pattern.
  divergence: GR is more permissive (continues without filtering); anodizer halts the whole stage.

- field: `Abbrev`
  goreleaser: no validation (Go int).
  anodizer: no validation; negative values (e.g. `-1`) trigger the "omit hash" branch (lib.rs:482-484). Parity.

- field: `header`/`footer` (anodizer-only — supports `from_url`/`from_file`)
  anodizer: only inline strings on `ChangelogConfig.header`/`footer` (config.rs:4288-4290) — `String`, not `ContentSource`. The `release` block (config.rs:1563-1565) takes `ContentSource`. Asymmetric typing across the two header/footer surfaces.

### Code smells (anodizer-internal)

- stage-changelog/lib.rs:842-843 — comment "we intentionally generate it for testing/preview purposes" justifies the snapshot divergence. Should be a CLI/config opt-in (e.g. `changelog.snapshot: true`) rather than a silent always-on behavior.
- stage-changelog/lib.rs:382 — SCM-mode default format hard-codes `ShortSHA`; GR uses `.SHA`. Out-of-band override silently swaps semantic of `SHA` template variable (see default-divergence above).
- stage-changelog/lib.rs:1138 + 1140 — per-crate body and combined markdown both built in the same loop; combined is what gets header/footer treatment. Two outputs, two distinct truths — easy to skew.
- stage-changelog/lib.rs:1186-1195 (`fetch_git_commits`) — `unwrap_or_default()` swallows `get_commits_between_paths` errors; bad git config produces an empty changelog with no warning.
- stage-changelog/lib.rs:849-855 + 875-884 — `--release-notes` path bypass duplicates the dist-write logic; should share with the main path.

---

## milestone pipe

### Real MISSING fields (anodizer MilestoneConfig vs GoReleaser config.Milestone)

None.

| GR field | GR ref | anodizer field | anodizer ref |
|---|---|---|---|
| `Repo` | config.go:671 | `repo` (`ScmRepoConfig`) | config.rs:5844 |
| `Close` | config.go:672 | `close` | config.rs:5846 |
| `FailOnError` | config.go:673 | `fail_on_error` | config.rs:5848 |
| `NameTemplate` | config.go:674 | `name_template` | config.rs:5850 |

### Default divergences

- behavior: default `NameTemplate`
  goreleaser: milestone.go:13 + 27-29 — defaults to `"{{ .Tag }}"`.
  anodizer: cli/src/commands/release/milestones.rs:28 — defaults to `"{{ Tag }}"`.
  divergence: same render, different surface (no leading dot).

- behavior: `Repo` auto-detection at Default()
  goreleaser: milestone.go:31-41 — at Default() time, when `Repo.Name == ""`, runs `git.ExtractRepoFromConfig` (with snapshot exception) AND `repo.CheckSCM` (with snapshot exception); the resolved repo is persisted into `milestone.Repo`.
  anodizer: cli/src/commands/release/milestones.rs:108-172 (`resolve_milestone_repo`) — defers resolution to publish-time. Order: explicit `milestone.repo` → first matching crate's `release.{token_type}` → any release block → `git::detect_owner_repo()`. No persistence back into `milestone.repo`.
  divergence: anodizer's resolution is publish-time only (template variables built from `release` configs are not visible to other stages); GR's is Default()-time and propagates.

- behavior: `Pipe.Skip` semantics
  goreleaser: milestone.go:20 — `Skip == true` when `len(ctx.Config.Milestones) == 0`. The pipe runs even when no milestone has `Close: true` (and emits `pipe.Skip("closing not enabled")` per-iteration).
  anodizer: cli/src/commands/release/milestones.rs:20-22 — iterates and `continue`s when `close == false`. No top-level skip; a `milestones: [{close: false}]` config still walks every entry. Functional parity.

- behavior: `ContinueOnError == true` advertised by the pipe
  goreleaser: milestone.go:19 — explicit `func (Pipe) ContinueOnError() bool { return true }` so milestone failures don't abort the rest of the pipeline.
  anodizer: milestones.rs:88-103 — error handling is per-milestone: when `fail_on_error == true`, returns the error; otherwise warns. The function returns `Result<()>` and is called from the post-pipeline phase (cli/release/mod.rs:483 `run_post_pipeline`).
  divergence: anodizer surfaces failures only when `fail_on_error == true` (matching GR semantics for individual milestones), but anodizer has no analog of `ContinueOnError` for stage-level error propagation — the broader pipeline does not have a per-stage continue-on-error flag.

- behavior: empty `name` after rendering
  goreleaser: milestone.go:64-67 — renders, no empty check.
  anodizer: milestones.rs:33-36 — skips when rendered name is empty. anodizer-stricter.

- behavior: tokio runtime per-milestone-call
  goreleaser: milestone.go:48-54 — single sync `client.New(ctx)`, then loop.
  anodizer: milestones.rs:185, 313, 399 — one tokio runtime per `close_milestone_*` call (3 separate `Runtime::new()` per milestone). Overhead, not correctness.

- behavior: GitLab/Gitea API URL resolution
  goreleaser: milestone.go:48-49 — uses `client.New(ctx)` which honors `gitlab_urls`/`gitea_urls` from config.
  anodizer: milestones.rs:279-298 (`resolve_milestone_api_url`) — does its own URL resolution by stripping `/api/v4` or `/api/v1` suffix. Brittle: a custom path like `https://gitlab.example.com/private/api/v4/` strips to `https://gitlab.example.com/private`, but a path like `https://gitlab.example.com/api/v4` strips to `https://gitlab.example.com`. Parity, with edge-case fragility.

### Validation gaps

- field: `Repo.Owner` / `Repo.Name`
  goreleaser: milestone.go:36-38 — `repo.CheckSCM()` returns "invalid scm url" when owner/name aren't both set; bail unless snapshot.
  anodizer: milestones.rs:43-49 — checks both fields non-empty AND respects `fail_on_error` for the bail/warn decision. GR bails unconditionally (subject to snapshot exception); anodizer is more permissive without `fail_on_error`. Documented divergence.

- field: `name_template`
  goreleaser: milestone.go:64 — render is the only validation.
  anodizer: milestones.rs:29-32 — same. Parity.

- field: `close`
  goreleaser: milestone.go:60-62 — `if !milestone.Close { return pipe.Skip(...) }` returns from `doPublish` after the FIRST encountered uncloseable milestone, halting the rest. Code smell in GR itself: a config with `[{close: false}, {close: true}]` skips the second one.
  anodizer: milestones.rs:21-23 — `continue`s past entries with `close == false`, so other milestones still close. Anodizer is correct here; GR is wrong.

### Code smells (anodizer-internal)

- milestones.rs:185, 313, 399 — `tokio::runtime::Runtime::new()` invoked three times per milestone-close, once per `close_milestone_*`. Should reuse a single runtime built at the caller.
- milestones.rs:279-298 — `resolve_milestone_api_url` strips `/api/v4` and `/api/v1` from a fully-qualified API URL to derive the "base URL" then re-appends `/api/v4` or `/api/v1` in callers. Round-trips through a stripped form for no benefit; pass the full API base.
- milestones.rs:147-161 — "any release block" fallback iterates ALL crates again after the per-token-type loop. Two near-identical loops; refactor.
- milestones.rs:243-249, 358-361, 443-446 — "milestone not found → treat as success" silently returns `Ok(())`. Different from GoReleaser, which would never get here because GR closes by ID, not by-name lookup. Worth a verbose log so users learn that re-runs don't error.
- milestones.rs:457 — Gitea PATCH includes `"title": milestone_name` even though the goal is to close (not rename). Per Gitea API, `title` is optional on PATCH; sending it round-trips the title and asserts the title hasn't changed under our feet — minor surprise.

---

## Summary table

| pipe | MISSING | default-divergence | code-smell | validation-gap |
|---|---|---|---|---|
| release | 0 | 6 | 6 | 6 |
| changelog | 0 | 9 | 5 | 5 |
| milestone | 0 | 6 | 5 | 4 |

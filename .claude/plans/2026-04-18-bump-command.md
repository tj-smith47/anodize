# `anodizer bump` — Spec & Plan

Status: spec, not implemented. Handoff from a cross-repo brainstorm on 2026-04-18.

## Why this, why anodizer

cargo-release bumps + commits + tags + publishes as one atomic flow. `cargo set-version` (cargo-edit) bumps only — no git, no inference. `cargo-workspaces` has its own surface. None of them know what anodizer knows:

1. **Per-crate automatic level from Conventional Commits.** anodizer already parses commits (`stage-changelog::parse_commit_message`) and already reads commits scoped to a crate's paths (`git::get_commits_between_paths`). Deciding `feat→minor`, `fix→patch`, `BREAKING CHANGE→major` *per crate* is a one-line mapping on top of infra that exists today.
2. **"Only bump what changed."** anodizer already uses per-crate tag prefixes (`core-v0.4.2`, `cli-v0.4.2`) via `monorepo_tag_prefix()`. It can skip crates with no commits since their last tag. No other tool knows this.
3. **Cascade awareness.** Anodizer has the workspace graph and the release-tag history. "Bumping core major means cli must ship too" falls out of that context.
4. **Plan reflects the whole release.** Preview shows: bumps → changelog entries → tags that'd be created → what'd be signed → what'd be published. Single command, whole-pipeline visibility.
5. **Release-policy enforcement.** Anodizer already knows protected-branch, signing-required, and `.anodizer.yaml` constraints. `bump` refuses states that'd produce an invalid release.

**The headline:** `anodizer bump` with no arguments figures out which crates changed, at what level each needs to bump, and shows you. Impossible for any tool that only sees `Cargo.toml`.

## Scope

**In scope (v1):**
- `anodizer bump [LEVEL_OR_VERSION] [OPTIONS]`
- Per-crate Conventional Commit inference
- `-p/--package` (repeatable), `--workspace`, `--exclude`
- `--pre <IDENT>`, prerelease promotion (`bump release` or `--promote`)
- `--exact` (don't propagate to dependents' dep specs)
- `--dry-run`, `--yes`, dirty-tree guard (`--allow-dirty`)
- `--commit` (stages `Cargo.toml` + `Cargo.lock` + staged changelog if present), `--sign`
- Workspace inheritance (`version.workspace = true` → edit root)
- Skip `publish = false` crates by default
- `--output json` on `--dry-run`
- Bundled changelog: `--commit` also stages the changelog update if anodizer's changelog stage can render one
- Composition: `anodizer bump --commit && anodizer tag` is the intended flow

**Out of scope (v1):**
- Tag creation (`anodizer tag`'s job)
- Publishing (`anodizer publish`'s job)
- Push (user does this; or `anodizer tag --push`)
- Cross-workspace bumping (one workspace per invocation)

**Landed after the initial scaffold (now in scope):**
- Bundled changelog edits inside `--commit` — `crates/stage-changelog`
  exposes `render_crate_section` / `ChangelogUpdate` and bump writes the
  resulting `CHANGELOG.md` alongside `Cargo.toml` in a single commit.
- `--strict` enforcement against `crates[*].version` pins in `.anodizer.yaml`
  — strict mode refuses pin violations; non-strict logs a warning.
  Validation runs *before* the confirmation prompt so the user never
  approves an invalid plan.

## Syntax (final)

```
anodizer bump [LEVEL_OR_VERSION] [OPTIONS]
```

### Positional
- `patch` | `minor` | `major` — semver level
- `1.2.3` | `1.2.3-rc.1` — explicit version
- `release` — strip prerelease suffix (`1.0.0-rc.1` → `1.0.0`)
- Omitted — infer per-crate from Conventional Commits since each crate's last tag

### Selection (mutually exclusive group)
- `-p, --package <NAME>` — repeatable. Hidden alias `--crate`.
- `--workspace` / `--all` — every member not marked `publish = false` or excluded.
- Default when unambiguous: single-crate repo → implicit; multi-crate → requires explicit selection.

### Modifiers
- `--exclude <NAME>` — repeatable; pairs with `--workspace`
- `--pre <IDENT>` — append `-<ident>` (e.g. `--pre rc.1`)
- `--exact` — don't rewrite dependents' `[dependencies]` version specs
- `--allow-dirty` — override the uncommitted-changes guard
- `-y, --yes` — skip confirmation prompt
- `--dry-run` — emit plan, write nothing
- `--commit` — stage edits + one commit
- `--sign` — GPG-sign the commit (requires `--commit`)
- `--commit-message <TMPL>` — override default message template
- `--output <FMT>` — `text` (default) or `json`; JSON on `--dry-run` gives a machine-consumable plan

### Defaults (what matters)
- No selection, single-crate workspace → that crate
- No selection, multi-crate workspace → error: "specify `-p` or `--workspace`"
- No positional → infer per-crate from commits
- Dirty tree → refuse unless `--allow-dirty`
- Interactive stdout → prompt before write; `-y` or non-tty skips
- Default commit message: `chore(release): bump <crate> → <version>` (or summary list when multiple)

## Behavior details worth locking

### Inference rules (Conventional Commits → level)
- `BREAKING CHANGE:` footer, or `!` after type — major
- `feat(...)` — minor
- `fix(...)`, `perf(...)` — patch
- Anything else (`chore`, `docs`, `refactor`, `test`, `build`, `ci`, `style`) — no bump unless a tracked dep forces one
- Config hooks: respect existing `TagConfig.major_string_token` / `minor_string_token` / `patch_string_token` (same token set as `anodizer tag`)

### Propagation (default = propagate dep specs)
Matches `cargo set-version` default. Bumping `core` rewrites `cli`'s `[dependencies] core = "0.5.0"` to the new version. Does **not** bump `cli`'s own version — that's cargo's distinction. `--exact` turns this off.

### Workspace inheritance
`[workspace.package] version = "X"` + member `version.workspace = true`:
- Bumping any inheriting member bumps the root `[workspace.package]`
- All inheriting members are affected atomically
- Members with their own literal `version = "..."` are independent

### Non-publishable skip
`publish = false` in `[package]` → excluded from `--workspace`. Explicit `-p <name>` on such a crate is an error unless `--include-private` (probably not v1 — decide if needed).

### Plan table
```
Crate              Current   →   Next      Level    Reason
anodizer-core       0.4.2     →   0.5.0     minor    3 feat commits
anodizer-cli        0.4.2     →   0.4.3     patch    1 fix commit
anodizer-stage-foo  0.4.2     →   —         skip     no commits since core-v0.4.2
```
JSON form mirrors this as an array of objects.

### Composition with changelog
If `--commit` and the workspace has a changelog stage config:
- Render the changelog section for each bumped crate
- Stage the changelog file(s)
- Single commit includes both `Cargo.toml` edits and changelog edits
- Message: `chore(release): bump <crate> → <version>`

This is #9 from the gap list — what separates `anodizer bump` from "cargo set-version with a wrapper".

## Implementation plan

### Files to add
- `crates/cli/src/commands/bump.rs` — entry point + `BumpOpts` struct (mirror `TagOpts` pattern from `tag.rs`)
- `crates/cli/src/commands/bump/plan.rs` — build the plan table (pure function, testable)
- `crates/cli/src/commands/bump/cargo_edit.rs` — `Cargo.toml` rewriting (use `toml_edit` to preserve formatting/comments)
- `crates/cli/src/commands/bump/inference.rs` — commits → level mapping
- `crates/cli/tests/bump_integration.rs` — assert_cmd end-to-end

### Files to edit
- `crates/cli/src/lib.rs` — add `Bump { ... }` variant to the `Commands` enum
- `crates/cli/src/main.rs` — wire dispatch
- `.anodizer.yaml` (self-host) — optional bump config section if we add one

### Dependencies to add
- `toml_edit` (already transitively available via `cargo_toml`? verify; prefer toml_edit for format preservation)
- `semver` (already present)
- `cargo_metadata` or `guppy` for workspace graph — **check what's already in the tree** before adding

### Reuse (do not reinvent)
- `stage-changelog::parse_commit_message` — Conventional Commit parser
- `core::git::get_commits_between_paths` — per-crate commit scan
- `core::git::find_latest_tag_matching_with_prefix` — last tag per crate
- `core::config::TagConfig.*_string_token` — token overrides
- `core::log::StageLogger` — structured output for the plan table
- `cli::commands::tag::crate_name` semantics — the per-crate tag_prefix logic

### Phases (rough)
1. **Scaffold**: `Bump` command variant, `bump.rs` skeleton, `BumpOpts`, `--help` wiring. No behavior.
2. **Cargo.toml edit**: `toml_edit`-based rewriter with tests (single crate, single bump).
3. **Plan builder**: workspace walk, version resolution, plan table (pure, no IO).
4. **Inference**: commit-scoped per-crate level decision. Uses existing parser.
5. **Propagation**: dependent-spec rewriting. `--exact` to skip.
6. **Workspace inheritance**: `[workspace.package]` handling.
7. **Guards**: dirty tree, confirmation prompt, `--yes`.
8. **`--commit` + changelog bundling**: stage + commit + optional sign.
9. **JSON output**: plan → serde → stdout.
10. **Integration tests**: assert_cmd, fixture workspaces in `tests/fixtures/bump/`.

Each phase should end with `cargo test -p anodizer-cli` green before moving on.

## Testing

- **Unit**: `inference.rs` (commits → level), `cargo_edit.rs` (toml_edit roundtrips, preserving formatting), `plan.rs` (workspace graph → plan).
- **Integration** (assert_cmd):
  - Single-crate `patch` bump
  - Multi-crate `--workspace` with mixed inferred levels
  - `--dry-run` produces plan but no file changes
  - `--dry-run --output json` emits parseable JSON
  - Dirty tree refused without `--allow-dirty`
  - `publish = false` crate skipped from `--workspace`
  - `[workspace.package]` inheritance bumps root, not member
  - `--exact` does not rewrite dependents
  - `--commit` produces one commit with expected message
  - Propagation rewrites `[dependencies] core = "0.X.Y"` when core bumps

Fixture layout: `tests/fixtures/bump/<scenario>/` — one workspace per scenario, with a pre-seeded git repo (commits staged for inference tests).

## Open questions (decide during implementation)

1. **`--infer` as an explicit flag, or just the default when positional omitted?** My vote: positional-omitted = infer. Explicit `--infer` is noise.
2. **What to do when inference says "no bump" for every selected crate?** Probably exit 0 with "nothing to bump" message. Consider exit code: this is a legitimate no-op, not an error.
3. **`cargo_metadata` vs `guppy`?** `cargo_metadata` is simpler and sufficient. `guppy` is overkill unless we need transitive dep queries.
4. **Pre-release increment semantics.** `1.0.0-rc.1` + `--pre rc.2` → `1.0.0-rc.2`? Or error because `rc.2` isn't a level? My vote: `--pre rc.N` replaces whatever prerelease is there, so `1.0.0-rc.1` + `--pre rc.2` = `1.0.0-rc.2`. For increment-by-one, add a dedicated `--pre-bump` or require explicit.
5. **Should `bump` ever push?** My vote: no. Pushing is `anodizer tag`'s business (and arguably not even that — composable).
6. **How does this interact with `anodizer.spec` / `.anodizer.yaml` dist config?** At minimum, refuse bumps that would violate pinned versions elsewhere in the spec. Probably v1.1.
7. **Exit codes.** Should match anodizer's existing taxonomy (whatever it is). If it doesn't have one, this is a chance to introduce one — see cfgd's `crates/cfgd-core/src/exit.rs` for a recent precedent (Success=0, Error=1, ConfigInvalid=4, etc. with stable wire values locked by test).

## Success criteria for v1

- `anodizer bump` with no args in a cfgd-like workspace prints a plan, prompts, and on confirmation edits only the crates that actually have changes since their last tag, each at its inferred level.
- `anodizer bump --workspace --dry-run --output json` produces a structured plan a CI script can parse.
- `anodizer bump patch -p cli --commit` does the narrowest thing: edits one Cargo.toml, optionally stages changelog, one commit.
- `anodizer bump --commit && anodizer tag` is the documented release flow and it works end-to-end on the anodizer repo itself (dogfood it).
- No `cargo set-version` user would find the syntax surprising (positional level, `-p`, `--workspace`, `--exclude`, `--exact` all line up).

## Reference commands

Final invocation set (for docs):
```
anodizer bump                                  # infer, workspace (if single-crate) or prompt
anodizer bump patch -p cli                     # single crate, explicit level
anodizer bump 0.5.0 -p core                    # explicit version
anodizer bump minor -p core -p cli             # multi-select, same level
anodizer bump minor --workspace                # whole workspace, same level
anodizer bump --workspace                      # whole workspace, inferred per-crate
anodizer bump major -p core --pre rc.1         # prerelease
anodizer bump release -p core                  # strip prerelease
anodizer bump minor --workspace --exclude scratchpad
anodizer bump minor -p core --dry-run --output json
anodizer bump patch -p cli --commit --sign
```

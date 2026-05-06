# Anodizer file-decomposition handoff prompt

Reusable prompt template for one carve at a time. The full plan is at
`.claude/plans/2026-05-03-file-decomposition.md` — read it first; that's the
source of truth for the sized inventory, per-file carve maps, and execution
order.

## How to use

Pick the next undone item from the **Execution order** in the plan. Fill the
template below with the chosen carve map and dispatch a subagent (or work
through it yourself, one submodule at a time).

## Standing rules (apply to every carve)

- **Zero behavior change.** Pure structural movement.
- **Preserve public API** via `pub use` re-exports in the parent module.
- **No grab-bag `utils.rs` / `helpers.rs` files.** Shared helpers go in
  `crates/core/src/util.rs` (already exists; don't bloat it).
- **No new abstraction layers** (no new traits / pub types invented to
  "justify" the split).
- **Use `task commit -- -m "..."`**, NOT bare `git commit` — sandbox blocks
  it. `task commit` runs `task lint` + commits. See `CLAUDE.local.md`. Subject
  must not contain `#none`.
- **Verify before committing**: `cargo fmt --all` + `task lint` (auto-run by
  `task commit`) + `task test`.
- **Never push.** Never amend.
- **Module-boundaries rule** (`.claude/rules/module-boundaries.md`) — any new
  submodule under a `stage-*` crate inherits the umbrella allow-list. New
  submodules under `crates/core/` calling `Command::new` must be added to the
  allow-list explicitly.

## Recipe

1. **Re-verify line numbers.** The plan's snapshot is dated; carve maps are
   stale relative to current `mod tests` insertions and helper additions.
   Run `grep -nE '^(pub |impl |fn |struct |enum |trait )' <file>` and align
   the spec ranges before extracting.
2. **For each new submodule**:
   - Create the file with the use-clause header (`Write` for headers <100 lines).
   - `sed -n 'X,Yp' <source> > /tmp/staging.rs` to extract the block.
   - `cat header.rs /tmp/staging.rs > <dest>.rs`.
   - Add `mod <name>;` (or `pub mod <name>;`) and `pub use <name>::{...}` in
     the parent.
3. **Test-block handling**:
   - If tests cleanly partition by submodule, classify per-submodule.
   - Otherwise externalize the whole `mod tests` block to a sibling
     `tests.rs` in the **same commit batch**:
     - `sed -n 'TEST_START,$p' <source> > tests.rs`
     - Strip the outer `mod tests { ... }` wrapper from the copied content
       (the file is the module body).
     - In the parent, replace the entire block with `#[cfg(test)] mod tests;`.
     - **Do NOT dedent the inner block** — raw-string literals contain
       indented YAML that breaks if dedented. `cargo fmt` normalizes.
   - Items reached from `tests.rs` via `super::*` need at least `pub(super)`
     visibility. Default convention: `pub(super)` items + `use submod::*;`
     glob in `mod.rs`.
4. **Verify**:
   - `cargo fmt --all`
   - `task test` (full workspace; `task lint` is run by `task commit`)
5. **Commit** with `task commit -- -m "refactor(<crate>): carve <file> into
   <dir>/"`. One commit per carve. Don't push. Don't amend.

## Prompt template (paste into Agent dispatch)

```
Carve `<RELATIVE_PATH>` into the layout below.

Snapshot: 2026-05-03 (re-verify line numbers before extracting).

```
<CARVE_MAP_FROM_PLAN>
```

Hard rules:
- Zero behavior change. Pure structural movement.
- Preserve public API via `pub use` re-exports in the parent module — no
  caller in any other crate should need to change.
- No grab-bag `utils.rs` / `helpers.rs` files.
- No new traits or pub types invented for the split.
- Externalize the bottom `#[cfg(test)] mod tests` block to a sibling
  `tests.rs` in this same commit if it's >1,500 lines after the prod carve.
- Use `task commit -- -m "..."`, NOT bare `git commit`.
- `cargo fmt --all` + `task test` must pass before committing.
- One commit on master. Don't push. Don't amend.

Recipe:
1. Re-verify the line ranges against the current file
   (`grep -nE '^(pub |impl |fn |struct |enum |trait )' <file>`).
2. For each submodule: extract with sed, prepend a header (use-clauses), add
   `mod` declaration + `pub use` in the parent.
3. Test-externalization: `sed -n '<TEST_START>,$p' <file> > tests.rs`, strip
   outer `mod tests { ... }`, leave `#[cfg(test)] mod tests;` declaration in
   parent. Do NOT dedent the inner block.
4. Visibility: items reached from `tests.rs` need at least `pub(super)`.
   Default: `pub(super)` items + `use submod::*;` glob in `mod.rs`.
5. Verify: `cargo fmt --all` + `task test` (must be all green).
6. Commit: `task commit -- -m "refactor(<crate>): carve <file> into <dir>/"`.

Report back: the commit SHA, before/after line counts, any deviations from
the carve map (with reasoning), and any visibility decisions you had to make.
```

## Worktree posture

Per user preference (memory: `feedback_no_worktrees_branches.md`): work on
master directly. Spawn a worktree only when 2+ concurrent agents are editing
**disjoint** files. The plan flags which steps are safely parallelizable.

## Status board

Mark steps DONE here as they land. Update with commit SHA + before→after stats.

| Step | Status | Commit | Before → After |
|---|---|---|---|
| 1. `stage-sign/lib.rs` test externalization | ✅ DONE | `5d105dc` | 2,921 → 435 + tests.rs (2,485) |
| 2. `stage-nfpm/lib.rs` test externalization | ✅ DONE | `c322e12` | 6,447 → 1,730 + tests.rs (4,716) |
| 3. `core/config.rs` Wave A (promote + 6 sections) | ✅ DONE | `f2f264f` + `31093fa` (vis-fix) | 12,217 → mod.rs 4,725 + 6 submodules + tests.rs (5,662) |
| 4. `core/config.rs` Wave B (remaining sections) | ✅ DONE | `35d7ecf`, `20a5e5f`, `d3c2fd0`, `aa8df91` (rescue), `7ab18f4` (cleanup) | mod.rs 4,725 → 930; 30+ submodule files in `config/` and `config/publishers/` |
| 5a. `stage-build/lib.rs` | ✅ DONE | `a94ec8d` + `8a3c7a4` + `4b421ba` + `0a4082d` (tests-glob fix) | 4,873 → lib.rs 76 + 7 prod submodules + tests.rs (2,333) |
| 5b. `stage-docker/lib.rs` | ✅ DONE | `2595484` + `3922bbb` + `e357e29` (tests-glob fix) | 5,039 → lib.rs 53 + 8 prod submodules + tests.rs (2,636) |
| Cleanup. `tests.rs` glob removal (sign + nfpm + config) | ✅ DONE | `2390161`, `89b5811`, `1db9b64` | 3 tests.rs files now use explicit imports |
| Cleanup. config/ submodule glob removal (39 files) | ✅ DONE | `cefbafc` (A1, 10 files), `fe93408` (A2, 9 files), `eed5c07` (B, 13 files), `55ad7a0` (C, 7 files) | Zero `use super::*;` in entire `crates/core/src/config/` |
| 5c. `stage-release/lib.rs` | ✅ DONE | `b6a63ec` | 5,474 → lib.rs 185 + run.rs 990 + 5 github/ submodules + tests.rs (3,193) |
| 6a. `stage-publish/homebrew.rs` | ✅ DONE | `b9a4de3` | 2,864 → mod.rs 29 + 6 prod submodules + tests.rs (787) |
| 6b. `stage-publish/util.rs` | ✅ DONE | `dc0de39` | 2,310 → mod.rs 57 + 9 prod submodules + tests.rs (798) |
| 7a. `stage-changelog/lib.rs` | ✅ DONE | `a163ce3` | 4,863 → lib.rs 26 + 4 prod submodules + fetch/ (4 files) + tests.rs (2,980) |
| 7b. `stage-nfpm/lib.rs` prod carve | ✅ DONE | `78eb3f5` | 1,730 → lib.rs 34 + yaml.rs 277 + builders.rs 194 + generate.rs 358 + command.rs 69 + run.rs 854 |
| 7c. `stage-source/lib.rs` | ✅ DONE | `5b25663` | 2,350 → lib.rs 22 + archive.rs 350 + sbom.rs 231 + run.rs 625 + tests.rs 1,141 |
| 8a. `core/template.rs` | ✅ DONE | `684afc6` | 3,578 → mod.rs 43 + static_render.rs 31 + base_tera.rs 1,162 + vars.rs 174 + render.rs 215 + tests.rs 1,981 |
| 8b. `core/git.rs` | ✅ DONE | `dbfd40e` | 2,349 → mod.rs 50 + semver.rs 122 + detect.rs 160 + status.rs 55 + remote.rs 119 + tags.rs 468 + commits.rs 295 + github_api.rs 195 + tests.rs 943 |
| 8c. `core/template_preprocess.rs` | ✅ DONE | `fe56e8a` | 2,077 → mod.rs 75 + tokens.rs 144 + go_blocks.rs 286 + dots_dollars.rs 123 + builtins.rs 298 + positional.rs 364 + methods.rs 41 + tests.rs 801 |
| 9a. `stage-publish/nix.rs` | ✅ DONE | `6503e02` | 1,563 → mod.rs 16 + binary.rs 97 + hashing.rs 91 + generate.rs 509 + publish.rs 476 + tests.rs 403 |
| 9b. `stage-publish/chocolatey.rs` | ✅ DONE | `79fb6c4` | 1,689 → mod.rs 14 + nuspec.rs 162 + install.rs 91 + package.rs 373 + publish.rs 431 + tests.rs 636 |
| 9c. `stage-snapcraft/lib.rs` | ✅ DONE | `d210bf5` | 3,154 → lib.rs 21 + yaml.rs 124 + arch.rs 34 + generate.rs 190 + command.rs 71 + build_stage.rs 571 + publish_stage.rs 164 + tests.rs 2,023 |
| 9d. `stage-announce/lib.rs` | ✅ DONE | `d5de0ca` | 3,306 → lib.rs 35 + helpers.rs 179 + dispatch.rs 19 + run.rs 896 + tests.rs 2,210 |
| 9e. `stage-archive/lib.rs` | ✅ DONE | `0cf1c8f` | 5,146 → lib.rs 121 + run.rs 961 + tests.rs 4,093 (existing siblings file_specs.rs/formats.rs untouched) |
| 9f. `stage-blob/lib.rs` | ✅ DONE | `3b7220f` | 2,240 → lib.rs 17 + provider.rs 34 + kms.rs 264 + store.rs 168 + upload.rs 300 + run.rs 322 + tests.rs 1,204 |
| 9g. `stage-checksum/lib.rs` | ✅ DONE | `c5afb12` | 3,077 → lib.rs 25 + hashing.rs 149 + run.rs 619 + tests.rs 2,329 |

## Lessons learned

- **Never write `use super::*;` in submodules or extracted test files.** That was the sin of Wave A/B and stage-build/docker — required 9 cleanup commits and dozens of file edits. Always start with explicit per-file imports. The shortcut "I'll add `use super::*;` and let cargo tell me what's missing" feels efficient but creates invisible coupling that compounds across siblings; it forces tests.rs into wildcard dependence on whatever lib.rs happens to expose; and removing it later is much more expensive than doing it right the first time.
- **`task commit` runs `git add -u`**, which only stages **modifications to tracked files**. NEW files are silently dropped. Wave B's first three commits each declared `mod foo;` for files that were never staged, leaving mid-history unbuildable. **Always**: explicitly `git add <path>` for every NEW file BEFORE running `task commit`. After every commit, verify `git status -s` shows zero `??` entries (apart from gitignored).
- **Private items in `mod.rs` ARE visible to children declared via `mod child;`.** No visibility widening needed for children to import private parent helpers via `use super::name;`. (Children-of-children — e.g., `config/publishers/homebrew.rs` reaching `config/mod.rs` items — DO need `pub(super)` or `pub(crate)`, or `use super::super::name;` paths.)
- **Visibility default**: `pub(super)` reaches `mod.rs` and any `tests.rs` child. Use `pub(crate)` only when a non-module sibling needs the symbol. Bare `pub` for previously-private items is a leak — never do it.
- **Mid-history broken commits + rescue commit** is acceptable per "never amend" rule. Optionally squash before push (user discretion).
- **Plan deviations are fine when the result is better** — co-locating schema/serde helpers with their types beat a separate `schema.rs`/`serde.rs`. Document the deviation rather than retrofitting code to an outdated plan.

# Parity File Consolidation Session

## Your Task

Consolidate the parity tracking files into a single, authoritative document. You are NOT implementing features — you are organizing and verifying documentation only.

## Context

There are currently 4 parity-related files that overlap, contradict, and have stale entries:

1. `.claude/specs/goreleaser-parity-matrix.md` — Original parity matrix. Tasks 1-9 were implemented this session but the file still shows many items as Missing/Partial that are now Implemented.
2. `.claude/specs/fresh-parity-gap-analysis.md` — Independent gap analysis that found ~40 NEW gaps not in the original matrix (sections B1-B39).
3. `.claude/specs/parity-gap-analysis.md` — Oldest gap analysis (2026-03-25 baseline, 138 tests). Fully superseded by the other docs but never deleted.
4. `.claude/specs/goreleaser-complete-feature-inventory.md` — Raw GoReleaser feature inventory. Reference doc, not a gap tracker.

The target output is: `.claude/specs/parity-session-index.md` — which already exists with the session structure and checkboxes. This file should become the single source of truth.

There are also 2 files that are NOT part of this consolidation (leave them alone):
- `.claude/specs/community-adoption-feature-gaps.md` — post-release work
- `.claude/specs/test-parity-gap-matrix.md` — separate plan item for test coverage

## GoReleaser Source

GoReleaser is cloned at `/opt/repos/goreleaser`. Use it to verify claims. When a file says "Missing" or "Implemented", check the actual anodizer code AND the GoReleaser source to confirm.

## Definition of Parity

Parity means **equal or superior implementation** of each GoReleaser feature:

1. **Config field parity**: Every GoReleaser config field has an equivalent that is *wired through to behavior*. A parsed-but-ignored field is Missing.
2. **Behavioral parity**: Each feature produces the same output given the same input. Wrong defaults = gap.
3. **Wiring parity**: Config fields flow through to the stage that uses them. A field that's set but never read is Missing.
4. **Error parity**: Every GoReleaser error case has an equivalent. Different error behavior (warn vs error) = gap.
5. **Auth parity**: Credential chains must match.
6. **Default parity**: Every default value must match or be explicitly better.

Every Missing and Partial item must eventually be addressed. There is no "high priority" vs "low priority", no "niche", no "low value". Pro features are not excluded — we give them to the people for free.

## Rules for the Output File

The consolidated `parity-session-index.md` must:

1. **Contain every gap from all 4 source files** — nothing lost in consolidation
2. **Mark completed items with [x]** — only if you verify the feature actually exists in our code AND is wired to behavior (not just a config field that's parsed but ignored)
3. **Group remaining items into sessions** (A-F + Z) as already structured
4. **Include the session rules** (read GoReleaser source first, review loop until zero findings, etc.)
5. **Include the parity definition** above
6. **Stay under 300 lines** — use concise checkbox lines, not tables with 5 columns
7. **Note the GoReleaser source path** for each session so implementers know where to look

## Process

### Step 1: Read all 4 source files completely

Read every line of all 4 files. Build a mental inventory of every item mentioned anywhere.

### Step 2: Read the current parity-session-index.md

Understand the existing structure and what's already checked off.

### Step 3: Cross-reference against actual code

For every item marked as completed in the index (Phase 1 checkboxes), spot-check 5-10 of them against the actual Rust source to verify they're really implemented. Use `grep` to find the config fields and stage wiring.

For every item currently in the 4 source files that ISN'T in the index yet, add it to the appropriate session.

### Step 4: Write the consolidated file

Rewrite `parity-session-index.md` with:
- Everything from the existing index
- Every gap from the 4 source files that wasn't already captured
- Proper completed/incomplete status based on actual code verification
- Deduplication (many items appear in multiple source files)

### Step 5: First self-review

Re-read the consolidated file. Then go back to each of the 4 source files and scan for any item that's NOT in the consolidated output. If you find one, add it.

### Step 6: Second self-review

Read the consolidated file one more time with fresh eyes. Check:
- Is every session's GoReleaser source path correct?
- Are the session rules present and clear?
- Is the parity definition included?
- Is the file under 300 lines?
- Are completed items actually verified against code (not just trusted from a previous session's claim)?

### Step 7: Delete superseded files

Go file by file through the 4 source files. For EACH file:
1. Read it one final time
2. For every single item in it, confirm it exists somewhere in the consolidated output
3. Only after confirming 100% coverage, delete the file
4. Do NOT delete `goreleaser-complete-feature-inventory.md` — it's a reference doc, keep it

Files to delete after verification:
- `.claude/specs/goreleaser-parity-matrix.md`
- `.claude/specs/fresh-parity-gap-analysis.md`
- `.claude/specs/parity-gap-analysis.md`

File to keep as reference:
- `.claude/specs/goreleaser-complete-feature-inventory.md`

### Step 8: Final verification

Run `ls .claude/specs/*parity*` and `ls .claude/specs/*gap*` to confirm only the intended files remain.

## Do NOT

- Implement any features
- Modify any Rust source code
- Create worktrees or branches
- Dismiss any item as "niche" or "low priority"
- Trust previous session claims without spot-checking code
- Leave any item from the source files out of the consolidated output

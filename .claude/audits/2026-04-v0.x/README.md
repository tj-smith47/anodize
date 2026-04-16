# Anodize v0.x audit wave — 2026-04

Shared scratch for the anodize-parity and dogfooding-evidence agent teams.

## Owners

- **Wave A (parity):** `goreleaser-inventory-mapper`, `parity-auditor-*` (build, archive, publishers, announcers), `pro-features-skeptic`.
- **Wave C (dogfooding):** `feature-matrix-builder`, `evidence-collector`.

## Files

- `wave-status.md` — live status of A and C. Source of truth for `/audit-wave` router.
- `pro-<feature>.md` — per-Pro-feature deep audit (pro-features-skeptic).
- `parity-<area>.md` — per-area parity gap findings.
- `evidence-<feature>.md` — per-feature dogfooding proof links (evidence-collector).
- `pro-features-summary.md` — rollup of Pro audits.

## Handoff

- Consolidated findings → `/opt/repos/anodize/.claude/known-bugs.md` (push-gate consumes).
- Dogfooding page → `/opt/repos/anodize/content/dogfooding/_index.md` + README link.

## Handoff

When the team reaches a state where further progress requires the user to push, tag, or cut a real release, the lead writes `HANDOFF.md` with:

```markdown
# Handoff — anodize v0.x — YYYY-MM-DD

## Live-test items outstanding
- <feature/finding> — requires <release gesture> to verify
  - Command(s): `<exact command the user runs>`
  - Watch for: <artifact name | workflow URL | log pattern | signature file>
  - Rollback if failed: <steps>

## Needs live-release evidence
(rows in the dogfooding matrix blocked on a real release — evidence-collector → feature-matrix-builder fills these in automatically after the user runs the release)
- <feature> — needs <release type>; capture <URL/asset>

## Ready for: <suggested next action>
(e.g., "/wt release-rc1 → anodize release --snapshot inside the worktree → inspect dist/")
```

Teams do not attempt the release. Hooks deny push/tag/release gestures during an active wave; that's correct behavior.

## Rules

- Findings cite `file:line` or docs URL + fetched date. No memory citations.
- "Pre-existing" is not a valid excuse. Every finding is in scope.
- Write evidence honestly. ❌ is better than a fake ✅.

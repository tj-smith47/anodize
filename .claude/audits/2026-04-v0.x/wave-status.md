# Audit Wave Status — 2026-04-v0.x

Updated by the audit teams as they run. `/audit-wave` router reads this to pick the next incomplete wave.

## Wave A — anodize parity (CLI)

- [ ] A1 — `goreleaser-inventory-mapper` refreshed inventory with `ecosystem_relevance` + `disposition` columns AND emitted the completion statement block
- [ ] A2 — `parity-auditor-build-archive` findings written (filtered to required+strongly-suggested)
- [ ] A3 — `parity-auditor-publishers` findings written (filtered to required+strongly-suggested)
- [ ] A4 — `parity-auditor-announcers` findings written (filtered to required+strongly-suggested)
- [ ] A5 — `pro-features-skeptic` all Pro features audited (filtered to required+strongly-suggested)
- [ ] A6 — `rust-safety-scanner` findings written to `safety.md` AND countersigned every OSS `disposition=remove` candidate in `bloat.md` (each entry has `countersigned_by:` or `countersign_rejected:`)
- [ ] A7 — `dedup` skill run complete, findings in `dedup.md`
- [ ] A8 — `pro-features-skeptic` countersigned every Pro `disposition=remove` candidate in `bloat.md`
- [ ] A9 — Inventory completion statement reads `Completion achieved: yes`
- [ ] A10 — Lead consolidated BLOCKER findings into `anodize/.claude/known-bugs.md`; any `disposition=—` (undecided) rows written to known-bugs as "needs human decision"

## Wave A — anodize-action parity (tracked in `/opt/repos/anodize-action/.claude/audits/2026-04-v0.x/wave-status.md`)

A8–A10 live there; must all check before Wave A is considered complete.

## Wave C — dogfooding evidence (blocked on A1 completing)

- [ ] C1 — `evidence-collector` per-feature evidence files written
- [ ] C2 — `feature-matrix-builder` produced `content/dogfooding/_index.md`
- [ ] C3 — `anodize/README.md` links to dogfooding page

## Notes

Wave B (cfgd audit) status lives in `/opt/repos/cfgd/.claude/audits/2026-04-v0.x/wave-status.md`.

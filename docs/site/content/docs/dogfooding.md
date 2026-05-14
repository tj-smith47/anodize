+++
title = "Dogfooding Matrix: Resilience and Determinism"
description = "Which release-resilience and determinism features have shipped in real releases of anodize, brontes, and cfgd."
weight = 90
template = "docs.html"
+++

Anodizer is dogfooded against itself and two downstream consumers:

- **anodize** (this repo) - the releaser, releasing itself.
- **brontes** - small Rust CLI consumed via `cargo install`.
- **cfgd** - multi-crate workspace (CLI + lib + operator + CSI driver).

The matrix below tracks which release-resilience and determinism features
have been exercised by a *real* CI release run for each project. It is not a
feature list - the canonical feature list is
[What works (with proof)](../../dogfooding/). This page is narrower: it
answers "has feature X actually fired on tag push in this repo's pipeline
since the resilience work merged?".

## Motivating incident

The resilience work was driven by the anodize **v0.2.0 cascade failure** on
2026-05-12. Five consecutive Release runs failed in the publish stage:

- [Run 25754442852](https://github.com/tj-smith47/anodizer/actions/runs/25754442852)
- [Run 25749311448](https://github.com/tj-smith47/anodizer/actions/runs/25749311448)
- [Run 25733952026](https://github.com/tj-smith47/anodizer/actions/runs/25733952026)
- [Run 25716942017](https://github.com/tj-smith47/anodizer/actions/runs/25716942017)
- [Run 25712992215](https://github.com/tj-smith47/anodizer/actions/runs/25712992215)

The post-mortem is in `.claude/known-bugs.md` under "v0.2.1 release blockers".
Every row in the matrix below ties back to a behavior that, had it been
present, would have either prevented the cascade or made recovery a single
`--rollback-only --from-run=<id>` invocation.

## Feature matrix

The work landed on master in **commit `625c026`** on **2026-05-14**. No
post-merge release has cut yet, so the honest answer for every Resilience
and Determinism row is "not yet - first downstream release exercises it".
Rows fill in as v0.2.x+ tags ship.

| Feature | anodize | brontes | cfgd |
|---|---|---|---|
| Three-group Submitter gate (default-on) | not yet - merged 625c026 on 2026-05-14, first v0.2.x release exercises | not yet - pending downstream upgrade past 625c026 | not yet - pending downstream upgrade past 625c026 |
| `--no-gate-submitter` override | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| Opt-in rollback (`--rollback=best-effort`) | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| `--rollback-only --from-run=<id>` replay | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| `--fail-fast` (pre-resilience work) | exercised in v0.1.x release runs (default off; opt-in flag) | available via CLI; no recorded fail-fast trigger yet | available via CLI; no recorded fail-fast trigger yet |
| `--allow-nondeterministic <name>=<reason>` | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| `--summary-json=<path>` audit-trail | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| `announce.gate_on` config (default `required_publishers`) | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| `anodize check determinism --runs=N` harness | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| `anodize check config` (post-restructure) | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| "Non-deterministic exemptions:" block in release body | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| Preflight rollback-scope checks | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| AnnounceStage emit-summary-on-skip | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |
| BlobStage writes to `ctx.publish_report` | not yet - merged 625c026 on 2026-05-14 | not yet - pending downstream upgrade | not yet - pending downstream upgrade |

## How a row gets filled in

A row flips from "not yet" to an evidence link when:

1. The consuming repo upgrades to an anodize containing commit `625c026` or
   later (typically by bumping `tj-smith47/anodizer-action` or `cargo
   install`-ing a published version that includes the merge).
2. A real tag push triggers the Release workflow.
3. The workflow either exercises the feature on the happy path (e.g. the
   Submitter gate evaluates and decides to fire) or trips it (e.g. a
   required publisher fails and the gate aborts the irreversible group).
4. The CI run URL and tag URL go in the cell, replacing "not yet -".

Cells should *never* link a run that did not actually execute the feature.
A run that didn't hit the affected codepath is not evidence.

## Evidence: most recent release runs per repo

These are real `gh run list` snapshots taken on 2026-05-14. They establish
the baseline against which future rows fill in.

### anodize (this repo)

Latest release: [v0.2.0](https://github.com/tj-smith47/anodizer/releases/tag/v0.2.0)
(2026-05-12; cascade-failed - see motivating incident above).

Most recent Release-workflow runs (all v0.2.0 cascade, all failed):

- [Run 25754442852](https://github.com/tj-smith47/anodizer/actions/runs/25754442852) - FAIL
- [Run 25749311448](https://github.com/tj-smith47/anodizer/actions/runs/25749311448) - FAIL
- [Run 25733952026](https://github.com/tj-smith47/anodizer/actions/runs/25733952026) - FAIL
- [Run 25716942017](https://github.com/tj-smith47/anodizer/actions/runs/25716942017) - FAIL
- [Run 25712992215](https://github.com/tj-smith47/anodizer/actions/runs/25712992215) - FAIL

Last clean releases (pre-cascade):

- [v0.1.1](https://github.com/tj-smith47/anodizer/releases/tag/v0.1.1) - 2026-04-21
- [v0.1.0](https://github.com/tj-smith47/anodizer/releases/tag/v0.1.0) - 2026-04-20

### brontes

Latest release: [v0.2.0](https://github.com/tj-smith47/brontes/releases/tag/v0.2.0)
(2026-05-14).

Most recent runs:

- [Run 25845316544](https://github.com/tj-smith47/brontes/actions/runs/25845316544) - FAIL
- [Run 25823099560](https://github.com/tj-smith47/brontes/actions/runs/25823099560) - OK
- [Run 25817782297](https://github.com/tj-smith47/brontes/actions/runs/25817782297) - FAIL

### cfgd

Latest release: [v0.3.5](https://github.com/tj-smith47/cfgd/releases/tag/v0.3.5)
(2026-04-20).

Most recent runs:

- [Run 24648508657](https://github.com/tj-smith47/cfgd/actions/runs/24648508657) - OK
- [Run 24648508015](https://github.com/tj-smith47/cfgd/actions/runs/24648508015) - OK
- [Run 24648507473](https://github.com/tj-smith47/cfgd/actions/runs/24648507473) - FAIL

## Truthfulness rule

Every URL on this page is a real GitHub run/tag URL captured by the agent
that wrote this matrix. No row is fabricated. When the resilience work
exercises in a real release, the responsible commit (or its release PR)
edits this file to replace the "not yet" cell with the run URL that proves
it. Rows that stay "not yet" for more than a few v0.2.x cuts are review
findings, not acceptable state.

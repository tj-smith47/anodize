# Session L: Config/Defaults & Announce Gaps

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close all config default and announce provider gaps identified in the 2026-04-06 deep audit.

**Architecture:** Small, targeted changes to defaults and announce wiring. Each change touches 1-2 files max.

**Tech Stack:** Rust, serde, tera templates, reqwest (announce HTTP)

---

## Already Verified Complete

These items were verified against GoReleaser source and are already correctly implemented:

- Snapshot `version_template` default `"{{ Version }}-SNAPSHOT-{{ ShortCommit }}"` — `release.rs:297`
- Checksum `algorithm` default `"sha256"` — `stage-checksum/lib.rs`
- Git `tag_sort` default `"-version:refname"` — `git.rs:414`
- Archive default files (LICENSE*, README*, CHANGELOG*) — `stage-archive/lib.rs`
- Token file paths (`~/.config/goreleaser/{github,gitlab,gitea}_token`) — `config.rs:437+`
- Discord color default `3_888_754` — `discord.rs:18`
- Mattermost username default `"anodize"` — `lib.rs:430`
- Mattermost attachments: top-level `text` omitted when attachments present — matches GoReleaser

## Task 1: Release name_template default

**Files:**
- Modify: `crates/stage-release/src/lib.rs:786-795`

- [ ] Change fallback from `tag.clone()` to rendering `"{{ Tag }}"` template default
- [ ] Add test for default name_template rendering

## Task 2: ANODIZE_FORCE_TOKEN env var

**Files:**
- Modify: `crates/cli/src/commands/helpers.rs:345-355`

- [ ] Read `ANODIZE_FORCE_TOKEN` env var when `config.force_token` is None
- [ ] Parse lowercase values "github"/"gitlab"/"gitea" into ForceTokenKind
- [ ] Add tests for env var fallback

## Task 3: Slack username default

**Files:**
- Modify: `crates/stage-announce/src/lib.rs:230`

- [ ] Add `.or(Some("anodize"))` fallback for username

## Task 4: Discord author default

**Files:**
- Modify: `crates/stage-announce/src/lib.rs:152`

- [ ] Add `.or(Some("anodize"))` fallback for author

## Task 5: Teams icon_url

**Files:**
- Modify: `crates/stage-announce/src/lib.rs:398-400`

- [ ] No anodize avatar URL exists — document in code comment

## Task 6: Webhook Content-Type

**Files:**
- Modify: `crates/stage-announce/src/lib.rs:302`

- [ ] Change `"application/json"` to `"application/json; charset=utf-8"`

## Task 7: Discord color docstring

**Files:**
- Modify: `crates/core/src/config.rs` (DiscordAnnounce color field)

- [ ] Fix docstring from 3553599 to 3888754

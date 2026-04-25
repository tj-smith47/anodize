# announcers — parity audit (Pass B)

Audited: 2026-04-25
References: `/opt/repos/anodizer` HEAD, `/opt/repos/goreleaser` HEAD.
GR config types: `pkg/config/config.go:1345-1480`. Anodizer config: `crates/core/src/config.rs:4681-5036`. Dispatch: `crates/stage-announce/src/lib.rs`.

Re-grading rule applied: Go-toolchain-only fields not flagged as MISSING. Brand-default substitution (`GoReleaser` -> `anodizer`) flagged as default-divergence (one entry per provider — many announcers do this in lockstep).

---

## discord

GR struct config.go:1404-1410. GR pipe internal/pipe/discord/discord.go. Anodizer cfg config.rs:4795-4812; impl stage-announce/src/discord.rs + lib.rs:138-199.

### MISSING fields
- none.

### Default divergences
- `author` default: GR=`GoReleaser` (discord.go:19), anodizer=`anodizer` (lib.rs:166). Brand divergence.
- `icon_url` default: GR=`https://goreleaser.com/static/avatar.png` (discord.go:21), anodizer=None (lib.rs:187). Anodizer omits because no hosted avatar — embeds render without footer icon.

### Code smells
- lib.rs:160 — `require_rendered` runs `ctx.render_template` on the literal config string `webhook_url`; OK but error message just says "missing webhook_url" even when env vars are also unset.
- discord.rs:31-40 — when `author` is None but `icon_url` set, payload emits `author: { icon_url }` with no `name`; Discord may reject.
- lib.rs:169-186 — color parsing accepts only u32 base-10; GR uses `strconv.Atoi` which is signed int. Negative values rejected by anodizer; GR would accept and downcast.

### Validation gaps
- GR errors when `DISCORD_WEBHOOK_ID` / `DISCORD_WEBHOOK_TOKEN` env vars are missing (env tag `notEmpty`). Anodizer falls back to `webhook_url` which is more permissive — correct, anodizer-only intentional.

### Anodizer-only intentional
- `webhook_url` config field. GR has no config field; uses env-only.

---

## slack

GR struct config.go:1393-1402; pipe internal/pipe/slack/slack.go. Anodizer cfg config.rs:4953-4975; impl stage-announce/src/slack.rs + lib.rs:251-291.

### MISSING fields
- none.

### Default divergences
- `username` default: GR=`GoReleaser` (slack.go:17), anodizer=`anodizer` (lib.rs:265). Brand divergence.

### Code smells
- lib.rs:266-267 — `icon_emoji` / `icon_url` are NOT template-rendered; GR doesn't render them either, but anodizer renders `channel`/`username`. Inconsistent rendering policy across icon vs other fields.
- slack.rs:30-32 — defensive `as_object_mut().unwrap_or_else(|| return message.to_string())` is unreachable; obscures intent.
- lib.rs:269-276 — `serde_json::to_value(blocks)` happens even when no template vars — wasteful but harmless.

### Validation gaps
- GR errors when `SLACK_WEBHOOK` env var missing AND no webhook_url config (anodizer aligns: lib.rs:259).
- Neither GR nor anodizer validate `webhook_url` syntactically.

### Anodizer-only intentional
- `webhook_url` config field (GR uses env-only).
- Typed `SlackBlock` / `SlackAttachment` with template rendering on text fields.

---

## teams

GR struct config.go:1412-1418; pipe internal/pipe/teams/teams.go uses `messagecard` (legacy MS Teams MessageCard format). Anodizer cfg config.rs:4866-4882; impl stage-announce/src/teams.rs + lib.rs:445-479.

### MISSING fields
- none.

### Default divergences
- `icon_url` default: GR=`https://goreleaser.com/static/avatar.png` (teams.go:15), anodizer=None (lib.rs:467, comment justifies). Cards render without header avatar.
- `color` default: GR=`#2D313E`, anodizer=`#2D313E` (lib.rs:466). Match.

### Code smells
- teams.rs:80-99 — anodizer emits Adaptive Card v1.4 (`AdaptiveCard` schema, `themeColor` extension). GR posts a legacy MessageCard via `goteamsnotify` library. Different output format; old Teams clients may not render Adaptive Card.
- teams.rs:97-99 — `themeColor` placed on outer `message` envelope instead of inside the card; Teams documentation places it inside `MessageCard` not Adaptive Card. May not render the color band.
- lib.rs:457-461 — title template `{{ ProjectName }} {{ Tag }} is out!` hardcoded; GR also uses this default but stores in config struct after `Default()`. Anodizer renders inline so YAML round-trip differs.

### Validation gaps
- Both: webhook_url not URL-validated.

### Anodizer-only intentional
- `webhook_url` config field; GR uses env-only `TEAMS_WEBHOOK`.

---

## twitter

GR struct config.go:1373-1376; pipe internal/pipe/twitter/twitter.go uses go-twitter library v1 API (`Statuses.Update`). Anodizer cfg config.rs:4773-4781; impl stage-announce/src/twitter.rs + lib.rs:579-611.

### MISSING fields
- none.

### Default divergences
- none.

### Code smells
- twitter.rs:14 — anodizer POSTs to `https://api.x.com/2/tweets` (Twitter v2 API). GR uses v1 statuses endpoint via go-twitter library. v1 Statuses endpoint was decommissioned 2023; GR's path likely broken. Anodizer is correct for current API but diverges.
- twitter.rs:41-80 — hand-rolled OAuth1 signature; relies on BTreeMap ordering (correct) but no test for parameter ordering edge cases.
- lib.rs:583-597 — four sequential env reads emit four distinct error messages; consider one pre-check that lists all missing.

### Validation gaps
- No length validation on `message` (Twitter limits 280 chars; GR also doesn't enforce).

---

## mastodon

GR struct config.go:1378-1382 (note: `Server` is `yaml:"server"` without omitempty — required). Pipe internal/pipe/mastodon/mastodon.go requires THREE env vars: `MASTODON_CLIENT_ID`, `MASTODON_CLIENT_SECRET`, `MASTODON_ACCESS_TOKEN`. Anodizer cfg config.rs:4783-4793; impl stage-announce/src/mastodon.rs + lib.rs:616-639.

### MISSING fields
- none.

### Default divergences
- none.

### Code smells
- mastodon.rs:8-12 — anodizer authenticates with bearer token only (form-encoded `status` to `/api/v1/statuses`). GR (mastodon.go:42-52) uses go-mastodon library which issues client_id+client_secret+access_token. Anodizer's simpler flow is correct for Mastodon API (access token alone is sufficient) but diverges from GR's required env-var set.
- lib.rs:619-624 — empty `server` is a soft-skip (status log) where GR's Skip() returns true with no log. Slight UX divergence (anodizer logs, GR silent).
- lib.rs:632 — `mastodon::send_mastodon` called without User-Agent header (other senders use anodizer USER_AGENT). Consistency gap.

### Validation gaps
- `server` not URL-validated.
- No retry on transient 5xx (other senders also lack this; consistency).

### Env-var divergence (intentional, simplification)
- GR requires 3 env vars; anodizer requires 1 (`MASTODON_ACCESS_TOKEN`). Document migration note for users moving from GR.

---

## bluesky

GR struct config.go:1466-1470 (no `pds_url` field). Pipe internal/pipe/bluesky/bluesky.go has `pdsURL` baked at `New()` (line 31) defaulting to `https://bsky.social`; not user-configurable. Anodizer cfg config.rs:4715-4729; impl stage-announce/src/bluesky.rs + lib.rs:644-679.

### MISSING fields
- none.

### Default divergences
- `pds_url` default: GR=`https://bsky.social` (bluesky.go:20), anodizer=`https://bsky.social` (bluesky.rs:6). Match.

### Code smells
- bluesky.rs:51 — `Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)` — milliseconds precision; GR uses `time.RFC3339` which is seconds precision. Bluesky accepts both but YAML round-trip / fixture diffs.
- bluesky.rs:60-67 — facet detection uses `message.find(url)` (substring match). GR (bluesky.go:69) uses `strings.Index(msg, ctx.ReleaseURL)` — same logic. Match.
- lib.rs:658 — `release_url` extracted from template_vars then passed; if `ReleaseURL` template var is missing, no facet emitted (GR identical behavior).

### Validation gaps
- `username` not validated as Bluesky handle (e.g. `user.bsky.social`); GR doesn't either.

### Anodizer-only intentional
- `pds_url` config field — supports self-hosted PDS instances. GR pipe has the field on `Pipe` struct but no config exposure.

---

## webhook

GR struct config.go:1363-1371; pipe internal/pipe/webhook/webhook.go. Anodizer cfg config.rs:4814-4845; impl stage-announce/src/webhook.rs + lib.rs:296-368.

### MISSING fields
- none.

### Default divergences
- `message_template`: GR=`{ "message": "{{...}}"}` JSON envelope (webhook.go:21), anodizer=same `r#"{"message":"{{...}}"}"#` (lib.rs:33-34). Match.
- `content_type`: GR=`application/json; charset=utf-8` (webhook.go:26), anodizer same (lib.rs:347). Match.
- `expected_status_codes`: GR=`[200, 201, 202, 204]` (webhook.go:29-31), anodizer same (webhook.rs:24-26). Match.

### Code smells
- lib.rs:317-337 — Authorization precedence intentionally diverges from GR (config wins over env); documented at config.rs:4823-4834. Behavioral divergence noted.
- lib.rs:340-341 — User-Agent set to `anodizer_core::http::USER_AGENT` (e.g. `anodizer/x.y.z`). GR sends `goreleaser` literal (webhook.go:24). Brand divergence.
- webhook.rs:13-15 — `webhook_body` ignores `content_type` parameter (just returns message as-is). Naming implies more transformation than happens; consider deleting the wrapper.

### Validation gaps
- lib.rs:302-307 — anodizer validates `endpoint_url` parses as `reqwest::Url`. GR uses `url.ParseRequestURI` (webhook.go:73) — stricter (requires absolute URL); anodizer's `Url::parse` accepts relative-with-base which differs slightly.

### Anodizer-only intentional
- Typed `expected_status_codes: Vec<u16>`.

---

## smtp / email

GR struct config.go:1431-1441 — yaml key is `smtp`. Pipe internal/pipe/smtp/smtp.go. Anodizer cfg config.rs:4908-4933 — yaml key is `email` (no `smtp` alias). Impl stage-announce/src/email.rs + lib.rs:749-837.

### MISSING fields
- none.

### Default divergences
- YAML config key: GR=`announce.smtp`, anodizer=`announce.email`. **Configs migrating from GR break silently** — `announce.smtp:` is ignored. Anodizer should add `#[serde(alias = "smtp")]` on AnnounceConfig.email.
- `port` default: GR=0 (errors out — `errNoPort` smtp.go:98) requiring user to set port or `SMTP_PORT`; anodizer=587 (lib.rs:814). Anodizer is more permissive; users relying on GR's hard-fail get unexpected 587 connect attempts.
- `subject_template`: GR=`{{ .ProjectName }} {{ .Tag }} is out!` (smtp.go:18); anodizer same (lib.rs:768). Match.
- `body_template`: GR=`You can view details from: {{ .ReleaseURL }}` (smtp.go:19); anodizer same (lib.rs:773). Match.

### Code smells
- lib.rs:827-832 — sendmail/msmtp fallback when `host` unset. GR errors instead (`errNoHost`). Anodizer-only feature; documented intent but verify user expectation.
- email.rs:57-71 — port-465 selects SMTPS, all other ports STARTTLS. GR uses `gomail.NewDialer` which auto-detects. Anodizer behaviour-matches for 465 but plain SMTP (port 25) without TLS will fail under anodizer's STARTTLS branch.
- email.rs:54 — `Credentials::new(username, password)` always required; GR's gomail also requires creds via Dialer. Match.
- lib.rs:797-803 — `username` defaulted from `SMTP_USERNAME` env then errors when empty. GR matches via env tag (smtp.go:33,112-114). Match.

### Validation gaps
- lib.rs:754-759 — `from` validated to contain `@`. GR has no such check (relies on gomail to reject).
- lib.rs:761-763 — empty `to` validated; GR doesn't pre-check.

### Anodizer-only intentional
- Sendmail/msmtp fallback when SMTP host unset.
- Renamed `body_template` -> `message_template` with `#[serde(alias = "body_template")]` (config.rs:4929).

---

## mattermost

GR struct config.go:1420-1429; pipe internal/pipe/mattermost/mattermost.go. Anodizer cfg config.rs:4884-4906; impl stage-announce/src/mattermost.rs + lib.rs:484-531.

### MISSING fields
- none.

### Default divergences
- `username` default: GR=`GoReleaser` (mattermost.go:20), anodizer=`anodizer` (lib.rs:498). Brand divergence.
- `color` default: GR `#2D313E` BUT GR's Default() sets `ctx.Config.Announce.Teams.Color` (mattermost.go:48-49) — cross-pipe write bug. Anodizer correctly reads `cfg.color` from MattermostAnnounce (lib.rs:509) — deliberate upstream-bug fix; documented at lib.rs:502-508.
- `title_template`: GR=`{{ .ProjectName }} {{ .Tag }} is out!` (mattermost.go:22), anodizer same (lib.rs:514).

### Code smells
- mattermost.rs:35 — top-level `text: ""` always emitted (matches GR zero-value JSON serialization without omitempty). Documented at mattermost.rs:32-34.
- mattermost.rs:53-60 — single attachment block always emitted; matches GR.
- lib.rs:495-500 — channel/username/icon_url/icon_emoji template-rendered; GR doesn't render these.

### Validation gaps
- webhook_url not URL-validated.

### Anodizer-only intentional
- `webhook_url` config field (GR uses env-only `MATTERMOST_WEBHOOK`).
- Reads `mattermost.color` (not Teams.Color); fixes upstream bug.

---

## telegram

GR struct config.go:1448-1456; pipe internal/pipe/telegram/telegram.go (no `bot_token` config — env-only `TELEGRAM_TOKEN`). Anodizer cfg config.rs:4847-4864; impl stage-announce/src/telegram.rs + lib.rs:373-440.

### MISSING fields
- none.

### Default divergences
- `message_template`: GR=`{{ print .ProjectName " " .Tag " is out! Check it out at " .ReleaseURL | mdv2escape }}` (telegram.go:18). Anodizer=`{{ ProjectName ~ " " ~ Tag ~ " is out! Check it out at " ~ ReleaseURL | mdv2escape }}` (lib.rs:386). Tera vs Go-template syntax — same semantic output, but flagged as code-smell since template engine difference impacts user-supplied templates too. Per re-grading rule, template-engine syntax is not MISSING; cross-engine port (Go-template `print` -> Tera `~`) is a behavioral footgun for users copying GR docs.
- `parse_mode`: GR=`MarkdownV2` (telegram.go:20,49), anodizer=`MarkdownV2` (lib.rs:394). Match.

### Code smells
- lib.rs:395-405 — unknown `parse_mode` warns and defaults to MarkdownV2. GR silently overwrites without warn (telegram.go:45-50). Anodizer's behavior is better.
- lib.rs:409-426 — `message_thread_id` parsed to i64 with explicit error. GR (telegram.go:122-128) parses int64 too; match.

### Validation gaps
- lib.rs:383 — `chat_id` required (anodizer). GR's `getMessageDetails` doesn't fail when chat_id is empty; sends with empty `chat_id` and Telegram API errors. Anodizer fails earlier — better.

### Anodizer-only intentional
- `bot_token` config field (GR uses env-only `TELEGRAM_TOKEN`).

---

## reddit

GR struct config.go:1384-1391; pipe internal/pipe/reddit/reddit.go. Anodizer cfg config.rs:4935-4951; impl stage-announce/src/reddit.rs + lib.rs:536-574.

### MISSING fields
- none.

### Default divergences
- `title_template`: GR=`{{ .ProjectName }} {{ .Tag }} is out!` (reddit.go:13), anodizer same (lib.rs:551). Match.
- `url_template`: GR=`{{ .ReleaseURL }}` (reddit.go:14), anodizer same (lib.rs:554). Match.

### Code smells
- reddit.rs:25-33 — anodizer's password-grant OAuth2 flow uses Basic Auth + `grant_type=password`. GR uses `caarlos0/go-reddit/v3` which does same flow. Match.
- reddit.rs:48-58 — POSTs to `https://oauth.reddit.com/api/submit` with `kind=link`; matches GR's `SubmitLink`.
- lib.rs:539-547 — `application_id`, `username`, `sub` all `require_rendered` — GR doesn't pre-validate; anodizer fails earlier.

### Validation gaps
- No subreddit name format validation (e.g. starts with letter, length ≤ 21).
- Reddit API rate-limit headers not surfaced.

---

## linkedin

GR struct config.go:1443-1446; pipe internal/pipe/linkedin/linkedin.go. Anodizer cfg config.rs:4749-4757; impl stage-announce/src/linkedin.rs + lib.rs:684-703.

### MISSING fields
- none.

### Default divergences
- `message_template`: GR=`{{ .ProjectName }} {{ .Tag }} is out! Check it out at {{ .ReleaseURL }}` (linkedin.go:10), anodizer uses shared `DEFAULT_MESSAGE_TEMPLATE` (lib.rs:28-29). Match.

### Code smells
- linkedin.rs:56-79 — anodizer prefers `/v2/userinfo` then falls back to `/v2/me` on 403. GR uses `client.Share` from a custom doc.go; the fallback strategy is anodizer-original (and arguably more robust given LinkedIn API deprecations).
- lib.rs:691-694 — explicit empty-string check after env var resolution; GR's env-tag `notEmpty` does this implicitly. Match.

### Validation gaps
- Neither validates that `LINKEDIN_ACCESS_TOKEN` is a JWT or non-empty after rendering.

---

## opencollective

GR struct config.go:1458-1463; pipe internal/pipe/opencollective/opencollective.go. Anodizer cfg config.rs:4759-4771; impl stage-announce/src/opencollective.rs + lib.rs:708-744.

### MISSING fields
- none.

### Default divergences
- `title_template`: GR=`{{ .Tag }}` (opencollective.go:17), anodizer=`{{ Tag }}` (opencollective.rs:6). Match.
- `message_template`: GR=`{{ .ProjectName }} {{ .Tag }} is out!<br/>Check it out at <a href="{{ .ReleaseURL }}">{{ .ReleaseURL }}</a>` (opencollective.go:18), anodizer same (opencollective.rs:7). Match.

### Code smells
- opencollective.rs:14-46 — two-step GraphQL flow (createUpdate, publishUpdate) matches GR. The second `publishUpdate` mutation is in opencollective.rs (continued past line 80) — verify it always fires even when create succeeds without an id.
- lib.rs:711-714 — empty `slug` is a soft-skip with status log; GR's `Skip()` returns true silently when slug empty (opencollective.go:28). UX divergence.

### Validation gaps
- `slug` not regex-validated (OC slugs allow lowercase alphanumeric + hyphen).
- HTTP `Personal-Token` header sent over plain HTTPS — no token format check.

---

## Cross-cutting findings (apply to all 13)

- **Brand-default substitution**: many providers (slack, mattermost, discord) replace GR default username/author `GoReleaser` with `anodizer`. Counted once per affected provider above. Decision is intentional but a CHANGELOG migration note for GR users would prevent surprise.
- **Skip-when-empty UX**: GR's pattern is `Skip() returns true` (silent). Anodizer's pattern logs a status line ("opencollective: slug is empty — skipping"). Better UX, but consistent across mastodon/opencollective only — bluesky/discourse/etc. error instead.
- **Env-var fallbacks for `*_WEBHOOK`**: anodizer uniformly accepts `<PROVIDER>_WEBHOOK` env when `webhook_url` config absent (slack/teams/mattermost). GR uses env-only via `caarlos0/env` `notEmpty` tag. Anodizer-only intentional.
- **Template engine**: anodizer uses Tera; GR uses Go text/template. Default templates ported with appropriate syntax, but user-supplied templates copied from GR docs will fail (e.g. `print` builtin, `.ProjectName` dotted accessor with leading dot). Documented globally.
- **`announce.skip` field**: present in both (config.go:1346, config.rs:4684); anodizer accepts string-or-bool, GR same.
- **YAML key rename `smtp`->`email`**: only divergent key name in this set; missing serde alias breaks GR migration silently.
- **Webhook envelope JSON for non-webhook providers**: only `webhook` uses JSON envelope; all others use plain-text default. Match GR.

---

## Summary table

| announcer | MISSING | default-divergence | code-smell | validation-gap |
|---|---|---|---|---|
| discord | 0 | 2 | 3 | 1 |
| slack | 0 | 1 | 3 | 2 |
| teams | 0 | 1 | 3 | 1 |
| twitter | 0 | 0 | 3 | 1 |
| mastodon | 0 | 0 | 3 | 2 |
| bluesky | 0 | 0 | 3 | 1 |
| webhook | 0 | 0 | 3 | 1 |
| smtp | 0 | 2 | 4 | 2 |
| mattermost | 0 | 1 | 3 | 1 |
| telegram | 0 | 1 | 2 | 1 |
| reddit | 0 | 0 | 3 | 2 |
| linkedin | 0 | 0 | 2 | 1 |
| opencollective | 0 | 0 | 2 | 2 |
| **TOTAL** | **0** | **8** | **37** | **18** |

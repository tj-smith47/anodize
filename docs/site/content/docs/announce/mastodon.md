+++
title = "Mastodon"
description = "Post release announcements to a Mastodon instance"
weight = 46
template = "docs.html"
+++

## Config

```yaml
announce:
  mastodon:
    enabled: true
    server: "https://mastodon.social"
    message_template: "{{ .ProjectName }} {{ .Tag }} is out! Check it out at {{ .ReleaseURL }}"
```

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | bool | Enable Mastodon announcements |
| `server` | string | Full URL of your Mastodon instance (e.g. `https://mastodon.social`) |
| `message_template` | string | Toot text (templates supported). Default: `{{ .ProjectName }} {{ .Tag }} is out! Check it out at {{ .ReleaseURL }}` |

## Environment variables

| Variable | Required | Description |
|----------|----------|-------------|
| `MASTODON_CLIENT_ID` | Yes | OAuth application client ID |
| `MASTODON_CLIENT_SECRET` | Yes | OAuth application client secret |
| `MASTODON_ACCESS_TOKEN` | Yes | User access token for the posting account |

All three variables must be present and non-empty. Only `MASTODON_ACCESS_TOKEN`
is used for the actual API request (Bearer auth to `POST /api/v1/statuses`).
The client ID and secret are validated for GoReleaser compatibility and
forward-compatibility with future OAuth flows.

## Empty server handling

If `server` renders to an empty string, anodize logs a warning and skips the
Mastodon announcement without failing the pipeline.

## Obtaining credentials

1. Log in to your Mastodon instance.
2. Go to **Preferences → Development → New Application**.
3. Grant the `write:statuses` scope.
4. Copy the **Client key** → `MASTODON_CLIENT_ID`.
5. Copy the **Client secret** → `MASTODON_CLIENT_SECRET`.
6. Copy the **Your access token** → `MASTODON_ACCESS_TOKEN`.

## Example

```yaml
announce:
  mastodon:
    enabled: true
    server: "https://fosstodon.org"
    message_template: "{{ .ProjectName }} {{ .Tag }} is out! {{ .ReleaseURL }} #rustlang"
```

```
MASTODON_CLIENT_ID=abc123
MASTODON_CLIENT_SECRET=def456
MASTODON_ACCESS_TOKEN=xyz789
```

+++
title = "Milestones"
description = "Automatically close milestones after a release"
weight = 88
template = "docs.html"
+++

Anodizer can automatically close milestones on GitHub, GitLab, or Gitea after a release completes.

## Minimal config

```yaml
milestones:
  - close: true
```

## Milestone config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `close` | bool | `false` | Close the matching milestone |
| `name_template` | string | `{{ .Tag }}` | Milestone name to match (template) |
| `repo` | object | auto-detected | Override the repository (`owner`, `name`) |
| `fail_on_error` | bool | `false` | Fail the pipeline if milestone closing fails |

## Behavior

- Only acts when `close: true`
- The milestone name template is rendered and matched against open milestones
- Repository is auto-detected from the first crate's release config (GitHub/GitLab/Gitea)
- Provider is auto-detected from the release config
- Errors are logged as warnings by default; set `fail_on_error: true` to make them fatal

### Provider-specific behavior

| Provider | How milestones are found | How they are closed |
|----------|--------------------------|---------------------|
| GitHub | Paginated listing (100/page), title match | PATCH `state: "closed"` |
| GitLab | API filter by title | PUT `state_event: "close"` |
| Gitea | API filter by name | PATCH `state: "closed"` |

## Custom milestone name

Match a milestone with a name different from the tag:

```yaml
milestones:
  - close: true
    name_template: "v{{ .Version }}"
```

## Full example

```yaml
milestones:
  - close: true
    name_template: "{{ .Tag }}"
    fail_on_error: true
```

+++
title = "Gemfury"
description = "Push packages to Gemfury (fury.io)"
weight = 84
template = "docs.html"
+++

Anodize can push deb, rpm, and apk packages to [Gemfury](https://fury.io/) repositories.

## Minimal config

```yaml
fury:
  - account: myorg
```

## Gemfury config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `account` | string | **required** | Gemfury account name (template) |
| `ids` | list | none | Filter by build IDs |
| `formats` | list | `["apk", "deb", "rpm"]` | Package format filter |
| `secret_name` | string | `FURY_TOKEN` | Environment variable name for the API token |
| `disable` | string/bool | none | Disable this config |

## Environment variables

| Variable | Description |
|----------|-------------|
| `FURY_TOKEN` | Gemfury push token (or custom name via `secret_name`) |

## Behavior

- Pushes matching `linux_package` and `archive` artifacts via HTTP PUT to `https://push.fury.io/v1/{account}/`
- Authenticates with Bearer token
- Matches artifacts by file extension against the format filter
- Supports multiple entries and ID filtering

## Full example

```yaml
fury:
  - account: myorg
    formats:
      - deb
      - rpm
    secret_name: MY_FURY_TOKEN
```

+++
title = "NPM"
description = "Publish NPM binary wrapper packages"
weight = 86
template = "docs.html"
+++

Anodize can publish NPM packages that wrap your compiled binaries, allowing users to install them via `npm install -g`.

## Minimal config

```yaml
npms:
  - name: "@myorg/myapp"
```

## NPM config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | none | Unique identifier |
| `name` | string | **required** | NPM package name (e.g., `@myorg/myapp`) |
| `description` | string | none | Package description |
| `homepage` | string | none | Homepage URL |
| `keywords` | list | none | NPM package keywords |
| `license` | string | none | License identifier |
| `author` | string | none | Package author |
| `repository` | string | none | Git repository URL |
| `bugs` | string | none | Bug tracker URL |
| `access` | string | none | NPM access level (`"public"` or `"restricted"`) |
| `tag` | string | `latest` | NPM dist-tag |
| `format` | string | `tgz` | Download archive format (`tgz` or `zip`) |
| `ids` | list | none | Filter by build IDs |
| `url_template` | string | auto-derived | Download URL for binaries (template) |
| `extra_files` | list | none | Additional files to include |
| `templated_extra_files` | list | none | Template-rendered extra files |
| `extra` | map | none | Extra fields merged into package.json root |
| `if` | string | none | Template condition; skip if result is `"false"` or empty |
| `disable` | string/bool | none | Disable this config |

## How it works

Anodize generates:

1. A `package.json` with a `postinstall` script
2. A `postinstall.js` that detects the user's OS and architecture, downloads the correct binary from your release, and installs it

Users install with `npm install -g @myorg/myapp` and get a working binary.

## Authentication

NPM authentication uses the standard `npm` CLI auth mechanism. Configure credentials via `.npmrc` or `npm login` before running the release.

## Conditional publishing

Use `if` to conditionally skip NPM publishing:

```yaml
npms:
  - name: "@myorg/myapp"
    if: "{{ ne .Prerelease \"\" }}"
```

## Full example

```yaml
npms:
  - name: "@myorg/myapp"
    description: "A fast CLI tool"
    homepage: "https://example.com/myapp"
    license: MIT
    author: "My Org"
    repository: "https://github.com/myorg/myapp"
    bugs: "https://github.com/myorg/myapp/issues"
    access: public
    keywords:
      - cli
      - tool
    extra:
      engines:
        node: ">=14"
```

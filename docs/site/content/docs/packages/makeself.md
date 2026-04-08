+++
title = "Makeself"
description = "Create self-extracting archives with makeself"
weight = 66
template = "docs.html"
+++

Anodize can create self-extracting `.run` archives using [makeself](https://makeself.io/).

## Minimal config

```yaml
makeselfs:
  - script: ./install.sh
```

## Makeself config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | `default` | Unique identifier |
| `ids` | list | all builds | Filter by build IDs |
| `name_template` | string | `{project}_{version}_{os}_{arch}.run` | Output filename (template) |
| `name` | string | project name | Display name embedded in the archive |
| `script` | string | **required** | Startup script path (template) |
| `description` | string | none | LSM metadata description |
| `maintainer` | string | none | LSM metadata maintainer |
| `keywords` | list | none | LSM metadata keywords |
| `homepage` | string | none | LSM metadata homepage URL |
| `license` | string | none | LSM metadata license |
| `compression` | string | makeself default | Compression: `gzip`, `bzip2`, `xz`, `lzo`, `compress`, or `none` |
| `extra_args` | list | none | Extra `makeself` CLI arguments |
| `files` | list | none | Additional files to include |
| `goos` | list | `["linux", "darwin"]` | Target OS filter |
| `goarch` | list | all | Target architecture filter |
| `disable` | string/bool | none | Disable this config |

### File entries

Each entry in `files` can specify:

| Field | Alias | Type | Description |
|-------|-------|------|-------------|
| `source` | `src` | string | Source file path |
| `destination` | `dst` | string | Destination path inside archive |
| `strip_parent` | — | bool | Strip parent directory from source path |

## Prerequisites

The `makeself` command must be installed and available on PATH.

## Behavior

- Groups binary artifacts by platform (os + arch), creating one `.run` per platform
- Generates an embedded LSM (Linux Software Map) metadata file
- The `.run` extension is auto-appended if not present in the output name
- IDs must be unique across all makeself configs
- Skippable with `--skip makeself`

## Compression

Override the default compression method:

```yaml
makeselfs:
  - script: ./install.sh
    compression: xz
```

## Full example

```yaml
makeselfs:
  - id: installer
    script: ./scripts/install.sh
    name: "My App Installer"
    description: "Self-extracting installer for My App"
    maintainer: "Alice <alice@example.com>"
    license: MIT
    homepage: "https://example.com/myapp"
    compression: xz
    goos:
      - linux
    files:
      - src: config.example.yaml
        dst: config.yaml
      - src: LICENSE
    extra_args:
      - "--nox11"
```

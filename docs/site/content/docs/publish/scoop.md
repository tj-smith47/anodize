+++
title = "Scoop"
description = "Generate Scoop manifests for Windows package management"
weight = 4
template = "docs.html"
+++

Anodizer generates Scoop JSON manifests and pushes them to your bucket repository.

## Minimal config

```yaml
crates:
  - name: myapp
    publish:
      scoop:
        repository:
          owner: myorg
          name: scoop-bucket
```

## Scoop config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `repository.owner` | string | — | GitHub owner of the bucket repo |
| `repository.name` | string | — | Bucket repository name |
| `description` | string | none | Manifest description |
| `license` | string | none | License identifier |

## Generated manifest

The manifest includes:
- Download URL for the Windows archive
- SHA-256 checksum
- Binary extraction path
- `checkver` and `autoupdate` templates for automatic updates

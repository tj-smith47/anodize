+++
title = "Docker Digests"
description = "Configure Docker image digest artifact creation"
weight = 69
template = "docs.html"
+++

After pushing Docker images, Anodizer captures the image digest (sha256 hash) and writes it to artifact files. Digests provide immutable references to images, useful for signing and pinning.

## Minimal config

Digest creation is enabled by default. To customize:

```yaml
crates:
  - name: myapp
    docker_digest:
      name_template: "{{ .ProjectName }}_{{ .Version }}_digest.txt"
```

## Docker digest config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `disable` | string/bool | `false` | Disable digest artifact creation |
| `name_template` | string | tag-based naming | Template for digest artifact filename |

## Behavior

- After each Docker image push, the sha256 digest is extracted via `docker inspect`
- A per-tag digest file is written to the dist directory
- A combined `digests.txt` file is written containing all digest lines
- Digest files are registered as `docker_digest` artifacts
- The digest is also stored in artifact metadata under the `digest` key

## Disabling digests

```yaml
crates:
  - name: myapp
    docker_digest:
      disable: true
```

## Using digests

Digests are primarily useful for:

- **Docker signing** — tools like `cosign` sign by digest rather than tag
- **Manifest pinning** — Docker manifests use digests to reference specific image layers
- **Immutable references** — digests guarantee the exact image content, unlike mutable tags

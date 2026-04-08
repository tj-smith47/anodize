+++
title = "Artifactory"
description = "Upload artifacts to JFrog Artifactory"
weight = 80
template = "docs.html"
+++

Anodize can upload release artifacts to JFrog Artifactory repositories.

## Minimal config

```yaml
artifactories:
  - name: production
    target: "https://artifactory.example.com/repo/path/"
```

## Artifactory config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | **required** | Identifier used for env var lookup |
| `target` | string | **required** | Upload URL (template, artifact-specific vars available) |
| `mode` | string | `archive` | Artifact selection: `"archive"` or `"binary"` |
| `username` | string | env fallback | HTTP basic auth username |
| `password` | string | env fallback | HTTP basic auth password |
| `ids` | list | none | Filter by build IDs |
| `exts` | list | none | Filter by file extensions |
| `method` | string | `PUT` | HTTP method (`PUT` or `POST`) |
| `checksum_header` | string | `X-Checksum-SHA256` | Header name for SHA-256 checksum |
| `custom_headers` | map | none | Extra HTTP headers (template-rendered) |
| `checksum` | bool | `false` | Include checksum files |
| `signature` | bool | `false` | Include signature files |
| `meta` | bool | `false` | Include metadata.json and artifacts.json |
| `custom_artifact_name` | bool | `false` | Use artifact name as-is (don't append to target URL) |
| `extra_files` | list | none | Additional files to upload |
| `extra_files_only` | bool | `false` | Only upload extra files, skip artifacts |
| `client_x509_cert` | string | none | Path to client TLS certificate |
| `client_x509_key` | string | none | Path to client TLS private key |
| `trusted_certificates` | string | none | Path to CA certificate bundle |
| `skip` | string/bool | none | Skip this config |

## Environment variables

Credentials are resolved in this order:

| Variable | Fallback |
|----------|----------|
| Username | config value, then `ARTIFACTORY_{NAME}_USERNAME` |
| Password | `ARTIFACTORY_{NAME}_SECRET`, then `ARTIFACTORY_SECRET`, then config value |

Where `{NAME}` is the uppercased `name` field.

## Target URL templating

The `target` URL and `custom_headers` values support artifact-specific template variables:

| Variable | Description |
|----------|-------------|
| `{{ .ArtifactName }}` | Artifact filename |
| `{{ .ArtifactExt }}` | File extension |
| `{{ .Os }}` | Target OS |
| `{{ .Arch }}` | Target architecture |
| `{{ .Target }}` | Rust target triple |

## Full example

```yaml
artifactories:
  - name: production
    target: "https://artifactory.example.com/myapp/{{ .Version }}/{{ .ArtifactName }}"
    mode: archive
    custom_headers:
      X-Build-Number: "{{ .Env.BUILD_NUMBER }}"
    checksum: true
    signature: true
```

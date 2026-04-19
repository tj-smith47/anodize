+++
title = "Source RPM"
description = "Build source RPMs (.src.rpm) from your project"
weight = 68
template = "docs.html"
+++

Anodizer can build source RPM packages (`.src.rpm`) using `rpmbuild`.

## Minimal config

```yaml
srpm:
  enabled: true
```

## SRPM config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `false` | Enable SRPM generation |
| `package_name` | string | project name | RPM package name |
| `file_name_template` | string | `{{ PackageName }}-{{ Version }}.src.rpm` | Output filename (template) |
| `spec_file` | string | auto-generated | Path to RPM spec file template |
| `epoch` | string | none | RPM epoch |
| `section` | string | none | RPM section |
| `maintainer` | string | package name | Package maintainer |
| `vendor` | string | none | Package vendor |
| `summary` | string | package name | Summary line |
| `group` | string | none | RPM group |
| `description` | string | package name | Package description |
| `license` | string | `MIT` | License identifier |
| `license_file_name` | string | none | License file to include |
| `url` | string | `""` | Homepage URL |
| `packager` | string | none | RPM packager field |
| `compression` | string | none | Compression: `gzip`, `xz`, `zstd`, `none` |
| `docs` | list | none | Documentation files to include |
| `contents` | list | none | Additional contents (same format as nFPM contents) |
| `signature` | object | none | RPM signing configuration |
| `disable` | string/bool | none | Disable this config |

### Signature config

| Field | Type | Description |
|-------|------|-------------|
| `key_file` | string | Path to GPG key file |
| `passphrase` | string | GPG passphrase (falls back to `SRPM_PASSPHRASE` env var) |

## Prerequisites

- `rpmbuild` must be installed and available on PATH
- Exactly one source archive artifact must exist (from the archive stage with `format: tar.gz` or similar)

## Auto-generated spec

When no `spec_file` is provided, Anodizer generates a minimal RPM spec with `%autosetup`, `%build`, `%install`, `%files`, and `%changelog` sections.

## Custom spec file

Provide your own `.spec` template for full control:

```yaml
srpm:
  enabled: true
  spec_file: rpm/myapp.spec
```

The spec file is rendered through the template engine with additional variables:

| Variable | Description |
|----------|-------------|
| `{{ .PackageName }}` | RPM package name |
| `{{ .Source }}` | Source archive filename |
| `{{ .Summary }}` | Package summary |
| `{{ .License }}` | License identifier |
| `{{ .URL }}` | Homepage URL |
| `{{ .Description }}` | Package description |

## Behavior

- The `.src.rpm` extension is auto-appended if not present
- Respects global `--skip-sign` — signature config is cleared when signing is skipped
- Skippable with `--skip srpm`

## Full example

```yaml
srpm:
  enabled: true
  package_name: myapp
  summary: "A fast CLI tool"
  description: "My application does great things"
  license: Apache-2.0
  url: "https://example.com/myapp"
  vendor: "My Org"
  maintainer: "Alice <alice@example.com>"
  compression: xz
  docs:
    - README.md
    - CHANGELOG.md
  signature:
    key_file: /path/to/gpg-key.asc
```

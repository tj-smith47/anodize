+++
title = "NSIS"
description = "Create Windows installers with NSIS"
weight = 67
template = "docs.html"
+++

Anodize can create Windows `.exe` installers using [NSIS (Nullsoft Scriptable Install System)](https://nsis.sourceforge.io/).

## Minimal config

```yaml
crates:
  - name: myapp
    nsis:
      - {}
```

## NSIS config fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `id` | string | none | Unique identifier |
| `ids` | list | all builds | Filter by build IDs |
| `name` | string | `{{ ProjectName }}_{{ Version }}_{{ Arch }}_setup.exe` | Output installer filename (template) |
| `script` | string | built-in default | Path to a custom NSIS `.nsi` script (template) |
| `extra_files` | list | none | Additional files to include |
| `templated_extra_files` | list | none | Template-rendered extra files |
| `replace` | bool | `false` | Remove source archives, keeping only the installer |
| `mod_timestamp` | string | none | Reproducible build timestamp (template) |
| `disable` | string/bool | none | Disable this config |

## Prerequisites

The `makensis` command must be installed and available on PATH.

## Default script

When no custom `script` is provided, Anodize uses a built-in NSIS script with:

- Modern UI 2 (MUI2) interface
- Install and uninstall sections
- Desktop shortcut creation
- Admin execution level
- Installation to `$PROGRAMFILES\{ProjectName}`
- Uninstaller registration

The default script uses these template variables: `{{ ProjectName }}`, `{{ NsisOutputFile }}`, `{{ NsisBinaryPath }}`, `{{ NsisBinaryName }}`.

## Custom script

Provide your own `.nsi` script for full control:

```yaml
crates:
  - name: myapp
    nsis:
      - script: installer/myapp.nsi
```

Custom scripts are rendered through the template engine, so all template variables are available.

## Behavior

- Only processes Windows binary artifacts
- The `.exe` extension is auto-appended if not present in `name`
- Output is placed in `dist/windows/`
- `mod_timestamp` is applied to the staging directory and final output
- Skippable with `--skip nsis`

## Full example

```yaml
crates:
  - name: myapp
    nsis:
      - name: "MyApp_{{ Version }}_{{ Arch }}_setup.exe"
        extra_files:
          - LICENSE
          - README.md
        mod_timestamp: "{{ .CommitTimestamp }}"
```

+++
title = "Artifacts"
description = "Artifact types and the generated artifacts.json reference"
weight = 5
template = "docs.html"
+++

Anodize tracks every file produced during a release as an **artifact**. At the end of the pipeline, all artifacts are written to `artifacts.json` in the dist directory.

## Artifact types

| Type | Description |
|------|-------------|
| `binary` | Raw build output from `cargo build` |
| `uploadable_binary` | Binary marked for upload (checksummed, signed, released) |
| `universal_binary` | macOS universal binary (lipo merge of x86_64 + aarch64) |
| `library` | Shared/static library output |
| `header` | C header file from `cbindgen` |
| `c_archive` | C-compatible static archive |
| `c_shared` | C-compatible shared library |
| `wasm` | WebAssembly module |
| `py_wheel` | Python wheel package |
| `py_sdist` | Python source distribution |
| `archive` | Compressed archive (tar.gz, zip, etc.) |
| `source_archive` | Source tarball |
| `makeself` | Self-extracting archive |
| `linux_package` | Linux package (deb, rpm, apk, archlinux) |
| `snap` | Snap package |
| `publishable_snapcraft` | Snap package ready for Snapcraft upload |
| `flatpak` | Flatpak bundle |
| `source_rpm` | Source RPM (.src.rpm) |
| `disk_image` | macOS DMG disk image |
| `installer` | Windows installer (NSIS, MSI) |
| `macos_package` | macOS .pkg installer |
| `docker_image` | Docker image (legacy single-platform) |
| `docker_image_v2` | Docker image (Buildx multi-platform) |
| `publishable_docker_image` | Docker image ready for registry push |
| `docker_manifest` | Docker multi-arch manifest list |
| `docker_digest` | Docker image digest reference |
| `checksum` | Checksum file (SHA-256, etc.) |
| `signature` | Cryptographic signature |
| `certificate` | Signing certificate |
| `sbom` | Software Bill of Materials |
| `metadata` | The generated metadata.json file |
| `uploadable_file` | Template-rendered file for upload |
| `brew_formula` | Homebrew formula |
| `brew_cask` | Homebrew cask |
| `nixpkg` | Nix package expression |
| `scoop_manifest` | Scoop manifest |
| `publishable_chocolatey` | Chocolatey package ready for push |
| `winget_installer` | WinGet installer manifest |
| `winget_default_locale` | WinGet default locale manifest |
| `winget_version` | WinGet version manifest |
| `pkg_build` | AUR PKGBUILD |
| `src_info` | AUR .SRCINFO |
| `source_pkg_build` | AUR source package PKGBUILD |
| `source_src_info` | AUR source package .SRCINFO |
| `krew_plugin_manifest` | Krew plugin manifest |

## artifacts.json format

Each entry in `artifacts.json` contains:

```json
{
  "kind": "archive",
  "path": "dist/myapp-1.0.0-linux-amd64.tar.gz",
  "name": "myapp-1.0.0-linux-amd64.tar.gz",
  "target": "x86_64-unknown-linux-gnu",
  "crate_name": "myapp",
  "metadata": {
    "format": "tar.gz"
  },
  "size": 4404019
}
```

| Field | Description |
|-------|-------------|
| `kind` | Artifact type (see table above) |
| `path` | Path relative to the project root |
| `name` | Canonical filename |
| `target` | Rust target triple (null for non-build artifacts) |
| `crate_name` | Crate that produced this artifact |
| `metadata` | Freeform key-value pairs (format, id, binary, etc.) |
| `size` | File size in bytes (present only when `report_sizes: true`) |

## Uploadable artifacts

Not all artifact types are uploaded to releases. The following types are included in release uploads, checksumming, and signing:

`archive`, `uploadable_binary`, `source_archive`, `uploadable_file`, `makeself`, `linux_package`, `publishable_snapcraft`, `flatpak`, `source_rpm`, `sbom`, `py_wheel`, `py_sdist`, `checksum`, `signature`, `certificate`, `disk_image`, `installer`, `macos_package`

## Size reporting

Enable artifact size reporting to see a summary table at the end of the pipeline:

```yaml
report_sizes: true
```

This populates the `size` field in `artifacts.json` and prints a formatted table showing the size of each build output and uploadable artifact.

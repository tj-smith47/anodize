+++
title = "What anodizer builds"
description = "Artifacts the `anodizer release` pipeline produces: binaries, archives, packages, installers, containers, and signing material."
weight = 20
template = "section.html"
+++

# What anodizer builds

Output formats and the `builds[]` / `archives[]` / `dockers[]` / `signs[]`
keys that drive them. Native binaries for 6 targets ship on every release
(linux amd64/arm64, darwin amd64/arm64, windows amd64/arm64), built with
cargo + cargo-zigbuild + cross.

## Build

| Key | Status | Notes |
|---|---|---|
| `builds[].goos` / `builds[].goarch` | ✅ Verified | [v0.1.1 assets](https://github.com/tj-smith47/anodizer/releases/tag/v0.1.1) cover 6 targets (`*-linux-amd64.tar.gz` to `*-windows-arm64.zip`) |
| `universal_binaries[]` | ✅ Verified | [cfgd v0.3.5](https://github.com/tj-smith47/cfgd/releases/tag/v0.3.5) ships `cfgd-0.3.5-darwin-all.tar.gz` via `lipo` |
| `upx[]` | ✅ Verified | v0.1.1 binaries are UPX-packed |
| `builds[].overrides` | ✅ Verified | Used in production configs |
| `builds[].hooks.pre` / `post` | ✅ Verified | Wired |
| `builds[].mod_timestamp` | ✅ Verified | Reproducible build, wired in build config |
| `builds[].builder: prebuilt` | ✅ Verified | Tested |
| `builds[].buildmode` (no-compile) | ✅ Verified | Wired |
| `report_sizes` | ✅ Verified | Wired |

## Archives and checksums

| Format | Status | Notes |
|---|---|---|
| `tar.gz` | ✅ Verified | [`anodizer-0.1.1-linux-amd64.tar.gz`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer-0.1.1-linux-amd64.tar.gz) |
| `zip` | ✅ Verified | [`anodizer-0.1.1-windows-amd64.zip`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer-0.1.1-windows-amd64.zip) |
| `tar.xz`, `tar.zst`, `tgz` | ✅ Verified | Second `archives[]` entry emits all three; live artifacts attach to releases >= v0.2.0 |
| `source.format` | ✅ Verified | [`anodizer-0.1.1-source.tar.gz`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer-0.1.1-source.tar.gz) |
| `makeselfs[]` | ✅ Verified | [`anodizer-0.1.1-linux-amd64-installer.run`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer-0.1.1-linux-amd64-installer.run) (4 platforms) |

| Key | Status | Notes |
|---|---|---|
| `checksum.algorithm` | ✅ Verified | sha256 default. [`anodizer-0.1.1-checksums.txt`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer-0.1.1-checksums.txt). Full list: sha1/224/256/384/512, sha3-*, blake2s/2b, blake3, crc32, md5 |
| `checksum.split` | ✅ Verified | Per-artifact sidecar checksums wired |

## Linux packages

| Format | Status | Notes |
|---|---|---|
| `.deb` | ✅ Verified | [`anodizer_0.1.1_linux_amd64.deb`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer_0.1.1_linux_amd64.deb) (amd64 + arm64) |
| `.rpm` | ✅ Verified | [`anodizer_0.1.1_linux_amd64.rpm`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer_0.1.1_linux_amd64.rpm) (amd64 + arm64) |
| `.apk` | ✅ Verified | [`anodizer_0.1.1_linux_amd64.apk`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer_0.1.1_linux_amd64.apk) |
| `.src.rpm` | ✅ Verified | [`anodizer-0.1.1-1.src.rpm`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer-0.1.1-1.src.rpm) |
| `.snap` | ✅ Verified | [snapcraft.io/anodizer](https://snapcraft.io/anodizer), `latest/stable` channel |
| `archlinux`, `ipk`, `termux.deb` | 🤝 Help wanted | nFPM dispatch covered; not shipped live |

| Key | Status | Notes |
|---|---|---|
| `nfpms[].scripts` | ✅ Verified | Maintainer scripts: preinstall/postinstall/preremove/postremove |
| `nfpms[].contents` | ✅ Verified | cfgd ships `LICENSE` and `README` to `/usr/share/doc/cfgd/` |
| `NFPM_PASSPHRASE` env chain | ✅ Verified | Signed package env priority chain wired |

## macOS and Windows installers

Code-signing material and a real macOS or Windows runner are required
before these can ship live. Implementation is complete and unit-tested.

| Format | Status | Notes |
|---|---|---|
| `.dmg` | 🤝 Help wanted | Needs `dmgs[]` configured |
| `.pkg` | 🤝 Help wanted | Needs `pkgs[]` configured |
| `.app` bundle | 🤝 Help wanted | Needs `app_bundles[]` configured |
| `.msi` | 🤝 Help wanted | Needs `wixl`/`candle`/`light` on the runner |
| `.exe` (NSIS) | 🤝 Help wanted | Needs `makensis` on the runner |

| Key | Status | Notes |
|---|---|---|
| `notarize.macos` | 🤝 Help wanted | Cross-platform (rcodesign). Implementation requires `sign.certificate` (P12 file), `sign.password`, and `notarize.{issuer_id, key, key_id}`, i.e. an Apple Developer Program membership. Not dogfoodable on Linux runners without a paid Apple account |
| `notarize.macos_native` | 🤝 Help wanted | Needs Apple Developer cert on a macOS runner |

## Container images

| Key | Status | Notes |
|---|---|---|
| `dockers[]` | ✅ Verified | [ghcr.io/tj-smith47/cfgd](https://github.com/tj-smith47?tab=packages&repo_name=cfgd) (`cfgd`, `cfgd-operator`, `cfgd-csi`) |
| `docker_manifests[]` | ✅ Verified | Multi-arch (linux/amd64 + linux/arm64). Three manifests per cfgd release |
| `docker_v2` | ✅ Verified | cfgd ships three images per release with the modern config |
| `dockers[].build_args` / `labels` / `annotations` | ✅ Verified | All in use in cfgd's config |
| `docker_v2.sbom: true` | ✅ Verified | Three cfgd images carry inline SBOM |
| `docker_digest.name_template` | ✅ Verified | cfgd writes a digest manifest |
| `dockers[].use: buildx` | ✅ Verified | Default in CI |
| `docker_manifests[].use: docker` / `podman` | 🤝 Help wanted | Backend selector for `docker manifest create / push`. cfgd configures `docker_manifests[]` but the entries are bypassed because `docker_v2` already pushes multi-arch indexes (`docker: skipping manifest ... already pushed as multi-arch by docker_v2`). No live release exercises the non-buildx backend |
| `docker_hub.description` | 🤝 Help wanted | We use ghcr; needs a Docker Hub-anchored release |

## Signing

| Key | Status | Notes |
|---|---|---|
| `signs[]` (cosign) | ✅ Verified | [cfgd v0.3.5 cosign bundle](https://github.com/tj-smith47/cfgd/releases/download/v0.3.5/cfgd-0.3.5-checksums.txt.cosign.bundle). Cosign keyless for binaries and checksums |
| `signs[]` (gpg) | ✅ Verified | [`anodizer-0.1.1-checksums.txt.sig`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer-0.1.1-checksums.txt.sig) |
| `signs[].artifacts` | ✅ Verified | `archive`/`binary`/`checksum`/`sbom` per-artifact signatures wired |
| `docker_signs[]` | ✅ Verified | cfgd signs all three GHCR images per release with cosign |
| `binary_signs[]` | ✅ Verified | Build-time binary signing wired |
| `sboms[]` | ✅ Verified | CycloneDX via syft. [`anodizer-0.1.1.cdx.json`](https://github.com/tj-smith47/anodizer/releases/download/v0.1.1/anodizer-0.1.1.cdx.json) |
| `${artifact}` / `${document}` substitution | ✅ Verified | Wired |

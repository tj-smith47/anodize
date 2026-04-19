# Changelog — anodize-stage-pkg

## [0.2.0] - 2026-04-19

### Features

* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* af1641509a3ec71aa29f9eb6208a505fea8cc404 add platform-specific packaging stages — snapcraft, dmg, msi, pkg
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* dc0fb7a9da942bea0c287f720f85e756becd5b70 packaging parallelism + per-packager ConventionalFileName + DRY sweeps #none

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 216cd10b957e2683e08178b27a8a12ad45430f9d address 10 review findings for Task 4 (OSS template vars)
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* 64c9bab4765630a426095ddeff4fdd3b9b3db7f1 macOS PKG test, use zigbuild for cross-compilation
* a4a64d0d546b1985400efd2bfd28b66163324f31 unify extra_files to ExtraFileSpec across DMG, PKG stages

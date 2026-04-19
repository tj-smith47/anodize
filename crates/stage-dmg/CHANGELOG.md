# Changelog — anodize-stage-dmg

## [0.2.0] - 2026-04-19

### Features

* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* af1641509a3ec71aa29f9eb6208a505fea8cc404 add platform-specific packaging stages — snapcraft, dmg, msi, pkg
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* dc0fb7a9da942bea0c287f720f85e756becd5b70 packaging parallelism + per-packager ConventionalFileName + DRY sweeps #none
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 216cd10b957e2683e08178b27a8a12ad45430f9d address 10 review findings for Task 4 (OSS template vars)
* 0f44e2d7fdbdc962706af3fbeec1abb4e2b47493 address all code review findings
* bf6489a673763056b9c31f82641df077764a75b5 CI failures — docs freshness, packaging, tests, snapshot
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* a4a64d0d546b1985400efd2bfd28b66163324f31 unify extra_files to ExtraFileSpec across DMG, PKG stages

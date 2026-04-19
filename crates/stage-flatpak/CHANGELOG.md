# Changelog — anodize-stage-flatpak

## [0.2.0] - 2026-04-19

### Features

* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* 52dc4d0674596c7b5280297b88baf70a87911a14 Taskfile, cross-workflow artifacts, disable support, CI hardening
* dc0fb7a9da942bea0c287f720f85e756becd5b70 packaging parallelism + per-packager ConventionalFileName + DRY sweeps #none

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 216cd10b957e2683e08178b27a8a12ad45430f9d address 10 review findings for Task 4 (OSS template vars)
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step

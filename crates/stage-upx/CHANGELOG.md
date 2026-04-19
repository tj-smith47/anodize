# Changelog — anodize-stage-upx

## [0.2.0] - 2026-04-19

### Features

* e50fdd2c403824f2e32b67d26ea1d4c801ee2703 implement HTTP upload for Artifactory/Fury/CloudSmith + promote Homebrew Cask to top-level config
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 10a99885775db125e6eb2fcdb17bf19470619b24 add UPX binary compression stage
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* dc0fb7a9da942bea0c287f720f85e756becd5b70 packaging parallelism + per-packager ConventionalFileName + DRY sweeps #none
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* 10cbd0dc37fc2627e86ede11a23bd6ef4c5711f6 address all 8 code review findings for UPX binary compression

# Changelog — anodize-stage-notarize

## [0.2.0] - 2026-04-19

### Features

* e50fdd2c403824f2e32b67d26ea1d4c801ee2703 implement HTTP upload for Artifactory/Fury/CloudSmith + promote Homebrew Cask to top-level config
* 52dc4d0674596c7b5280297b88baf70a87911a14 Taskfile, cross-workflow artifacts, disable support, CI hardening
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none

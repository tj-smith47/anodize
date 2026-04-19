# Changelog — anodize-stage-snapcraft

## [0.2.0] - 2026-04-19

### Features

* a96cf297a32b5171a87c5d12a581dd88d9b4088c Session M — missing stages, milestones, cross-cutting parity
* 1685d6f36f908c80f158eae59ae3958d69d70b54 add 24 missing app fields, hooks, and structured extra_files
* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* af1641509a3ec71aa29f9eb6208a505fea8cc404 add platform-specific packaging stages — snapcraft, dmg, msi, pkg
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* dc0fb7a9da942bea0c287f720f85e756becd5b70 packaging parallelism + per-packager ConventionalFileName + DRY sweeps #none
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 673571dd9668035ddcb0d9a9ad0b9c23ad11ff41 upgrade all disable/skip_upload fields to Option<StringOrBool>
* eed4b33e87a1ca55275c7f6c3fbe2a9b4eb29898 address 7 code review findings for Task 7
* 216cd10b957e2683e08178b27a8a12ad45430f9d address 10 review findings for Task 4 (OSS template vars)
* 0f44e2d7fdbdc962706af3fbeec1abb4e2b47493 address all code review findings
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 91f7d7f13df7deebe4f54ccca223129f11ff1324 strict-mode bulletproofing + targets subcommand + publisher safety #none

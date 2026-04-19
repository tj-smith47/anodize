# Changelog — anodize-stage-nsis

## [0.2.0] - 2026-04-19

### Features

* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* c5c8026c22dc896660390980ea3ae134b73b316b add stage-nsis crate for NSIS Windows installer generation
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* dc0fb7a9da942bea0c287f720f85e756becd5b70 packaging parallelism + per-packager ConventionalFileName + DRY sweeps #none
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 3230ff923f6e334350daf05603d66a69dd3c1725 address code quality review findings
* 216cd10b957e2683e08178b27a8a12ad45430f9d address 10 review findings for Task 4 (OSS template vars)
* 0f44e2d7fdbdc962706af3fbeec1abb4e2b47493 address all code review findings
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 23f25f6ea042e5355985e1d6f4fcb427381d32f4 final review findings — SBOM arg ordering, AppBundle replace/disable, NSIS mod_timestamp

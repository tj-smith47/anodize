# Changelog — anodize-stage-nfpm

## [0.2.0] - 2026-04-19

### Features

* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* 3f9861df7afe12a14f2a6d33761ce78539008e8e add Release and Epoch template vars for file_name_template
* 4214474b5a5d3db6fcfc4ad54f0d6eb55eb3452f add libdirs, changelog, and owner/group template rendering
* 30333462b49190a4c37e96004a210f2f0ef796d6 Session K — nFPM IPK format, template rendering, publisher behavioral gaps
* d7e6615d5eeb8f7589496e00b8d5c48a8b6e57a9 Linux package generation via nfpm
* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 2c78e3c173f52ebc780954dd51266c748d966f9a add scripts, package metadata, and content type/file_info to nfpm stage
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* dc0fb7a9da942bea0c287f720f85e756becd5b70 packaging parallelism + per-packager ConventionalFileName + DRY sweeps #none
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 75c56d73bedc1a31a0470178cbcb27bf096111c9 nfpm test gating, Windows test cross-platform, stage-release pkg dep #none
* 882d224b038e2e58d13b3dba5996ebbada120ab0 strengthen Release/Epoch test assertions and add combined test
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* b49a23f88b4e2f98f85814d21d53ed315c3435f2 Windows test compat for /dev/null paths, unused var warning
* 97549d213dddec2aed5995ed0ace0df6a84e9199 address all code review findings for Tasks 8-12
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* ba1bbf19fd6307bca864eda2a8cb936488eb91d0 checksum archives-disabled check, nfpm per-target iteration and template support
* fbe8ddd1d46124fecd05a1f80a0cc1b22a826b21 clippy unnecessary_unwrap and fmt in nfpm test
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* 9687d2e0038336ab90dc89c5febe03ea8181d1e8 nfpm mode serialization and initial tag version
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E

### Others

* 4f603d1d030f88cd2582039ad9e593dc0fce6ab7 replace hand-built format strings with Tera templates and serde serialization
* a6a2f986ccdb28c3ea3fe4d3c33ac6b5dc07858d harden unwrap paths, secret handling, path traversal, and regex injection
* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules
* 5de4f85a4e9ab108bd058dd071bbc92922d900e9 add 60 stage behavior tests verifying config fields produce correct output
* 6b0452c6855e459b604cf36b30330ef8555cb54c add error path tests for nfpm, changelog, and checksum stages

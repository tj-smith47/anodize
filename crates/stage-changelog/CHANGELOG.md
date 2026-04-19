# Changelog — anodize-stage-changelog

## [0.2.0] - 2026-04-19

### Features

* d4916567136050891b33a46cb22d4f8a27202c72 add gitlab and gitea backends
* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* 91527e8c12e7fe5689ac7794918e59c64387b28f wire GitConfig fields to tag discovery behavior
* 7cf4bfaec3851d786cc51b4660b018012255e06c expand ArtifactKind to 38 variants + changelog Pro features
* c0acd12f745ac8832da2061ef14394c41f6662ce Session H — release & changelog behavioral gaps
* efb3d3b550c643c3dcb78593e9ee0c79aecc69fe conventional commit changelog generation
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 52dc4d0674596c7b5280297b88baf70a87911a14 Taskfile, cross-workflow artifacts, disable support, CI hardening
* 6d8b2adadab8b0433203124c50996bb8dec3d56f add filters.include, abbrev, and use: github-native to changelog config
* 01030a6fe4be2a18948af275a0800a2ae0575adb add make_latest, changelog header/footer, and disable fields to config
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 6b654913b0b6acf9922da95b03b0b7edf2aa51fd address code review findings for gitlab/gitea backends
* d0cf3c5951007d4eab01a98abba9c27411acf0d1 close 4 GoReleaser parity gaps in GitConfig wiring
* c71179146f31bb7b311738f14a2c985dc1d5ae7e address all code review findings for monorepo config feature
* 1f067c4c0e9fa6bad26d91b5f3b3c5add27ef88d per-crate changelogs, ReleaseURL template var, single tokio runtime
* a5ec51da01245cad586a469eb27874b52db278a6 dry-run, lazy regex, invalid regex warnings, initial release support
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* b49a23f88b4e2f98f85814d21d53ed315c3435f2 Windows test compat for /dev/null paths, unused var warning
* 97549d213dddec2aed5995ed0ace0df6a84e9199 address all code review findings for Tasks 8-12
* d6607d0bd37e3ef1e2b062951e53470b0843d630 address all v0.3.2 release failures across 4 workspace crates #none
* b224243165c8229f6f3488cc6e724208ecfd482f address code review findings from Task 3C
* d64d36607430dc9537fdf56895410abf37abc568 address code review issues for Tasks 2I and 2J
* aac8408209b439f1091e65ef4fa941caab23c903 address post-fix code review findings for Tasks 11+12
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* fe6b4dcf01c3a8103177f26cfdef45d3fb5040b5 changelog tests use tempdir for dist to avoid Windows file lock contention
* fbe8ddd1d46124fecd05a1f80a0cc1b22a826b21 clippy unnecessary_unwrap and fmt in nfpm test
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 17bb8f7834b1451e6acdb7ba6c1785e32ec4898e resolve flaky tests, duplicated helpers, and unfalsifiable assertions
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E

### Others

* ecd50adb6b49550bd9c902e03726389c10a00b57 deep dedup pass + wire all dead CLI flags and config fields
* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* a6a2f986ccdb28c3ea3fe4d3c33ac6b5dc07858d harden unwrap paths, secret handling, path traversal, and regex injection
* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules
* 5de4f85a4e9ab108bd058dd071bbc92922d900e9 add 60 stage behavior tests verifying config fields produce correct output
* f2b7f162f6d34d30bb61ab1ba5f053a45c9aecd4 add deep integration tests for archive, checksum, changelog, and publish stages
* 6b0452c6855e459b604cf36b30330ef8555cb54c add error path tests for nfpm, changelog, and checksum stages

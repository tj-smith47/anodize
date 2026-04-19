# Changelog — anodize-stage-checksum

## [0.2.0] - 2026-04-19

### Features

* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* 14fbec88434766e4d5f302defc481545227dc405 SHA256/SHA512 checksum generation
* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 01030a6fe4be2a18948af275a0800a2ae0575adb add make_latest, changelog header/footer, and disable fields to config
* 191b5c20759b385fb2ae491303bc2efb2b61436d add sha1, sha224, sha384, blake2b, blake2s algorithms and extra_files/ids to checksum stage
* 26a9e353f60b74c7f93d5410232656548d35c30f add shared test helpers module and mock GitHub client trait
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* fede6499895cb0648b693fb5dd78a9b866259277 publisher sha256 key, darwin-universal arch, EasyCLA-blocking commit author #none
* eed4b33e87a1ca55275c7f6c3fbe2a9b4eb29898 address 7 code review findings for Task 7
* 0f44e2d7fdbdc962706af3fbeec1abb4e2b47493 address all code review findings
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* 4e655d929ee55ec2886094fc7dc87841d608b2d9 Windows test compat, workflow improvements, lint cleanup
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* ba1bbf19fd6307bca864eda2a8cb936488eb91d0 checksum archives-disabled check, nfpm per-target iteration and template support
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* c9f5cc3bf62e498df7bbb5fb283a94f9c6a1223f wire TestContextBuilder into stage tests, add mock GitHub release test
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E

### Others

* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules
* 5de4f85a4e9ab108bd058dd071bbc92922d900e9 add 60 stage behavior tests verifying config fields produce correct output
* f2b7f162f6d34d30bb61ab1ba5f053a45c9aecd4 add deep integration tests for archive, checksum, changelog, and publish stages
* 6b0452c6855e459b604cf36b30330ef8555cb54c add error path tests for nfpm, changelog, and checksum stages

# Changelog — anodize-stage-sign

## [0.2.0] - 2026-04-19

### Features

* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* e50fdd2c403824f2e32b67d26ea1d4c801ee2703 implement HTTP upload for Artifactory/Fury/CloudSmith + promote Homebrew Cask to top-level config
* 472f947baa9e7bae817c66f45f80e0b85700f9ba Session J — sign & docker behavioral gaps
* 5b6757a8bf3433bc73ab2c9975859ce22b90f485 GPG and cosign signing
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 19801c9642266985acc6ad2dd97cd6b35e22bc08 SkipMemento wiring + BinarySignStage + anodize build parity #none
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 56bea2d3b97636ee773b4fa88b0faf6380a603a7 migrate sign to signs[] array with backward compat and new fields
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* c30bc496b809b738263a909bf358a6da06edb515 change DockerSignConfig.output to StringOrBool for GoReleaser parity
* 0adcfc1ca6d6e36af8dfc7e70cb20451f4e7e888 sign ID default, use-backend default, COPY/ADD file listing, push digest capture
* 15300788f0dd1c8b0681dd96e863f2b914a29c5c SignConfig.output to StringOrBool + template evaluation
* 83c57f31a5a15393dedb1175b82f82f64997a8f0 remove unused StringOrBool import from lib scope
* eed4b33e87a1ca55275c7f6c3fbe2a9b4eb29898 address 7 code review findings for Task 7
* bb0f87646e010af0302c89cc5e2e2e2592e952a3 resolve template vars in sign args, apply docker_signs artifact filter
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* a9da09ca5145da42d5545b5124b7e43bb3813474 Windows compat for sign stage tests that execute shell commands
* 10cbd0dc37fc2627e86ede11a23bd6ef4c5711f6 address all 8 code review findings for UPX binary compression
* 9244126789a27977823ef15dd60584fdfafb00fb address code review issues for Task 2E sign stage
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* f7d483dd094e184b56cc26011cce6e42eab553a7 docker_signs as separate stage post-docker, UploadableBinary filter for winget/homebrew/publisher util #none
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* dcd3fc7b065d4d70618d2294b3b24b727b420e7e handle dot-less template placeholders in sign arg resolution
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 19409b4f43b0250cbf6bd633f4b222539ce3e60e strengthen stage behavior tests to verify actual behavior, not just is_ok()
* c9f5cc3bf62e498df7bbb5fb283a94f9c6a1223f wire TestContextBuilder into stage tests, add mock GitHub release test
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E

### Others

* ecd50adb6b49550bd9c902e03726389c10a00b57 deep dedup pass + wire all dead CLI flags and config fields
* a6a2f986ccdb28c3ea3fe4d3c33ac6b5dc07858d harden unwrap paths, secret handling, path traversal, and regex injection
* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules
* 5de4f85a4e9ab108bd058dd071bbc92922d900e9 add 60 stage behavior tests verifying config fields produce correct output

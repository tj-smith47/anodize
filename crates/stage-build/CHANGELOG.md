# Changelog — anodize-stage-build

## [0.2.0] - 2026-04-19

### Features

* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* c3ac8c67d2d302da4a45efcf467d0b5bff390c37 cargo/zigbuild/cross build orchestration
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 34186a9ad564c60266c4cb7edd0fd5a6011904f3 add Rust-specific first-class features (binstall, version sync, cdylib/wasm32)
* 20fd04a3254b138acb044da256df83c0027cc141 add jsonschema command, env_files, config versioning, build ignore/overrides
* 058e8651cc71c94d0dd739ab7cbe46889debb10a add macOS universal binary support via lipo
* ae5053bd36ef6919d79e18a87fc7cd13b8038258 add reproducible build support with SOURCE_DATE_EPOCH
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 9e40eb16a7486d60d1cb381eab748aef42ab5ccb keep cargo for same-OS cross-arch when host toolchain is universal
* 68c9ef715071ba9dc44b938a2349731e487312ca use cargo for native targets even under Auto strategy
* eed4b33e87a1ca55275c7f6c3fbe2a9b4eb29898 address 7 code review findings for Task 7
* c9a7bec11934107f66ba32339c19f13a2d983898 copy_from skip compilation, workspace target dir, auto default strategy, per-crate cross
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* f608424b79264651db334a145df3e86f991b332d address all 11 code review findings for Task 5M (CLI + Config)
* 10cbd0dc37fc2627e86ede11a23bd6ef4c5711f6 address all 8 code review findings for UPX binary compression
* 0beb30ee22095f52eef8dfc0b50c52b738be9eca address all split/merge code review findings
* 5e97b0bb4108b161cdc50b21e70158a335181bf4 address code review issues for Task 5A
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 91f7d7f13df7deebe4f54ccca223129f11ff1324 strict-mode bulletproofing + targets subcommand + publisher safety #none
* 1b3bb18a40b7cea67cfc8dfcabcd94abbc5d975e universal binary ids filter matches on binary name metadata
* d6b4c72da7d2d1459969222044cc3885e9b8367c universal_binaries skip is not a strict error on non-macOS builds
* ce3e3962742afaa7d5e8c0157558f5a57fbc22c2 version_sync updates workspace deps, respects Cargo.toml version, skips CI
* fee9ad1c4652d5e166a94922210725052769de19 wire stage-scoped template vars (Binary, ArtifactName, ArtifactPath)
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E
* a73a3abc6d6eb73ea7fe143c80297c6fa7b7876f wire up ids filter and replace field, make lipo failure non-fatal

### Others

* ecd50adb6b49550bd9c902e03726389c10a00b57 deep dedup pass + wire all dead CLI flags and config fields
* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* a6a2f986ccdb28c3ea3fe4d3c33ac6b5dc07858d harden unwrap paths, secret handling, path traversal, and regex injection
* c8b1dd32d6b37a5d7f484d95886ca847d4d05928 add 28 error path tests across config, template, build, release, and CLI
* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules

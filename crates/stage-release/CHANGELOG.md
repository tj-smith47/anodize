# Changelog — anodize-stage-release

## [0.2.0] - 2026-04-19

### Features

* 4c6d86c7f8d7cb40b97e7983699e3bfa6c2cfe96 Session L — config defaults, ANODIZE_FORCE_TOKEN, announce provider parity
* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* 80e85168be8a119c3ac198e46903aa26a428158d add GitLab release backend
* b73a9bde9d0248f2cfb78e33264b82ce3a298219 add Gitea release backend
* a5b4cbb83afa37303f4cbd67fa24ff88829d1ccc add release.tag template override for GitHub release tag
* 7a6db86ba9cf56e62c39c9d5c6213cabeddcc3dd support GitHub Enterprise URLs (api/upload/download/skip_tls_verify)
* c0acd12f745ac8832da2061ef14394c41f6662ce Session H — release & changelog behavioral gaps
* fadf467fbcd358f46b747759dfee41eef42755e8 GitHub Release creation via octocrab
* a691cb2c0f01f2e0961f9cbdc5ac9128b0f6ed1d add 8 Pro template variables for GoReleaser parity
* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 802e58ae4f3f113469513b36e6b68aeb78395830 add release stage enhancements (header/footer, extra_files, skip_upload, make_latest, replace options)
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline
* 4013c81ca15400721b914b9ee79964d092b1b86b wire github-native changelog to GitHub API generate_release_notes

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 75c56d73bedc1a31a0470178cbcb27bf096111c9 nfpm test gating, Windows test cross-platform, stage-release pkg dep #none
* a2f0b4be8965ea1f80dadc88affcf7c3fed4c3a3 address code review findings for token type wiring
* 084e41ffdfd822196794fbc6f96da159b9949536 address all code review findings for GitLab release backend
* 2d81439e5adc3c50737ba800255318c9d851b546 address code review findings for GitHub Enterprise URL support
* 2e6a9f4ad4fa297d29659e1fa01241eece7437a7 address code review findings for release.tag feature
* 5b4370877fc544eb4e23210b5f4527af68b3252f document Gitea pagination improvement over GoReleaser
* 1f067c4c0e9fa6bad26d91b5f3b3c5add27ef88d per-crate changelogs, ReleaseURL template var, single tokio runtime
* 0f44e2d7fdbdc962706af3fbeec1abb4e2b47493 address all code review findings
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* cbcf63096563a9785327e16b28d8e470df3d89eb address code review findings from Tasks 3A and 3B
* d64d36607430dc9537fdf56895410abf37abc568 address code review issues for Tasks 2I and 2J
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* fd4fc26fe18524e9c5ad3ed5ded7ba7b74632c9a do not upload raw binaries to GitHub releases, add missing artifact kinds
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 19409b4f43b0250cbf6bd633f4b222539ce3e60e strengthen stage behavior tests to verify actual behavior, not just is_ok()
* c9ce92919183fc4a97d39840c458ae4dd2efa98c template-render release header/footer and use double-newline join
* 86faad2ddcdc273ac10c1ef781d13279306b0ffd whole-token #none + github-release size-match idempotency #none
* c9f5cc3bf62e498df7bbb5fb283a94f9c6a1223f wire TestContextBuilder into stage tests, add mock GitHub release test
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E

### Others

* ecd50adb6b49550bd9c902e03726389c10a00b57 deep dedup pass + wire all dead CLI flags and config fields
* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* c8b1dd32d6b37a5d7f484d95886ca847d4d05928 add 28 error path tests across config, template, build, release, and CLI
* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules
* 5de4f85a4e9ab108bd058dd071bbc92922d900e9 add 60 stage behavior tests verifying config fields produce correct output

# Changelog — anodize-stage-publish

## [0.2.0] - 2026-04-19

### Features

* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* a96cf297a32b5171a87c5d12a581dd88d9b4088c Session M — missing stages, milestones, cross-cutting parity
* 30333462b49190a4c37e96004a210f2f0ef796d6 Session K — nFPM IPK format, template rendering, publisher behavioral gaps
* 04d05e2281c6ebae96e712053ed5eaa85bf590a5 add Artifactory upload publisher
* 817fef82bb9f9c6b09893c6a3be0325c9a6b28d8 add CloudSmith package publisher
* 348c050f2175b65a2980d8eba877a8b68d50b846 add DockerHub description sync stage
* 59cf502cd26990b11555d4214e80df90c66611f0 add GemFury deb/rpm/apk publisher
* baa9b85fc8bc60e273b6e90ba50cb738e5d81284 add NPM package publisher
* e50fdd2c403824f2e32b67d26ea1d4c801ee2703 implement HTTP upload for Artifactory/Fury/CloudSmith + promote Homebrew Cask to top-level config
* 758f6336861bf9077183ffb7fc156597d34a57b6 index_timeout: 0 skips poll with a warning
* 815b47d1d45a110e2233888a561dc0b6f588125f crates.io, Homebrew, and Scoop publishing
* 2d3d0dacfda0da99f0fe5c277bcb86551322c6ee scope change detection to crate path for monorepo workspaces
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 2c3502ed6fa5f10a4c09d227a8e74d719de889c7 Session A publisher config field parity (all tests pass)
* ee45cc4929949742aa515fa8eae70340fa062ab0 add AUR and Krew publishers
* dc35e7ff05298a5982dfa61a72682711e52fd7a0 add Chocolatey and WinGet publishers
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* 01677832fb944719bb63506e5fa5abb7a97b92ca implement Homebrew Cask + fix remaining review suggestions
* a8a78e8de46a22183ee258aad5de2bd8bf94506e native nupkg creation + HTTP push — eliminates choco CLI dependency #none
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 3e93533cd9befc728cd1c81dd95604b3ec31b88e upgrade skip_publish to StringOrBool and add disable field
* 673571dd9668035ddcb0d9a9ad0b9c23ad11ff41 upgrade all disable/skip_upload fields to Option<StringOrBool>
* fede6499895cb0648b693fb5dd78a9b866259277 publisher sha256 key, darwin-universal arch, EasyCLA-blocking commit author #none
* d0aa4caaf0a980f6fe71055cf2c195496eecc65f address all code review findings for Artifactory stage
* db70c00f85517428e26bb3d326b11246f2ab50c1 address all code review findings for CloudSmith stage
* f017b2de8a7488b442f751110c68243f5d9e0384 address all code review findings for DockerHub stage
* 1c095d93c39bb663cbb09ad9ebf3da45f7db5ee7 address all code review findings for GemFury stage
* f29380086c30e383964e324efc44022c10891de9 address all code review findings for NPM stage
* d00682fadd9405fbf5694dcdca0aee55405cc7a5 address final cross-cutting review findings for Session F
* 5e716bfdbd20a4d159ab18b9082db07859cc19c4 check each crate's Cargo.toml version for already-published
* de99152c3390f3bd0332e35658bc2e845891b8d6 search all workspaces for transitive deps
* 97357b95259f1d4f834fa40d845f587f62011394 HOMEBREW_TAP_TOKEN, selected_crates filter, license, backoff, formula grouping
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* 733268280e0e582f2cc87e3c7eca29299a2a52d4 address all 10 code review findings for AUR + Krew publishers
* 02f61b531e89fe7f2906f5c24b457d08507c790f address all 10 code review findings for Chocolatey + WinGet publishers
* aac46c4682276d735a5402249a5ed993ac83523e address all 4 findings from Session 5 final review
* 0a0c94dbb261c120d80cf8a053c71daf11c43c55 address all code review findings for Session A
* b7087379695f3d7fba3336ffff3d032723990f3d address all remaining review findings (round 2)
* d6607d0bd37e3ef1e2b062951e53470b0843d630 address all v0.3.2 release failures across 4 workspace crates #none
* b224243165c8229f6f3488cc6e724208ecfd482f address code review findings from Task 3C
* 794f83cc08c93e8bbcb7e31dafd2bab233981fce always base publisher branch on default + force-with-lease orphan replace #none
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 85ad8683703d470108f3104f3108805fe863b9aa check-then-act publisher push + conventional-commit gate on tag #none
* 8ae51f91a61d1c38dcb76149bb0a3b4389724d91 chocolatey push uses multipart/form-data + NuGet UA (matches choco push wire protocol) #none
* 9cdfefd9e11f1ad3b1b16ed1211105594e935336 chocolatey skips gracefully when version already on feed (matches crates.io idempotency) #none
* f3a32f96297be6f818f9d39a80e938318d533360 correct Nix maybe_submit_pr argument count and semantics
* f7d483dd094e184b56cc26011cce6e42eab553a7 docker_signs as separate stage post-docker, UploadableBinary filter for winget/homebrew/publisher util #none
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* c9fd0596468eb11e057574c1a81b9e13a29c3a53 explicit fetch refspec — single-branch clone blocks tracking ref creation #none
* 86e4efe99d79f07184c8a254ec4cc1c4cd0d7a10 force color in CI; publish transitive workspace deps
* f3c18414e1f4d6bfb40773121b4f16f812917b07 idempotent commit_and_push (skip when nothing staged), version_sync Cargo.lock #none
* 4532d8e988c40dbccf4dde8ae5238cb65677b386 idempotent push + collect-then-bail for publish stage #none
* 128e0034a9463335991398d8b431a7ca7be00fb3 krew default upstream + per-crate previous_tag prefix filter #none
* 248c904ea0ea91d2f1e7ee586ecb61d7c2cf3402 native nupkg (no choco CLI), skip before hooks on tag-triggered CI #none
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* d437316c3739952e5a76be0066132e63eb13d585 publish stage skips already-published crates for idempotent retries
* 5165fc642749aba43515de6aa5ccfc3f7fe1e559 query upstream default_branch for PR base — unblocks winget (master) #none
* 19409b4f43b0250cbf6bd633f4b222539ce3e60e strengthen stage behavior tests to verify actual behavior, not just is_ok()
* 91f7d7f13df7deebe4f54ccca223129f11ff1324 strict-mode bulletproofing + targets subcommand + publisher safety #none
* c4ec7e9e50d05ffb6282d737f976533e921f07b7 switch from native-tls to rustls-tls for cross-compilation
* 37d55950fe6b973cf9969b39b0ca29110bbad97e wire Nix SSH/PR support, AUR directory field (round 3)
* d5b978b29c9c393e41f1ca74de5fd7b324e59214 wire PR workflow, SSH transport, and remaining review suggestions
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E
* 4a8f0945788265349b82f98b8653703e8597d017 wire up publisher-specific token env vars for winget, nix, krew #none

### Others

* ecd50adb6b49550bd9c902e03726389c10a00b57 deep dedup pass + wire all dead CLI flags and config fields
* 4f603d1d030f88cd2582039ad9e593dc0fce6ab7 replace hand-built format strings with Tera templates and serde serialization
* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* cf2660101294d316ed36dc82cb146036af53233e fix token leak in git clone URLs + scoop env var copy-paste bug
* a6a2f986ccdb28c3ea3fe4d3c33ac6b5dc07858d harden unwrap paths, secret handling, path traversal, and regex injection
* 5de4f85a4e9ab108bd058dd071bbc92922d900e9 add 60 stage behavior tests verifying config fields produce correct output
* f2b7f162f6d34d30bb61ab1ba5f053a45c9aecd4 add deep integration tests for archive, checksum, changelog, and publish stages

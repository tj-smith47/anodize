# Changelog — anodize-stage-archive

## [0.2.0] - 2026-04-19

### Features

* 22ac7f2ca1db0b4f6daab458c4740895e0337fb0 add Amd64 version suffix to default name template
* 3d30d2ca1b27d643b166a9edcb9f6debf7109069 add duplicate destination path detection
* f8df5a2f1507fe62475c3f3d0d783d4ba9a0dbf4 archive UniversalBinary/Header/CArchive/CShared types
* 81eb363847d7613303499b4510205da3333cb5c9 implement LCP-based glob directory preservation
* ee4f0931013bbcfddf5cefec361236456cc473f4 sort entries by destination for reproducibility
* 065cb0879ce54b355a920307461d9fa5cf9bc785 template-render FileInfo owner/group/mtime fields
* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* e50fdd2c403824f2e32b67d26ea1d4c801ee2703 implement HTTP upload for Artifactory/Fury/CloudSmith + promote Homebrew Cask to top-level config
* de79d45b923c19c23ceb2c1b8f2ff5bbd84cd80d tar.gz and zip archive creation
* a691cb2c0f01f2e0961f9cbdc5ac9128b0f6ed1d add 8 Pro template variables for GoReleaser parity
* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* ae5053bd36ef6919d79e18a87fc7cd13b8038258 add reproducible build support with SOURCE_DATE_EPOCH
* 26a9e353f60b74c7f93d5410232656548d35c30f add shared test helpers module and mock GitHub client trait
* 3d174d070ea6f65685bb1df4082837c8a55bc7a6 add tar.xz, tar.zst, binary archive formats with glob and wrap_in_directory support
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 3dfcbbd57f65dffa3b1ecaf9c2f27a23f945ce75 error on missing binary files instead of silent skip
* 216cd10b957e2683e08178b27a8a12ad45430f9d address 10 review findings for Task 4 (OSS template vars)
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* b224243165c8229f6f3488cc6e724208ecfd482f address code review findings from Task 3C
* 230eab98b18f6a776ad1a8f398a4331b4b95c700 address code review issues for Tasks 2B, 2G, 2H
* f56788c8c341eabacaa069fc1acf86746759baa8 archive path normalization, Zola 0.22.1 config, Windows test
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* 4532d8e988c40dbccf4dde8ae5238cb65677b386 idempotent push + collect-then-bail for publish stage #none
* 49a6f9f57b450ddd3984de7714057c8e05c29078 normalize tar uid/gid and read CommitTimestamp from context for reproducibility
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 91f7d7f13df7deebe4f54ccca223129f11ff1324 strict-mode bulletproofing + targets subcommand + publisher safety #none
* c9f5cc3bf62e498df7bbb5fb283a94f9c6a1223f wire TestContextBuilder into stage tests, add mock GitHub release test
* fee9ad1c4652d5e166a94922210725052769de19 wire stage-scoped template vars (Binary, ArtifactName, ArtifactPath)
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E

### Others

* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* 69f8669b9c3d55b116a548c1995cde0caef1e817 add test confirming binaries filter is wired
* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules
* 5de4f85a4e9ab108bd058dd071bbc92922d900e9 add 60 stage behavior tests verifying config fields produce correct output
* f2b7f162f6d34d30bb61ab1ba5f053a45c9aecd4 add deep integration tests for archive, checksum, changelog, and publish stages

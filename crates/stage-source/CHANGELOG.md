# Changelog — anodize-stage-source

## [0.2.0] - 2026-04-19

### Features

* e50fdd2c403824f2e32b67d26ea1d4c801ee2703 implement HTTP upload for Artifactory/Fury/CloudSmith + promote Homebrew Cask to top-level config
* 0777ad16d9fa2798a15f8e55cb5b4cf10afff82e implement file metadata (info) for extra files via Rust tar crate
* 8984159c1678425916b37acf7db0b5f39911467b implement strip_parent for extra files
* a691cb2c0f01f2e0961f9cbdc5ac9128b0f6ed1d add 8 Pro template variables for GoReleaser parity
* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 468c2668facb88ec47461eaacb36e507dba94af6 add source archive and SBOM generation stage
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* 70a256f9edfff2425c5a47ec622ec4d3ea1d47db rewrite SBOM stage to subprocess model and update source archive
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* d7d1458d986c80b9f04fc4bdfa008f80ab888151 migrate SbomConfig.env to HashMap + add env deserializer tests
* 3230ff923f6e334350daf05603d66a69dd3c1725 address code quality review findings
* cb0ec5d025452162891a634ca0c47077a9329709 address code quality review findings
* 216cd10b957e2683e08178b27a8a12ad45430f9d address 10 review findings for Task 4 (OSS template vars)
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* 73f8ec52931cbf2bdb4c6339e1e7d0834ebd2721 address all 10 code review findings for source archives + SBOM
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* 10f2ae254c7190f96e50e913e748ba945cee14ac expand glob patterns in source stage extra_files
* 23f25f6ea042e5355985e1d6f4fcb427381d32f4 final review findings — SBOM arg ordering, AppBundle replace/disable, NSIS mod_timestamp
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none

### Others

* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none

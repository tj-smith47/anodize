# Changelog — anodize-core

## [0.2.0] - 2026-04-19

### Features

* d11f5b587759dde85a6990d7ee29fa2612c4695a add Bluesky provider
* f37eb1036083b5647c2f4edda95b445e8ec4a982 add Discourse provider
* 672d56f1546586ff8802e2a130ec6e3704cfe0b9 add LinkedIn provider
* c6c65f6c59c07013b7b4c76999c4083047dae180 add Mastodon provider
* 2c2fada700085279afaebdaa4bf1aef2a6f59429 add OpenCollective provider
* 0f5e5643126ba019ed8459e477ffff39c3e8a2da add Reddit provider
* 773f455e8983766a0f69e6e2d2da03915e59cc68 add Twitter/X provider with OAuth 1.0a
* 98435ab2c929febe5a299cd38a9af7f1cf79e59d add expected_status_codes to webhook provider
* 016a285a0dc25fd217b1dfb276f50af3af0daa78 add icon_url to Teams provider
* 828b074ab04a18d454ef45262cccfb0c43e08f32 add template-conditional announce.skip field
* b5e075ba2e9a63106b60d5476218815930489339 add title_template to Mattermost provider
* 49edfe115c46f8da69ff8805a0ee9de205bb2ad1 replace sendmail with SMTP transport via lettre
* 0d64c62c4ad80311845f5a40769e1c490378b92b type Slack blocks/attachments for better schema validation
* f8df5a2f1507fe62475c3f3d0d783d4ba9a0dbf4 archive UniversalBinary/Header/CArchive/CShared types
* e4a8f117075fe1777aaf7eb6c3796f9fa0d68126 add 5 missing GoReleaser artifact types
* d4916567136050891b33a46cb22d4f8a27202c72 add gitlab and gitea backends
* 68594969ba5d1e8be6e21d53c4994a73353d4e19 add DockerDigestConfig type for digest file control
* 6b763b22103eac82d6e97960c34fd2eecaeaabf6 add DockerHub, Artifactory, Fury, CloudSmith, NPM publisher configs
* 137d01018b589811f838ea5f9db4578ffea3c957 add GitConfig struct for tag discovery and sorting
* c6225e1739712943896012f96509c38ec6a43a64 add NSIS, AppBundle, SBOM rewrite, Source archive config structs
* 8558e3ef1d6c35c55c54c22e38204b3753ed6653 add NsisConfig, AppBundleConfig, rewrite SbomConfig and SourceConfig
* 70abf225208a1f37beb17e0409d0ae5fc468b299 add platform URL configs and force_token for multi-SCM support
* 4c6d86c7f8d7cb40b97e7983699e3bfa6c2cfe96 Session L — config defaults, ANODIZE_FORCE_TOKEN, announce provider parity
* a695d3dc02d6e283f28ac3151eeebe41cfc2ab08 SkipPush template support + expand Docker parity plan
* 0e59ac0de0d48249859bbc16efdd66a8c9a00c6a Go-style template engine
* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* 5abd27d5e4bf483bdb62a81fba8e00b9903a5485 add ScmTokenType enum and resolution logic
* 63051864d8a9bbd251ca36f751e50399d6e66cc2 add output secret redaction for docker commands
* bf368646bd8061252c3be89d442ee1fe34ae2a93 artifact registry
* df1f813084b6324e9684158076bb93c1d73720a5 config schema with serde deserialization
* 1bb8c3c0a48f8688fb136ec607d60ff5a4ae56f8 git state detection and semver parsing
* bf64cf28e326499b40416d518f5bc5d098512704 target triple to OS/arch mapping
* cb6ea428d2617c1521c72fa25973f557cc590961 add legacy goos/goarch/goarm/goamd64 fields to DockerConfig
* ae7718b2e4ef67976837decb66a81f95b8699bce add GoReleaser env list form and env_files structured token files
* 91527e8c12e7fe5689ac7794918e59c64387b28f wire GitConfig fields to tag discovery behavior
* a96cf297a32b5171a87c5d12a581dd88d9b4088c Session M — missing stages, milestones, cross-cutting parity
* 4214474b5a5d3db6fcfc4ad54f0d6eb55eb3452f add libdirs, changelog, and owner/group template rendering
* 30333462b49190a4c37e96004a210f2f0ef796d6 Session K — nFPM IPK format, template rendering, publisher behavioral gaps
* 7cf4bfaec3851d786cc51b4660b018012255e06c expand ArtifactKind to 38 variants + changelog Pro features
* e50fdd2c403824f2e32b67d26ea1d4c801ee2703 implement HTTP upload for Artifactory/Fury/CloudSmith + promote Homebrew Cask to top-level config
* a5b4cbb83afa37303f4cbd67fa24ff88829d1ccc add release.tag template override for GitHub release tag
* c0acd12f745ac8832da2061ef14394c41f6662ce Session H — release & changelog behavioral gaps
* 1685d6f36f908c80f158eae59ae3958d69d70b54 add 24 missing app fields, hooks, and structured extra_files
* efb3d3b550c643c3dcb78593e9ee0c79aecc69fe conventional commit changelog generation
* 2d3d0dacfda0da99f0fe5c277bcb86551322c6ee scope change detection to crate path for monorepo workspaces
* c1de526dc8b8ec0ced75ae0d68cd74736fa3bdf4 version_sync Cargo.toml before tagging #none
* a691cb2c0f01f2e0961f9cbdc5ac9128b0f6ed1d add 8 Pro template variables for GoReleaser parity
* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* a76a559a50c384b06c0cb3ad2bae2f539979cfee add Go-style positional syntax for replace, split, contains
* df21273c5c965e87c29e96c6232d2aa1ddc6a3e7 add `in` and `reReplaceAll` Pro template functions
* 0f0a3847f15737ba0ee5655534ae2192035ba7d7 add custom template variables (.Var.*) for user-defined values
* 4e5187d47f958951befd1c9e0742eea31746c66d add template_files config section and rendering stage
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 2c3502ed6fa5f10a4c09d227a8e74d719de889c7 Session A publisher config field parity (all tests pass)
* 52dc4d0674596c7b5280297b88baf70a87911a14 Taskfile, cross-workflow artifacts, disable support, CI hardening
* 2e051d362c8d4e49f6862d2bafb6ecc174ae4582 Tera custom filters (tolower, toupper, trimprefix, trimsuffix) + 16 new tests
* a2a88f1e065afc66634cf3d74a1f340b300fc354 Tera template support (default), Go-style compat via auto-detect, update docs
* d711d39e3339a4fcef126a34896952fa58df6421 add --nightly flag for automated rolling releases
* ee45cc4929949742aa515fa8eae70340fa062ab0 add AUR and Krew publishers
* dc35e7ff05298a5982dfa61a72682711e52fd7a0 add Chocolatey and WinGet publishers
* 34186a9ad564c60266c4cb7edd0fd5a6011904f3 add Rust-specific first-class features (binstall, version sync, cdylib/wasm32)
* 61f843b35b180577d374d50e6a29e21bffcff3e9 add Telegram, Teams, Mattermost, and email announce providers
* 10a99885775db125e6eb2fcdb17bf19470619b24 add UPX binary compression stage
* 10388fb247ec1e7419bf5efa6a51c4b32493aa35 add `anodize tag` command for auto-tagging based on commit directives
* ec6ad18689d9e2b47adb4fd5261d664add0d439f add cloud storage upload (S3/GCS/Azure) and split/merge CI fan-out
* bdd232279723ad37dba7fb09e29da63562c94077 add completion, healthcheck commands and new CLI flags for release/build
* e39f1425eeb77a4968c763fb73848bfd241fd08b add config includes with deep merge support
* 68e4930283e3c4c4710f6c13cc738382d363ffce add custom publishers for generic post-release artifact publishing
* 6d8b2adadab8b0433203124c50996bb8dec3d56f add filters.include, abbrev, and use: github-native to changelog config
* 20fd04a3254b138acb044da256df83c0027cc141 add jsonschema command, env_files, config versioning, build ignore/overrides
* 058e8651cc71c94d0dd739ab7cbe46889debb10a add macOS universal binary support via lipo
* 01030a6fe4be2a18948af275a0800a2ae0575adb add make_latest, changelog header/footer, and disable fields to config
* a192a1590dcb82fe29bb5eeac179760100d9915a add monorepo workspace support for independent project roots
* af1641509a3ec71aa29f9eb6208a505fea8cc404 add platform-specific packaging stages — snapcraft, dmg, msi, pkg
* 802e58ae4f3f113469513b36e6b68aeb78395830 add release stage enhancements (header/footer, extra_files, skip_upload, make_latest, replace options)
* 30d43921135faa6d2965c2a40b2610ef709f90a3 add report_sizes, metadata.json output, and env config fields
* ae5053bd36ef6919d79e18a87fc7cd13b8038258 add reproducible build support with SOURCE_DATE_EPOCH
* 2c78e3c173f52ebc780954dd51266c748d966f9a add scripts, package metadata, and content type/file_info to nfpm stage
* 191b5c20759b385fb2ae491303bc2efb2b61436d add sha1, sha224, sha384, blake2b, blake2s algorithms and extra_files/ids to checksum stage
* 26a9e353f60b74c7f93d5410232656548d35c30f add shared test helpers module and mock GitHub client trait
* 28edbc6ad6842474276d02c198c74790b6ef3e0d add skip_push, extra_files, and push_flags to docker stage
* 468c2668facb88ec47461eaacb36e507dba94af6 add source archive and SBOM generation stage
* 3d174d070ea6f65685bb1df4082837c8a55bc7a6 add tar.xz, tar.zst, binary archive formats with glob and wrap_in_directory support
* 3ac261cbc8e130f9dbcb85063bfb69b7afdcf4cf add xtask crate with Tera-based gen-docs for CLI and config reference
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* 2925006e3f0fa1ef20d0b3af8dac2261f00bec7e auto-detect GitHub owner/name from git remote when release.github is omitted
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* 01677832fb944719bb63506e5fa5abb7a97b92ca implement Homebrew Cask + fix remaining review suggestions
* 56bea2d3b97636ee773b4fa88b0faf6380a603a7 migrate sign to signs[] array with backward compat and new fields
* 605ddce94aada7b4af49443df6c13b3bcca11bdf migrate template engine from regex to Tera
* 009a145610309c55bfbae4e56dae61348b742802 populate all template variables from git info and time
* 02a933655e4ab4129d6e8e99f25882a9ab644964 register GoReleaser-compat Tera filters + add 13 template tests
* 42117bb315c1bdc732224bcaedc0d9bcd21628b6 replace blob CLI-shelling with object_store SDK
* 70a256f9edfff2425c5a47ec622ec4d3ea1d47db rewrite SBOM stage to subprocess model and update source archive
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* 1c70bc903e4eeba171a189ce7aeabd8fbe9db352 shared parallelism + skip-memento + tag hooks + ignore_tags wiring #none
* 3d1e30a0dac8fad61c602740452e6468987c8d11 split/merge rewrite for GoReleaser Pro parity
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* be43d3b6a8e58cca4dfb65727b23faafbc446b01 use StringOrBool for skip field and strengthen template test
* d393a3735d1f5afb2f27f51cb57095049ecc03ca address code quality review findings
* e97fe0cc05d46e87296d36be0e479933ea06eafa add Library and Wasm to size_reportable_kinds
* 65ab1306c43218a9e9af9e89054c4999237e1fe8 add UploadableBinary, fix uploadable_kinds to use PublishableSnapcraft
* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 3e93533cd9befc728cd1c81dd95604b3ec31b88e upgrade skip_publish to StringOrBool and add disable field
* 1ed1fc42794a9c783479f8a24785a41eae670710 wire verbose/debug to ContextOptions, improve find_config error message
* 7ed80f486db6e935ddf1677fcfdf8e965a992240 address code review findings for platform URL configs
* c30bc496b809b738263a909bf358a6da06edb515 change DockerSignConfig.output to StringOrBool for GoReleaser parity
* d7d1458d986c80b9f04fc4bdfa008f80ab888151 migrate SbomConfig.env to HashMap + add env deserializer tests
* 4d68b2b5b41fef76ed47bfe51c944fd035002034 strengthen publisher config tests per code review
* 673571dd9668035ddcb0d9a9ad0b9c23ad11ff41 upgrade all disable/skip_upload fields to Option<StringOrBool>
* 7c6883a8c9897fc4175bad8bba3b6d675b254462 wire deserialize_env_map to all env fields for GoReleaser list-of-strings compat
* 1920b6241a3d3e17029ed7fb23dacf18b0daecfe add missing NfpmConfig fields (contents, dependencies, overrides)
* da0a155d7b84887a30ac011e0bbea1cdec079160 repo URL, lazy regex compilation, semver prerelease sort ordering
* cc0571b140a3bbebc872a0c697597561544fb21d validate PrereleaseConfig accepts only 'auto', use serde_json::Value for format-neutral nfpm overrides
* 8353e036402f9a91c21e931cf8f893af1ebc8643 DockerDigest review fixes — collision, artifacts, errors, docs
* 9d94db54754d55e1450b596252d2650743b50efd legacy push, combined digests, retry codes, redact sort
* fede6499895cb0648b693fb5dd78a9b866259277 publisher sha256 key, darwin-universal arch, EasyCLA-blocking commit author #none
* e18f274dc07d8576f4dc014701b26a3c438258e0 address all code quality review findings for GitConfig
* d0cf3c5951007d4eab01a98abba9c27411acf0d1 close 4 GoReleaser parity gaps in GitConfig wiring
* d757b6f7f73a2c5f65142981fb0936f337a19948 address all code/spec review findings for config includes from URL
* d09bea66265fa3d65c5c744bbbadd625c3f258cc address code quality review findings for MetadataConfig
* 79197be3b3fb9c1257dbc10676a542966ea8a558 correct metadata.json content and split artifact list into artifacts.json
* c71179146f31bb7b311738f14a2c985dc1d5ae7e address all code review findings for monorepo config feature
* db70c00f85517428e26bb3d326b11246f2ab50c1 address all code review findings for CloudSmith stage
* 1f067c4c0e9fa6bad26d91b5f3b3c5add27ef88d per-crate changelogs, ReleaseURL template var, single tokio runtime
* dcbb79b2acad834e5fd5d9b3a1b48d6ea29a8495 address all code review findings for SCM module
* c647361629d4c7cf1d436cf0ea41fb28d0a6318a use GoReleaser-style template syntax in release URL templates
* 15300788f0dd1c8b0681dd96e863f2b914a29c5c SignConfig.output to StringOrBool + template evaluation
* eed4b33e87a1ca55275c7f6c3fbe2a9b4eb29898 address 7 code review findings for Task 7
* c9a7bec11934107f66ba32339c19f13a2d983898 copy_from skip compilation, workspace target dir, auto default strategy, per-crate cross
* a5ec51da01245cad586a469eb27874b52db278a6 dry-run, lazy regex, invalid regex warnings, initial release support
* 97357b95259f1d4f834fa40d845f587f62011394 HOMEBREW_TAP_TOKEN, selected_crates filter, license, backoff, formula grouping
* 216cd10b957e2683e08178b27a8a12ad45430f9d address 10 review findings for Task 4 (OSS template vars)
* 9fcf897b517a94e4fcd60d259daf803c4b4ba7b3 address 9 code review findings for positional syntax
* 29b3b0dc53ccff7d8c717a218e5caa3e1280aaf5 address 9 review findings for `in` and `reReplaceAll`
* fe38a0187fb253e5937204b76708018716665e88 register `in` filter, fix list regex for escaped/mixed quotes
* 0f44e2d7fdbdc962706af3fbeec1abb4e2b47493 address all code review findings
* 840b75d61ec95f1990ab0e991f50e2cff86ad0d2 address code review findings for template_files stage
* 6311dccf028fa6af4d772bb8206a3a9d4147c7bc add #[serial] to token file tests that mutate env vars
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* bf8f940168d38fda8513b64292971d0d51a5955b R5 audit — eliminate remaining raw eprintln, double env load, VERSION_PLACEHOLDERS copy
* d8d0942db46a3546f974dac734af01474e74e98f Zola 0.22.1 config compat, Windows find_binary test
* c8ac32f483445304e6cc4399b2deec28037779af add flatpak/notarize to workspace + fix ChangelogConfig construction
* 733268280e0e582f2cc87e3c7eca29299a2a52d4 address all 10 code review findings for AUR + Krew publishers
* 02f61b531e89fe7f2906f5c24b457d08507c790f address all 10 code review findings for Chocolatey + WinGet publishers
* 73f8ec52931cbf2bdb4c6339e1e7d0834ebd2721 address all 10 code review findings for source archives + SBOM
* f608424b79264651db334a145df3e86f991b332d address all 11 code review findings for Task 5M (CLI + Config)
* f04bab0538d2d02d0498e98b8e94a0c93ef25a5a address all 6 code review findings for announce providers
* 10cbd0dc37fc2627e86ede11a23bd6ef4c5711f6 address all 8 code review findings for UPX binary compression
* 0a0c94dbb261c120d80cf8a053c71daf11c43c55 address all code review findings for Session A
* 97549d213dddec2aed5995ed0ace0df6a84e9199 address all code review findings for Tasks 8-12
* b7087379695f3d7fba3336ffff3d032723990f3d address all remaining review findings (round 2)
* 0beb30ee22095f52eef8dfc0b50c52b738be9eca address all split/merge code review findings
* d6607d0bd37e3ef1e2b062951e53470b0843d630 address all v0.3.2 release failures across 4 workspace crates #none
* c47ffde321b183eceb74131d971dbe036262d54f address code quality review findings in config doc comments
* 64cb7038671dc9bb6d907f2d9415bf1cf508b9d0 address code review findings for monorepo workspace support
* cbcf63096563a9785327e16b28d8e470df3d89eb address code review findings from Tasks 3A and 3B
* 2ce684874ec377af07e9020c2533dd65376e2021 address code review issues for Task 2A CLI completeness
* 230eab98b18f6a776ad1a8f398a4331b4b95c700 address code review issues for Tasks 2B, 2G, 2H
* d64d36607430dc9537fdf56895410abf37abc568 address code review issues for Tasks 2I and 2J
* 7e6c83bbdd713b003b3ab2393d8433448ecd893c address deep review findings across template, artifact, util, helpers
* aac8408209b439f1091e65ef4fa941caab23c903 address post-fix code review findings for Tasks 11+12
* 76526184c5b1386cf3471afef1a585e1b799f0d4 anchor branch regex and make tag push conditional on remote
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* ba1bbf19fd6307bca864eda2a8cb936488eb91d0 checksum archives-disabled check, nfpm per-target iteration and template support
* e84f0ec79d524ad40e259ff7985e4fd129175a98 derive Clone on TemplateVars, replace fragile clone_template_vars
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* 23f25f6ea042e5355985e1d6f4fcb427381d32f4 final review findings — SBOM arg ordering, AppBundle replace/disable, NSIS mod_timestamp
* ca68e87df3f86fd2a860b750d2f80ef7445ac06b find_binary PATHEXT on Windows, workspace skip field #none
* a5028f36f272037fa9d7362cc9e86cae9b3fedd5 gate MockGitHubClient behind test-helpers feature, fix temp dir in tests
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* c03d441733a7e9cc333cdd3ece594623feeb3dbc repair YAML indentation in 120 extracted config parsing integration tests
* 17bb8f7834b1451e6acdb7ba6c1785e32ec4898e resolve flaky tests, duplicated helpers, and unfalsifiable assertions
* fbb83bb934b1a165cec92e1e7a7cdca3aae186a5 split/merge env poisoning — macOS HOME leaked into Linux docker builds #none
* c0ebc5dbceb5e93b425649937d1b4b770517b821 stable docs default, Windows test compat, Zola deprecated field
* 91f7d7f13df7deebe4f54ccca223129f11ff1324 strict-mode bulletproofing + targets subcommand + publisher safety #none
* 94d9417686c655bcf4085f294a026deebe70b114 support Tera-native tag_template in tag matching, remove last TODO, run cargo fmt
* a4a64d0d546b1985400efd2bfd28b66163324f31 unify extra_files to ExtraFileSpec across DMG, PKG stages
* c9f5cc3bf62e498df7bbb5fb283a94f9c6a1223f wire TestContextBuilder into stage tests, add mock GitHub release test
* fee9ad1c4652d5e166a94922210725052769de19 wire stage-scoped template vars (Binary, ArtifactName, ArtifactPath)
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E
* a73a3abc6d6eb73ea7fe143c80297c6fa7b7876f wire up ids filter and replace field, make lipo failure non-fatal

### Others

* f782d890983f0c2bb5be1ea8c301a3ff0d792ecf Merge branch 'worktree-docsite'
* fbcd944952855d634d1574994ee88f4c2e199e9a Revert "feat(docker): add legacy goos/goarch/goarm/goamd64 fields to DockerConfig"
* ecd50adb6b49550bd9c902e03726389c10a00b57 deep dedup pass + wire all dead CLI flags and config fields
* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* 6c11bc7b5c3ee5a1c2ec025819eae830d479afff eliminate all unsafe code from production
* a6a2f986ccdb28c3ea3fe4d3c33ac6b5dc07858d harden unwrap paths, secret handling, path traversal, and regex injection
* d775f40783a6f2f2634182995075475f661e19e6 add 202 config parsing depth tests for every field and variation
* c8b1dd32d6b37a5d7f484d95886ca847d4d05928 add 28 error path tests across config, template, build, release, and CLI
* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules
* 3c1f06cf37519b4d2f00aac6c7a8221ac87f4c91 add error path tests for config validation, failed builds, and unknown fields

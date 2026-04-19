# Changelog — anodize

## [0.2.0] - 2026-04-19

### Features

* 015283438ddd7045e719ab2c7c31366c34eba7b1 add resolve-tag subcommand for monorepo tag→crate mapping
* 13e9fbb002652fdf04e4e68d662b5db6b8a2b1e9 config loading, pipeline assembly, and command stubs
* 15bb0ede381c2c5587dd8485843f95295164e9cc init, check commands, change detection, hooks, colored output
* 96b24d895ae0b9fb0fb8ad3ffad878be92e30230 wire token type resolution from config/env into Context
* 4c6d86c7f8d7cb40b97e7983699e3bfa6c2cfe96 Session L — config defaults, ANODIZE_FORCE_TOKEN, announce provider parity
* ae7718b2e4ef67976837decb66a81f95b8699bce add GoReleaser env list form and env_files structured token files
* 91527e8c12e7fe5689ac7794918e59c64387b28f wire GitConfig fields to tag discovery behavior
* a96cf297a32b5171a87c5d12a581dd88d9b4088c Session M — missing stages, milestones, cross-cutting parity
* 2d3d0dacfda0da99f0fe5c277bcb86551322c6ee scope change detection to crate path for monorepo workspaces
* c1de526dc8b8ec0ced75ae0d68cd74736fa3bdf4 version_sync Cargo.toml before tagging #none
* a691cb2c0f01f2e0961f9cbdc5ac9128b0f6ed1d add 8 Pro template variables for GoReleaser parity
* 9e3e8ea2b441e7bd8788cb4c539be94b00566289 add ArtifactExt, Target, Checksums, Outputs template variables
* 0f0a3847f15737ba0ee5655534ae2192035ba7d7 add custom template variables (.Var.*) for user-defined values
* 4e5187d47f958951befd1c9e0742eea31746c66d add template_files config section and rendering stage
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* e41e6be15dd36a5113371031bbca237ee469f366 README, config reference, GitHub Action, integration tests, dotfile config default
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* 19801c9642266985acc6ad2dd97cd6b35e22bc08 SkipMemento wiring + BinarySignStage + anodize build parity #none
* 571de80a96c28349c316469187d8d6e568c75093 add --config / -f global flag to specify config file path
* d711d39e3339a4fcef126a34896952fa58df6421 add --nightly flag for automated rolling releases
* 9e4cde874793a738ba93f05616ea453cf11f10f8 add --timeout flag to release and build commands
* afa159e9ae955ce4d862ede309aafea8544d3943 add --workspace flag to build and check commands
* 843fdb68317c2be9713326cc733995dece9699da add E2E test infrastructure for integration testing
* 10a99885775db125e6eb2fcdb17bf19470619b24 add UPX binary compression stage
* 10388fb247ec1e7419bf5efa6a51c4b32493aa35 add `anodize tag` command for auto-tagging based on commit directives
* ec6ad18689d9e2b47adb4fd5261d664add0d439f add cloud storage upload (S3/GCS/Azure) and split/merge CI fan-out
* bdd232279723ad37dba7fb09e29da63562c94077 add completion, healthcheck commands and new CLI flags for release/build
* e39f1425eeb77a4968c763fb73848bfd241fd08b add config includes with deep merge support
* 68e4930283e3c4c4710f6c13cc738382d363ffce add custom publishers for generic post-release artifact publishing
* 6d8b2adadab8b0433203124c50996bb8dec3d56f add filters.include, abbrev, and use: github-native to changelog config
* 20fd04a3254b138acb044da256df83c0027cc141 add jsonschema command, env_files, config versioning, build ignore/overrides
* 01030a6fe4be2a18948af275a0800a2ae0575adb add make_latest, changelog header/footer, and disable fields to config
* a192a1590dcb82fe29bb5eeac179760100d9915a add monorepo workspace support for independent project roots
* af1641509a3ec71aa29f9eb6208a505fea8cc404 add platform-specific packaging stages — snapcraft, dmg, msi, pkg
* 30d43921135faa6d2965c2a40b2610ef709f90a3 add report_sizes, metadata.json output, and env config fields
* 191b5c20759b385fb2ae491303bc2efb2b61436d add sha1, sha224, sha384, blake2b, blake2s algorithms and extra_files/ids to checksum stage
* 26a9e353f60b74c7f93d5410232656548d35c30f add shared test helpers module and mock GitHub client trait
* 468c2668facb88ec47461eaacb36e507dba94af6 add source archive and SBOM generation stage
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* 2925006e3f0fa1ef20d0b3af8dac2261f00bec7e auto-detect GitHub owner/name from git remote when release.github is omitted
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 1267439012db850b5d09e5d5d8f6be1491ec3c0a comprehensive --strict mode across all stages
* 9214ddb1e361c46725d07b79c4296f78c26b127d integration tests and README
* 56bea2d3b97636ee773b4fa88b0faf6380a603a7 migrate sign to signs[] array with backward compat and new fields
* 009a145610309c55bfbae4e56dae61348b742802 populate all template variables from git info and time
* 42117bb315c1bdc732224bcaedc0d9bcd21628b6 replace blob CLI-shelling with object_store SDK
* 70a256f9edfff2425c5a47ec622ec4d3ea1d47db rewrite SBOM stage to subprocess model and update source archive
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* 1c70bc903e4eeba171a189ce7aeabd8fbe9db352 shared parallelism + skip-memento + tag hooks + ignore_tags wiring #none
* 3d1e30a0dac8fad61c602740452e6468987c8d11 split/merge rewrite for GoReleaser Pro parity
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* 8205e26781432b484bdf716509423d61ac6b3712 inference uses .anodize.yaml tag_template, not name-v fallback #none
* 75c56d73bedc1a31a0470178cbcb27bf096111c9 nfpm test gating, Windows test cross-platform, stage-release pkg dep #none
* 1de3d974121930ea548abf3547643eb6228eaab6 add target triple validation and GPG/cosign env checks to check command
* a2f0b4be8965ea1f80dadc88affcf7c3fed4c3a3 address code review findings for token type wiring
* 1ed1fc42794a9c783479f8a24785a41eae670710 wire verbose/debug to ContextOptions, improve find_config error message
* 673571dd9668035ddcb0d9a9ad0b9c23ad11ff41 upgrade all disable/skip_upload fields to Option<StringOrBool>
* e18f274dc07d8576f4dc014701b26a3c438258e0 address all code quality review findings for GitConfig
* d0cf3c5951007d4eab01a98abba9c27411acf0d1 close 4 GoReleaser parity gaps in GitConfig wiring
* d757b6f7f73a2c5f65142981fb0936f337a19948 address all code/spec review findings for config includes from URL
* 2278136d559fd9e0354aa58c58f8792f0363124f correct header key expansion test assertion
* d09bea66265fa3d65c5c744bbbadd625c3f258cc address code quality review findings for MetadataConfig
* 79197be3b3fb9c1257dbc10676a542966ea8a558 correct metadata.json content and split artifact list into artifacts.json
* c71179146f31bb7b311738f14a2c985dc1d5ae7e address all code review findings for monorepo config feature
* 51905543a2237059c0e82554279c5630b9252fa1 apply workspace overlay when --crate matches a workspace
* aaceb419ad9bebb0c2b1494a5b1a776f9e169073 flatten workspaces for crate resolution
* cb0ec5d025452162891a634ca0c47077a9329709 address code quality review findings
* 0f44e2d7fdbdc962706af3fbeec1abb4e2b47493 address all code review findings
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* bf6489a673763056b9c31f82641df077764a75b5 CI failures — docs freshness, packaging, tests, snapshot
* bf8f940168d38fda8513b64292971d0d51a5955b R5 audit — eliminate remaining raw eprintln, double env load, VERSION_PLACEHOLDERS copy
* 79638da4ef73e0ec7881e9c7591a88123b3db150 Windows hook tests, docs check diagnostic output
* 0427353d6fad28d6c72e8a94a4e80918421db7dd action injection safety, nfpm docs, IsDraft template var, unused import, clippy warnings
* c8ac32f483445304e6cc4399b2deec28037779af add flatpak/notarize to workspace + fix ChangelogConfig construction
* 73f8ec52931cbf2bdb4c6339e1e7d0834ebd2721 address all 10 code review findings for source archives + SBOM
* f608424b79264651db334a145df3e86f991b332d address all 11 code review findings for Task 5M (CLI + Config)
* aac46c4682276d735a5402249a5ed993ac83523e address all 4 findings from Session 5 final review
* 0334bdfe30000b2081d62df32e659f5b01f77db2 address all code review findings from Task 3D documentation review
* 0beb30ee22095f52eef8dfc0b50c52b738be9eca address all split/merge code review findings
* d6607d0bd37e3ef1e2b062951e53470b0843d630 address all v0.3.2 release failures across 4 workspace crates #none
* 64cb7038671dc9bb6d907f2d9415bf1cf508b9d0 address code review findings for monorepo workspace support
* cbcf63096563a9785327e16b28d8e470df3d89eb address code review findings from Tasks 3A and 3B
* 2ce684874ec377af07e9020c2533dd65376e2021 address code review issues for Task 2A CLI completeness
* 9244126789a27977823ef15dd60584fdfafb00fb address code review issues for Task 2E sign stage
* 230eab98b18f6a776ad1a8f398a4331b4b95c700 address code review issues for Tasks 2B, 2G, 2H
* d64d36607430dc9537fdf56895410abf37abc568 address code review issues for Tasks 2I and 2J
* 7e6c83bbdd713b003b3ab2393d8433448ecd893c address deep review findings across template, artifact, util, helpers
* 634c79ca6581160425d11e52fb73da95c46f7d62 address final review findings — help text, mobile nav, aria, CI check
* 76526184c5b1386cf3471afef1a585e1b799f0d4 anchor branch regex and make tag push conditional on remote
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 9152ade728000bcd376b0d9ab4a53f61e12d12a3 cargo.toml override wins over bump=None short-circuit in tag #none
* 85ad8683703d470108f3104f3108805fe863b9aa check-then-act publisher push + conventional-commit gate on tag #none
* fbe8ddd1d46124fecd05a1f80a0cc1b22a826b21 clippy unnecessary_unwrap and fmt in nfpm test
* 4ff610db20928377a5bf360850582bf30cbb13df copy binary artifacts into split dist for cross-machine merge
* e84f0ec79d524ad40e259ff7985e4fd129175a98 derive Clone on TemplateVars, replace fragile clone_template_vars
* f7d483dd094e184b56cc26011cce6e42eab553a7 docker_signs as separate stage post-docker, UploadableBinary filter for winget/homebrew/publisher util #none
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* ca68e87df3f86fd2a860b750d2f80ef7445ac06b find_binary PATHEXT on Windows, workspace skip field #none
* 86e4efe99d79f07184c8a254ec4cc1c4cd0d7a10 force color in CI; publish transitive workspace deps
* d1bcc93a56a038005a84cd7d8a451f4104a19f00 have resolve_git_context fall back to workspaces when crates empty
* f3c18414e1f4d6bfb40773121b4f16f812917b07 idempotent commit_and_push (skip when nothing staged), version_sync Cargo.lock #none
* 128e0034a9463335991398d8b431a7ca7be00fb3 krew default upstream + per-crate previous_tag prefix filter #none
* 248c904ea0ea91d2f1e7ee586ecb61d7c2cf3402 native nupkg (no choco CLI), skip before hooks on tag-triggered CI #none
* 9687d2e0038336ab90dc89c5febe03ea8181d1e8 nfpm mode serialization and initial tag version
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* e4bac94d07ee82bdee22a6fc3c315a292a1d8c28 remove needless borrows in E2E tests to fix clippy warnings
* 17bb8f7834b1451e6acdb7ba6c1785e32ec4898e resolve flaky tests, duplicated helpers, and unfalsifiable assertions
* 7a3ade0be618aab96271518e8acfc1c471572c42 reverse config include merge order so base config overrides includes
* 6d6ebe9c7d3439889f959995e4aabb4cc6aec855 run formatter
* 408e3dba7aecaaff6544f87dc145ccff452f6383 skip before hooks in split mode, gitignore zig cache
* a049bccf9aa863b8c296ec4da12f43bd9093f262 skip dirty-dist check in merge mode
* fbb83bb934b1a165cec92e1e7a7cdca3aae186a5 split/merge env poisoning — macOS HOME leaked into Linux docker builds #none
* c0ebc5dbceb5e93b425649937d1b4b770517b821 stable docs default, Windows test compat, Zola deprecated field
* 13eabefa8e4d1ff713a011c795d3f228432b775c strengthen workspace E2E assertions and add change detection test
* 91f7d7f13df7deebe4f54ccca223129f11ff1324 strict-mode bulletproofing + targets subcommand + publisher safety #none
* 94d9417686c655bcf4085f294a026deebe70b114 support Tera-native tag_template in tag matching, remove last TODO, run cargo fmt
* 85108ea9adf103e32838e0445f5bbbef61d7861a version_sync must regenerate Cargo.lock before committing, revert hook skip #none
* ce3e3962742afaa7d5e8c0157558f5a57fbc22c2 version_sync updates workspace deps, respects Cargo.toml version, skips CI
* 86faad2ddcdc273ac10c1ef781d13279306b0ffd whole-token #none + github-release size-match idempotency #none
* c9f5cc3bf62e498df7bbb5fb283a94f9c6a1223f wire TestContextBuilder into stage tests, add mock GitHub release test
* 4f34bf8c4209ec44ab353306bebca330b6ed5022 wire nightly name_template, gate verbose output, guard snapshot+nightly
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E
* a5a3042ca53c302a70fe4a27c9b8a02c5019ab62 write metadata.json before release stage for include_meta #none

### Others

* f782d890983f0c2bb5be1ea8c301a3ff0d792ecf Merge branch 'worktree-docsite'
* addcb5c1578ae54b0157a80beadfb64caec7c5eb DRY context setup + UX improvements #none
* b5d2af54a576cfa066b415e35b4473af44f6724c split release.rs into release/{mod,milestones,split}.rs #none
* ecd50adb6b49550bd9c902e03726389c10a00b57 deep dedup pass + wire all dead CLI flags and config fields
* 2af0e4ea6dbf6b80a08caff44c37d935e3f6c23d extract CLI types to lib.rs for xtask introspection
* da916faaa69d8fceec3a80b5d7a609e765dc31fe reorder pipeline stages to match GoReleaser parity
* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* 6c11bc7b5c3ee5a1c2ec025819eae830d479afff eliminate all unsafe code from production
* a6a2f986ccdb28c3ea3fe4d3c33ac6b5dc07858d harden unwrap paths, secret handling, path traversal, and regex injection
* a200115f19e03294795a08b4cfc7e6da04886cbc add 22 E2E pipeline tests for multi-format, changelog, workspace, and round-trip scenarios
* c8b1dd32d6b37a5d7f484d95886ca847d4d05928 add 28 error path tests across config, template, build, release, and CLI
* 3c1f06cf37519b4d2f00aac6c7a8221ac87f4c91 add error path tests for config validation, failed builds, and unknown fields

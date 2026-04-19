# Changelog — anodize-stage-docker

## [0.2.0] - 2026-04-19

### Features

* 68594969ba5d1e8be6e21d53c4994a73353d4e19 add DockerDigestConfig type for digest file control
* 62bc47c638445c61fbbcb77a436755be476e540f Stage trait and Context, wire up stage crate stubs
* 63051864d8a9bbd251ca36f751e50399d6e66cc2 add output secret redaction for docker commands
* a081ff86b0fdf40ce7cd571d459d47e9dd2056bd Levenshtein suggestions, project marker warnings, daemon check
* 6840db5c76815af6021b6f4fd721cd4db76791da V2 --iidfile digest capture instead of docker inspect
* 5a27115638d579aef3d325362fb43e90ebb4b6cb add Docker V2 API support with annotations, SBOM, disable, and build_args
* cb6ea428d2617c1521c72fa25973f557cc590961 add legacy goos/goarch/goarm/goamd64 fields to DockerConfig
* 0bdd3e8d5ce7f06d159782e641749610284ec48b inject context env vars into docker/manifest commands
* e50fdd2c403824f2e32b67d26ea1d4c801ee2703 implement HTTP upload for Artifactory/Fury/CloudSmith + promote Homebrew Cask to top-level config
* 472f947baa9e7bae817c66f45f80e0b85700f9ba Session J — sign & docker behavioral gaps
* 9379592ef4a7d8b1dd32d818da09ae974bc1c165 multi-arch Docker image builds via buildx
* 0c7d61dc27874e1e6589322803ec21317b950f90 GoReleaser parity — Session 6 gap closures + specs
* 1b9bee413d89bdb8167e4b8e317ae70adb2d220d Session 5.5 observability + deep audit fixes across entire codebase
* ae4916ebfb684df5b7663d94490a1ef23bbbe926 add --progress=plain to docker buildx for CI diagnostics #none
* 28edbc6ad6842474276d02c198c74790b6ef3e0d add skip_push, extra_files, and push_flags to docker stage
* 98765725221efcaa3cc9d57bc5f03cee5eeae1e3 anodize bump --commit bundles changelog + --strict version-pin gate #none
* cc098a9e876be05f586d7ff8ec8b85178f39aa34 complete Phase 1 parity tasks 1-9 (1,736 tests pass)
* 18f300c58484f9007f63ba2749f062cc9bc9b693 scaffold workspace with core and stage crate stubs
* e575fa81397f48df45a77cf678977d86f9470795 v0.1.0 release preparation
* 809ddaeed7e0d4c5b5af2bd1a44edc1b802414f7 wire NSIS and AppBundle stages into workspace, CLI, and pipeline

### Bug Fixes

* 86ecb943eabb244798f71f407b6a8fc107233465 match GoReleaser report_sizes behavior with type filter and size storage
* c30bc496b809b738263a909bf358a6da06edb515 change DockerSignConfig.output to StringOrBool for GoReleaser parity
* 8353e036402f9a91c21e931cf8f893af1ebc8643 DockerDigest review fixes — collision, artifacts, errors, docs
* f42e4fe2e0d5582e3c25ee149996b419af382c5f UX diagnostics, dedup/sort, skip_push template wiring
* 4f3e946d9f4b5ecc48e2f80aaae2370f874b6308 V2 staging layout, manifest digest, additional parity fixes
* e3594e9a70b8e4a27395475b7d0de5c76e619dab deep parity audit — critical behavioral fixes
* 2250cdc8cca0344f7fe3ca6c21bae9cecff5de65 improve env passthrough test + add probe comments
* 9d94db54754d55e1450b596252d2650743b50efd legacy push, combined digests, retry codes, redact sort
* 76ba5595076b1937077f3de27e5c7baf08263fb7 redact output.stderr/stdout bytes before check_output
* 0adcfc1ca6d6e36af8dfc7e70cb20451f4e7e888 sign ID default, use-backend default, COPY/ADD file listing, push digest capture
* 15300788f0dd1c8b0681dd96e863f2b914a29c5c SignConfig.output to StringOrBool + template evaluation
* 0e5ed75e7f459fd48868f7703d5fef4c5f48fa5b wire up build_flag_templates, fix dry-run multi-platform incompatibility
* 0f44e2d7fdbdc962706af3fbeec1abb4e2b47493 address all code review findings
* 8207788a9b4163501014fd12eb3126f95bcc9255 68 GoReleaser parity bugs across all stages
* 97549d213dddec2aed5995ed0ace0df6a84e9199 address all code review findings for Tasks 8-12
* c0e62906db01a768a05f754143690b40cc8aae72 cargo fmt, clippy, and add CI auto-tag step
* 2cb51c5d2c04bc12dfe6364087867ece3ade2963 drain known-bugs (W1+W2+S1-S4 safety, S1-S6 pro, S1-S5 dedup) #none
* a7d9766fc991ca3219fdaa1939af3985e8b21ff3 parity sweep — 31 GoReleaser parity fixes across release/sign/changelog/publishers/packaging/announce #none
* 7f681f5421d13e54129228b87d5c747f4f9fca95 skip docker_manifests for tags already pushed as multi-arch by docker_v2 #none
* fbb83bb934b1a165cec92e1e7a7cdca3aae186a5 split/merge env poisoning — macOS HOME leaked into Linux docker builds #none
* 91f7d7f13df7deebe4f54ccca223129f11ff1324 strict-mode bulletproofing + targets subcommand + publisher safety #none
* a2ea9c5625dff18aaea772052848aceb796e7054 template-render push_flags and guard against directories in docker extra_files
* 5c62e6f3ee5cfb70a09a08963b21e1699db8f351 wire up git variable population and CI improvements for Task 3E

### Others

* fbcd944952855d634d1574994ee88f4c2e199e9a Revert "feat(docker): add legacy goos/goarch/goarm/goamd64 fields to DockerConfig"
* 4e13532258f33f3aa1b96383f7a37394cdd47b68 improve iidfile comments + add V2 digest read test
* 441b3264a59007c448b1ea046f02ba57e982f2f7 unwrap/expect -> ?/context (142 -> 0 non-test lib sites) + publisher cleanup #none
* 0fda8df225204be08c7c0e3213269028789117ac add subdirectory path test for project markers
* 806264fb7fb919aa0b21b240836b02b0e74d3dcf add 56 error path tests across all stages and core modules
* 5de4f85a4e9ab108bd058dd071bbc92922d900e9 add 60 stage behavior tests verifying config fields produce correct output

# File Size Audit and Decomposition (anodizer)

Mirror of cfgd's `/opt/repos/cfgd/.claude/plans/file-size-audit-and-decomposition.md`
adapted to anodizer's stage-crate layout. Same goal, same hard rules.

## Goal

Carve every Rust file >1,500 prod lines (or >2,500 raw) into focused submodules
that:

1. Preserve all tests — every test still passes after the split.
2. Preserve all public API — no caller edits (`pub use` re-exports from the
   parent module).
3. Are cohesive — single responsibility per submodule.
4. Don't introduce duplication or new abstraction layers.
5. Don't break the [module-boundaries rule](../rules/module-boundaries.md) —
   stage-* allow-list stays intact (any new submodule under a `stage-*` crate
   inherits the umbrella allow-list; no new top-level crate).

## Sized inventory (snapshot 2026-05-03)

Sorted by **prod** lines. Prod = lines before the bottom `^mod tests {` block.

### Tier 1 — must carve (≥1,500 prod lines)

| # | File | Total | **Prod** | Tests | Notes |
|---|---|---:|---:|---:|---|
| 1 | `crates/core/src/config.rs` | 12,217 | **6,553** | 5,664 | ~50 banner-delimited sections; cleanest carve in repo |
| 2 | `crates/stage-build/src/lib.rs` | 4,873 | **2,533** | 2,340 | universal-binary, target-validation, profile-detect, `impl Stage::run` (~1.2k) |
| 3 | `crates/stage-docker/src/lib.rs` | 5,039 | **2,413** | 2,626 | command-builders + retry/backoff + v2 path + extra-files staging |
| 4 | `crates/stage-release/src/lib.rs` | 5,474 | **2,284** | 3,190 | already has `gitea`/`gitlab`/`release_body` siblings; GitHub path lives in `lib.rs` |
| 5 | `crates/stage-publish/src/homebrew.rs` | 2,864 | **2,075** | 789 | formula gen + cask gen + publish-formula + publish-cask + helpers |
| 6 | `crates/stage-changelog/src/lib.rs` | 4,863 | **1,888** | 2,975 | grouping + rendering + GH/GL/Gitea fetchers + `impl Stage::run` |
| 7 | `crates/stage-nfpm/src/lib.rs` | 6,447 | **1,729** | 4,718 | yaml-types + format-builders + `impl Stage::run`; tests dominate |
| 8 | `crates/core/src/template.rs` | 3,578 | **1,600** | 1,978 | static parser + tera context + helpers + `TemplateVars` |
| 9 | `crates/stage-publish/src/util.rs` | 2,310 | **1,521** | 789 | PR creation (gh + API), token injection, fork sync, artifact filter |
| 10 | `crates/core/src/git.rs` | 2,349 | **1,403** | 946 | semver, info, commits, tags, gh-api, remote parse — already module-shaped |
| 11 | `crates/core/src/template_preprocess.rs` | 2,077 | **1,277** | 800 | go→tera translator; rewriters per syntax form |
| 12 | `crates/stage-source/src/lib.rs` | 2,350 | **1,210** | 1,140 | source archive + cyclonedx + spdx generators + `impl Stage::run` |

### Tier 2 — should carve (1,000–1,500 prod lines)

| # | File | Total | **Prod** | Tests | Notes |
|---|---|---:|---:|---:|---|
| 13 | `crates/stage-publish/src/nix.rs` | 1,563 | **1,159** | 404 | hex/base32/SRI helpers + nix expr generator + publish flow |
| 14 | `crates/stage-snapcraft/src/lib.rs` | 3,154 | **1,140** | 2,014 | yaml-types + arch helpers + `SnapcraftStage` + `SnapcraftPublishStage` |
| 15 | `crates/stage-announce/src/lib.rs` | 3,306 | **1,102** | 2,204 | dispatcher orchestrator over already-split provider files |
| 16 | `crates/stage-archive/src/lib.rs` | 5,146 | **1,065** | 4,081 | already partly split; `impl Stage::run` is the bloat |
| 17 | `crates/stage-blob/src/lib.rs` | 2,240 | **1,048** | 1,192 | KMS + s3/gcs/azure store builders + put-options + upload loop |
| 18 | `crates/stage-publish/src/chocolatey.rs` | 1,689 | **1,047** | 642 | nuspec gen + install-script gen + nupkg packing + push |

### Tier 3 — borderline (500–1,000 prod lines, defer unless scope-adjacent)

| File | Total | Prod | Tests |
|---|---:|---:|---:|
| `crates/stage-publish/src/winget.rs` | 1,546 | 998 | 548 |
| `crates/stage-notarize/src/lib.rs` | 2,102 | 973 | 1,129 |
| `crates/cli/src/commands/tag.rs` | 1,460 | 894 | 566 |
| `crates/cli/src/pipeline.rs` | 1,493 | 848 | 645 |
| `crates/cli/src/commands/helpers.rs` | 1,671 | 843 | 828 |
| `crates/core/src/context.rs` | 2,173 | 810 | 1,363 |
| `crates/stage-checksum/src/lib.rs` | 3,077 | 757 | 2,320 |
| `crates/stage-flatpak/src/lib.rs` | 1,662 | 700 | 962 |
| `crates/stage-msi/src/lib.rs` | 2,254 | 692 | 1,562 |
| `crates/stage-publish/src/aur.rs` | 1,452 | 678 | 774 |

### Test-bloat — externalize to sibling `tests.rs` even without prod carve

| File | Inline `#[cfg(test)]` lines |
|---|---:|
| `crates/stage-nfpm/src/lib.rs` | 4,718 |
| `crates/stage-archive/src/lib.rs` | 4,081 |
| `crates/stage-release/src/lib.rs` | 3,190 |
| `crates/stage-changelog/src/lib.rs` | 2,975 |
| `crates/stage-docker/src/lib.rs` | 2,626 |
| `crates/stage-sign/src/lib.rs` | 2,487 |
| `crates/stage-checksum/src/lib.rs` | 2,320 |
| `crates/stage-build/src/lib.rs` | 2,340 |
| `crates/stage-announce/src/lib.rs` | 2,204 |
| `crates/stage-snapcraft/src/lib.rs` | 2,014 |
| `crates/stage-msi/src/lib.rs` | 1,562 |
| `crates/cli/tests/integration.rs` | 4,216 (already in `tests/`, but split-by-domain would help) |
| `crates/core/tests/config_parsing_tests.rs` | 3,767 (same) |

Rule: if the originating `mod tests` block is >1,500 lines after the production
carve lands, externalize it to a sibling `tests.rs` in the **same commit batch**.
For files that are mostly tests (stage-sign, stage-archive, stage-nfpm), the
test split is the primary win — production carve is secondary.

## Anti-patterns to avoid

- **No `utils.rs` / `helpers.rs` grab-bag files.** Shared helpers go in
  `crates/core/src/util.rs` (already exists at 562 lines — don't bloat it).
- **No new abstraction layers.** Splitting `lib.rs` → `lib.rs` + `foo.rs` +
  `bar.rs` with `pub use` is fine; introducing new traits to "justify" the
  split is not.
- **No movement across crates.** Keep submodules inside the originating crate.
- **No tests moved away from their code.** Per-submodule `#[cfg(test)] mod
  tests` if classifiable; otherwise externalize the whole block to a sibling
  `tests.rs` (`mod tests;` declaration) in the same commit.
- **Zero behavior change.** Pure structural refactor.

## Per-file carve maps (Tier 1)

Line numbers below are against snapshot 2026-05-03; **re-verify before executing**
since `mod tests` and helper additions will shift offsets.

### A. `crates/core/src/config.rs` — 6,553 prod lines  ⭐ start here

This file is exceptionally clean: ~50 banner-delimited sections, one config
struct group per section. Promote the file to a directory and carve by section.

```
core/src/config/
  mod.rs                # Config root struct + Default impl + validators
                        # (validate_version, validate_tag_sort,
                        # validate_release_backends, validate_defaults_axis,
                        # validate_format_overrides, validate_homebrew_cask_url_template)
                        # + the few `pub use` re-exports + tests.rs declaration
  include.rs            # IncludeSpec, IncludeFilePath, IncludeUrlConfig          [L7–51]
  env_files.rs          # EnvFilesConfig + EnvFilesTokenConfig + helpers          [L645–908]
  defaults.rs           # Defaults, PublishDefaults, DefaultsCrateBlock,
                        # DefaultsWorkspaceBlock                                  [L911–1052]
  build.rs              # BuildIgnore, BuildOverride, CrossStrategy,
                        # CrateConfig, UniversalBinaryConfig, BuildConfig,
                        # BuildHooksConfig, ArchiveHooksConfig                    [L1054–1330]
  archives.rs           # ArchivesConfig (untagged enum), WrapInDirectory,
                        # ArchiveConfig, FormatOverride, ArchiveFileSpec,
                        # FileInfo, parse_octal_mode, VALID_ARCHIVE_FORMATS,
                        # ExtraFileSpec, TemplatedExtraFile, ChecksumConfig,
                        # ContentSource                                           [L1332–1825]
  release.rs            # ReleaseConfig, ScmRepoConfig, GitHubConfig alias,
                        # ForceTokenKind, GitHub/GitLab/Gitea URLs,
                        # PrereleaseConfig, MakeLatestConfig, SkipPushConfig      [L1827–2280]
  publishers/
    mod.rs              # RepositoryConfig, PullRequestConfig, CommitAuthorConfig,
                        # CommitSigningConfig, PublishConfig                      [L2282–2483]
    homebrew.rs         # HomebrewConfig + cask types + ScoopConfig               [L2485–2860]
    chocolatey.rs       # ChocolateyConfig + ChocolateyDependency                 [L2862–2950]
    winget.rs           # WingetConfig + WingetDependency                         [L2952–3034]
    aur.rs              # AurConfig                                               [L3036–3103]
    cargo.rs            # CargoPublishConfig                                      [L2433–2483 from publishers/mod]
  docker.rs             # DockerRetryConfig, DockerV2Config,
                        # DockerDigestConfig, DockerManifestConfig                [L3122–3233]
  nfpm.rs               # NfpmConfig + NfpmLibdirs + NfpmScripts + NfpmContent
                        # + per-format (rpm/deb/apk/archlinux/ipk) + signature    [L3235–3634]
  snapcraft.rs          # SnapcraftConfig + App/Layout/ExtraFileSpec              [L3636–3839]
  installers.rs         # DmgConfig, MsiConfig, PkgConfig, NsisConfig,
                        # AppBundleConfig, FlatpakConfig                          [L3841–4057]
  blob.rs               # BlobConfig                                              [L4059–4123]
  partial.rs            # PartialConfig                                           [L4125–4136]
  binstall.rs           # BinstallConfig                                          [L4138–4153]
  notarize.rs           # NotarizeConfig + MacOSSignNotarizeConfig +
                        # MacOSSignConfig + MacOSNotarizeApiConfig +
                        # native variants + MacOSNativeArtifactKind               [L4155–4357]
  source.rs             # SourceFileEntry/Info, SourceConfig + custom serde       [L4359–4500]
  sbom.rs               # SbomConfig + custom serde                               [L4502–4672]
  version_sync.rs       # VersionSyncConfig                                       [L4674–4685]
  changelog.rs          # ChangelogConfig (huge — biggest single section)         [L4687–4951]
  upx.rs                # UpxConfig                                               [L4964–5055]
  snapshot_nightly.rs   # SnapshotConfig + NightlyConfig + MetadataConfig         [L5057–5109]
  templatefiles.rs      # TemplateFileConfig                                      [L5111–5134]
  announce.rs           # AnnounceConfig (large — providers union)                [L5136–5552]
  dockerhub.rs          # DockerHub description sync                              [L5554–5604]
  artifactory.rs        # Artifactory publisher                                   [L5606–5657]
  cloudsmith.rs         # CloudSmith publisher                                    [L5659–5688]
  publisher.rs          # PublisherConfig (generic)                               [L5690–5728]
  hooks.rs              # HooksConfig                                             [L5730–5796]
  git.rs                # GitConfig                                               [L5798–5828]
  monorepo.rs           # MonorepoConfig                                          [L5830–5862]
  tag.rs                # TagConfig                                               [L5864–5914]
  workspace.rs          # WorkspaceConfig                                         [L5916–5958]
  string_or_bool.rs     # StringOrBool (load-bearing — used everywhere)           [L5960–6327]
  milestone.rs          # MilestoneConfig                                         [L6345–6388]
  upload.rs             # UploadConfig (generic HTTP upload)                      [L6390–6443]
  aur_source.rs         # AurSourceConfig                                         [L6445–6546]
  schema.rs             # signs_schema, upx_schema, sboms_schema, etc.
                        # (the custom schemars helpers scattered through file)
  serde.rs              # custom deserializers (deserialize_archives_config,
                        # deserialize_signs, deserialize_binary_signs,
                        # deserialize_source_files, deserialize_sboms)
  tests.rs              # externalize the bottom 5,664 lines as a single block
```

Order this carve in **two waves** (one PR each):
- **Wave A**: promote to `config/` directory + extract just the 4–6 biggest
  sections (`announce.rs`, `string_or_bool.rs`, `changelog.rs`, `nfpm.rs`,
  `snapcraft.rs`, `notarize.rs`). Externalize tests as `tests.rs` in same wave.
- **Wave B**: carve the remaining ~30 small sections.

Reason for two waves: a single PR moving 6,500 prod lines across 30+ files is
unreviewable. Tier 1A (the heavy-hitter sections + test externalization) buys
~50% of the file size reduction by itself.

### B. `crates/stage-build/src/lib.rs` — 2,533 prod lines

```
stage-build/src/
  lib.rs                # mod declarations, BuildCommand struct, BuildStage,
                        # impl Stage::run dispatcher (slimmed)
  binstall.rs           # (already exists, 199 lines)
  version_sync.rs       # (already exists, 260 lines)
  family.rs             # same_apple_family, same_windows_family,
                        # strip_glibc_suffix, target_for_validation,
                        # ensure_targets_installed                                [L95–960, 1245–1284]
  universal.rs          # build_universal_binary, project_universal_out_path,
                        # try_compile_glob (universal binary plumbing)            [L376–942]
  workspace.rs          # check_workspace_package, find_workspace_root,
                        # crate_has_binary_target, resolve_binary_path,
                        # cargo_target_dir                                        [L279–1162]
  profile.rs            # detect_cargo_profile, resolve_reproducible_epoch,
                        # parse_amd64_variant_from_rustflags, detect_amd64_variant [L331, 1196, 1286–1325]
  copy_from.rs          # resolve_copy_from                                       [L1211–1244]
  dynlink.rs            # is_dynamically_linked + ELF/Mach-O probes               [L973–1064]
  run.rs                # impl Stage for BuildStage (the orchestration)          [L1329–end of prod]
  tests.rs              # externalize the 2,340-line test block
```

### C. `crates/stage-docker/src/lib.rs` — 2,413 prod lines

```
stage-docker/src/
  lib.rs                # mod declarations, DockerStage, impl Stage::run dispatch
  detect.rs             # docker_supports_provenance, is_docker_daemon_available,
                        # check_buildx_driver, find_image_digest                  [L59–238]
  retry.rs              # is_retriable_error, is_retriable_error_v2,
                        # parse_duration_string, resolve_retry_params             [L100–353]
  command.rs            # build_docker_command, build_docker_v2_command,
                        # tag_suffix, platform_to_arch, find_sha256_digest        [L239–789]
  v2_resolve.rs         # resolve_backend, is_docker_v2_skipped, resolve_skip_push,
                        # resolve_digest_config, apply_docker_v2_defaults,
                        # is_docker_v2_sbom_enabled, generate_v2_image_tags       [L355–725]
  staging.rs            # stage_artifacts_v2, copy_dockerfile, stage_extra_files,
                        # warn_project_markers_in_extra_files,
                        # list_staging_dir_recursive                              [L764, 1217–1449]
  build.rs              # DockerBuildJob, DockerBuildResult, execute_docker_build [L811–1216]
  spelling.rs           # levenshtein_distance                                    [L22–58]
  run.rs                # impl Stage for DockerStage (orchestration)              [L1452–end]
  tests.rs              # externalize the 2,626-line test block
```

### D. `crates/stage-release/src/lib.rs` — 2,284 prod lines

Already has `gitea.rs`, `gitlab.rs`, `release_body.rs` siblings (765/474/293
lines). The remaining bloat is the GitHub path inside `lib.rs`.

```
stage-release/src/
  lib.rs                # mod declarations, ReleaseStage, dispatch by SCM
  gitea.rs              # (already exists)
  gitlab.rs             # (already exists)
  release_body.rs       # (already exists)
  github/
    mod.rs              # impl-Stage delegate for github backend (slimmed)
    client.rs           # build_octocrab_client, build_octocrab_client_insecure,
                        # DangerousNoCertVerifier                                 [L524–705]
    upload.rs           # populate_artifact_download_urls + upload-asset path     [L389–522 + run-time slice]
    encode.rs           # percent_encode_query                                    [L36–]
  tests.rs              # externalize the 3,190-line test block
```

### E. `crates/stage-publish/src/homebrew.rs` — 2,075 prod lines

```
stage-publish/src/homebrew/
  mod.rs                # public API (publish_to_homebrew, publish_cask,
                        # publish_top_level_homebrew_casks) + dispatch
  formula/
    mod.rs              # FormulaOptions + generate_formula + generate_formula_with_opts [L822–1262]
    deps.rs             # build_depends_directives, build_conflicts_directives,
                        # build_uninstall_directives                              [L1992–end]
  cask/
    mod.rs              # CaskParams + generate_cask + CaskGenResult              [L230–693]
    publish.rs          # publish_cask body                                       [L696–820]
    finder.rs           # find_top_level_cask_artifact                            [L1953–1990]
  publish_formula.rs    # publish_to_homebrew body                                [L1263–1690]
  publish_top.rs        # publish_top_level_homebrew_casks body                   [L1692–1951]
  tests.rs              # externalize the 789-line test block (or per-submod tests)
```

### F. `crates/stage-changelog/src/lib.rs` — 1,888 prod lines

```
stage-changelog/src/
  lib.rs                # mod declarations + ChangelogStage + impl Stage::run
  group.rs              # GroupedCommits, group_commits_inner,
                        # compile_filter_patterns                                 [L42–449]
  render.rs             # render_groups, render_commit_line,
                        # render_crate_section, InsertionMode, ChangelogUpdate,
                        # merge_into_changelog                                    [L450–905, 598–631]
  fetch/
    mod.rs              # fetch_git_commits, parse_git_log_records,
                        # fetch_git_commits_in, today_yyyy_mm_dd                  [L771–831, 1387]
    github.rs           # fetch_github_commits                                    [L1435–1592]
    gitlab.rs           # fetch_gitlab_commits                                    [L1593–1745]
    gitea.rs            # fetch_gitea_commits                                     [L1746–1888]
  preempt.rs            # should_preempt_scm_to_git, relative_filter              [L756, 1374]
  tests.rs              # externalize the 2,975-line test block
```

### G. `crates/stage-nfpm/src/lib.rs` — 1,729 prod lines

```
stage-nfpm/src/
  lib.rs                # mod declarations + NfpmStage + impl Stage::run
  filename.rs           # (already exists, 618 lines)
  yaml/
    mod.rs              # NfpmYamlConfig + NfpmYamlScripts + NfpmYamlContent +
                        # NfpmYamlFileInfo + NfpmYamlSignature + is_empty_vec    [L23–153]
    rpm.rs              # NfpmYamlRpmScripts + NfpmYamlRpm                       [L154–181]
    deb.rs              # NfpmYamlDebTriggers + NfpmYamlDebScripts + NfpmYamlDeb [L182–229]
    apk.rs              # NfpmYamlApkScripts + NfpmYamlApk                       [L230–245]
    archlinux.rs        # NfpmYamlArchlinuxScripts + NfpmYamlArchlinux           [L246–263]
    ipk.rs              # NfpmYamlIpk + NfpmYamlIpkAlternative                   [L264–300]
  generate.rs           # NfpmLibraryPaths, generate_nfpm_yaml,
                        # generate_nfpm_yaml_with_env, build_yaml_signature,
                        # build_yaml_rpm/deb/apk/archlinux/ipk,
                        # resolve_passphrase_from_env                            [L301–822]
  command.rs            # nfpm_command, validate_format,
                        # is_arch_supported_for_format, format_extension          [L824–887, 1713–1729]
  run.rs                # NfpmJob + impl Stage for NfpmStage                     [L889–1712]
  tests.rs              # externalize the 4,718-line test block (this is the win)
```

### H. `crates/core/src/template.rs` — 1,600 prod lines

```
core/src/template/
  mod.rs                # public API: parse_static, render_static, render,
                        # extract_artifact_ext, validate_single_env_only         [L47–74, 1499–1599]
  vars.rs               # TemplateVars + impl + Default + clear_per_target_vars,
                        # PER_TARGET_VARS, PER_ARTIFACT_VARS,
                        # clear_per_artifact_vars                                [L1227–1400]
  context.rs            # build_tera_context, build_tera_context_for_template    [L1402–1497]
  helpers.rs            # hex_encode, expand_tilde, value_to_string,
                        # translate_go_time_format, increment_version,
                        # VersionPart enum                                       [L75–1226]
  tests.rs              # externalize the 1,978-line test block
```

### I. `crates/stage-publish/src/util.rs` — 1,521 prod lines

```
stage-publish/src/util/
  mod.rs                # public re-exports
  pr.rs                 # create_pr_via_gh_cli, create_pr_via_api,
                        # gh_is_available, sync_fork                              [L356–1115]
  token.rs              # inject_token_in_url                                     [L218–]
  branch.rs             # fetch_default_branch                                    [L1116–1228]
  artifact_filter.rs    # artifact_to_os_artifact, filter_by_variant              [L1230–1521]
  tests.rs              # externalize tests
```

### J. `crates/core/src/git.rs` — 1,403 prod lines

Promote to `git/` directory.

```
core/src/git/
  mod.rs                # public re-exports + git_output (private helper) +
                        # is_git_dirty + render_ignore_patterns                  [L14–243]
  semver.rs             # SemVer + impls + parse_semver, parse_semver_tag,
                        # compare_prerelease                                     [L50–167]
  info.rs               # GitInfo, detect_git_info, strip_url_credentials,
                        # local_git_user_name, local_git_user_email              [L169, 220–386]
  commits.rs            # Commit, get_commits_between, get_all_commits,
                        # parse_commit_output, get_last_commit_messages,
                        # get_commit_messages_between, has_commits_since_tag,
                        # has_changes_since, get_short_commit, get_current_branch,
                        # get_*_path variants                                    [L198, 574–1029]
  tags.rs               # has_version_placeholder, extract_tag_prefix,
                        # strip_monorepo_prefix, find_latest_tag_matching*,
                        # collect_semver_tags, get_all_semver_tags,
                        # get_branch_semver_tags, find_previous_tag,
                        # create_and_push_tag                                    [L387–768, 1156–end]
  github_api.rs         # gh_api_get, gh_api_get_paginated, gh_api_post,
                        # create_tag_via_github_api                              [L769–953]
  remote.rs             # parse_github_remote, detect_github_repo,
                        # parse_remote_owner_repo, detect_owner_repo             [L1040–1155]
  stage.rs              # stage_and_commit                                       [L1030–1039]
  tests.rs              # externalize the 946-line test block
```

### K. `crates/core/src/template_preprocess.rs` — 1,277 prod lines

```
core/src/template_preprocess/
  mod.rs                # public preprocess() + static_regex helper             [L24–94]
  go_blocks.rs          # preprocess_go_blocks + tera_block                     [L95, 105–261]
  dollar.rs             # strip_dollar_vars                                     [L262–315]
  dots.rs               # preprocess_strip_dots                                 [L316–393]
  list_subexpr.rs       # preprocess_list_subexpr                               [L394–440]
  builtins.rs           # preprocess_go_builtins, rewrite_*                     [L441–681]
  map.rs                # preprocess_map_syntax                                 [L683–737]
  positional/
    mod.rs              # preprocess_positional_syntax + lookup_positional      [L739, 1101–1110]
    tokens.rs           # Token, tokenize_block, significant_tokens,
                        # extract_block_parts, format_arg_value, token_to_str   [L815, 896–1024, 1256–1278]
    rewrite.rs          # try_rewrite_control_block, try_rewrite_standalone,
                        # try_rewrite_piped, PositionalSyntax                   [L841–1255]
  methods.rs            # preprocess_method_calls                               [L794–814]
  tests.rs              # externalize the 800-line test block
```

### L. `crates/stage-source/src/lib.rs` — 1,210 prod lines

```
stage-source/src/
  lib.rs                # SourceStage + impl Stage::run                          [L598–1188]
  archive.rs            # SourceArchiveInputs + create_source_archive +
                        # get_repo_root + find_cargo_lock                        [L24–349, 1189–1209]
  cargo_lock.rs         # CargoPackage + parse_cargo_lock                        [L372–413]
  sbom/
    mod.rs              # public re-exports
    cyclonedx.rs        # generate_cyclonedx                                    [L414–479]
    spdx.rs             # generate_spdx + deterministic_uuid_from               [L480–597]
  tests.rs              # externalize the 1,140-line test block
```

## Tier 2 carve sketches

Lower priority — execute in a follow-up wave once Tier 1 lands.

- **stage-publish/nix.rs (1,159)** → `nix/{mod,base32,sri,binary_probe,
  generator,publish}.rs`
- **stage-snapcraft/lib.rs (1,140)** → `{lib, yaml, arch, generate, command,
  upload, run}.rs` + `tests.rs`
- **stage-announce/lib.rs (1,102)** → just externalize tests + slim dispatcher;
  providers are already split. Move `dispatch`, `render_message`,
  `render_json_template`, `resolve_webhook_headers`, `is_enabled`,
  `require_env*` into `dispatch.rs`.
- **stage-archive/lib.rs (1,065)** → primarily test-externalization win;
  `impl Stage::run` body splits into `stage/run.rs` + `stage/staging.rs`.
- **stage-blob/lib.rs (1,048)** → `{lib, kms, store/{s3,gcs,azure},
  put_options, upload, run}.rs`
- **stage-publish/chocolatey.rs (1,047)** → `chocolatey/{mod,nuspec,
  install_script,nupkg,push,feed_hash}.rs`

## Execution order

Mechanical-ness first; what unblocks downstream work next.

1. **`stage-sign/lib.rs` test externalization** — 6 prod lines, 2,487 test
   lines. Trivial, mechanical, biggest test-bloat ratio in repo. Use this as
   the warm-up to validate the recipe in this codebase.
2. **`stage-nfpm/lib.rs` test externalization** — 4,718 test lines. Same
   pattern, larger payoff. Optionally bundle with the production carve (G).
3. **`crates/core/src/config.rs` Wave A** — promote to `config/` + extract the
   six biggest sections + externalize tests. Highest blast-radius file in the
   repo touches almost every other crate via `pub use`.
4. **`crates/core/src/config.rs` Wave B** — remaining ~30 sections. Can
   parallelize across worktrees once Wave A is on master.
5. **`stage-build/lib.rs`** + **`stage-docker/lib.rs`** + **`stage-release/lib.rs`**
   in parallel worktrees (different stage-* crates, no overlap).
6. **`stage-publish/homebrew.rs`** + **`stage-publish/util.rs`** — both inside
   stage-publish; serialize.
7. **`stage-changelog/lib.rs`** + **`stage-nfpm/lib.rs` prod carve** + **`stage-source/lib.rs`** in parallel worktrees.
8. **`core/template.rs`** + **`core/git.rs`** + **`core/template_preprocess.rs`** in parallel worktrees (same crate, but disjoint files — confirm no cross-deps before parallelizing).
9. **Tier 2** — sketches above, one PR each.

## Recipe (anodizer-specific)

Cribbed from cfgd's recipe, adapted for anodizer commit gating:

1. **Use `task commit -- -m "..."`, NOT `git commit`.** Sandbox blocks bare
   `git commit`. `task commit` runs `task lint` first then commits. See
   `CLAUDE.local.md` for the full pattern. Subject **must not** include
   `#none`; omit issue marker if there is none.
2. **Verify before committing**: `cargo fmt --all` + `task lint` + `task test`.
   `task commit` runs lint as a precondition; full test suite is still your job.
3. **`pub use` re-exports preserve callers** — no caller should change. Add a
   `pub use` line for every type/fn/const moved out of the parent module.
4. **Use sed-based block-moves for big carves**. `Write` for files <100 lines;
   `sed -n 'X,Yp' source > /tmp/staging.rs` then `cat headers.rs
   /tmp/staging.rs > dest.rs` for the rest. Keeps you under the 16k
   assistant-output-token cap.
5. **Test-block externalization pattern**:
   - `sed -n 'TEST_START,$p' lib.rs > tests.rs`
   - Strip the outer `mod tests { ... }` wrapper from the copied content
     (the file IS the module body).
   - In `lib.rs`, replace the entire test block with `#[cfg(test)] mod tests;`.
   - **Do NOT dedent the inner block** — `r#"..."#` literals contain
     indented YAML that breaks if dedented. `cargo fmt` normalizes.
6. **Visibility**: items reached from `tests.rs` via `super::*` need at least
   `pub(super)` (or `pub(crate)` if a sibling submodule needs them). Pick one
   per carve and stick with it. Default to `pub(super)` + `use submod::*;`
   glob in `mod.rs`.
7. **Worktrees only for parallel non-interfering agents.** Per the user's
   preference: work on master directly unless 2+ concurrent agents are editing
   disjoint file sets.

## Hard constraints

- Zero behavior change. Pure structural movement.
- Preserve public API via `pub use` re-exports. No caller edits.
- No grab-bag `utils.rs` / `helpers.rs` files.
- No new abstraction layers (no new traits / pub types).
- Never `git push`. Never `git commit` directly (use `task commit`).
- Don't commit anything under `.claude/` (gitignored).
- Always `task test` before claiming done; **never** mark complete on
  unverified work.
- Each carve = exactly one commit on master, message of the form
  `refactor(<area>): carve <file> into <dir>/`.

## Reusable handoff prompt

See `.claude/plans/decomposition-handoff-prompt.md` for the prompt template
to feed to a subagent (or to use yourself, one chunk at a time).

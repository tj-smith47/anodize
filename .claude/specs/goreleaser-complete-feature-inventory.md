# GoReleaser Complete Feature Inventory

> Comprehensive inventory of every GoReleaser feature, config field, and capability.
> Researched from goreleaser.com documentation, March 2026.

---

## 1. Builds

### Builder Types
- **Go** (default builder)
- **Rust** (`builder: rust`)
- **Zig** (`builder: zig`)
- **Bun** (`builder: bun`)
- **Deno** (`builder: deno`)
- **Python** (`builder: python`) -- "Coming soon" as of research date
- **UV** (`builder: uv`) -- Python UV builder
- **Poetry** (`builder: poetry`) -- Python Poetry builder
- **Pre-built/Import** (`builder: prebuilt`) -- import pre-compiled binaries

### Go Builder Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project dir name |
| `main` | string | `.` |
| `binary` | string (template) | project dir name |
| `dir` | string | `.` |
| `command` | string | `build` |
| `flags` | []string | `[]` |
| `ldflags` | []string (template) | default version/commit |
| `asmflags` | []string | `[]` |
| `gcflags` | []string | `[]` |
| `tags` | []string | `[]` |
| `env` | []string (template) | os.Environ() + env config |
| `tool` | string (template) | `go` |
| `gobinary` | string | `go` (deprecated, use `tool`) |
| `goos` | []string | `[darwin, linux, windows]` |
| `goarch` | []string | `[386, amd64, arm64]` |
| `goarm` | []string | `[6]` |
| `goamd64` | []string | `[v1]` |
| `goarm64` | []string | `[v8.0]` (v2.4+) |
| `gomips` | []string | `[hardfloat]` (v2.4+) |
| `go386` | []string | `[sse2]` (v2.4+) |
| `goppc64` | []string | `[power8]` (v2.4+) |
| `goriscv64` | []string | `[rva20u64]` (v2.4+) |
| `targets` | []string | generated from goos/goarch matrix |
| `ignore` | []object | `[]` (goos/goarch combos to skip) |
| `buildmode` | string | standard |
| `mod_timestamp` | string (template) | empty |
| `overrides` | []object | `[]` (target-specific overrides) |
| `hooks.pre[]` | []object | `{}` |
| `hooks.post[]` | []object | `{}` |
| `skip` | bool (template) | `false` |
| `no_unique_dist_dir` | bool | `false` |
| `no_main_check` | bool | `false` |

### Rust Builder Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project dir name |
| `builder` | string | `rust` |
| `binary` | string | project dir name |
| `targets` | []string | x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-gnu, aarch64-unknown-linux-gnu, aarch64-apple-darwin |
| `dir` | string | `.` |
| `tool` | string | `cargo` (can use `cross`) |
| `command` | string | `zigbuild` |
| `flags` | []string (template) | `--release` |
| `env` | []string (template) | inherited |
| `hooks.pre/post` | []object | - |
| `skip` | bool | `false` |

### Zig Builder Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project dir name |
| `builder` | string | `zig` |
| `binary` | string | project dir name |
| `targets` | []string | x86_64-linux, x86_64-macos, x86_64-windows, aarch64-linux, aarch64-macos |
| `dir` | string | `.` |
| `tool` | string | `zig` |
| `command` | string | `build` |
| `flags` | []string | `-Doptimize=ReleaseSafe` |
| `env` | []string | inherited |
| `hooks.pre/post` | []object | - |
| `skip` | bool | `false` |

### Bun Builder Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project dir name |
| `builder` | string | `bun` |
| `binary` | string | project dir name |
| `targets` | []string | linux-x64-modern, linux-arm64, darwin-x64, darwin-arm64, windows-x64-modern |
| `dir` | string | `.` |
| `main` | string | from package.json or `.` |
| `tool` | string (template) | `bun` |
| `command` | string | `build` |
| `flags` | []string (template) | `[--compile]` |
| `env` | []string (template) | inherited |
| `hooks.pre/post` | []object | - |
| `skip` | bool | `false` |

### Deno Builder Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project dir name |
| `builder` | string | `deno` |
| `binary` | string | project dir name |
| `targets` | []string | x86_64-pc-windows-msvc, x86_64-apple-darwin, aarch64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu |
| `dir` | string | `.` |
| `main` | string | `main.ts` |
| `tool` | string (template) | `deno` |
| `command` | string (template) | `compile` |
| `flags` | []string (template) | `[]` |
| `env` | []string (template) | inherited |
| `hooks.pre/post` | []object | - |
| `skip` | bool | `false` |

### UV Builder Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project dir name |
| `builder` | string | `uv` |
| `dir` | string | `.` |
| `buildmode` | string | `wheel` (also: `sdist`) |
| `tool` | string | `uv` |
| `command` | string | `build` |
| `flags` | []string (template) | - |
| `env` | []string | os.Environ() + env config |
| `hooks.pre/post` | []object | - |

### Poetry Builder Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project dir name |
| `builder` | string | `poetry` |
| `dir` | string | `.` |
| `buildmode` | string | `wheel` (also: `sdist`) |
| `tool` | string | `poetry` |
| `command` | string | `build` |
| `flags` | []string | - |
| `env` | []string | inherited |
| `hooks.pre/post` | []object | - |

### Build Hooks (all builders)
Hook object fields:
| Field | Type | Default |
|-------|------|---------|
| `cmd` | string (template) | required |
| `dir` | string | - |
| `output` | bool | `false` |
| `env` | map[string]string | - |

Hook template variables: `.Name`, `.Ext`, `.Path`, `.Target`
Env precedence: global env > build env > hook env

### Verifiable Builds (gomod)
| Field | Type | Default |
|-------|------|---------|
| `gomod.proxy` | bool | - |
| `gomod.env` | []string | merged env |
| `gomod.mod` | string | - |
| `gomod.gobinary` | string | `go` |
| `gomod.dir` | string | `''` |

### macOS Universal Binaries
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project name |
| `ids` | []string | - |
| `name_template` | string (template) | `{{ .ProjectName }}` |
| `replace` | bool | - |
| `mod_timestamp` | string | - |
| `hooks.pre/post` | []object | - |

### UPX Compression
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | - |
| `ids` | []string | - |
| `goos` | []string | - |
| `goarch` | []string | - |
| `goarm` | []string | - |
| `goamd64` | []string | - |
| `compress` | string | `1`-`9` or `best` |
| `lzma` | bool | - |
| `brute` | bool | - |

---

## 2. Archives

### Core Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | `default` |
| `ids` | []string | empty (v2.8+) |
| `format` | string | `tar.gz` (deprecated) |
| `formats` | []string | `[tar.gz]` (v2.6+, replaces `format`) |
| `meta` | bool | `false` |
| `name_template` | string (template) | varies |
| `wrap_in_directory` | bool/string | `false` |
| `strip_binary_directory` | bool | `false` |
| `allow_different_binary_count` | bool | `false` |

### File Inclusion
| Field | Type | Default |
|-------|------|---------|
| `files` | []string/object | `[LICENSE*, README*, CHANGELOG, ...]` |
| `templated_files` | []object | empty (Pro) |

File object properties:
- `src` (string): source glob
- `dst` (string): destination directory
- `strip_parent` (bool): remove parent paths
- `info.owner` (string), `info.group` (string), `info.mtime` (RFC3339Nano), `info.mode` (octal)

### Binary Metadata (builds_info)
| Field | Type | Default |
|-------|------|---------|
| `builds_info.group` | string | - |
| `builds_info.owner` | string | - |
| `builds_info.mode` | octal | - |
| `builds_info.mtime` | RFC3339Nano | copied from source |

### Format Overrides
| Field | Type |
|-------|------|
| `format_overrides[].goos` | string |
| `format_overrides[].formats` | []string |

### Archive Hooks (Pro)
| Field | Type |
|-------|------|
| `hooks.before[]` | []object (cmd, output, dir, env) |
| `hooks.after[]` | []object (cmd, output, dir, env) |

### Supported Formats
`tar.gz`, `tgz`, `tar.xz`, `txz`, `tar.zst`, `tzst` (v2.1+), `tar`, `gz`, `zip`, `binary`, `none`

---

## 3. Checksums

| Field | Type | Default |
|-------|------|---------|
| `name_template` | string (template) | `{{ .ProjectName }}_{{ .Version }}_checksums.txt` (normal) or `{{ .ArtifactName }}.{{ .Algorithm }}` (split) |
| `algorithm` | string | `sha256` |
| `split` | bool | `false` |
| `disable` | bool (template) | `false` |
| `ids` | []string | empty (all published) |
| `extra_files` | []object | empty |
| `templated_extra_files` | []object | empty (Pro) |

Extra files object: `glob`, `name_template`

### Supported Algorithms
`sha256`, `sha512`, `sha1`, `crc32`, `md5`, `sha224`, `sha384`, `sha3-256`, `sha3-512`, `sha3-224`, `sha3-384`, `blake2s`, `blake2b`, `blake3`

---

## 4. Release (GitHub/GitLab/Gitea)

### GitHub-Specific
| Field | Type | Default |
|-------|------|---------|
| `github.owner` | string | - |
| `github.name` | string | - |
| `ids` | []string | empty (all) |
| `draft` | bool | `false` |
| `replace_existing_draft` | bool | - |
| `use_existing_draft` | bool | - (v2.5+) |
| `replace_existing_artifacts` | bool | - |
| `target_commitish` | string | - |
| `tag` | string (template) | - (Pro) |
| `discussion_category_name` | string | - |
| `prerelease` | string/bool | `auto`/`true`/`false` |
| `make_latest` | bool | - (v2.6+) |
| `mode` | string | `keep-existing`/`append`/`prepend`/`replace` |
| `header` | string (template) | - |
| `header.from_url` | object | - (Pro) |
| `header.from_file` | object | - (Pro) |
| `footer` | string (template) | - |
| `footer.from_url` | object | - (Pro) |
| `footer.from_file` | object | - (Pro) |
| `name_template` | string (template) | `{{.Tag}}` |
| `disable` | bool | `false` |
| `skip_upload` | bool | `false` |
| `extra_files` | []object | - |
| `templated_extra_files` | []object | - (Pro) |
| `include_meta` | bool | - |

### GitLab-Specific
| Field | Type |
|-------|------|
| `gitlab.owner` | string |
| `gitlab.name` | string (or project ID) |
| `ids` | []string |
| `name_template` | string (template) |
| `disable` | bool |
| `mode` | string |
| `extra_files` | []object |

### Gitea-Specific
| Field | Type |
|-------|------|
| `gitea.owner` | string |
| `gitea.name` | string |
| `ids` | []string |
| `name_template` | string (template) |
| `disable` | bool |
| `mode` | string |
| `extra_files` | []object |

### GitHub Enterprise URLs
| Field | Type |
|-------|------|
| `github_urls.api` | string |
| `github_urls.upload` | string |
| `github_urls.download` | string |
| `github_urls.skip_tls_verify` | bool |

---

## 5. Changelog

| Field | Type | Default |
|-------|------|---------|
| `disable` | string (template) | `false` |
| `use` | string | `git` (`github`, `gitlab`, `gitea`, `github-native`) |
| `format` | string (template) | provider-dependent |
| `sort` | string | empty (`asc`, `desc`) |
| `abbrev` | int | 0 (0=default, -1=remove, N=length) |
| `paths` | []string | monorepo.dir (Pro, git only) |
| `title` | string (template) | `Changelog` (Pro, v2.12+) |
| `divider` | string | none (Pro) |
| `filters.exclude` | []string | regex patterns |
| `filters.include` | []string | regex patterns |
| `groups[].title` | string | - |
| `groups[].regexp` | string | RE2 regex |
| `groups[].order` | int | - |
| `groups[].groups[]` | []object | nested subgroups (Pro, single level) |

### AI Enhancement (Pro)
| Field | Type | Default |
|-------|------|---------|
| `ai.use` | string | - (`anthropic`, `openai`, `ollama`) |
| `ai.model` | string | provider default |
| `ai.prompt` | string/object | default |
| `ai.prompt.from_url` | object | - |
| `ai.prompt.from_file` | object | - |

### Format Template Variables
`SHA`, `Message`, `Authors`, `Logins`

---

## 6. Signing

### signs[] (archive/checksum signing)
| Field | Type | Default |
|-------|------|---------|
| `id` | string | `default` |
| `cmd` | string | `gpg` |
| `signature` | string (template) | `${artifact}.sig` |
| `args` | []string (template) | `[--output, ${signature}, --detach-sign, ${artifact}]` |
| `artifacts` | enum | `none` -- values: `none`, `all`, `checksum`, `source`, `package`, `installer`, `diskimage`, `archive`, `sbom`, `binary` |
| `ids` | []string | - |
| `if` | string (template) | - |
| `stdin` | string (template) | - |
| `stdin_file` | string | - |
| `certificate` | string (template) | - |
| `env` | []string | - |
| `output` | bool | `false` (v2.13+) |

Template variables: `${artifact}`, `${artifactID}`, `${certificate}`, `${signature}`

### docker_signs[] (same fields apply)
### binary_signs[] (same fields apply, deprecated since v2.12)

---

## 7. Docker

### Docker Images (deprecated since v2.12)
| Field | Type | Default |
|-------|------|---------|
| `id` | string | - |
| `image_templates` | []string (template) | - |
| `goos` | string | `linux` |
| `goarch` | string | `amd64` |
| `goarm` | string | `6` |
| `goamd64` | string | `v1` |
| `ids` | []string | - |
| `dockerfile` | string | `Dockerfile` |
| `templated_dockerfile` | string | - (Pro) |
| `extra_files` | []string | - |
| `templated_extra_files` | []object | - (Pro, src/dst/mode) |
| `use` | string | `docker` (`buildx`, `podman`) |
| `build_flag_templates` | []string (template) | - |
| `skip_build` | bool | `false` (Pro) |
| `skip_push` | bool/`auto` | `false` |
| `push_flags` | []string | - |
| `retry.attempts` | int | `10` |
| `retry.delay` | duration | `10s` |
| `retry.max_delay` | duration | `5m` |

### Docker Images v2 (replacement)
| Field | Type | Default |
|-------|------|---------|
| `id` | string | project name |
| `dockerfile` | string | `Dockerfile` |
| `ids` | []string | - |
| `images` | []string | - |
| `tags` | []string (template) | - |
| `extra_files` | []string | - |
| `labels` | map[string]string | - |
| `annotations` | map[string]string | - |
| `platforms` | []string | `[linux/amd64, linux/arm64]` |
| `disable` | string (template) | - |
| `sbom` | bool | `true` |
| `build_args` | map[string]string | - |
| `flags` | []string | - |
| `skip_push` | bool | - |
| `push_flags` | []string | - |
| `retry.attempts` | int | `10` |
| `retry.delay` | duration | `10s` |
| `retry.max_delay` | duration | `5m` |

### Docker Manifests
| Field | Type | Default |
|-------|------|---------|
| `id` | string | - |
| `name_template` | string (template) | - |
| `image_templates` | []string (template) | - |
| `create_flags` | []string | - |
| `push_flags` | []string | - |
| `skip_push` | bool/`auto` | `false` |
| `use` | string | `docker` (`podman`) |
| `retry.attempts` | int | `10` |
| `retry.delay` | duration | `10s` |
| `retry.max_delay` | duration | `5m` |

---

## 8. nFPM (Linux Packages)

### Core Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | `default` |
| `package_name` | string (template) | ProjectName |
| `file_name_template` | string (template) | - |
| `ids` | []string | all builds |
| `if` | string (template) | - (Pro, v2.4+) |
| `vendor` | string | - |
| `homepage` | string | inferred from metadata |
| `maintainer` | string | inferred from metadata |
| `description` | string | inferred from metadata |
| `license` | string | inferred from metadata |
| `formats` | []string | apk, deb, rpm, termux.deb, archlinux |
| `umask` | octal | `0o002` |
| `bindir` | string | `/usr/bin` |
| `libdirs.header` | string | `/usr/include` |
| `libdirs.cshared` | string | `/usr/lib` |
| `libdirs.carchive` | string | `/usr/lib` |
| `epoch` | int | auto from semver |
| `prerelease` | string | auto from semver |
| `version_metadata` | string | auto from semver |
| `release` | string | - |
| `section` | string | - |
| `priority` | string | - |
| `meta` | bool | `false` |
| `changelog` | string | - (YAML path) |
| `goamd64` | string | - (v2.14+) |
| `mtime` | string (template) | - (v2.6+) |

### Dependencies
| Field | Type |
|-------|------|
| `dependencies` | []string |
| `provides` | []string |
| `recommends` | []string |
| `suggests` | []string |
| `conflicts` | []string |
| `replaces` | []string |

### Contents
| Field | Type |
|-------|------|
| `contents[].src` | string |
| `contents[].dst` | string |
| `contents[].type` | string (config, config\|noreplace, symlink, tree, ghost, dir) |
| `contents[].file_info.mode` | octal |
| `contents[].file_info.mtime` | string (template, v2.6+) |
| `contents[].file_info.owner` | string (template, v2.6+) |
| `contents[].file_info.group` | string (template, v2.6+) |
| `templated_contents` | []object (Pro) |

### Scripts
| Field | Type |
|-------|------|
| `scripts.preinstall` | string |
| `scripts.postinstall` | string |
| `scripts.preremove` | string |
| `scripts.postremove` | string |
| `templated_scripts` | object (Pro) |

### RPM-Specific
| Field | Type | Default |
|-------|------|---------|
| `rpm.summary` | string | first line of description |
| `rpm.group` | string | - (deprecated, CentOS 5) |
| `rpm.packager` | string | maintainer fallback |
| `rpm.buildhost` | string | os.Hostname() (v2.10+) |
| `rpm.compression` | string | `gzip` (also: `lzma`, `xz`) |
| `rpm.prefixes` | []string | - |
| `rpm.scripts.pretrans` | string | - |
| `rpm.scripts.posttrans` | string | - |
| `rpm.signature.key_file` | string (template) | - |

### Deb-Specific
| Field | Type | Default |
|-------|------|---------|
| `deb.lintian_overrides` | []string | - |
| `deb.scripts.rules` | string | - |
| `deb.scripts.templates` | string | - |
| `deb.triggers.interest` | []string | - |
| `deb.triggers.interest_await` | []string | - |
| `deb.triggers.interest_noawait` | []string | - |
| `deb.triggers.activate` | []string | - |
| `deb.triggers.activate_await` | []string | - |
| `deb.triggers.activate_noawait` | []string | - |
| `deb.breaks` | []string | - |
| `deb.compression` | string | `gzip` (also: `xz`, `zstd`, `none`) |
| `deb.signature.key_file` | string (template) | - |
| `deb.signature.type` | string | `origin` (also: `maint`, `archive`) |
| `deb.fields` | map[string]string | - |
| `deb.predepends` | []string | - |

### APK-Specific
| Field | Type |
|-------|------|
| `apk.scripts.preupgrade` | string |
| `apk.scripts.postupgrade` | string |
| `apk.signature.key_file` | string (template) |
| `apk.signature.key_name` | string (template, default: maintainer email) |

### Archlinux-Specific
| Field | Type |
|-------|------|
| `archlinux.scripts.preupgrade` | string |
| `archlinux.scripts.postupgrade` | string |
| `archlinux.pkgbase` | string |
| `archlinux.packager` | string |

### IPK-Specific (v2.1+)
| Field | Type |
|-------|------|
| `ipk.abi_version` | string |
| `ipk.alternatives[].priority` | int |
| `ipk.alternatives[].target` | string |
| `ipk.alternatives[].link_name` | string |
| `ipk.auto_install` | bool |
| `ipk.essential` | bool |
| `ipk.fields` | map[string]string |
| `ipk.predepends` | []string |
| `ipk.tags` | []string |

### Overrides
`overrides` section allows format-specific overrides for `deb`, `rpm`, `apk` with the same field structure.

### Signing Passphrase Env Vars
Priority order: `NFPM_[ID]_[FORMAT]_PASSPHRASE` > `NFPM_[ID]_PASSPHRASE` > `NFPM_PASSPHRASE`

---

## 9. Publish - Homebrew

### Homebrew Formulas (deprecated since v2.10, section: `brews` or `homebrew_formulas`)
| Field | Type | Default |
|-------|------|---------|
| `name` | string | project name |
| `alternative_names` | []string | - (Pro) |
| `ids` | []string | all |
| `goarm` | string | `6` |
| `goamd64` | string | `v1` |
| `app` | string | - (Pro, DMG app) |
| `url_template` | string (template) | client-dependent |
| `url_headers` | []string | - |
| `download_strategy` | string | - |
| `custom_require` | string | - |
| `custom_block` | string | - |
| `homepage` | string | inferred |
| `description` | string | inferred |
| `license` | string | inferred |
| `caveats` | string | - |
| `install` | string | `bin.install` default |
| `extra_install` | string | - |
| `post_install` | string | - |
| `test` | string | - |
| `dependencies[].name` | string | - |
| `dependencies[].os` | string | `mac`/`linux` |
| `dependencies[].type` | string | `optional` |
| `dependencies[].version` | string | - |
| `conflicts` | []string | - |
| `plist` | string | - (deprecated by Homebrew) |
| `service` | string | - |
| `commit_msg_template` | string (template) | - |
| `directory` | string | `Formula` |
| `skip_upload` | bool/`auto` | `false` |
| `repository.owner` | string | - |
| `repository.name` | string | - |
| `repository.branch` | string | default branch |
| `repository.token` | string | - |
| `repository.token_type` | string | - (Pro: github/gitlab/gitea) |
| `repository.pull_request.enabled` | bool | `false` |
| `repository.pull_request.draft` | bool | `false` |
| `repository.pull_request.check_boxes` | bool | `false` (Pro) |
| `repository.pull_request.body` | string | - (v2.12+) |
| `repository.pull_request.base.owner` | string | - |
| `repository.pull_request.base.name` | string | - |
| `repository.pull_request.base.branch` | string | - |
| `repository.git.url` | string | - |
| `repository.git.private_key` | string | - |
| `repository.git.ssh_command` | string | - |
| `commit_author.name` | string | inferred |
| `commit_author.email` | string | inferred |
| `commit_author.signing.enabled` | bool | `false` (v2.11+) |
| `commit_author.signing.key` | string | - |
| `commit_author.signing.program` | string | `gpg` |
| `commit_author.signing.format` | string | `openpgp` (also: `x509`, `ssh`) |

### Homebrew Casks (new, section: `homebrew_casks`)
| Field | Type | Default |
|-------|------|---------|
| `name` | string | project name |
| `alternative_names` | []string | - (Pro) |
| `ids` | []string | all |
| `binaries` | []string | cask name |
| `app` | string | - (Pro, DMG app) |
| `manpages` | []string | - |
| `completions.bash` | string | - |
| `completions.zsh` | string | - |
| `completions.fish` | string | - |
| `generate_completions_from_executable.executable` | string | first binary |
| `generate_completions_from_executable.args` | []string | - |
| `generate_completions_from_executable.base_name` | string | binary name |
| `generate_completions_from_executable.shell_parameter_format` | string | - |
| `generate_completions_from_executable.shells` | []string | `[bash, zsh, fish]` |
| `url.template` | string | SCM-dependent |
| `url.verified` | string | - |
| `url.using` | string | - |
| `url.cookies` | map | - |
| `url.referer` | string | - |
| `url.headers` | []string | - |
| `url.user_agent` | string | - |
| `url.data` | map | - |
| `commit_msg_template` | string (template) | - |
| `directory` | string | `Casks` |
| `caveats` | string | - |
| `homepage` | string | inferred |
| `description` | string | inferred |
| `skip_upload` | bool/`auto` | `false` |
| `custom_block` | string | - |
| `dependencies` | []object | - |
| `conflicts` | []object | - |
| `hooks` | object | - (v2.13+, pre/post install/uninstall) |
| `service` | string | - |
| `zap` | object | - |
| `uninstall` | object | - |
| `repository.*` | (same as formulas) | - |
| `commit_author.*` | (same as formulas) | - |

---

## 10. Publish - Scoop

| Field | Type | Default |
|-------|------|---------|
| `name` | string | project name |
| `url_template` | string (template) | - |
| `directory` | string | - |
| `use` | string | `archive` (also: `msi`, `nsis`) |
| `commit_msg_template` | string (template) | - |
| `homepage` | string | - |
| `description` | string | - |
| `license` | string | - |
| `skip_upload` | bool/`auto` | `false` |
| `persist` | []string | - |
| `pre_install` | []string | - |
| `post_install` | []string | - |
| `depends` | []string | - |
| `shortcuts` | [][]string | - |
| `goamd64` | string | `v1` |
| `repository.owner` | string | - |
| `repository.name` | string | - |
| `repository.branch` | string | default branch |
| `repository.token` | string | - |
| `repository.token_type` | string | - |
| `repository.pull_request.*` | (same structure as Homebrew) | - |
| `repository.git.*` | (same structure as Homebrew) | - |
| `commit_author.*` | (same structure as Homebrew) | - |

---

## 11. Publish - Chocolatey

| Field | Type | Default |
|-------|------|---------|
| `name` | string | project name |
| `ids` | []string | all |
| `package_source_url` | string | - |
| `owners` | string | - |
| `title` | string | project name |
| `authors` | string | - |
| `project_url` | string | required |
| `use` | string | `archive` (also: `msi`, `nsis`) |
| `url_template` | string (template) | - |
| `icon_url` | string | - |
| `copyright` | string (template) | - |
| `license_url` | string | - |
| `require_license_acceptance` | bool | `false` |
| `project_source_url` | string | - |
| `docs_url` | string | - |
| `bug_tracker_url` | string | - |
| `tags` | string | - |
| `summary` | string | - |
| `description` | string | - |
| `release_notes` | string | - |
| `dependencies` | []object | - |
| `api_key` | string | - |
| `source_repo` | string | - |
| `skip_publish` | bool | `false` |
| `goamd64` | string | `v1` |

---

## 12. Publish - Winget

| Field | Type | Default |
|-------|------|---------|
| `name` | string | project name |
| `publisher` | string | required |
| `short_description` | string | inferred |
| `license` | string | inferred |
| `publisher_url` | string | - |
| `publisher_support_url` | string | - |
| `privacy_url` | string | - |
| `package_identifier` | string | project name |
| `package_name` | string | name value |
| `ids` | []string | all |
| `use` | string | `''` (archives/binaries) |
| `goamd64` | string | `v1` |
| `product_code` | string | - |
| `url_template` | string (template) | - |
| `commit_msg_template` | string (template) | - |
| `path` | string | `manifests/<publisher>/<name>/<version>` |
| `homepage` | string | inferred |
| `description` | string | inferred |
| `license_url` | string | - |
| `copyright` | string | - |
| `copyright_url` | string | - |
| `skip_upload` | bool | `false` |
| `release_notes` | string | - |
| `release_notes_url` | string | - |
| `installation_notes` | string | - |
| `tags` | []string | - |
| `dependencies[].package_identifier` | string | - |
| `dependencies[].minimum_version` | string | - |
| `repository.*` | (same structure as Homebrew) | - |
| `commit_author.*` | (same structure as Homebrew) | - |

---

## 13. Publish - AUR

| Field | Type | Default |
|-------|------|---------|
| `name` | string | ProjectName-bin |
| `ids` | []string | empty |
| `homepage` | string | inferred |
| `description` | string | inferred |
| `maintainers` | []string | inferred |
| `contributors` | []string | - |
| `license` | string | inferred |
| `private_key` | string | - |
| `git_url` | string | - |
| `skip_upload` | bool/string | `false` |
| `provides` | []string | project name |
| `conflicts` | []string | project name |
| `depends` | []string | - |
| `optdepends` | map[string]string | - |
| `backup` | []string | - |
| `package` | string | default install to /usr/bin/ |
| `install` | string | - |
| `commit_msg_template` | string | `Update to {{ .Tag }}` |
| `goamd64` | string | `v1` |
| `git_ssh_command` | string | `ssh -i {{ .KeyPath }} -o StrictHostKeyChecking=accept-new -F /dev/null` |
| `url_template` | string | client-dependent |
| `directory` | string | `.` |
| `disable` | bool/string | `false` |
| `commit_author.*` | (same structure) | - |

---

## 14. Publish - Krew

| Field | Type | Default |
|-------|------|---------|
| `name` | string | project name |
| `ids` | []string | - |
| `goarm` | string | `6` |
| `goamd64` | string | `v3` |
| `url_template` | string | SCM defaults |
| `commit_msg_template` | string | - |
| `homepage` | string | inferred |
| `description` | string | inferred |
| `short_description` | string | inferred |
| `caveats` | string | - |
| `skip_upload` | bool/string | `false` |
| `repository.*` | (same structure) | - |
| `commit_author.*` | (same structure) | - |

---

## 15. Publish - Nix

| Field | Type | Default |
|-------|------|---------|
| `name` | string | project name |
| `ids` | []string | all |
| `goamd64` | string | `v1` |
| `url_template` | string | client-dependent |
| `commit_msg_template` | string | `{{ .ProjectName }}: {{ .Tag }}` |
| `path` | string | `pkgs/<name>/default.nix` |
| `homepage` | string | inferred |
| `description` | string | inferred |
| `license` | string | inferred |
| `skip_upload` | bool/string | `false` |
| `dependencies[].name` | string | - |
| `dependencies[].os` | string | - |
| `install` | string | default mkdir/cp |
| `extra_install` | string | - |
| `post_install` | string | - |
| `formatter` | string | - |
| `repository.*` | (same structure) | - |
| `commit_author.*` | (same structure) | - |

---

## 16. Publish - Custom Publishers

| Field | Type | Default |
|-------|------|---------|
| `name` | string | required (unique) |
| `cmd` | string (template) | required |
| `dir` | string | - |
| `ids` | []string | - |
| `if` | string (template) | - |
| `checksum` | bool | - |
| `meta` | bool | - |
| `signature` | bool | - |
| `env` | []string | - |
| `disable` | string (template) | - |
| `extra_files` | []object | - |
| `templated_extra_files` | []object | - (Pro) |
| `output` | string | - (v2.11+) |

Template vars: `Version`, `Tag`, `ProjectName`, `ArtifactName`, `ArtifactPath`, `Os`, `Arch`, `Arm`, `.Env.NAME`

---

## 17. Publish - Artifactory

| Field | Type | Default |
|-------|------|---------|
| `name` | string | required |
| `target` | string (template) | required |
| `mode` | string | `archive` (also: `binary`) |
| `username` | string (template) | - |
| `password` | string (template) | - |
| `client_x509_cert` | string | - |
| `client_x509_key` | string | - |
| `trusted_certificates` | string | - |
| `ids` | []string | - |
| `exts` | []string | - |
| `matrix` | map | - (Pro) |
| `custom_artifact_name` | bool | `false` |
| `custom_headers` | map (template) | - |
| `checksum` | bool | - |
| `meta` | bool | - |
| `signature` | bool | - |
| `skip` | string (template) | - |
| `extra_files` | []object | - |
| `templated_extra_files` | []object | - (Pro) |
| `extra_files_only` | bool | `false` (v2.1+) |

---

## 18. Publish - Fury.io (Pro)

| Field | Type | Default |
|-------|------|---------|
| `account` | string (template) | required |
| `disable` | string (template) | `false` |
| `secret_name` | string | `FURY_TOKEN` |
| `ids` | []string | all |
| `formats` | []string | `[apk, deb, rpm]` |

---

## 19. Publish - CloudSmith (Pro)

| Field | Type | Default |
|-------|------|---------|
| `organization` | string (template) | required |
| `repository` | string (template) | required |
| `skip` | string (template) | - |
| `secret_name` | string | `CLOUDSMITH_TOKEN` |
| `ids` | []string | - |
| `formats` | []string | `[apk, deb, rpm]` |
| `distributions` | map | - (v2.8+) |
| `component` | string | - (v2.7+) |
| `republish` | bool (template) | - (v2.11+) |

---

## 20. Publish - NPM (Pro)

| Field | Type | Default |
|-------|------|---------|
| `id` | string | project name |
| `ids` | []string | all |
| `name` | string | required |
| `description` | string (template) | - |
| `homepage` | string | - |
| `keywords` | []string | - |
| `license` | string (template) | required |
| `author` | string (template) | - |
| `repository` | string (template) | - |
| `bugs` | string (template) | - |
| `extra_files` | []object | `[README*, LICENSE*]` |
| `templated_extra_files` | []object | - |
| `access` | string | `public`/`restricted` |
| `tag` | string (template) | `latest` (v2.13+) |
| `format` | string | - |
| `if` | string (template) | - |
| `disable` | string (template) | - |
| `url_template` | string | - (v2.10+) |
| `extra` | map | - (v2.13+) |

---

## 21. Announce Providers

### Discord
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | `true` |
| `message_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out! Check it out at {{ .ReleaseURL }}` |
| `author` | string | `GoReleaser` |
| `color` | string (decimal) | `3888754` |
| `icon_url` | string | `https://goreleaser.com/static/avatar.png` |
Env: `DISCORD_WEBHOOK_ID`, `DISCORD_WEBHOOK_TOKEN`

### Slack
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool | - |
| `message_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out! Check it out at {{ .ReleaseURL }}` |
| `channel` | string | - |
| `username` | string | `""` |
| `icon_emoji` | string | `""` |
| `icon_url` | string | `""` |
| `blocks` | []object (template) | `[]` |
| `attachments` | []object (template) | `[]` |
Env: `SLACK_WEBHOOK`

### Telegram
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | - |
| `chat_id` | string (template) | - |
| `message_thread_id` | int | - (v2.15) |
| `message_template` | string (template) | MarkdownV2 escaped default |
| `parse_mode` | string | `MarkdownV2` (also: `HTML`) |
Env: `TELEGRAM_TOKEN`

### Teams
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | - |
| `title_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out!` |
| `message_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out! Check it out at {{ .ReleaseURL }}` |
| `color` | string (hex) | `#2D313E` |
| `icon_url` | string | `https://goreleaser.com/static/avatar.png` |
Env: `TEAMS_WEBHOOK`

### Mattermost
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | `true` |
| `title_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out!` |
| `message_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out! Check it out at {{ .ReleaseURL }}` |
| `color` | string (hex) | `#2D313E` |
| `channel` | string | required |
| `username` | string | - |
| `icon_emoji` | string | - |
| `icon_url` | string | - |
Env: `MATTERMOST_WEBHOOK`

### Webhook
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | - |
| `skip_tls_verify` | bool | - |
| `message_template` | string (template) | default |
| `content_type` | string | `application/json; charset=utf-8` |
| `endpoint_url` | string | - |
| `headers` | map[string]string | - |
| `expected_status_codes` | []int | `[200, 201, 202, 204]` |
Env: `BASIC_AUTH_HEADER_VALUE`, `BEARER_TOKEN_HEADER_VALUE`

### SMTP / Email
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | - |
| `host` | string | `$SMTP_HOST` |
| `port` | int | `$SMTP_PORT` |
| `from` | string | required |
| `to` | []string | required |
| `username` | string | `$SMTP_USERNAME` |
| `subject_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out!` |
| `body_template` | string (template) | `You can view details from: {{ .ReleaseURL }}` |
Env: `SMTP_PASSWORD`, `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`

### Reddit
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool | `true` |
| `application_id` | string | - |
| `username` | string | - |
| `url_template` | string (template) | `{{ .ReleaseURL }}` |
| `title_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out!` |
Env: `REDDIT_SECRET`, `REDDIT_PASSWORD`

### Twitter/X
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | - |
| `message_template` | string (template) | default |
Env: `TWITTER_CONSUMER_KEY`, `TWITTER_CONSUMER_SECRET`, `TWITTER_ACCESS_TOKEN`, `TWITTER_ACCESS_TOKEN_SECRET`

### Mastodon
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | `true` |
| `message_template` | string (template) | default |
| `server` | string | `https://mastodon.social` |
Env: `MASTODON_CLIENT_ID`, `MASTODON_CLIENT_SECRET`, `MASTODON_ACCESS_TOKEN`

### Bluesky
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool | `true` |
| `message_template` | string (template) | default |
| `username` | string | required |
Env: `BLUESKY_APP_PASSWORD`

### LinkedIn
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | `true` |
| `message_template` | string (template) | default |
Env: `LINKEDIN_ACCESS_TOKEN`

### OpenCollective
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool | `true` |
| `slug` | string | required |
| `title_template` | string (template) | `{{ .Tag }}` |
| `message_template` | string (template) | HTML default |
Env: `OPENCOLLECTIVE_TOKEN`

### Discourse
| Field | Type | Default |
|-------|------|---------|
| `enabled` | bool (template) | - |
| `server` | string | required (FQDN, no trailing slash) |
| `title_template` | string (template) | `{{ .ProjectName }} {{ .Tag }} is out!` |
| `message_template` | string (template) | default |
| `username` | string | `system` |
| `category_id` | int | required |
Env: `DISCOURSE_API_KEY`

---

## 22. Templates

### Common Variables
| Variable | Description |
|----------|-------------|
| `.ProjectName` | project identifier |
| `.Version` | release version (v stripped) |
| `.Branch` | current git branch |
| `.Tag` | current git tag |
| `.PreviousTag` | prior git tag |
| `.ShortCommit` | abbreviated commit hash |
| `.FullCommit` | complete commit hash |
| `.Commit` | commit hash (deprecated) |
| `.CommitDate` | UTC commit date (RFC 3339) |
| `.CommitTimestamp` | UTC commit date (Unix) |
| `.GitURL` | remote repository URL |
| `.GitTreeState` | `clean` or `dirty` |
| `.IsGitClean` / `.IsGitDirty` | booleans |
| `.Major`, `.Minor`, `.Patch` | SemVer components |
| `.Prerelease` | prerelease identifier |
| `.RawVersion` | `{Major}.{Minor}.{Patch}` |
| `.ReleaseNotes` | generated notes |
| `.IsDraft`, `.IsSnapshot`, `.IsNightly`, `.IsSingleTarget` | state flags |
| `.Env` | environment variables map |
| `.Date` | current UTC date (RFC 3339) |
| `.Now` | current `time.Time` struct |
| `.Timestamp` | current Unix timestamp |
| `.ModulePath` | Go module path |
| `.ReleaseURL` | download URL |
| `.Summary` | git summary |
| `.TagSubject` | annotated tag subject |
| `.TagContents` | full annotated tag message |
| `.TagBody` | tag message body |
| `.Runtime.Goos` / `.Runtime.Goarch` | runtime identifiers |
| `.Outputs` | custom outputs (v2.11+) |

### Pro-Exclusive Variables
| Variable | Description |
|----------|-------------|
| `.PrefixedTag` | tag with monorepo prefix |
| `.PrefixedPreviousTag` | previous tag with prefix |
| `.PrefixedSummary` | summary with prefix |
| `.IsRelease`, `.IsMerging` | state flags (v2.8+) |
| `.Artifacts` | current artifacts list |
| `.Metadata` | project metadata (v2.13+) |
| `.Metadata.Description` | project summary |
| `.Metadata.Homepage` | project URL |
| `.Metadata.License` | licensing info |
| `.Metadata.Maintainers` | maintainer list |
| `.Metadata.ModTimestamp` | mod timestamp template |
| `.Var.variableName` | custom variables |

### Single-Artifact Variables
`.Os`, `.Arch`, `.Arm`, `.Mips`, `.Amd64`, `.Arm64`, `.Mips64`, `.Ppc64`, `.Riscv64`, `.I386`,
`.Target`, `.Binary`, `.ArtifactID` (Pro), `.ArtifactName`, `.ArtifactPath`, `.ArtifactExt`

### nFPM-Specific Variables
`.Release`, `.Epoch`, `.PackageName`, `.ConventionalFileName`, `.ConventionalExtension`, `.Format`

### Release Body Variables
`.Checksums` -- checksum file contents or filename/content map

### Template Functions

**String:**
- `replace "text" "old" "new"` -- ReplaceAll
- `split "text" "sep"` -- split string
- `tolower "TEXT"` -- lowercase
- `toupper "text"` -- uppercase
- `trim " text "` -- trim whitespace
- `trimprefix "prefix_text" "prefix"` -- remove prefix
- `trimsuffix "text_suffix" "suffix"` -- remove suffix
- `contains "haystack" "needle"` -- substring check
- `title "text"` -- English title case

**Path:**
- `dir .Path` -- directory component
- `base .Path` -- filename component
- `abs .Path` -- absolute path

**Filtering:**
- `filter "text" "regex"` -- grep-like matching
- `reverseFilter "text" "regex"` -- inverse matching

**Version:**
- `incpatch "v1.2.4"` -- increment patch
- `incminor "v1.2.4"` -- increment minor
- `incmajor "v1.2.4"` -- increment major

**Environment:**
- `envOrDefault "VAR" "default"` -- env with fallback
- `isEnvSet "VAR"` -- existence check

**Hash (v2.9+):**
- `blake2b`, `blake2s`, `blake3` (v2.15+)
- `crc32`, `md5`, `sha1`
- `sha224`, `sha256`, `sha384`, `sha512`
- `sha3_224`, `sha3_256`, `sha3_384`, `sha3_512`

**File I/O (v2.12+):**
- `readFile "/path"` -- read or empty
- `mustReadFile "/path"` -- read or fail

**Data Structures:**
- `list "a" "b" "c"` -- create string slice
- `map "KEY" "VALUE"` -- create map from pairs
- `indexOrDefault $m "KEY" "default"` -- map lookup with fallback

**Encoding/Escaping:**
- `mdv2escape "text"` -- MarkdownV2 escaping
- `urlPathEscape "path/seg"` -- URL path encoding (v2.5+)

**Miscellaneous:**
- `time "02/01/2006"` -- formatted current UTC time
- `englishJoin` -- English conjunction joining (v2.14+)

**Pro-Exclusive Functions:**
- `in (list "a" "b") "a"` -- slice membership
- `reReplaceAll "(.*)" "foo" "bar-$1"` -- regex replace (v2.8+)

---

## 23. Global Hooks

### before / after hooks
| Field | Type | Default |
|-------|------|---------|
| `hooks[].cmd` | string (template) | required |
| `hooks[].output` | bool | `false` |
| `hooks[].dir` | string | - |
| `hooks[].env` | []string (template) | - |
| `hooks[].if` | string (template) | - (v2.7+) |

---

## 24. Blob / Cloud Storage

| Field | Type | Default |
|-------|------|---------|
| `provider` | string | `s3`, `azblob`, `gs` |
| `bucket` | string | required |
| `endpoint` | string | - (S3-compatible) |
| `region` | string | - (S3 only) |
| `disable_ssl` | bool | - (S3 only) |
| `ids` | []string | - |
| `if` | string (template) | - |
| `disable` | bool | - |
| `directory` | string (template) | `{{ .ProjectName }}/{{ .Tag }}` |
| `extra_files` | []object | - |
| `templated_extra_files` | []object | - (Pro) |
| `extra_files_only` | bool | - |
| `s3_force_path_style` | bool | `true` |
| `acl` | string | - |
| `cache_control` | string | - |
| `content_disposition` | string | `attachment;filename={{.Filename}}` |
| `include_meta` | bool | - |

---

## 25. Source Archives

| Field | Type | Default |
|-------|------|---------|
| `source.enabled` | bool | - |
| `source.name_template` | string (template) | `{{ .ProjectName }}-{{ .Version }}` |
| `source.format` | string | `tar.gz` (also: `tar`, `tgz`, `zip`) |
| `source.prefix_template` | string (template) | `{{ .ProjectName }}-{{ .Version }}/` |
| `source.files` | []object | - (src/dst/strip_parent/info) |
| `source.templated_files` | []object | - (Pro) |

---

## 26. Snapcraft

### Core Fields
| Field | Type | Default |
|-------|------|---------|
| `id` | string | `default` |
| `ids` | []string | all |
| `name_template` | string (template) | complex default |
| `name` | string | project name |
| `title` | string | - |
| `icon` | string | - |
| `publish` | bool | - |
| `summary` | string | max 79 chars |
| `description` | string | under 100 words |
| `disable` | bool | - |
| `channel_templates` | []string | varies by grade |
| `grade` | string | `stable` |
| `confinement` | string | `strict` (also: `devmode`, `classic`) |
| `license` | string | SPDX |
| `base` | string | e.g. `core18` |
| `assumes` | []string | - |
| `hooks` | map | - |
| `extra_files` | []object | src/dst/mode |
| `templated_extra_files` | []object | - (Pro) |
| `layout` | map | bind/symlink/type |
| `plugs` | map | personal-files etc. |
| `slots` | map | - |

### App Sub-Fields (per app)
`args`, `adapter`, `after`, `aliases`, `autostart`, `before`, `bus_name`, `command_chain`, `common_id`, `completer`, `command`, `daemon` (simple/forking/notify/dbus), `desktop`, `environment`, `extensions`, `install_mode` (disable/enable), `passthrough`, `plugs`, `post_stop_command`, `refresh_mode` (endure/restart), `reload_command`, `restart_condition` (always/on-failure), `slots`, `sockets`, `start_timeout`, `stop_command`, `stop_mode`, `stop_timeout`, `timer`, `watchdog_timeout`

---

## 27. DMG (macOS Disk Images) -- Pro

| Field | Type | Default |
|-------|------|---------|
| `id` | string | project name |
| `name` | string (template) | `{{.ProjectName}}_{{.Arch}}` |
| `ids` | []string | all |
| `use` | string | `binary` (also: `appbundle`) |
| `if` | string (template) | - |
| `goamd64` | string | `v1` |
| `extra_files` | []object | - |
| `templated_extra_files` | []object | - |
| `replace` | bool | `false` |
| `mod_timestamp` | string (template) | - |

---

## 28. MSI (Windows Installer) -- Pro

| Field | Type | Default |
|-------|------|---------|
| `id` | string | project name |
| `name` | string (template) | `{{.ProjectName}}_{{.MsiArch}}` |
| `wxs` | string | required |
| `ids` | []string | all |
| `goamd64` | string | `v1` |
| `extra_files` | []string | - |
| `extensions` | []string | - |
| `disable` | string (template) | - |
| `replace` | bool | `false` |
| `mod_timestamp` | string (template) | - |
| `version` | string | inferred (v3 or v4) |
| `hooks.before[]` | []object | - |
| `hooks.after[]` | []object | - |

Architectures: amd64, 386, arm64 only.

---

## 29. NSIS (Windows Installer) -- Pro

| Field | Type | Default |
|-------|------|---------|
| `id` | string | project name |
| `name` | string (template) | `{{.ProjectName}}_{{.Arch}}_setup` |
| `script` | string | required (templated) |
| `ids` | []string | all |
| `goamd64` | string | `v1` |
| `extra_files` | []object | - |
| `templated_extra_files` | []object | - |
| `disable` | string (template) | - |
| `replace` | bool | - |
| `mod_timestamp` | string (template) | - |

Template vars: `.Name`, `.Arch` (x86/x64/arm64), `.ProgramFiles`

---

## 30. PKG (macOS Packages) -- Pro (v2.14+)

| Field | Type | Default |
|-------|------|---------|
| `id` | string | project name |
| `name` | string (template) | `{{.ProjectName}}_{{.Arch}}` |
| `ids` | []string | all |
| `use` | string | `binary` (also: `appbundle`) |
| `if` | string (template) | - |
| `identifier` | string (template) | required |
| `install_location` | string (template) | `/usr/local/bin` |
| `scripts` | string (template) | - (dir with preinstall/postinstall) |
| `replace` | bool | `false` |
| `mod_timestamp` | string (template) | - |

---

## 31. App Bundles (macOS .app) -- Pro

| Field | Type | Default |
|-------|------|---------|
| `id` | string | project name |
| `name` | string (template) | `{{.ProjectName}}` |
| `ids` | []string | all |
| `if` | string (template) | - |
| `icon` | string (template) | - (must be .icns) |
| `bundle` | string (template) | - (reverse DNS) |
| `mod_timestamp` | string (template) | - |
| `extra_files` | []object | - (src/dst/info.mtime) |
| `templated_extra_files` | []object | - (src/dst/info.mtime) |

---

## 32. SBOM

| Field | Type | Default |
|-------|------|---------|
| `id` | string | `default` |
| `documents` | []string | varies by artifact type |
| `cmd` | string | `syft` |
| `args` | []string | `[$artifact, --output, spdx-json=$document, --enrich, all]` |
| `env` | []string | `[SYFT_FILE_METADATA_CATALOGER_ENABLED=true]` |
| `artifacts` | string | `archive` (also: `any`, `source`, `package`, `installer`, `diskimage`, `binary`, `sbom`) |
| `ids` | []string | - |
| `disable` | bool (template) | `true` |

Template vars: `${artifact}`, `${artifactID}`, `${document}`, `${document#}`

---

## 33. Flatpak

| Field | Type | Default |
|-------|------|---------|
| `id` | string | `default` |
| `ids` | []string | all |
| `name_template` | string (template) | detailed default |
| `app_id` | string | required (reverse DNS) |
| `runtime` | string | required |
| `runtime_version` | string | required |
| `sdk` | string | required |
| `command` | string | first binary |
| `finish_args` | []string | - (sandbox permissions) |
| `disable` | string (template) | - |

---

## 34. Ko (Container Image Builder)

| Field | Type | Default |
|-------|------|---------|
| `id` | string | - |
| `build` | string | - (build ID to import from) |
| `main` | string | `build.main` |
| `working_dir` | string | `build.dir` |
| `base_image` | string | `cgr.dev/chainguard/static` |
| `labels` | map | - |
| `annotations` | map | - |
| `user` | string | - |
| `repositories` | []string | `[$KO_DOCKER_REPO]` |
| `repository` | string | `$KO_DOCKER_REPO` (deprecated) |
| `platforms` | []string | `[linux/amd64]` |
| `tags` | []string (template) | `[latest]` |
| `creation_time` | string (template) | - |
| `ko_data_creation_time` | string (template) | - |
| `sbom` | string | `spdx` (also: `none`) |
| `sbom_directory` | string | - |
| `local_domain` | string | `goreleaser.ko.local` |
| `ldflags` | []string | `build.ldflags` |
| `flags` | []string | `build.flags` |
| `env` | []string | `build.env` |
| `disable` | string (template) | - |
| `bare` | bool | - |
| `preserve_import_paths` | bool | - |
| `base_import_paths` | bool | - |

---

## 35. Notarize (macOS)

### Cross-Platform (anchore/quill)
| Field | Type | Default |
|-------|------|---------|
| `notarize.macos[].enabled` | bool | `false` |
| `notarize.macos[].ids` | []string | project name |
| `notarize.macos[].sign.certificate` | string | - (P12 path or base64) |
| `notarize.macos[].sign.password` | string | - |
| `notarize.macos[].sign.entitlements` | string | - |
| `notarize.macos[].notarize.issuer_id` | string | - (App Store Connect UUID) |
| `notarize.macos[].notarize.key_id` | string | - |
| `notarize.macos[].notarize.key` | string | - (P8 path or base64) |
| `notarize.macos[].notarize.wait` | bool | `false` |
| `notarize.macos[].notarize.timeout` | duration | `10m` |

### Native (codesign/xcrun) -- Pro
| Field | Type | Default |
|-------|------|---------|
| `notarize.macos_native[].enabled` | bool | `false` |
| `notarize.macos_native[].ids` | []string | project name |
| `notarize.macos_native[].use` | string | `dmg` (also: `pkg`) |
| `notarize.macos_native[].sign.keychain` | string | - |
| `notarize.macos_native[].sign.identity` | string | - |
| `notarize.macos_native[].sign.options` | []string | - |
| `notarize.macos_native[].sign.entitlements` | string | - |
| `notarize.macos_native[].notarize.profile_name` | string | - |
| `notarize.macos_native[].notarize.wait` | bool | `false` |

---

## 36. DockerHub Description Sync -- Pro

| Field | Type | Default |
|-------|------|---------|
| `username` | string | `{{ .Env.DOCKER_USERNAME }}` |
| `secret_name` | string | `DOCKER_PASSWORD` |
| `images` | []string | - |
| `disable` | string (template) | - |
| `description` | string | global metadata |
| `full_description.from_url` | string | - |
| `full_description.from_file` | string | - |

---

## 37. Split & Merge -- Pro

### Configuration
| Field | Type | Default |
|-------|------|---------|
| `partial.by` | string | `goos` (also: `target`) |

### Commands
- `goreleaser release --clean --split` (with `GOOS`/`GOARCH` or `GGOOS`/`GGOARCH` env vars)
- `goreleaser continue --merge`
- `goreleaser publish --merge`
- `goreleaser announce --merge`

`GOOS`/`GOARCH`: affects build targets AND hook execution
`GGOOS`/`GGOARCH`: filters targets only, does not affect hooks

---

## 38. Project Configuration

### General
| Field | Type | Default |
|-------|------|---------|
| `project_name` | string | inferred from SCM |
| `dist` | string | `./dist` |

### Git Configuration
| Field | Type | Default |
|-------|------|---------|
| `git.tag_sort` | string | `-version:refname` (also: `semver`, `smartsemver` Pro) |
| `git.prerelease_suffix` | string | - |
| `git.ignore_tags` | []string (template) | - |
| `git.ignore_tag_prefixes` | []string (template) | - (Pro) |

### Monorepo -- Pro
| Field | Type | Default |
|-------|------|---------|
| `monorepo.tag_prefix` | string | - |
| `monorepo.dir` | string | - |

### Includes -- Pro
| Field | Type |
|-------|------|
| `includes[].from_file.path` | string |
| `includes[].from_url.url` | string |
| `includes[].from_url.headers` | map[string]string |

### Metadata -- Pro
| Field | Type | Default |
|-------|------|---------|
| `metadata.mod_timestamp` | string (template) | - |
| `metadata.maintainers` | []object | - (v2.1+) |
| `metadata.license` | string (template) | - (v2.1+) |
| `metadata.homepage` | string (template) | - (v2.1+) |
| `metadata.description` | string (template) | - (v2.1+) |
| `metadata.full_description.from_url` | string | - |
| `metadata.full_description.from_file` | string | - |
| `metadata.commit_author.*` | (same structure) | - (v2.12+) |

### Environment
| Field | Type |
|-------|------|
| `env` | []string (template) |
| `env_files.github_token` | string (default: `~/.config/goreleaser/github_token`) |
| `env_files.gitlab_token` | string (default: `~/.config/goreleaser/gitlab_token`) |
| `env_files.gitea_token` | string (default: `~/.config/goreleaser/gitea_token`) |

### Template Files -- Pro
| Field | Type | Default |
|-------|------|---------|
| `template_files[].id` | string | `default` |
| `template_files[].src` | string (template) | - |
| `template_files[].dst` | string (template) | - |
| `template_files[].mode` | octal | `0655` |

---

## 39. Environment Variables

### Authentication Tokens
| Variable | Purpose |
|----------|---------|
| `GITHUB_TOKEN` | GitHub API auth (repo scope) |
| `GITLAB_TOKEN` | GitLab API auth (api scope) |
| `GITEA_TOKEN` | Gitea API auth |
| `GORELEASER_FORCE_TOKEN` | Force specific token when multiple set |

### Announce Provider Tokens
| Variable | Provider |
|----------|----------|
| `DISCORD_WEBHOOK_ID` | Discord |
| `DISCORD_WEBHOOK_TOKEN` | Discord |
| `SLACK_WEBHOOK` | Slack |
| `TELEGRAM_TOKEN` | Telegram |
| `TEAMS_WEBHOOK` | Teams |
| `MATTERMOST_WEBHOOK` | Mattermost |
| `SMTP_PASSWORD`, `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME` | Email |
| `REDDIT_SECRET`, `REDDIT_PASSWORD` | Reddit |
| `TWITTER_CONSUMER_KEY`, `TWITTER_CONSUMER_SECRET`, `TWITTER_ACCESS_TOKEN`, `TWITTER_ACCESS_TOKEN_SECRET` | Twitter/X |
| `MASTODON_CLIENT_ID`, `MASTODON_CLIENT_SECRET`, `MASTODON_ACCESS_TOKEN` | Mastodon |
| `BLUESKY_APP_PASSWORD` | Bluesky |
| `LINKEDIN_ACCESS_TOKEN` | LinkedIn |
| `OPENCOLLECTIVE_TOKEN` | OpenCollective |
| `DISCOURSE_API_KEY` | Discourse |

### Publisher Tokens
| Variable | Publisher |
|----------|----------|
| `FURY_TOKEN` | Fury.io |
| `CLOUDSMITH_TOKEN` | CloudSmith |
| `DOCKER_PASSWORD` | DockerHub |
| `KO_DOCKER_REPO` | Ko |

### nFPM Signing
| Variable | Purpose |
|----------|---------|
| `NFPM_PASSPHRASE` | Generic passphrase |
| `NFPM_[ID]_PASSPHRASE` | Per-config passphrase |
| `NFPM_[ID]_[FORMAT]_PASSPHRASE` | Per-config-per-format passphrase |

---

## 40. CLI Commands

### Commands
| Command | Description |
|---------|-------------|
| `goreleaser release` | Full release pipeline |
| `goreleaser build` | Build binaries only |
| `goreleaser check` | Validate config |
| `goreleaser healthcheck` | Verify dependencies |
| `goreleaser init` | Generate example config |
| `goreleaser completion` | Generate shell completions |
| `goreleaser jsonschema` | Generate JSON schema |
| `goreleaser changelog` | Preview changelog (Pro) |
| `goreleaser continue` | Continue a previously prepared release (with `--merge`) |
| `goreleaser publish` | Publish artifacts (with `--merge`) |
| `goreleaser announce` | Announce release (with `--merge`) |
| `goreleaser man` | Generate man pages |

### Release Flags
| Flag | Type | Default |
|------|------|---------|
| `--auto-snapshot` | bool | auto-snapshot if dirty |
| `--clean` | bool | remove dist/ |
| `-f, --config` | string | config file path |
| `--draft` | bool | set release to draft |
| `--fail-fast` | bool | abort on first error |
| `--id` | []string | build IDs (Pro) |
| `-k, --key` | string | Pro license key |
| `--nightly` | bool | nightly build (Pro) |
| `-p, --parallelism` | int | concurrent tasks |
| `--prepare` | bool | prepare only, publish later (Pro) |
| `--release-notes` | string | release notes file |
| `--release-notes-tmpl` | string | notes template file |
| `--skip` | []string | skip options |
| `--snapshot` | bool | unversioned snapshot |
| `--split` | string | split by GOOS (Pro, implies --prepare) |
| `--timeout` | duration | `1h0m0s` |

### Build Flags
| Flag | Type | Default |
|------|------|---------|
| `--auto-snapshot` | bool | - |
| `--clean` | bool | - |
| `-f, --config` | string | - |
| `--id` | []string | build IDs |
| `-p, --parallelism` | int | CPU count |
| `--single-target` | bool | current GOOS/GOARCH only |
| `--skip` | []string | - |
| `--snapshot` | bool | - |
| `--timeout` | duration | `1h0m0s` |

### Check Flags
| Flag | Type |
|------|------|
| `-q, --quiet` | bool |
| `--soft` | bool (Pro, exit 1 only on syntax errors) |
| `--verbose` | bool |

---

## 41. Artifacts JSON

Generated at `dist/artifacts.json` with fields per artifact:
- `name`, `path`, `type`, `target`
- `goos`, `goarch`, `goamd64`, `goarm`, `gomips`
- `extra.ID`, `extra.Checksum`, `extra.Format`, `extra.Size`, `extra.Binaries`, `extra.Digest`

30+ artifact type classifications including: Binary, Archive, Package, Docker Image, Docker Manifest, MSI, NSIS, DMG, Snap, Flatpak, NPM, Python wheel, Homebrew formula, Checksum, Signature, SBOM, Certificate.

---

## 42. Pro-Only Features Summary

1. macOS Installers (.pkg)
2. Windows NSIS Installers (.exe)
3. Smart SemVer Tag Sorting (smartsemver)
4. NPM Registry Publishing
5. Native macOS Signing & Notarization (codesign/xcrun)
6. AI-Enhanced Release Notes (anthropic/openai/ollama)
7. Conditional Artifact Filtering (`if` statements)
8. macOS App Bundles (.app)
9. CloudSmith Repository Integration
10. Global Metadata Defaults
11. Pre-Publishing Hooks
12. Cross-Platform Publishing (e.g. GitLab + Homebrew)
13. DockerHub Description Sync
14. macOS Disk Images (.dmg)
15. Windows MSI Installers (.msi) with Wix
16. Single-Target Release Building
17. Template Files
18. Artifacts Template Variable
19. Split & Merge Builds
20. Enhanced Changelog (path filtering, subgroups, dividers)
21. Archive Hooks
22. Multi-Stage Release (prepare/publish/announce)
23. Changelog Preview command
24. Nightly Builds
25. Prebuilt Binary Import
26. Podman Support
27. GemFury Repository Integration
28. Configuration File Reuse (includes)
29. Global After Hooks
30. Monorepo Support
31. Custom Template Variables
32. Flatpak packages
33. Templated extra files, contents, scripts, dockerfiles

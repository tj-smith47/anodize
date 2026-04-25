# checksum + sign + notarize + sbom — parity audit (Pass B)

Audited: 2026-04-25
Re-graded with Go-toolchain rule applied (Goos/Goarch/Goarm/Goamd64/etc. are NOT MISSING in a Rust tool).

References: `/opt/repos/anodizer` HEAD, `/opt/repos/goreleaser` HEAD.

## Summary
- **MISSING-FIELD: 1** (sign `binary_signs` config not surfaced as a top-level YAML key)
- **Default divergences (user-impacting): 5**
- **Validation gaps: 6**
- **Code smells: 13**
- **Anodizer-only intentional fields: 11** (verified wired)

---

## checksum pipe

GoReleaser source: `pkg/config/config.go:1078` (`type Checksum`), `internal/pipe/checksums/checksums.go`.
Anodizer source: `crates/core/src/config.rs:1462` (`ChecksumConfig`), `crates/stage-checksum/src/lib.rs`.

### Real MISSING fields
None. Field map (GR → anodizer): `NameTemplate→name_template`, `Algorithm→algorithm`, `Split→split`, `IDs→ids`, `Disable→disable` (with bool|template upgrade), `ExtraFiles→extra_files`.

### Default divergences
- **`name_template` default rendering — eager vs lazy**
  - goreleaser: `internal/pipe/checksums/checksums.go:44-50` writes the default into `cs.NameTemplate` during `Default()`; the persisted config carries the rendered default.
  - anodizer: `crates/stage-checksum/src/lib.rs:482-490` defaults are computed inline at run-time (`format!("{project}_{version}_checksums.txt")`); the config field stays `None`. Runtime parity holds; YAML round-trip and `--debug-config` introspection differ.
- **artifact-source kind list narrower than GoReleaser**
  - goreleaser: `internal/pipe/checksums/checksums.go:181` filters by `artifact.ReleaseUploadableTypes()` (`internal/artifact/artifact.go:132-148`) which includes `UploadableArchive`, `UploadableBinary`, `UploadableFile`, `UploadableSourceArchive`, `Makeself`, `LinuxPackage`, `Flatpak`, `SourceRPM`, `SBOM`, `PyWheel`, `PySdist`, `Checksum` (excluded by `Not(ByType(Checksum))`), `Signature`, `Certificate`.
  - anodizer: `crates/stage-checksum/src/lib.rs:261-272` enumerates `Archive, LinuxPackage, Binary, UploadableBinary, SourceArchive, Sbom, Snap, DiskImage, Installer, MacOsPackage`. Missing from the source-list: `Makeself`, `Flatpak`, `SourceRpm`, `Signature`, `Certificate`, `UploadableFile` (extra_files surrogate exists but only via the `extra_files:` key, not registry-injected uploadable files). Anodizer-extra: `Snap`, `DiskImage`, `Installer`, `MacOsPackage` (Pro-equivalents).
- **extra-file synthetic kind label**
  - goreleaser: `internal/pipe/checksums/checksums.go:193-198` registers extra files with `Type: UploadableFile`.
  - anodizer: `crates/stage-checksum/src/lib.rs:296` registers extra files with `kind: ArtifactKind::Archive`. Misleading metadata; consumers filtering by kind will see "archive" for an arbitrary extra file.

### Validation gaps
- **algorithm not validated at load time**
  - goreleaser: invalid algorithms surface at hash time (`internal/artifact/artifact.go:366-405`) but the supported list is closed (sha1/sha224/sha256/sha384/sha512/md5/crc32/blake2b/blake2s/blake3).
  - anodizer: `crates/stage-checksum/src/lib.rs:108-125` returns `bail!("unsupported checksum algorithm: ...")` at first artifact, after the rest of the pipeline has already started. Add a `Default()`-time check; otherwise a typo in `algorithm: sha257` only fails after build/archive completes.
- **`split` + `name_template` interaction silent**
  - goreleaser: `internal/pipe/checksums/checksums.go:44-50` swaps the default to `{{ .ArtifactName }}.{{ .Algorithm }}` when split; user-provided `name_template` is honored.
  - anodizer: `crates/stage-checksum/src/lib.rs:400-419` honors user template but provides no warning when a user sets a non-`{{ .ArtifactName }}` template under split mode (every artifact would write to the same path). Validate uniqueness or warn.

### Code smells
- **`crates/stage-checksum/src/lib.rs:374` — `unwrap_or("unknown")`** for the artifact filename string lookup. A real artifact with a non-UTF8 filename silently becomes "unknown" in the checksum line; bytes-vs-string mismatch is not surfaced.
- **`crates/stage-checksum/src/lib.rs:390` — `.unwrap_or_else(|_| filename.to_string())`** swallows template-render errors on `extra_name_template`. GoReleaser propagates template errors. Use `?`.
- **`crates/stage-checksum/src/lib.rs:474-478` — inherited GoReleaser sort divergence** for filenames containing two-space sequences. Pinned by `test_combined_sort_doublespace_divergence` (line 676). Acknowledged; carrying a known bug for parity. Document in user-facing notes.
- **`crates/stage-checksum/src/lib.rs:289-303` — extra-file synthetic Artifact uses `crate_name` of the current crate**, but extra files are workspace-scoped. In multi-crate runs the same extra file is checksummed once per crate.
- **`crates/stage-checksum/src/lib.rs:443, :527` — `name: String::new()`** on registered Checksum artifacts. Other artifact registrations populate `name` with the filename; this drift breaks consumers reading `artifact.name` (e.g., publisher templates).

### Anodizer-only intentional
- `templated_extra_files` (config.rs:1475) — Pro feature replication, wired at stage-checksum/lib.rs:308-326.
- `disable: bool|template` (config.rs:1469) — bool-or-template upgrade; GoReleaser's `Disable` is plain bool.

---

## sign pipe

GoReleaser source: `pkg/config/config.go:909` (`type Sign`), `:923` (`BinarySign`), `internal/pipe/sign/sign.go`, `sign_binary.go`, `sign_docker.go`.
Anodizer source: `crates/core/src/config.rs:4435` (`SignConfig`), `:4470` (`DockerSignConfig`), `crates/stage-sign/src/lib.rs`.

### Real MISSING fields
- **`binary_signs:` key documented separately in GoReleaser; anodizer reuses `SignConfig`**
  - goreleaser: `pkg/config/config.go:923` defines `BinarySign` as a separate type (identical fields, but documented + jsonschema-enumerated for `artifacts: binary|none` only).
  - anodizer: `crates/core/src/config.rs:93` types `binary_signs: Vec<SignConfig>` (re-uses the generic SignConfig). Functional parity exists, but the JSON schema does not constrain `artifacts` to `binary|none` for `binary_signs`, so YAML completion accepts e.g. `binary_signs: [{artifacts: archive}]` which is then ignored at runtime by `BinarySignStage` (lib.rs:454-458 hard-filters to Binary). Validation gap below.

### Default divergences
- **`signs:` default `cmd` chooses gpg → cosign**
  - goreleaser: `internal/pipe/sign/sign.go:48,52-58,64-67` defaults `cfg.Cmd` to gpg (or `git config gpg.program` if set).
  - anodizer: `crates/stage-sign/src/lib.rs:140-153, :414-418` matches (gpg + git override). Parity.
- **`signs:` default `Args` for non-gpg signing**
  - goreleaser: `internal/pipe/sign/sign.go:71-73` always defaults to `["--output", "$signature", "--detach-sig", "$artifact"]` (gpg-shaped) regardless of `cmd`. Cosign users always override.
  - anodizer: `crates/stage-sign/src/lib.rs:427-434` mirrors `["--output", "{{ .Signature }}", "--detach-sign", "{{ .Artifact }}"]`. **Differs in token form** — anodizer uses Tera-ish `{{ .X }}` placeholders; GR uses `$artifact`-style shell vars resolved by `os.Expand`. Anodizer also resolves shell-style `${artifact}` later (lib.rs:602-608, 599 `expand_shell_vars`), so user configs written with GR syntax still work — but the default doc presents Tera syntax that doesn't match GoReleaser docs verbatim.
- **`signs:` default `Signature` template**
  - goreleaser: `internal/pipe/sign/sign.go:68-70` defaults to `${artifact}.sig`.
  - anodizer: `crates/stage-sign/src/lib.rs:99` accepts `default_template: None` (falls back to `{artifact}.sig` via `resolve_signature_path`). Parity. **Note**: anodizer does not persist the default into the config field (lazy default), unlike GR.
- **`docker_signs:` default `Cmd`**
  - goreleaser: `internal/pipe/sign/sign_docker.go:36-38` defaults to `cosign`.
  - anodizer: `crates/stage-sign/src/lib.rs:977-981` defaults to `cosign`. Parity.
- **`docker_signs:` invalid `artifacts` filter**
  - goreleaser: `internal/pipe/sign/sign_docker.go:78-79` returns `error "invalid list of artifacts to sign"` for unknown values.
  - anodizer: `crates/stage-sign/src/lib.rs:1028-1037` **logs a warning and falls back to `images`** instead of erroring. User typos (`artifacts: imgaes`) silently sign the wrong set.
- **`binary_signs:` default `Signature` template form**
  - goreleaser: `internal/pipe/sign/sign_binary.go:16` `${artifact}_{{ .Os }}_{{ .Arch }}{{ with .Arm }}v{{ . }}{{ end }}{{ with .Mips }}_{{ . }}{{ end }}{{ if not (eq .Amd64 "v1") }}{{ .Amd64 }}{{ end }}`.
  - anodizer: `crates/stage-sign/src/lib.rs:26` `{{ .Artifact }}_{{ Os }}_{{ Arch }}{% if Arm %}v{{ Arm }}{% endif %}{% if Mips %}_{{ Mips }}{% endif %}{% if Amd64 and Amd64 != "v1" %}{{ Amd64 }}{% endif %}`. Tera syntax is functionally equivalent; the leading variable is `{{ .Artifact }}` (Tera-style with leading dot) vs `${artifact}` (shell). Both code paths resolve to same string at sign time, but a user copy-pasting GR examples for `binary_signs.signature` won't be able to round-trip without translation.

### Validation gaps
- **`docker_signs.artifacts` not validated up-front (GR: returns error in Run; anodizer: warns + silent fallback)**. Same as default-divergence above; flagged separately because the lack of validation lets the misconfiguration ride into Publish without detection.
- **`signs.artifacts: none` warns at Run, not Default**
  - goreleaser: `internal/pipe/sign/sign.go:74-75` defaults `Artifacts` to `"none"`; with explicit `"none"` returns `pipe.ErrSkipSignEnabled` at Run (`:113-114`).
  - anodizer: `crates/stage-sign/src/lib.rs:408-411` records skip with `remember_skip(label, sub_label, "artifacts: none")`. Parity-ish but no error/skip propagation upstream — the per-sign-cfg skip is only recorded; downstream consumers reading `ctx.config.signs` see the entries as if active.
- **`binary_signs.artifacts` constraint not jsonschema-enforced**
  - goreleaser: `pkg/config/config.go:929` `jsonschema:"enum=binary,enum=none"` constrains `BinarySign.Artifacts` at config-load time.
  - anodizer: `crates/core/src/config.rs:4441` allows any string (no `schemars` enum on `SignConfig.artifacts`). YAML editors accept `binary_signs: [{artifacts: archive}]`, runtime then silently filters to Binary.
- **`signs.ids` warning when `artifacts=checksum|source`**
  - goreleaser: `internal/pipe/sign/sign.go:96-97, 100-101` logs `"when artifacts is X, ids has no effect. ignoring"`.
  - anodizer: `crates/stage-sign/src/lib.rs` no equivalent warning (search for `"ids has no effect"`); `ids` filter is applied silently regardless.
- **`signs.env` render error fallback swallows**
  - goreleaser: `internal/pipe/sign/sign.go:180-185, 310-320` propagates env-template errors via `templateEnvS`.
  - anodizer: `crates/stage-sign/src/lib.rs:712-718` `unwrap_or_else` warns and uses raw value. A typo in `env: ["KEY={{ .NonExistent }}"]` silently passes the literal template string to the signer.
- **`signs.env` is `HashMap` (anodizer) vs ordered `[]string` (GR) — non-deterministic ordering**
  - goreleaser: `pkg/config/config.go:918` `Env []string` keeps user-defined order; later entries can reference earlier ones via `WithEnvS(out)` rolling context.
  - anodizer: `crates/core/src/config.rs:4456` `Option<HashMap<String, String>>` loses insertion order; envs cannot reference previously-defined envs in template. Migration hazard for users porting `env: ["FOO=$BAR", "BAZ={{ .Env.FOO }}"]`.

### Code smells
- **`crates/stage-sign/src/lib.rs:643` — `.unwrap_or("")`** on `sig_path.file_name()` produces a Signature artifact with empty name when path has no filename component (impossible-but-handled). Bail or log.
- **`crates/stage-sign/src/lib.rs:712` — env render fallback uses raw value** without bail (see validation gap above).
- **`crates/stage-sign/src/lib.rs:140-153` — `default_sign_cmd()` shells out to git for every sign config**; should cache on `OnceLock` like GR's `sync.OnceValue` (sign.go:52-59).
- **`crates/stage-sign/src/lib.rs:725-732` — shell_vars iter promotes all non-empty shell vars into env even when user did not request them**. GR sets these only when explicitly templated. Surface area: a user's signing tool that happens to read `$signature` or `$digest` from env will see anodizer-injected values.
- **`crates/stage-sign/src/lib.rs:892-907` — the `SignStage.run` body invokes both `signs` and `binary_signs` loops**. GR splits into two distinct pipes (`Pipe` and `BinaryPipe`). Anodizer's `BinarySignStage` (lib.rs:836) duplicates the `binary_signs` execution path, so when both run in sequence, `binary_signs` could execute twice in `anodizer release` if both stages are scheduled. Verify pipeline order doesn't double-execute.
- **`crates/stage-sign/src/lib.rs:3729 lines** — single file holds SignStage, BinarySignStage, DockerSignStage, helpers, tests. Split.

### Anodizer-only intentional
- `if_condition` (`if:`) on `SignConfig` and `DockerSignConfig` (config.rs:4464, :4496) — template gate, no GR analog.
- `output: bool|template` upgrade — GR `Output string` is jsonschema `oneof_type=string;boolean`; anodizer mirrors via `StringOrBool`.
- `env: HashMap` (per validation gap, this is a divergence from GR's ordered list; documented as deliberate "schema is map for ergonomics" but the order-loss side effect is not documented).

---

## notarize pipe

GoReleaser source: `pkg/config/config.go:938` (`type Notarize`), `:942` (`MacOSSignNotarize`), `:949` (`MacOSNotarize`), `:957` (`MacOSSign`); `internal/pipe/notary/macos.go`.
Anodizer source: `crates/core/src/config.rs:3948` (`NotarizeConfig`), `crates/stage-notarize/src/lib.rs`.

### Real MISSING fields
None on the cross-platform path. anodizer's `MacOSSignNotarizeConfig` covers `IDs/Enabled/Sign(certificate, password, entitlements)/Notarize(issuer_id, key, key_id, timeout, wait)` — full parity with GoReleaser's `MacOSSignNotarize`.

GoReleaser exposes `Notarize.MacOS` only. anodizer adds:
- `disable: bool|template` (config.rs:3951) — anodizer-only top-level disable.
- `macos_native: Vec<MacOSNativeSignNotarizeConfig>` (config.rs:3955) — anodizer-only second mode (codesign + xcrun + stapler).

### Default divergences
- **`Notarize.Timeout` default 10 minutes**
  - goreleaser: `internal/pipe/notary/macos.go:32-34` `if n.Notarize.Timeout == 0 { n.Notarize.Timeout = 10 * time.Minute }`. Stored as `time.Duration`.
  - anodizer: `crates/stage-notarize/src/lib.rs:294, :511-515` `or_else(|| Some("10m".to_string()))`. Stored as `String` (config.rs:3996). Parity at runtime; YAML/round-trip differs (anodizer accepts any duration string format the consumer parses; GR enforces `time.Duration`).
- **`IDs` default to project name**
  - goreleaser: `internal/pipe/notary/macos.go:35-37` `n.IDs = []string{ctx.Config.ProjectName}`.
  - anodizer: `crates/stage-notarize/src/lib.rs:301-307` matches.
- **Timestamp URL hard-coded**
  - goreleaser: `internal/pipe/notary/macos.go:95` `WithTimestampServer("http://timestamp.apple.com/ts01")`.
  - anodizer: `crates/stage-notarize/src/lib.rs:351-352` matches. Both inflexible.

### Validation gaps
- **`MacOSSign.Certificate`/`Password` required but only validated at Run**
  - goreleaser: relies on `quill/load.P12` to error at Run (macos.go:72-75).
  - anodizer: `crates/stage-notarize/src/lib.rs:262-267` `ok_or_else` at Run with descriptive `notarize: macos[N] sign.certificate is required`. Parity, but neither validates at Default/Validate time. User finds out only after running through build+archive+sign.
- **No validation that `macos` and `macos_native` aren't both populated for same artifact set** — anodizer-specific concern: two configs both targeting the same darwin Binary would sign twice, second sign breaks the first signature. No guard in `crates/stage-notarize/src/lib.rs:200-225`.
- **`StringOrBool::is_disabled` swallows template render errors**
  - `crates/stage-notarize/src/lib.rs:206-211` calls `is_disabled(|s| ctx.render_template(s))`. Inspect the closure semantics — render failures default to "not disabled" without surfacing the error. Same pattern in checksum/sbom; cross-cutting.
- **`enabled` default**
  - goreleaser: `internal/pipe/notary/macos.go:53-58` `tmpl.Bool(cfg.Enabled)` returns `false` for empty. Disabled by default.
  - anodizer: `crates/stage-notarize/src/lib.rs:248` `is_enabled(&cfg.enabled, ctx)`. Verify default; if empty/None defaults to `false` parity holds, but anodizer-only `disable` field at top of NotarizeConfig duplicates the per-cfg `enabled` check, doubling the surface.
- **API key file existence not pre-checked**
  - goreleaser: relies on `quill.Notarize` (Go library) to fail at submission.
  - anodizer: `crates/stage-notarize/src/lib.rs:280-292` validates non-empty but does not stat the `key` file. A typo'd path produces a deep `rcodesign notary-submit` error mid-run.
- **`macos_native.use_` field not enum-validated**
  - anodizer: `crates/core/src/config.rs:4012` `pub use_: Option<String>` accepts any string. Code at runtime presumably matches `dmg` / `pkg`; jsonschema doesn't enforce.

### Code smells
- **`crates/stage-notarize/src/lib.rs:343-358` — sign args list is built imperatively** with `push` instead of a typed builder; entitlements branch mutates the vector mid-construction. Refactor to a builder.
- **`crates/stage-notarize/src/lib.rs:390-407` — only `--max-wait` is gated on `wait`**, not `--api-issuer`/`--api-key`/`--api-key-path`. Verify rcodesign accepts the wait-related flags as documented; current code mixes concerns.
- **`crates/stage-notarize/src/lib.rs:294, :511, :512-515` — `"10m"` literal duplicated** in two code paths (`run_cross_platform` and `run_native`). Lift to a `DEFAULT_NOTARIZE_TIMEOUT` const matching GR's `10 * time.Minute`.
- **`crates/stage-notarize/src/lib.rs:104` — `sensitive_flags` allow-list** is hard-coded `["--p12-password", "--api-key-path"]`. Add a `defense-in-depth` pass that also redacts known env vars (`*_PASSWORD`, `*_TOKEN`, `*_KEY`).
- **`crates/stage-notarize/src/lib.rs:309-322` — darwin filter uses `is_darwin(target)` on `Binary | UniversalBinary`** but does not re-filter `UploadableBinary`. GoReleaser filters `Binary | UniversalBinary` (macos.go:79-82). Parity, but anodizer adds `UploadableBinary` for sign — divergence between sign and notarize artifact-source sets across the same darwin binary.
- **`crates/stage-notarize/src/lib.rs:722, :886` — `xcrun stapler staple` invocations assume macOS host**; anodizer-only `macos_native` mode silently fails on non-darwin runners with a confusing error from `xcrun: command not found`. Add an explicit `cfg!(target_os = "macos")` precondition.

---

## sbom pipe

GoReleaser source: `pkg/config/config.go:895` (`type SBOM`), `internal/pipe/sbom/sbom.go`.
Anodizer source: `crates/core/src/config.rs:4192` (`SbomConfig`), `crates/stage-sbom/src/lib.rs`.

### Real MISSING fields
None. Field map: `ID→id`, `Cmd→cmd`, `Env→env`, `Args→args`, `Documents→documents`, `Artifacts→artifacts`, `IDs→ids`, `Disable→disable`. anodizer's `disable` is bool|template upgrade.

### Default divergences
- **`env` ordering**
  - goreleaser: `pkg/config/config.go:898` `Env []string`. Iterated in user-given order, each entry rendered with previously-rendered envs in scope (`internal/pipe/sbom/sbom.go:283-292`).
  - anodizer: `crates/core/src/config.rs:4202` `Option<HashMap<String, String>>`. Order non-deterministic; envs cannot reference each other through templates. Same divergence as `signs.env`.
- **`artifacts: binary` default `documents` template**
  - goreleaser: `internal/pipe/sbom/sbom.go:71` `"{{ .Binary }}_{{ .Version }}_{{ .Os }}_{{ .Arch }}.sbom.json"`.
  - anodizer: `crates/stage-sbom/src/lib.rs:354-356` matches verbatim.
- **`artifacts` filter values — anodizer adds `diskimage`/`installer`**
  - goreleaser: `internal/pipe/sbom/sbom.go:115-138` accepts `source|archive|binary|package|any`. Unknown values return `fmt.Errorf("invalid list of artifacts to catalog: %s", cfg.Artifacts)`.
  - anodizer: `crates/stage-sbom/src/lib.rs:444-458` accepts the GR set plus `diskimage|installer`, AND falls back to `archive` with a warn log on unknown — instead of erroring. User typos silently catalog the wrong artifact set.
- **`use_builtin` mode (anodizer-only)**
  - goreleaser: no built-in SBOM generation; always shells out.
  - anodizer: `crates/stage-sbom/src/lib.rs:339, :674-718` parses `Cargo.lock` directly when both `cmd` and `args` are unset. No GR analog. Wired and tested. Document as anodizer-extra.
- **multi-document validation**
  - goreleaser: `internal/pipe/sbom/sbom.go:91-93` errors when `Artifacts != "any" && len(Documents) > 1`.
  - anodizer: `crates/stage-sbom/src/lib.rs:364-370` matches semantics.

### Validation gaps
- **Unknown `artifacts` value: anodizer warns + falls back; GR errors**. Same as default-divergence above; flagged again because it's a user-facing surprise.
- **`syft` defaults applied even when `cmd` is overridden but `args` is empty**
  - goreleaser: `internal/pipe/sbom/sbom.go:78-87` only applies syft-specific arg/env defaults `if cfg.Cmd == "syft"`.
  - anodizer: `crates/stage-sbom/src/lib.rs:373-385, :388-399` matches the gate (`if cmd == "syft"`). Parity.
- **`documents` rendering does not abs-path inside dist as in GR**
  - goreleaser: `internal/pipe/sbom/sbom.go:296-322` joins relative paths against `ctx.Config.Dist`, then `filepath.Abs`, then computes a subprocess-relative dist path (so subprocess cwd is `dist` and paths are relative to it).
  - anodizer: `crates/stage-sbom/src/lib.rs:550-562` renders the template, then `dist.join(doc_path)` after the fact (`:625`). Different: anodizer's command runs with `current_dir(dist)` (lib.rs:594) and passes the rendered path directly. If user supplies an absolute document path, anodizer joins on top of dist (creating `/dist/abs/path`) instead of using it as-is. **GR handles abs paths**; anodizer does not. Verify with a test.
- **`env` render fallback / ordering** — see default divergence; lack of ordered semantics makes some valid GR configs silently broken.
- **No `WithArtifact` template context in the `any` mode**
  - goreleaser: `internal/pipe/sbom/sbom.go:128-135, :273-281` skips per-artifact context for `any`.
  - anodizer: `crates/stage-sbom/src/lib.rs:487-498` matches.
- **Subprocess env passthrough list duplicated between sbom and core**
  - goreleaser: `internal/pipe/sbom/sbom.go:30` `passthroughEnvVars = [...]`.
  - anodizer: `crates/stage-sbom/src/lib.rs:596-605` inlines the same list. DRY violation if other stages need the same passthrough. Lift to `anodizer_core::env::PASSTHROUGH_VARS`.

### Code smells
- **`crates/stage-sbom/src/lib.rs:572-574` — `ctx.render_template(&s)` immediately followed by `unwrap_or_else` in the surrounding map** would lose template errors silently; verify the actual code (line range is the args render). Use `?`.
- **`crates/stage-sbom/src/lib.rs:411-439` — binary-mode dedup loop** is hand-rolled; should use a shared helper (mirrors GoReleaser's `artifact.ByBinaryLikeArtifacts` helper). Documented in source comment but not factored.
- **`crates/stage-sbom/src/lib.rs:638-649` — Sbom Artifact registration uses `name: String::new()`** like checksum stage. Drift from artifact-naming convention.
- **`crates/stage-sbom/src/lib.rs:684-692` — built-in mode infers `spdx` vs `cyclonedx` by substring search on the document name**. A document named `cyclonedx-spdx-comparison.json` would be classified `spdx`. Use an explicit `format` field.
- **`crates/stage-sbom/src/lib.rs:619-620` — error message `"sbom[{}]: '{}' failed: {}"`** prints raw stderr. Trim the stderr to last N lines for readability; raw multi-MB stderr blobs swamp logs.
- **`crates/stage-sbom/src/lib.rs:662-668` — clears 6 per-target template vars at end** with `set("X", "")`. A dedicated `clear_artifact_vars()` helper should live in `Context`.

### Anodizer-only intentional
- `diskimage` / `installer` artifacts filter values (lib.rs:448-449) — Pro-equivalents.
- Built-in `Cargo.lock` SBOM mode (lib.rs:674-918) — verified, default when both `cmd` and `args` are unset.
- `disable: bool|template` upgrade.

---

## Cross-cutting (apply to all four pipes)

- **Lazy defaults vs eager Default()**: GoReleaser persists the rendered default into the config struct during `Default()`; anodizer computes the default at run-time. Effect: `--debug-config` / YAML round-trip differs. Pipes affected: checksum (name_template), sign (cmd, args, signature, artifacts, id), notarize (timeout), sbom (id, cmd, args, documents, artifacts).
- **`env: HashMap` vs ordered `[]string`**: signs/sboms env loses user-given order, breaking `BAR=$FOO`-style chaining. Cross-cutting.
- **`StringOrBool::is_disabled` swallows template errors**: cross-cutting; appears in checksum, sbom, notarize, and likely others.
- **Lack of `Default()`-time validation**: every `bail!` in the four stages fires at Run, by which point build/archive/sign work has already been done. GR's pattern is `Default()` returns config errors before the pipeline starts.

---

## Summary table

| pipe | MISSING | default-divergence | code-smell | validation-gap |
|---|---|---|---|---|
| checksum | 0 | 3 | 5 | 2 |
| sign | 1 | 6 | 5 | 5 |
| notarize | 0 | 3 | 5 | 5 |
| sbom | 0 | 5 | 6 | 5 |
| **totals** | **1** | **17** | **21** | **17** |

Cross-cutting items (lazy defaults, env ordering, is_disabled error swallow, Default-time validation) are not double-counted.

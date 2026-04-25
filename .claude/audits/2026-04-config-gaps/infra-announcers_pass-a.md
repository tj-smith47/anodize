# infra publishers (artifactory + dockerhub + upload + blob) — parity audit (Pass A)

Audited: 2026-04-25
References: `/opt/repos/anodizer` HEAD, `/opt/repos/goreleaser` HEAD.

**Re-grading rule:** Go-toolchain-specific fields (PySdist/PyWheel artifact types,
gocloud.dev secret-keeper URLs, Go-template `.` prefix on var names) are NOT
MISSING-FIELD findings in a Rust tool.

DockerHub's GoReleaser implementation is **closed-source Pro** (no
`internal/pipe/dockerhub/` directory). The only available reference is
`/opt/repos/goreleaser/www/content/customization/publish/dockerhub.md`. The
DockerHub findings below are scored against that doc.

## artifactory

GR types: `pkg/config/config.go:1212` (`Upload` struct, reused for both
`artifactories:` and `uploads:`).
GR pipe: `internal/pipe/artifactory/artifactory.go`.
GR shared HTTP: `internal/http/http.go`.
Anodizer config: `crates/core/src/config.rs:5098` (`ArtifactoryConfig`).
Anodizer stage: `crates/stage-publish/src/artifactory.rs`.

### Real MISSING fields

None. Every GR `Upload` field with a Rust analog is present on
`ArtifactoryConfig`.

### Default divergences

- **Password lookup order reversed.** GR (`http.go:168-178`) cascades
  `config.password` (template-rendered) → `ARTIFACTORY_{NAME}_SECRET`. Anodizer
  (`artifactory.rs:407-414`) cascades env → config. A user with both set will
  see env override config in anodizer, config override env in GR. Same bug in
  `upload.rs:67-74`.
- **Username falls back to env only when field is `None`.** GR
  (`http.go:155-165`) treats an empty rendered string the same as missing;
  anodizer (`artifactory.rs:400-405`) only consults env when the YAML key is
  absent. `username: ""` in YAML therefore disables the env fallback in
  anodizer.
- **Default `checksum_header`.** GR (`artifactory.go:24-25`) hard-defaults
  `X-Checksum-SHA256`. Anodizer (`artifactory.rs:417-420`) defaults the same
  string — **parity OK** but worth noting for the matrix.

### Code smells

- `artifactory.rs:179-213` — `render_artifact_url` is a hand-rolled
  string-replace pass over `{{ .ArtifactName }}`, `{{.ArtifactName}}`,
  `{{ .Os }}`, `{{ .Arch }}`. GR runs the full template engine with the
  artifact context (`http.go:306`). Whitespace variants (`{{  .Arch }}`),
  pipe expressions (`{{ .Arch | upper }}`), and any non-listed artifact
  field will silently be skipped or mis-rendered.
- `artifactory.rs:198-205` — when `custom_artifact_name=false`, the
  artifact name is appended *after* `{{ .ArtifactName }}` substitution,
  so a template that uses `{{ .ArtifactName }}` followed by the implicit
  append produces `…/myapp-1.0.tar.gz/myapp-1.0.tar.gz`.
- `artifactory.rs:308-322` — JSON error parser only inspects `errors[].message`;
  Artifactory also emits `errors[].status` (per `Error` struct in
  `artifactory.go:60-63`). Status codes never appear in the bubbled message.
- `artifactory.rs:1` — single-file 1119-line module mixes generic HTTP
  upload helpers (`build_reqwest_client`, `upload_single_artifact`,
  `collect_upload_artifacts`) with Artifactory-specific orchestration.
  `upload.rs:6` reaches across via `crate::artifactory::…` for the
  shared helpers; that import shape couples the generic publisher to
  the Artifactory namespace.
- `artifactory.rs:33-44` — `artifact_kinds_for_mode("archive")` omits
  `SourceRpm`, `Sbom`, `Snap`, `DiskImage`, `Installer`, `MacOsPackage`.
  GR (`upload.go:230-239`) includes `UploadableSourceArchive` and (via the
  release-uploadable list elsewhere) treats `LinuxPackage` as covering
  rpm/deb/apk. Anodizer-built MSI/PKG/DMG/SRPM/Snap will silently be
  skipped by an `archive`-mode upload.
- `artifactory.rs:436-497` — dry-run path renders the `target` template
  via `ctx.render_template`, but the *live* upload re-renders via
  `render_artifact_url` (string replace). Dry-run output therefore lies
  about whether template-engine features will work at upload time.

### Validation gaps

- `artifactory.rs:266-269` — `req.basic_auth(username, Some(password))`
  is only set when **both** are non-empty. A user setting only `username`
  (with empty password) silently uploads anonymously rather than erroring.
  Cross-check at lines 502-515 catches this in live mode but **only after**
  client cert validation; the early dry-run path (line 434) does not flag
  it.
- `artifactory.rs:160-164` — `Certificate::from_pem_bundle` errors if the
  PEM is empty after parsing; GR (`http.go:134-136`) `AppendCertsFromPEM`
  returns false but the call site bubbles a `misconfigured(...)` Skip,
  not a hard error. Anodizer fails loud where GR skips with reason.
- `artifactory.rs:367-368` — `validate_upload_mode` lower-cases nothing;
  `mode: "Archive"` (capital A) errors out. GR (`upload.go:228`)
  `strings.ToLower(upload.Mode)` accepts mixed case.

## dockerhub

GR reference: `www/content/customization/publish/dockerhub.md` (Pro pipe;
no OSS source).
Anodizer config: `crates/core/src/config.rs:5046` (`DockerHubConfig`).
Anodizer stage: `crates/stage-publish/src/dockerhub.rs`.

### Real MISSING fields

None. The doc lists `username`, `secret_name`, `images`, `disable`,
`description`, `full_description: { from_url, from_file }`. All present on
`DockerHubConfig`.

### Default divergences

- **`secret_name` default.** Doc states `DOCKER_PASSWORD`; anodizer
  (`dockerhub.rs:140`) defaults `DOCKER_PASSWORD`. Match.
- **`username` default.** Doc states `{{ .Env.DOCKER_USERNAME }}` (template
  fallback). Anodizer (`dockerhub.rs:74-79`) bails when `username` is
  unset/empty rather than trying `DOCKER_USERNAME`. Behavioural divergence:
  the GR doc explicitly says "Default: `{{ .Env.DOCKER_USERNAME }}`".
- **`description` source.** Doc says "Default: inferred from global
  metadata." Anodizer treats empty description as no-op
  (`dockerhub.rs:131-137`); never reads from `ctx.config.metadata` /
  `crate.description`.

### Code smells

- `dockerhub.rs:32-39` — `resp.text().unwrap_or_default()` swallows the
  body-read error (returns empty string) when assembling the error message
  for a non-2xx response. The user sees the HTTP status but loses the body
  read failure.
- `dockerhub.rs:178-183` — `image.splitn(2, '/').collect::<Vec<&str>>()`
  followed by `parts.len() == 2` defaults the namespace to `"library"` for
  bare names. The GR doc warns bare names need Docker Inc permissions,
  which `dockerhub.rs:99-105` warns about, but the call still proceeds —
  Docker Hub will return 404 and surface it via the PATCH error. Worth
  failing fast.
- `dockerhub.rs:90-95` — short-description length warning uses
  `s.len()` (UTF-8 bytes), not graphemes; a 50-character emoji
  description would falsely log "100 chars". Docker Hub itself counts
  graphemes for the 100-char limit.
- `dockerhub.rs:119` — fresh `reqwest::blocking::Client::new()` per entry
  with no timeout, no connection pooling shared across the whole stage.

### Validation gaps

- `dockerhub.rs:74-79` — username required, but no validation that
  the `secret_name` env var exists *before* hitting the dry-run skip
  (line 108). A misconfigured secret name slips through dry-run
  unnoticed.
- `dockerhub.rs:97-105` — bare image names emit a warn but do not block
  the live PATCH; user gets a 404 from Docker Hub instead of an
  upfront config error.
- `dockerhub.rs:99` — `image.contains('/')` accepts `myorg/myapp/extra`
  (which is invalid for the `repositories/{ns}/{name}/` API endpoint).
  No path-component validation.
- No validation that exactly one of `from_url` / `from_file` is set when
  `full_description` block is provided. `resolve_full_description`
  (`dockerhub.rs:46`) bails at runtime; config-time validation would
  catch it earlier.

## upload (custom publisher)

GR types: `pkg/config/config.go:1212` (`Upload` struct, shared).
GR pipe: `internal/pipe/upload/upload.go`.
Anodizer config: `crates/core/src/config.rs:5859` (`UploadConfig`).
Anodizer stage: `crates/stage-publish/src/upload.rs`.

### Real MISSING fields

None.

### Default divergences

- **Field name `disable` vs GR `skip`.** GR (`config.go:1231`) calls the
  template-skip field `skip`; anodizer (`config.rs:5900`) calls it
  `disable` with **no `skip` serde alias**. A user copying
  `skip: "{{ .IsSnapshot }}"` from a GR config sees the field silently
  ignored. ArtifactoryConfig (`config.rs:5140`) correctly uses `skip`,
  so the two structurally-identical types have *opposite* spellings on
  the same semantic.
- **Password lookup order reversed.** Same bug as artifactory:
  `upload.rs:67-74` consults env before config.
- **`name` default `"upload"`.** GR (`upload.go:30`) requires `Name`
  via `CheckConfig`; missing-name yields a `Skip` (treated as warning,
  upload skipped). Anodizer (`upload.rs:30`) silently substitutes
  `"upload"` and proceeds, generating `UPLOAD_UPLOAD_USERNAME` /
  `UPLOAD_UPLOAD_SECRET` env-var names.
- **Empty `target` errors with "missing required" but GR Skips.** GR
  (`http.go:101-104`) returns `pipe.Skip(...)`; anodizer
  (`upload.rs:37-39`) `bail!`s. Hard-fail where GR soft-skips.

### Code smells

- `upload.rs:6` — depends on `crate::artifactory::{validate_upload_mode,
  collect_upload_artifacts, build_reqwest_client, upload_single_artifact}`.
  The shared HTTP helpers should live in a `crate::http_upload` module so
  that neither publisher reaches across.
- `upload.rs:14-19` — early-return on empty `Vec` is identical to
  `artifactory.rs:346-349` and `dockerhub.rs:58-61`. Repeats three times.
- `upload.rs:97-103` — empty-artifacts logged as `verbose` instead of
  `status`, while `artifactory.rs:544-550` logs same situation as
  `status`. Consistency drift.
- `upload.rs:135-165` — duplicates the "render target URL with artifact
  context, then append name unless `custom_artifact_name`" block from
  `artifactory.rs:179-213`, with subtly different template behaviour
  (`upload.rs` uses the full template engine, artifactory uses string
  replace). Two sites of the same logic, drifted.

### Validation gaps

- `upload.rs:60-74` — **no cross-validation** that username and password
  are both set or both empty. GR (`http.go:126-132`) explicitly
  `misconfigured(...)`s when only one of the pair is present.
  artifactory.rs has this check (lines 502-515); upload.rs does not.
- `upload.rs` — **no mTLS pair check**. GR (`http.go:138-149`) errors when
  `client_x509_cert` and `client_x509_key` are unevenly set, and
  `tls.LoadX509KeyPair` validates the pair. Anodizer relies on
  `build_reqwest_client` to bail (`artifactory.rs:152-156`), but only
  *after* artifact collection — a misconfigured cert wastes the artifact
  filter pass.
- `upload.rs:33-34` — `validate_upload_mode` rejects `Archive` /
  `Binary` (case-sensitive). GR `strings.ToLower` accepts mixed case.
- No `trusted_certificates` PEM-empty check (same as artifactory).
- `UploadConfig.password` field exists in struct but is never
  documented as the cascade target. The `// Since v2.12` comment in GR
  (`config.go:1234`) marks it as the new pattern; anodizer doc
  (`config.rs:5870`) just says "env var template recommended" without
  noting that it's the second-stage fallback.

## blob

GR types: `pkg/config/config.go:1192` (`Blob` struct).
GR pipe: `internal/pipe/blob/blob.go`, `internal/pipe/blob/upload.go`.
Anodizer config: `crates/core/src/config.rs:3858` (`BlobConfig`).
Anodizer stage: `crates/stage-blob/src/lib.rs`.

### Real MISSING fields

None on the GR side. Anodizer adds `id`, `parallelism`, and
`templated_extra_files` (Pro feature) on top.

### Default divergences

- **`Directory` default uses Tera syntax not Go-template.** GR
  (`blob.go:27`) defaults `{{ .ProjectName }}/{{ .Tag }}`; anodizer
  (`lib.rs:747`) defaults `{{ ProjectName }}/{{ Tag }}` (no `.`).
  Functionally equivalent through anodizer's renderer but a YAML copied
  from a GR config that overrides directory still works (renderer
  accepts both); a YAML copied from anodizer to a GR project would
  break.
- **`ContentDisposition` default omitted (intentional).** GR
  (`blob.go:32`) defaults `attachment;filename={{.Filename}}`. Anodizer
  (`lib.rs:412-422` + comment) deliberately omits this so images/PDFs
  preview in-browser. Documented divergence; users migrating from GR
  will see different download behaviour. The sentinel `"-"` to disable
  is preserved.
- **`s3_force_path_style` default `true` when endpoint is set.** GR
  (`upload.go:60-65`) defaults to `true` only if `S3ForcePathStyle == nil`
  AND `endpoint != ""`; anodizer (`lib.rs:294`) does the same. Match.
  But anodizer **also** applies `force_path` when `endpoint` is empty
  (`lib.rs:296-298`); GR does not include the path-style query param
  unless an endpoint is set. Anodizer's branch only triggers when the
  user explicitly sets `s3_force_path_style`, so impact is low —
  document.
- **`release_uploadable_kinds()` excludes installers.** Anodizer
  `crates/core/src/artifact.rs:403-418` omits `DiskImage`, `Installer`,
  `MacOsPackage` — the kinds emitted by `stage-dmg`, `stage-msi`,
  `stage-nsis`, `stage-pkg`. `collect_artifacts` (`lib.rs:491-518`) uses
  this list, so blob upload **silently skips DMG/MSI/PKG/NSIS
  installers**. GR's equivalent (`artifact.go:132-148`) doesn't have
  these kinds (Pro-only types in GR), but anodizer ships these stages
  in OSS, and they should appear in releases and blobs. Real bug.
- **`include_meta` adds `Metadata` kind.** Match
  (`lib.rs:492-494` ↔ `upload.go:163-165`). OK.

### Code smells

- `lib.rs:90-251` — `encrypt_with_kms` shells out to `aws`, `gcloud`, `az`
  CLIs for client-side encryption; GR uses native `gocloud.dev/secrets`
  Go libraries. The CLI dependency is an undocumented runtime requirement
  — there is no preflight check for `aws --version` etc.
- `lib.rs:99-114, 151-166, 206-225` — three nearly identical
  `Command::new(...).spawn()` blocks differ only in argv shape and
  output parsing. Should fold into one helper that takes a per-provider
  policy.
- `lib.rs:497-500` — `.filter(|a| a.crate_name == crate_name)` couples
  blob upload to anodizer's per-crate model. GR has no per-crate
  filtering; this means `ids:` filtering is effectively `(crate_name +
  metadata["id"])` AND-ed. Document or expose `crate:` field on
  BlobConfig if cross-crate blob uploads should be possible.
- `lib.rs:506-516` — `id` filter checks both `metadata["id"]` and
  `metadata["name"]`. GR `ByIDs` only matches the artifact's `Extra["ID"]`
  (`artifact.go:680-690`). Two-key match means an `ids: [foo]` filter
  matches artifacts whose *name* happens to be `foo`, even when the
  build id is different. False positives.
- `lib.rs:592-604` — `format_remote_path` mirrors the cloud URL scheme
  for log output. Hard-coded scheme strings duplicate `Provider::display_name`
  (`lib.rs:42-48`). One source of truth would be cleaner.
- `lib.rs:528-590` — `upload_files_owned` spins up a fresh tokio
  `Runtime::new()` per blob job (`lib.rs:536`). With many blob configs
  this is N runtime creations. Cache or reuse.
- `lib.rs:317-346` — S3 ACL set as a default header
  (`x-amz-acl`) on the client; this means `acl:` applies to **every**
  PUT through the client, including KMS encryption side-channels.
  GR (`upload.go:106-126`) sets it per-write via `BeforeWrite`. Per-write
  is more flexible.
- `lib.rs:355-377` — GCS ACL passes the same string GR documents,
  but no enum validation; `acl: "publicRead"` (GCS uses camelCase
  `publicRead`, not `public-read`) silently 400s at upload.
- `lib.rs:660-665` — `BlobJob` clones `Arc<dyn ObjectStore>`,
  `Vec<(PathBuf, String)>`, `Vec<PutOptions>`, and the KMS tuple; the
  `Vec` clones could be `Arc<Vec<...>>` to avoid duplication.

### Validation gaps

- `lib.rs:720-725` — `provider.is_empty()` and `bucket.is_empty()` checks
  but no enum validation of `provider` until `Provider::parse`
  (`lib.rs:30-40`). The parse error fires *after* template rendering
  (`lib.rs:733`); a typo `provider: s4` only surfaces partway through
  Phase 1.
- `lib.rs:330-336` — S3 ACL whitelist correctly mirrors
  `upload.go:113-119`, but the comment claims `log-delivery-write` is
  omitted "to match upstream"; that omission is correct (GR also
  excludes it), but a user passing `log-delivery-write` (a valid AWS
  canned ACL) gets a hard error from anodizer where GR also errors —
  parity OK, but the comment is the only documentation of the choice.
- No validation that `kms_key:` URL schemes match `provider:`. A user
  setting `provider: gcs` with `kms_key: awskms://...` will encrypt
  with AWS KMS then upload to GCS — possibly intentional, but no warn.
- No validation that `cache_control` array values are individually
  RFC-7234 directives; a malformed value is forwarded to S3 / GCS
  unchanged.
- `lib.rs:412-422` — content-disposition rendered with `Filename` set,
  but if the template references unrelated artifact vars (e.g.
  `{{ Os }}`), they resolve to the empty string silently because
  `template_vars` is cloned from the global ctx, not per-artifact.

## Summary table

| publisher | MISSING | default-divergence | code-smell | validation-gap |
|---|---|---|---|---|
| artifactory | 0 | 2 | 6 | 3 |
| dockerhub | 0 | 3 | 4 | 4 |
| upload | 0 | 4 | 4 | 5 |
| blob | 0 | 4 | 8 | 5 |

**Total: 0 MISSING, 13 default-divergence, 22 code-smell, 17 validation-gap.**

Cross-cutting:

- **The `username`/`password` env-cascade order reversal** (`env-first
  vs config-first`) appears in both `artifactory.rs:407` and
  `upload.rs:67`. Single fix-point.
- **Mode case-sensitivity** appears in both `artifactory.rs:367` and
  `upload.rs:33`. `validate_upload_mode` should `to_lowercase` first.
- **Field-name divergence `uploads.disable` vs `artifactories.skip`**
  (anodizer) vs GR canonical `skip` for both. The two structurally
  identical configs disagree on the same semantic in the same crate.
- **`release_uploadable_kinds()` omitting installers** affects blob
  silently. The same list is consulted by `stage-publish` (GitHub
  release) — needs cross-check that DMG/MSI/PKG actually attach to
  GitHub releases today.

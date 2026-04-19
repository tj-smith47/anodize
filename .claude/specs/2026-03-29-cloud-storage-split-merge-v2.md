# Cloud Storage + Split/Merge — Implementation Spec v2

> **Purpose:** Replace CLI-shelling blob upload with `object_store` SDK and bring split/merge to full GoReleaser Pro parity. Every config field, every behavior, every error path documented.

---

## Part 1: Blob Upload via `object_store`

### Dependency

```toml
# In workspace Cargo.toml [workspace.dependencies]:
object_store = { version = "0.13", features = ["aws", "gcp", "azure"] }

# In crates/stage-blob/Cargo.toml [dependencies]:
object_store.workspace = true
tokio.workspace = true
```

`object_store` 0.13.2 (Apache Arrow, 53M downloads, updated 2026-03-24). Unified `ObjectStore` trait with feature-flagged backends. ~318K SLoC dep tree — smallest multi-backend option. This is the Rust equivalent of Go CDK that GoReleaser uses.

### Config Changes to `BlobConfig`

Current → New:

| Field | Current Type | New Type | Reason |
|-------|-------------|----------|--------|
| `cache_control` | `Option<String>` | `Option<Vec<String>>` | GoReleaser uses `[]string`, joined with `", "` |
| `disable` | `Option<bool>` | `Option<String>` | GoReleaser supports template expressions: `"{{ if .IsSnapshot }}true{{ end }}"` |
| `content_disposition` | `Option<String>` | `Option<String>` | No type change, but add default `"attachment;filename={{Filename}}"` and `"-"` to disable |
| `s3_force_path_style` | `Option<bool>` | `Option<bool>` | No type change, but change semantic: defaults to `true` when `endpoint` is set |

Backward compatibility for `disable`: accept both `disable: true` (bool) and `disable: "{{ .IsSnapshot }}"` (string) via a custom deserializer or `serde(untagged)` enum.

### Provider Construction

Replace `build_upload_command()` CLI construction with `ObjectStore` builder pattern:

```rust
fn build_store(provider: Provider, config: &BlobConfig, ctx: &Context) -> Result<Box<dyn ObjectStore>> {
    match provider {
        Provider::S3 => {
            let mut builder = AmazonS3Builder::from_env()
                .with_bucket_name(&rendered_bucket);
            if let Some(ref region) = config.region {
                builder = builder.with_region(&render(region, ctx)?);
            }
            if let Some(ref endpoint) = config.endpoint {
                builder = builder.with_endpoint(&render(endpoint, ctx)?);
                // Smart default: force path style when custom endpoint is set
                // (MinIO, R2, DO Spaces all need this)
                let force_path = config.s3_force_path_style.unwrap_or(true);
                builder = builder.with_virtual_hosted_style_request(!force_path);
            } else if let Some(force_path) = config.s3_force_path_style {
                builder = builder.with_virtual_hosted_style_request(!force_path);
            }
            if config.disable_ssl.unwrap_or(false) {
                builder = builder.with_allow_http(true);
            }
            // KMS server-side encryption
            if let Some(ref kms_key) = config.kms_key {
                builder = builder
                    .with_server_side_encryption(S3ServerSideEncryption::AwsKms)
                    .with_sse_kms_key_id(kms_key);
            }
            Ok(Box::new(builder.build()?))
        }
        Provider::Gcs => {
            let builder = GoogleCloudStorageBuilder::from_env()
                .with_bucket_name(&rendered_bucket);
            Ok(Box::new(builder.build()?))
        }
        Provider::AzBlob => {
            let builder = MicrosoftAzureBuilder::from_env()
                .with_container_name(&rendered_bucket);
            Ok(Box::new(builder.build()?))
        }
    }
}
```

### Authentication Per Provider

**S3:** `AmazonS3Builder::from_env()` reads the full AWS credential chain:
- `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` + optional `AWS_SESSION_TOKEN`
- `AWS_PROFILE` → `~/.aws/credentials`
- `AWS_WEB_IDENTITY_TOKEN_FILE` + `AWS_ROLE_ARN` (OIDC federation for GitHub Actions)
- EC2 IMDS v1/v2 (IAM roles for instances)
- ECS container credentials

**GCS:** `GoogleCloudStorageBuilder::from_env()` reads:
- `GOOGLE_SERVICE_ACCOUNT` (path to service account JSON)
- Application Default Credentials (`gcloud auth application-default login`)
- Compute Engine metadata server

**Azure:** `MicrosoftAzureBuilder::from_env()` reads:
- `AZURE_STORAGE_ACCOUNT_NAME` + `AZURE_STORAGE_ACCOUNT_KEY`
- `AZURE_STORAGE_SAS_TOKEN`
- `AZURE_STORAGE_CONNECTION_STRING`
- Managed Identity / federated token

### Upload Flow

```rust
fn upload_blob(ctx: &Context, config: &BlobConfig, store: &dyn ObjectStore) -> Result<()> {
    let dir = render_directory(config, ctx)?;

    // 1. Collect artifacts to upload
    let artifacts = artifact_list(ctx, config);
    let extra = resolve_extra_files(config, ctx)?;

    // 2. Upload in parallel (bounded by ctx.parallelism)
    // Use tokio runtime that's already in the workspace
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let semaphore = Arc::new(Semaphore::new(ctx.options.parallelism));

        let mut handles = vec![];
        for artifact in &artifacts {
            let permit = semaphore.clone().acquire_owned().await?;
            let key = object_store::path::Path::from(format!("{}/{}", dir, artifact.name()));
            let data = tokio::fs::read(&artifact.path).await?;

            let put_opts = build_put_options(config, &artifact.name(), ctx)?;
            handles.push(tokio::spawn(async move {
                let result = store.put_opts(&key, data.into(), put_opts).await;
                drop(permit);
                result
            }));
        }
        // Same for extra_files...

        for handle in handles {
            handle.await??;
        }
        Ok(())
    })
}
```

### Put Options (Headers)

```rust
fn build_put_options(config: &BlobConfig, filename: &str, ctx: &Context) -> Result<PutOptions> {
    let mut attrs = Attributes::new();

    // Cache-Control: join array with ", "
    if let Some(ref cc) = config.cache_control {
        attrs.insert(Attribute::CacheControl, cc.join(", ").into());
    }

    // Content-Disposition: template-rendered, "-" disables
    let disp = config.content_disposition.as_deref()
        .unwrap_or("attachment;filename={{Filename}}");
    if disp != "-" {
        let rendered = render_with_extra(disp, ctx, &[("Filename", filename)])?;
        attrs.insert(Attribute::ContentDisposition, rendered.into());
    }

    Ok(PutOptions { attributes: attrs, ..Default::default() })
}
```

### S3 ACL Handling

`object_store` doesn't have a first-class ACL field on `PutOptions`. Two approaches:

1. **Custom headers via `TagSet`**: S3 ACLs can be set via the `x-amz-acl` header. Check if `object_store` supports custom request headers through `ClientOptions`.
2. **Fallback**: If `object_store` cannot set ACLs, shell out to `aws s3api put-object-acl` after upload. Document this limitation.

Research needed during implementation: verify `object_store`'s support for S3 canned ACLs. If supported via `Attributes::from_iter` or custom headers, use that. If not, use the put-object-acl fallback.

### Error Handling

`object_store::Error` is a typed enum:

```rust
fn handle_upload_error(err: object_store::Error, key: &Path, provider: Provider) -> anyhow::Error {
    match &err {
        object_store::Error::NotFound { .. } => {
            anyhow::anyhow!("bucket does not exist or key not found: {}", err)
        }
        object_store::Error::Unauthenticated { .. } => {
            anyhow::anyhow!(
                "authentication failed for {} — check credentials: {}",
                provider_name(provider), err
            )
        }
        object_store::Error::UnknownConfigurationKey { .. } => {
            anyhow::anyhow!("invalid configuration: {}", err)
        }
        _ => anyhow::anyhow!("failed to upload to {}: {}", key, err),
    }
}
```

No string matching needed (unlike GoReleaser). Typed errors give reliable handling.

### Extra Files Template Rendering

Current gap: `extra_files[].name` is not template-rendered. Fix:

```rust
fn resolve_extra_files(config: &BlobConfig, ctx: &Context) -> Result<Vec<(String, PathBuf)>> {
    let mut result = vec![];
    if let Some(ref extras) = config.extra_files {
        for ef in extras {
            let matches: Vec<PathBuf> = glob::glob(&ef.glob)?.filter_map(|r| r.ok()).collect();
            if matches.is_empty() {
                log::warn!("blobs: extra_files glob '{}' matched no files", ef.glob);
            }
            for path in matches {
                let upload_name = if let Some(ref name_tmpl) = ef.name {
                    // Template-render the name with standard vars + Filename
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    render_with_extra(name_tmpl, ctx, &[("Filename", &filename)])?
                } else {
                    path.file_name().unwrap_or_default().to_string_lossy().to_string()
                };
                result.push((upload_name, path));
            }
        }
    }
    Ok(result)
}
```

### Dry-Run Behavior

In dry-run mode, skip actual upload. Log what would happen:

```
[dry-run] blobs: would upload dist/myapp-v1.0.0-linux-amd64.tar.gz → s3://my-bucket/myapp/v1.0.0/myapp-v1.0.0-linux-amd64.tar.gz
```

Do NOT construct the `ObjectStore` in dry-run — avoid credential validation that might fail in local dev.

### Test Strategy

**Unit tests (no network, no Docker):**
- Config parsing: all field variations, backward compat for bool `disable`
- Provider parsing: all valid values, invalid value error
- Directory rendering: template vars, empty directory, trailing slashes
- Extra file resolution: glob matching, name template rendering
- Artifact filtering: by IDs, extra_files_only, include_meta
- Put options construction: cache_control array join, content_disposition template, `"-"` disable
- Error handling: all typed error variants mapped correctly
- S3 builder: endpoint + force_path_style smart default, region, disable_ssl, KMS
- GCS builder: bucket name
- Azure builder: container name

**Integration tests (with `InMemory` object store):**
- Full upload flow with `object_store::memory::InMemory`
- Verify uploaded keys and data match expected
- Parallel upload with semaphore
- Extra files alongside artifacts
- Content-disposition and cache-control headers on uploaded objects

**Smoke tests (optional, CI-only):**
- MinIO container via `testcontainers` crate if time permits
- Real S3-compatible upload/verify/delete cycle

---

## Part 2: Split/Merge — Full GoReleaser Pro Parity

### New Config Fields

```rust
// In PartialConfig:
pub struct PartialConfig {
    /// How to split builds: "goos" (by OS, default) or "target" (by full triple).
    /// "goos" matches GoReleaser's default.
    pub by: Option<String>,  // Change default from "target" to "goos"
}
```

### New Environment Variables

| Env Var | Purpose | Priority |
|---------|---------|----------|
| `TARGET` | Full target triple override (highest priority) | 1 |
| `ANODIZER_OS` | OS filter (equivalent to GoReleaser's `GGOOS`) | 2 |
| `ANODIZER_ARCH` | Arch filter (equivalent to GoReleaser's `GGOARCH`) | 3 |
| (host detection) | `rustc -vV` host triple | 4 (fallback) |

Why `ANODIZER_OS`/`ANODIZER_ARCH` instead of reusing `GGOOS`/`GGOARCH`: GoReleaser's env vars use Go's `GOOS`/`GOARCH` naming (`linux`, `darwin`, `windows`, `amd64`, `arm64`). Rust uses different names (`x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`). Using `ANODIZER_OS` makes the mapping explicit and avoids confusion. The values accepted map to the OS names from `target::map_target()`: `linux`, `darwin`, `windows`, `freebsd`, `netbsd`, `openbsd`, `android`.

### Target Resolution Module

New file: `crates/core/src/partial.rs`

```rust
/// Resolve the partial build target from environment and config.
pub fn resolve_partial_target(
    config: &PartialConfig,
    all_targets: &[String],
) -> Result<PartialTarget> {
    // Priority 1: TARGET env var — exact target triple
    if let Ok(target) = std::env::var("TARGET") {
        return Ok(PartialTarget::Exact(target));
    }

    // Priority 2: ANODIZER_OS + optional ANODIZER_ARCH
    if let Ok(os) = std::env::var("ANODIZER_OS") {
        let arch = std::env::var("ANODIZER_ARCH").ok();
        return Ok(PartialTarget::OsArch { os, arch });
    }

    // Priority 3: host detection
    let host = detect_host_target()?;
    let by = config.by.as_deref().unwrap_or("goos");
    match by {
        "goos" => {
            let (os, _) = target::map_target(&host);
            Ok(PartialTarget::OsArch { os, arch: None })
        }
        "target" => Ok(PartialTarget::Exact(host)),
        other => anyhow::bail!("partial.by: unknown value '{}' (expected 'goos' or 'target')", other),
    }
}

pub enum PartialTarget {
    /// Exact target triple match
    Exact(String),
    /// Match by OS (and optionally arch)
    OsArch { os: String, arch: Option<String> },
}

impl PartialTarget {
    /// Filter a list of target triples to those matching this partial target.
    pub fn filter_targets(&self, targets: &[String]) -> Vec<String> {
        match self {
            PartialTarget::Exact(t) => {
                targets.iter().filter(|tt| *tt == t).cloned().collect()
            }
            PartialTarget::OsArch { os, arch } => {
                targets.iter().filter(|tt| {
                    let (t_os, t_arch) = target::map_target(tt);
                    t_os == *os && arch.as_ref().map_or(true, |a| t_arch == *a)
                }).cloned().collect()
            }
        }
    }

    /// Return the dist subdirectory name for this partial target.
    pub fn dist_subdir(&self) -> String {
        match self {
            PartialTarget::Exact(t) => t.clone(),
            PartialTarget::OsArch { os, arch } => {
                if let Some(a) = arch {
                    format!("{}_{}", os, a)
                } else {
                    os.clone()
                }
            }
        }
    }
}

/// Detect the host target triple via `rustc -vV`.
fn detect_host_target() -> Result<String> {
    let output = std::process::Command::new("rustc")
        .args(["-vV"])
        .output()
        .context("failed to run rustc -vV for host target detection")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(host) = line.strip_prefix("host: ") {
            return Ok(host.trim().to_string());
        }
    }
    anyhow::bail!("could not detect host target from rustc -vV output")
}
```

### Artifact Serialization Format

Richer format matching GoReleaser's artifact JSON:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SplitArtifact {
    /// Artifact filename (basename).
    pub name: String,
    /// Full path to the artifact file.
    pub path: String,
    /// OS component of the target (e.g., "linux", "darwin", "windows").
    pub goos: Option<String>,
    /// Arch component (e.g., "amd64", "arm64").
    pub goarch: Option<String>,
    /// Full target triple (e.g., "x86_64-unknown-linux-gnu").
    pub target: Option<String>,
    /// Artifact kind for internal routing.
    #[serde(rename = "internal_type")]
    pub kind: String,
    /// Human-readable type string (e.g., "Binary", "Archive").
    #[serde(rename = "type")]
    pub type_s: String,
    /// Crate that produced this artifact.
    pub crate_name: String,
    /// Rich metadata (format, id, etc.).
    pub extra: HashMap<String, serde_json::Value>,
}
```

### Context Serialization

GoReleaser serializes the full context (config, git info, version state) alongside artifacts during split. The merge phase loads this instead of re-deriving from git.

```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct SplitContext {
    /// The full project config (frozen at split time).
    pub config: Config,
    /// Git info (tag, commit, branch, etc.).
    pub git_info: Option<GitInfo>,
    /// Template variables (all resolved values).
    pub template_vars: HashMap<String, String>,
    /// The partial target that was used for filtering.
    pub partial_target: String,
    /// Artifacts produced by this split job.
    pub artifacts: Vec<SplitArtifact>,
}
```

Written to `dist/{subdir}/context.json` during split. The merge phase loads all `context.json` files, verifies config consistency across split jobs, and reconstructs the full context.

### Split Mode (`--split`)

```
anodizer release --split [--clean]
```

1. Load config, resolve git info, populate template vars (same as normal release)
2. Resolve partial target from env vars / host detection
3. Filter build targets to matching subset
4. Route output to `dist/{subdir}/` (e.g., `dist/linux/`, `dist/x86_64-unknown-linux-gnu/`)
5. Run build pipeline (BuildStage + UpxStage only)
6. Serialize `SplitContext` to `dist/{subdir}/context.json`
7. Generate `dist/matrix.json` with all targets from config (not just the current split)

**Build target filtering integration:**

The `BuildStage` must respect the partial target. Add `partial_target: Option<PartialTarget>` to `ContextOptions`. The build stage checks this and filters its target list accordingly. This is equivalent to GoReleaser's `build.filter()` function.

**Dist subdirectory routing:**

When `--split` is active, set `config.dist` to `{original_dist}/{subdir}` before running the pipeline. This way all stages that write to `config.dist` automatically go to the right subdirectory.

### Matrix Generation

```rust
#[derive(Serialize, Deserialize, Debug)]
struct SplitMatrix {
    /// How the build was split.
    split_by: String,
    /// Matrix entries.
    include: Vec<MatrixEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MatrixEntry {
    /// OS name (when split_by=goos) or full target triple (when split_by=target).
    target: String,
    /// Suggested runner for GitHub Actions.
    runner: String,
}
```

Runner suggestion:
- `linux` → `ubuntu-latest`
- `darwin` → `macos-latest`
- `windows` → `windows-latest`
- Other → `ubuntu-latest` (cross-compile)

### Merge Mode (`--merge`)

```
anodizer continue --merge [--dist <path>]
```

New `continue` subcommand (matching GoReleaser Pro's `goreleaser continue`).

1. Scan `dist/*/context.json` files
2. Validate: all split contexts have the same config hash (detect config drift between split jobs)
3. Merge artifact lists from all split contexts (deduplicate by path)
4. Reconstruct `Context` from the first split context's config/git_info/template_vars
5. Load all artifacts into the merged context's `ArtifactRegistry`
6. Run post-build pipeline: Archive → NFpm → Snapcraft → DMG → MSI → Pkg → Source → Checksum → Changelog → Release → Publish → Docker → Sign → Blob → Announce
7. Run post-pipeline tasks (metadata, publishers, after hooks)

**Config hash validation:**

```rust
fn config_hash(config: &Config) -> String {
    let json = serde_json::to_string(config).unwrap();
    format!("{:x}", sha2::Sha256::digest(json.as_bytes()))
}
```

If config hashes differ across split contexts, warn (not error — config may have been intentionally updated between split runs, e.g., hotfix).

### New CLI Commands

**`anodizer continue --merge`:**

```rust
#[derive(clap::Args)]
pub struct ContinueOpts {
    /// Load artifacts from split jobs and run post-build stages.
    #[arg(long)]
    merge: bool,
    /// Custom dist directory (default: from config).
    #[arg(long)]
    dist: Option<PathBuf>,
    // ... standard flags: --dry-run, --skip, --token, etc.
}
```

**`anodizer publish`:**

Run only release + publish + blob stages from a completed dist/. Useful for CI pipelines that separate build from publish.

```rust
// Pipeline: ReleaseStage → PublishStage → BlobStage
```

**`anodizer announce`:**

Run only the announce stage from a completed dist/.

```rust
// Pipeline: AnnounceStage
```

Both `publish` and `announce` load artifacts from `dist/artifacts.json` (or `dist/metadata.json`), reconstruct context, and run their subset of stages.

### Build Stage Integration

Modify `ContextOptions`:

```rust
pub struct ContextOptions {
    // ... existing fields ...
    /// Partial build target (set by --split or --single-target).
    pub partial_target: Option<PartialTarget>,
}
```

In `BuildStage::run()`:

```rust
fn run(&self, ctx: &mut Context) -> Result<()> {
    for build_config in &crate_config.builds {
        let targets = if let Some(ref partial) = ctx.options.partial_target {
            partial.filter_targets(&build_config.targets)
        } else {
            build_config.targets.clone()
        };

        if targets.is_empty() && ctx.options.partial_target.is_some() {
            // No targets match this split — skip silently (not an error)
            continue;
        }

        for target in &targets {
            self.build_target(ctx, build_config, target)?;
        }
    }
    Ok(())
}
```

### Docker Handling During Merge

During split: Docker images are built and pushed to the registry (if not dry-run).

During merge: Docker stage creates multi-arch manifests (`docker manifest create`) referencing the images pushed during split. The stage detects merge mode and skips `docker build` + `docker push`, only running manifest operations.

### Error Messages

| Scenario | Error Message |
|----------|--------------|
| No targets match partial filter | `"split: no build targets match {partial_target}. Available targets: [...]"` |
| TARGET env var doesn't match any config target | `"split: TARGET={value} does not match any configured build target"` |
| No context.json files found during merge | `"merge: no context.json files found in {dist}/. Run 'anodizer release --split' first."` |
| Config hash mismatch across split contexts | `"merge: warning — config differs between split jobs (hash {a} vs {b}). Using config from {first_context}."` |
| Unknown artifact kind in context.json | `"merge: unknown artifact kind '{kind}' in {file}. Skipping."` |
| partial.by invalid value | `"partial.by: unknown value '{val}' (expected 'goos' or 'target')"` |

---

## Part 3: Migration Plan

### Phase 1: Config Changes (backward compatible)

1. Add `Option<Vec<String>>` for `cache_control` with custom deserializer that accepts both `String` and `Vec<String>`
2. Add `StringOrBool` newtype for `disable` field with custom deserializer
3. Change `PartialConfig.by` default from `"target"` to `"goos"`
4. All existing tests must still pass

### Phase 2: Blob Stage Rewrite

1. Add `object_store` to workspace deps
2. Rewrite `crates/stage-blob/src/lib.rs`:
   - Remove all CLI command construction (`build_upload_command`, `build_s3_command`, etc.)
   - Add `build_store()` function for provider construction
   - Add `upload_blob()` with parallel upload via tokio semaphore
   - Add `build_put_options()` for headers
   - Fix extra_files name template rendering
3. Update tests to use `object_store::memory::InMemory` where needed
4. Keep all existing config parsing tests (they test config, not upload mechanism)

### Phase 3: Split/Merge Rewrite

1. Add `crates/core/src/partial.rs` — target resolution module
2. Modify `ContextOptions` to include `partial_target`
3. Rewrite `run_split()`:
   - Use `partial::resolve_partial_target()`
   - Route dist to subdirectory
   - Serialize `SplitContext` (not just artifacts)
   - Generate matrix with runner suggestions
4. Rewrite `run_merge()`:
   - Load `SplitContext` from all subdirs
   - Validate config hashes
   - Reconstruct full context
5. Add `continue` subcommand to CLI
6. Add `publish` and `announce` subcommands
7. Integrate partial target into `BuildStage`

### Phase 4: Tests

**Blob tests (target: 40+ tests):**
- Config parsing: all field variations (15 tests)
- Provider construction: S3 with all options, GCS, Azure (6 tests)
- Upload flow with InMemory store (5 tests)
- Extra files: glob, name template rendering (4 tests)
- Artifact filtering: ids, extra_files_only, include_meta (3 tests)
- Put options: cache_control, content_disposition, disable patterns (4 tests)
- Error handling: each typed error variant (4 tests)
- Dry-run: no store constructed, log output correct (2 tests)

**Split/merge tests (target: 35+ tests):**
- Target resolution: TARGET env, ANODIZER_OS/ARCH, host fallback (6 tests)
- Target filtering: by goos, by target, no matches, partial matches (6 tests)
- Dist subdirectory: naming for goos vs target mode (3 tests)
- Context serialization: round-trip serialize/deserialize (2 tests)
- Config hash: matching hashes, mismatched hashes (2 tests)
- Split pipeline: only build+upx stages run (2 tests)
- Merge pipeline: loads artifacts, runs post-build stages (3 tests)
- Matrix generation: goos split, target split, runner suggestions (4 tests)
- Error paths: no context files, invalid artifact kind, bad partial.by value (4 tests)
- CLI: continue --merge, publish, announce subcommands parse (3 tests)
- E2E: split → merge round-trip with fixture project (2+ tests)

---

## Part 4: Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` (workspace) | Add `object_store` dep |
| `crates/stage-blob/Cargo.toml` | Add `object_store`, `tokio` deps |
| `crates/stage-blob/src/lib.rs` | Full rewrite — SDK upload replacing CLI shelling |
| `crates/core/src/config.rs` | `BlobConfig` field type changes, `PartialConfig.by` default change |
| `crates/core/src/partial.rs` | New — target resolution module |
| `crates/core/src/lib.rs` | Add `pub mod partial;` |
| `crates/core/src/context.rs` | Add `partial_target` to `ContextOptions` |
| `crates/core/src/artifact.rs` | Add `name()` method to `Artifact`, add `goos`/`goarch` fields |
| `crates/cli/src/commands/release.rs` | Rewrite `run_split`/`run_merge`, richer serialization format |
| `crates/cli/src/commands/mod.rs` | Add `continue`, `publish`, `announce` subcommands |
| `crates/cli/src/main.rs` | Wire new subcommands |
| `crates/cli/src/pipeline.rs` | Add `build_publish_pipeline()`, `build_announce_pipeline()` |
| `crates/stage-build/src/lib.rs` | Integrate `partial_target` filtering |

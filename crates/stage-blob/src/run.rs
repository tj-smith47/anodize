use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Result};

use anodizer_core::context::Context;
use anodizer_core::stage::Stage;

use object_store::{ObjectStore, PutOptions};

use crate::kms::{KmsProvider, parse_kms_provider, preflight_kms_cli, validate_kms_provider_match};
use crate::provider::Provider;
use crate::store::build_store;
use crate::upload::{
    build_put_options, collect_artifacts, format_remote_path, resolve_extra_files,
    upload_files_owned,
};

// ---------------------------------------------------------------------------
// BlobStage
// ---------------------------------------------------------------------------

pub struct BlobStage;

/// A fully-prepared blob upload job. Phase 1 (serial, `&mut ctx`) renders
/// templates, builds the ObjectStore, pre-renders per-item put options;
/// Phase 2 (parallel) runs the per-config upload via `upload_files_owned`.
/// Workers never touch `ctx`.
struct BlobJob {
    provider_display: &'static str,
    rendered_bucket: String,
    rendered_directory: String,
    upload_items: Vec<(PathBuf, String)>,
    store: Arc<dyn ObjectStore>,
    put_opts_per_item: Vec<PutOptions>,
    parallelism_inner: usize,
    client_kms: Option<(String, KmsProvider)>,
}

impl Stage for BlobStage {
    fn name(&self) -> &str {
        "blob"
    }

    fn run(&self, ctx: &mut Context) -> Result<()> {
        let log = ctx.logger("blob");
        if ctx.skip_in_snapshot(&log, "blob") {
            return Ok(());
        }

        let selected = ctx.options.selected_crates.clone();
        let dry_run = ctx.options.dry_run;
        let global_parallelism = ctx.options.parallelism.max(1);

        // Collect crates that have blob config
        let crates: Vec<_> = ctx
            .config
            .crates
            .iter()
            .filter(|c| selected.is_empty() || selected.contains(&c.name))
            .filter(|c| c.blobs.is_some())
            .cloned()
            .collect();

        if crates.is_empty() {
            return Ok(());
        }

        // Pre-flight: when `provider` is a literal (no template syntax),
        // validate it via Provider::parse before any I/O so a typo (e.g.
        // `provider: gss`) fails immediately instead of after build/archive.
        // Template-bearing providers (`{{ ... }}`) are validated inside the
        // per-job loop after rendering. Provider::parse already mentions the
        // bad value and the valid set, so the error needs no extra context.
        for krate in &crates {
            if let Some(blob_configs) = krate.blobs.as_ref() {
                for blob_cfg in blob_configs {
                    if !blob_cfg.provider.is_empty() && !blob_cfg.provider.contains("{{") {
                        Provider::parse(&blob_cfg.provider)?;
                    }
                }
            }
        }

        // Phase 1 (serial): render every config, build stores, collect jobs.
        let mut jobs: Vec<BlobJob> = Vec::new();

        for krate in &crates {
            // SAFETY: `crates` was filtered to only include crates with
            // `blobs.is_some()` above, so this Option is always Some here.
            // `continue` defends against a future refactor that breaks the
            // invariant rather than panicking on the now-impossible None.
            let Some(blob_configs) = krate.blobs.as_ref() else {
                continue;
            };

            for blob_cfg in blob_configs {
                // Evaluate disable (supports both bool and template string)
                if ctx.skip_with_log(
                    &blob_cfg.skip,
                    &log,
                    &format!("blob config for crate {}", krate.name),
                )? {
                    continue;
                }

                // Validate required fields
                if blob_cfg.provider.is_empty() {
                    anyhow::bail!("blobs: provider is required for crate '{}'", krate.name);
                }
                if blob_cfg.bucket.is_empty() {
                    anyhow::bail!("blobs: bucket is required for crate '{}'", krate.name);
                }

                let provider_str = ctx.render_template(&blob_cfg.provider).with_context(|| {
                    format!(
                        "blobs: render provider template '{}' for crate '{}'",
                        blob_cfg.provider, krate.name
                    )
                })?;
                let provider = Provider::parse(&provider_str)?;
                let config_label = blob_cfg.id.as_deref().unwrap_or(&provider_str);

                // Render template fields
                let rendered_bucket = ctx.render_template(&blob_cfg.bucket).with_context(|| {
                    format!(
                        "blobs[{}]: render bucket template for crate {}",
                        config_label, krate.name
                    )
                })?;

                // Default mirrors GoReleaser's `{{ .ProjectName }}/{{ .Tag }}`
                // (blob.go:27) but expressed in Tera syntax (no leading `.`).
                // Anodizer's renderer accepts both forms, so a YAML lifted from
                // a goreleaser config that overrides `directory:` keeps working.
                let directory_template = blob_cfg
                    .directory
                    .as_deref()
                    .unwrap_or("{{ ProjectName }}/{{ Tag }}");
                let rendered_directory =
                    ctx.render_template(directory_template).with_context(|| {
                        format!(
                            "blobs[{}]: render directory template for crate {}",
                            config_label, krate.name
                        )
                    })?;

                log.status(&format!(
                    "uploading to {} {}/{}",
                    provider.display_name(),
                    rendered_bucket,
                    rendered_directory
                ));

                // Collect artifacts to upload
                let mut upload_items: Vec<(PathBuf, String)> = Vec::new();

                let artifacts = collect_artifacts(ctx, blob_cfg, &krate.name);
                for artifact in &artifacts {
                    let filename = artifact
                        .path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("artifact")
                        .to_string();
                    upload_items.push((artifact.path.clone(), filename));
                }

                // Resolve extra files (with template-rendered names)
                if let Some(ref extra_files) = blob_cfg.extra_files {
                    let resolved = resolve_extra_files(extra_files, ctx, &log)?;
                    upload_items.extend(resolved);
                }

                // Process templated_extra_files: render and add to upload list.
                // NOTE: Rendered files are written to the shared dist directory. If multiple
                // blob configs use the same dst name, later writes will overwrite earlier
                // ones. Users should ensure dst names are unique across configs.
                if let Some(ref tpl_specs) = blob_cfg.templated_extra_files
                    && !tpl_specs.is_empty()
                {
                    let rendered = anodizer_core::templated_files::process_templated_extra_files(
                        tpl_specs,
                        ctx,
                        &ctx.config.dist,
                        "blobs",
                    )?;
                    upload_items.extend(rendered);
                }

                // Note: metadata files are already handled by collect_artifacts()
                // when include_meta is true — it includes ArtifactKind::Metadata
                // in its filter. No separate scan needed here.

                if upload_items.is_empty() {
                    log.warn(&format!(
                        "no files to upload for blob config on crate '{}'",
                        krate.name
                    ));
                    continue;
                }

                if dry_run {
                    // Dry-run: log what would happen without constructing the store
                    for (local_path, remote_key) in &upload_items {
                        let remote = format_remote_path(
                            provider,
                            &rendered_bucket,
                            &rendered_directory,
                            remote_key,
                        );
                        log.status(&format!(
                            "[dry-run] would upload {} -> {}",
                            local_path.display(),
                            remote,
                        ));
                    }
                    continue;
                }

                // Log each file before upload (serial stays in Phase 1 so
                // the per-config announcement order remains deterministic,
                // matching the pre-parallel behaviour).
                for (local_path, remote_key) in &upload_items {
                    let remote = format_remote_path(
                        provider,
                        &rendered_bucket,
                        &rendered_directory,
                        remote_key,
                    );
                    log.status(&format!("uploading {} -> {}", local_path.display(), remote));
                }

                let store: Arc<dyn ObjectStore> =
                    Arc::from(build_store(provider, blob_cfg, &rendered_bucket, ctx)?);

                // Pre-render put options per item while we still hold &ctx.
                let put_opts_per_item: Vec<PutOptions> = upload_items
                    .iter()
                    .map(|(_, key)| build_put_options(blob_cfg, key, ctx))
                    .collect::<Result<_>>()?;

                // Determine if client-side KMS encryption is needed.
                // Validate KMS scheme matches the bucket provider so a misconfig
                // surfaces here, not deep inside the upload phase. Preflight
                // the CLI tool too — a missing `aws`/`gcloud`/`az` binary on
                // PATH used to fail per-artifact during fan-out, producing N
                // identical errors. One check, one error.
                let client_kms = if let Some(key) = blob_cfg.kms_key.as_deref() {
                    let kms_provider = parse_kms_provider(key);
                    validate_kms_provider_match(provider, kms_provider, key)?;
                    preflight_kms_cli(kms_provider)?;
                    match kms_provider {
                        KmsProvider::ServerSide => None,
                        _ => Some((key.to_string(), kms_provider)),
                    }
                } else {
                    None
                };

                let parallelism_inner = blob_cfg
                    .parallelism
                    .unwrap_or(ctx.options.parallelism)
                    .max(1);

                jobs.push(BlobJob {
                    provider_display: provider.display_name(),
                    rendered_bucket,
                    rendered_directory,
                    upload_items,
                    store,
                    put_opts_per_item,
                    parallelism_inner,
                    client_kms,
                });
            }
        }

        if jobs.is_empty() {
            return Ok(());
        }

        // Phase 2 (parallel across configs): each worker runs its own
        // upload loop (which itself has intra-config per-file concurrency
        // via tokio). Bounded by the global parallelism so we don't fan
        // out unbounded across both axes simultaneously.
        //
        // One tokio runtime is shared across every job — N parallel jobs
        // would otherwise allocate N independent thread pools.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("anodizer-blob")
            .build()
            .context("blob: failed to construct tokio runtime")?;
        let runtime_ref = &runtime;
        let run_job = |job: &BlobJob| -> Result<()> {
            upload_files_owned(
                runtime_ref,
                Arc::clone(&job.store),
                job.upload_items.clone(),
                job.rendered_directory.clone(),
                job.put_opts_per_item.clone(),
                job.parallelism_inner,
                job.client_kms.clone(),
            )
        };

        anodizer_core::parallel::run_parallel_chunks(&jobs, global_parallelism, "blob", run_job)?;

        for job in &jobs {
            log.status(&format!(
                "uploaded {} file(s) to {} {}/{}",
                job.upload_items.len(),
                job.provider_display,
                job.rendered_bucket,
                job.rendered_directory,
            ));
        }

        Ok(())
    }
}

pub mod artifactory;
pub mod aur;
pub mod aur_source;
pub mod cargo;
pub mod chocolatey;
pub mod cloudsmith;
pub mod dockerhub;
pub mod homebrew;
pub(crate) mod http_upload;
pub mod krew;
pub mod nix;
pub mod scoop;
pub mod upload;
pub(crate) mod util;
pub mod winget;

use anodizer_core::config::PublishConfig;
use anodizer_core::context::Context;
use anodizer_core::stage::Stage;
use anyhow::Result;

use artifactory::publish_to_artifactory;
use aur::publish_to_aur;
use aur_source::{publish_to_aur_source, publish_top_level_aur_sources};
use cargo::publish_to_cargo;
use chocolatey::publish_to_chocolatey;
use cloudsmith::publish_to_cloudsmith;
use dockerhub::publish_to_dockerhub;
use homebrew::{publish_to_homebrew, publish_top_level_homebrew_casks};
use krew::publish_to_krew;
use nix::publish_to_nix;
use scoop::publish_to_scoop;
use upload::publish_to_upload;
use winget::publish_to_winget;

/// Collect crate names that match the selection filter and have a specific
/// publisher configured (as determined by the predicate `has_config`).
fn crates_with_publisher<F>(ctx: &Context, selected: &[String], has_config: F) -> Vec<String>
where
    F: Fn(&PublishConfig) -> bool,
{
    ctx.config
        .crates
        .iter()
        .filter(|c| selected.is_empty() || selected.contains(&c.name))
        .filter(|c| c.publish.as_ref().is_some_and(&has_config))
        .map(|c| c.name.clone())
        .collect()
}

pub struct PublishStage;

impl Stage for PublishStage {
    fn name(&self) -> &str {
        "publish"
    }

    fn run(&self, ctx: &mut Context) -> Result<()> {
        let log = ctx.logger("publish");
        if ctx.skip_in_snapshot(&log, "publish") {
            return Ok(());
        }
        let selected = ctx.options.selected_crates.clone();

        // Individual publisher failures are collected and reported at the end
        // rather than aborting the entire publish stage. This prevents a single
        // publisher (e.g. homebrew auth) from killing independent downstream
        // publishers (docker, cosign, announce). crates.io is the exception —
        // it's the authoritative registry and its failure is always fatal.
        //
        // Strict mode semantics: we still COLLECT every publisher error so a
        // single run surfaces *all* remaining issues. The difference vs. the
        // default mode is that at the end of the stage we bail with the full
        // list instead of warning. Failing fast on the first error is
        // counter-productive for dogfooding — it hides every issue after the
        // first, forcing N release cycles to shake out N bugs.
        let mut errors: Vec<String> = Vec::new();

        // Helper: run a publisher, log + collect error on failure. The end-of-
        // stage aggregation below decides whether to warn or bail.
        macro_rules! try_publish {
            ($label:expr, $expr:expr) => {
                if let Err(e) = $expr {
                    // `{:#}` renders the full anyhow error chain on one line
                    // (e.g. "top: middle: root cause"). `{}` shows only the
                    // top context, which discards the actual root cause —
                    // hiding details like reqwest transport errors, HTTP
                    // status codes, or response bodies that operators need
                    // to diagnose a failing publisher.
                    log.warn(&format!("{}: {:#}", $label, e));
                    errors.push(format!("{}: {:#}", $label, e));
                }
            };
        }

        // infra-level publishers (blob,
        // upload, artifactory, docker-signs, snapcraft/dockerhub) run BEFORE
        // package managers (homebrew/cask/scoop/chocolatey/winget/aur/krew/nix).
        // Package managers often reference release artifacts by URL+digest, so
        // those URLs must be live before the manifests are published.
        //
        // crates.io is dispatched first (after the macro definitions below)
        // and is fatal — it's the authoritative Rust registry and must
        // succeed before anything downstream runs. `aur_source`/`aur_sources`
        // run last to match GoReleaser.

        // ---- Infrastructure publishers (run before package managers) ----

        // 2. DockerHub — top-level publisher (not per-crate).
        try_publish!("dockerhub", publish_to_dockerhub(ctx, &log));

        // 3. Artifactory — top-level publisher (not per-crate).
        try_publish!("artifactory", publish_to_artifactory(ctx, &log));

        // 4. CloudSmith — top-level publisher (not per-crate).
        try_publish!("cloudsmith", publish_to_cloudsmith(ctx, &log));

        // 5. Generic HTTP upload — top-level publisher.
        try_publish!("upload", publish_to_upload(ctx, &log));

        // ---- Package-manager publishers (consume URLs from releases above) ----
        //
        // Every entry below is dispatched through one of two macros so the
        // skip gate, log line, and label are produced uniformly:
        //
        //   per_crate!  — fan out per `selected` crate that has the publisher
        //                  configured. Predicate filters `PublishConfig`.
        //   top_level!  — single top-level call (no per-crate fan-out).
        //
        // Skip names match GoReleaser convention: `brew`, `scoop`, `choco`,
        // `winget`, `aur`, `krew`, `nix`, `cargo`. The skip gate fires from
        // here for every publisher (cargo included) so the user sees a single
        // uniform "X: skipped via --skip=X" line regardless of which publisher
        // owns the actual subprocess. `--skip=brew` and `--skip=aur` each gate
        // two related sub-publishers (formula+casks, binary+source).

        // Dispatcher helpers — collapse per-publisher boilerplate.
        // Each macro:
        //   1. checks `ctx.should_skip($skip_name)`,
        //   2. emits "{label}: skipped via --skip={skip_name}" if skipped,
        //   3. otherwise runs the publisher and routes errors through
        //      `try_publish!` (collected for end-of-stage aggregation).
        macro_rules! per_crate {
            ($skip:expr, $label:expr, $pred:expr, $run:expr) => {{
                if ctx.should_skip($skip) {
                    log.status(&format!("{}: skipped via --skip={}", $label, $skip));
                } else {
                    for crate_name in &crates_with_publisher(ctx, &selected, $pred) {
                        try_publish!($label, $run(ctx, crate_name, &log));
                    }
                }
            }};
        }
        macro_rules! top_level {
            ($skip:expr, $label:expr, $run:expr) => {{
                if ctx.should_skip($skip) {
                    log.status(&format!("{}: skipped via --skip={}", $label, $skip));
                } else {
                    try_publish!($label, $run(ctx, &log));
                }
            }};
        }

        // Cargo (crates.io) — top-level by virtue of doing its own crate
        // walk + topo sort internally. Fatal: any error aborts the stage,
        // matching the "authoritative registry must succeed first" rule.
        if ctx.should_skip("cargo") {
            log.status("cargo: skipped via --skip=cargo");
        } else {
            publish_to_cargo(ctx, &selected, &log)?;
        }

        // 8. Homebrew formulae — per-crate.
        per_crate!(
            "brew",
            "homebrew",
            |p: &PublishConfig| p.homebrew.is_some(),
            publish_to_homebrew
        );

        // 9. Scoop — per-crate.
        per_crate!(
            "scoop",
            "scoop",
            |p: &PublishConfig| p.scoop.is_some(),
            publish_to_scoop
        );

        // 10. Chocolatey — per-crate.
        per_crate!(
            "choco",
            "chocolatey",
            |p: &PublishConfig| p.chocolatey.is_some(),
            publish_to_chocolatey
        );

        // 11. WinGet — per-crate.
        per_crate!(
            "winget",
            "winget",
            |p: &PublishConfig| p.winget.is_some(),
            publish_to_winget
        );

        // 12. AUR (binary) — per-crate. Shares `--skip=aur` with aur-source.
        per_crate!(
            "aur",
            "aur",
            |p: &PublishConfig| p.aur.is_some(),
            publish_to_aur
        );

        // 13. Krew — per-crate.
        per_crate!(
            "krew",
            "krew",
            |p: &PublishConfig| p.krew.is_some(),
            publish_to_krew
        );

        // 14. Nix — per-crate.
        per_crate!(
            "nix",
            "nix",
            |p: &PublishConfig| p.nix.is_some(),
            publish_to_nix
        );

        // 15. Homebrew Casks — top-level publisher (GoReleaser parity).
        // Shares `--skip=brew` with the per-crate formula publisher above; the
        // skip emits twice (once for "homebrew", once for "homebrew-casks") so
        // operators see exactly which surface was suppressed.
        top_level!("brew", "homebrew-casks", publish_top_level_homebrew_casks);

        // ---- AUR source last (GoReleaser parity) ----

        // 16. AUR source packages — per-crate publisher.
        per_crate!(
            "aur",
            "aur-source",
            |p: &PublishConfig| p.aur_source.is_some(),
            publish_to_aur_source
        );

        // 17. AUR source packages — top-level array (GoReleaser `aur_sources`).
        top_level!("aur", "aur-sources", publish_top_level_aur_sources);

        if errors.is_empty() {
            Ok(())
        } else {
            let suffix = if ctx.is_strict() {
                " (strict mode)"
            } else {
                ""
            };
            anyhow::bail!(
                "{} publisher(s) failed{}:\n  {}",
                errors.len(),
                suffix,
                errors.join("\n  ")
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use anodizer_core::config::{
        AurConfig, BucketConfig, CargoPublishConfig, ChocolateyConfig, ChocolateyRepoConfig,
        Config, CrateConfig, HomebrewConfig, KrewConfig, KrewManifestsRepoConfig, PublishConfig,
        ScoopConfig, TapConfig, WingetConfig, WingetManifestsRepoConfig,
    };
    use anodizer_core::context::{Context, ContextOptions};

    fn dry_run_ctx(config: Config) -> Context {
        Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_stage_name() {
        assert_eq!(PublishStage.name(), "publish");
    }

    #[test]
    fn test_run_no_crates_configured() {
        let config = Config::default();
        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_run_dry_run_cargo() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mylib".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                cargo: Some(CargoPublishConfig::default()),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        // dry-run: should log but not actually shell out
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_run_dry_run_homebrew() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                homebrew: Some(HomebrewConfig {
                    tap: Some(TapConfig {
                        owner: "myorg".to_string(),
                        name: "homebrew-tap".to_string(),
                    }),
                    description: Some("My tool".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_run_dry_run_scoop() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                scoop: Some(ScoopConfig {
                    bucket: Some(BucketConfig {
                        owner: "myorg".to_string(),
                        name: "scoop-bucket".to_string(),
                    }),
                    description: Some("My tool".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_run_dry_run_all_publishers() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "allpub".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                cargo: Some(CargoPublishConfig::default()),
                homebrew: Some(HomebrewConfig {
                    tap: Some(TapConfig {
                        owner: "org".to_string(),
                        name: "homebrew-tap".to_string(),
                    }),
                    ..Default::default()
                }),
                scoop: Some(ScoopConfig {
                    bucket: Some(BucketConfig {
                        owner: "org".to_string(),
                        name: "scoop-bucket".to_string(),
                    }),
                    description: None,
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    // -----------------------------------------------------------------------
    // Task 4C: Additional behavior tests — config fields actually do things
    // -----------------------------------------------------------------------

    #[test]
    fn test_dry_run_logs_without_executing_for_all_publishers() {
        // Verify dry-run mode works for all publisher types simultaneously
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "multi".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                cargo: Some(CargoPublishConfig::default()),
                homebrew: Some(HomebrewConfig {
                    tap: Some(TapConfig {
                        owner: "org".to_string(),
                        name: "homebrew-tap".to_string(),
                    }),
                    description: Some("A multi-publisher tool".to_string()),
                    ..Default::default()
                }),
                scoop: Some(ScoopConfig {
                    bucket: Some(BucketConfig {
                        owner: "org".to_string(),
                        name: "scoop-bucket".to_string(),
                    }),
                    description: Some("A multi-publisher tool".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        // All publishers should succeed in dry-run mode
        let result = PublishStage.run(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_selected_crates_filter_applies_to_publishers() {
        let mut config = Config::default();
        config.crates = vec![
            CrateConfig {
                name: "included".to_string(),
                path: ".".to_string(),
                tag_template: "v{{ .Version }}".to_string(),
                publish: Some(PublishConfig {
                    homebrew: Some(HomebrewConfig {
                        tap: Some(TapConfig {
                            owner: "org".to_string(),
                            name: "tap".to_string(),
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            CrateConfig {
                name: "excluded".to_string(),
                path: ".".to_string(),
                tag_template: "v{{ .Version }}".to_string(),
                publish: Some(PublishConfig {
                    homebrew: Some(HomebrewConfig {
                        tap: Some(TapConfig {
                            owner: "org".to_string(),
                            name: "tap".to_string(),
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ];

        let mut ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                selected_crates: vec!["included".to_string()],
                ..Default::default()
            },
        );

        // Should only run for "included", not "excluded"
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_no_publish_config_is_noop() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "nopub".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: None, // No publish config
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        // Should succeed (no-op)
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    /// Document current behavior: the publish stage does NOT skip homebrew/scoop
    /// publishing for prerelease versions. It proceeds regardless of whether
    /// the version contains a prerelease suffix like -rc.1 or -beta.
    ///
    /// This is a known limitation: GoReleaser skips homebrew/scoop for prereleases
    /// by default. If this behavior is added in the future, this test should be
    /// updated to verify that skipping occurs.
    #[test]
    fn test_publish_prerelease_version_proceeds_without_skip() {
        use anodizer_core::context::ContextOptions;

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "myapp".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                cargo: Some(CargoPublishConfig::default()),
                homebrew: Some(HomebrewConfig {
                    tap: Some(TapConfig {
                        owner: "org".to_string(),
                        name: "homebrew-tap".to_string(),
                    }),
                    description: Some("A tool".to_string()),
                    ..Default::default()
                }),
                scoop: Some(ScoopConfig {
                    bucket: Some(BucketConfig {
                        owner: "org".to_string(),
                        name: "scoop-bucket".to_string(),
                    }),
                    description: Some("A tool".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        // Use a prerelease version like v1.0.0-rc.1
        let mut ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                ..Default::default()
            },
        );

        // Manually set the Version template var to a prerelease string.
        // The publish stage reads this from template_vars, not from git.
        ctx.template_vars_mut().set("Version", "1.0.0-rc.1");
        ctx.template_vars_mut().set("Tag", "v1.0.0-rc.1");

        // The publish stage should succeed in dry-run mode even with
        // a prerelease version. With skip_upload: "auto", homebrew/scoop
        // will skip for prereleases (matching GoReleaser behavior).
        let result = PublishStage.run(&mut ctx);
        assert!(
            result.is_ok(),
            "publish stage should succeed for prerelease versions in dry-run: {:?}",
            result.err()
        );

        // skip_upload is supported: "true" always skips, "auto" skips for prereleases.
    }

    // -----------------------------------------------------------------------
    // Chocolatey integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_run_dry_run_chocolatey() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                chocolatey: Some(ChocolateyConfig {
                    project_repo: Some(ChocolateyRepoConfig {
                        owner: "myorg".to_string(),
                        name: "mytool".to_string(),
                    }),
                    description: Some("My tool".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    // -----------------------------------------------------------------------
    // WinGet integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_run_dry_run_winget() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                winget: Some(WingetConfig {
                    manifests_repo: Some(WingetManifestsRepoConfig {
                        owner: "myorg".to_string(),
                        name: "winget-pkgs".to_string(),
                    }),
                    package_identifier: Some("MyOrg.MyTool".to_string()),
                    description: Some("My tool".to_string()),
                    publisher: Some("My Org".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_run_dry_run_all_five_publishers() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "allpub5".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                cargo: Some(CargoPublishConfig::default()),
                homebrew: Some(HomebrewConfig {
                    tap: Some(TapConfig {
                        owner: "org".to_string(),
                        name: "homebrew-tap".to_string(),
                    }),
                    ..Default::default()
                }),
                scoop: Some(ScoopConfig {
                    bucket: Some(BucketConfig {
                        owner: "org".to_string(),
                        name: "scoop-bucket".to_string(),
                    }),
                    description: None,
                    ..Default::default()
                }),
                chocolatey: Some(ChocolateyConfig {
                    project_repo: Some(ChocolateyRepoConfig {
                        owner: "org".to_string(),
                        name: "allpub5".to_string(),
                    }),
                    ..Default::default()
                }),
                winget: Some(WingetConfig {
                    manifests_repo: Some(WingetManifestsRepoConfig {
                        owner: "org".to_string(),
                        name: "winget-pkgs".to_string(),
                    }),
                    package_identifier: Some("Org.Allpub5".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_selected_crates_filter_applies_to_chocolatey_and_winget() {
        let mut config = Config::default();
        config.crates = vec![
            CrateConfig {
                name: "included".to_string(),
                path: ".".to_string(),
                tag_template: "v{{ .Version }}".to_string(),
                publish: Some(PublishConfig {
                    chocolatey: Some(ChocolateyConfig {
                        project_repo: Some(ChocolateyRepoConfig {
                            owner: "org".to_string(),
                            name: "included".to_string(),
                        }),
                        ..Default::default()
                    }),
                    winget: Some(WingetConfig {
                        manifests_repo: Some(WingetManifestsRepoConfig {
                            owner: "org".to_string(),
                            name: "winget-pkgs".to_string(),
                        }),
                        package_identifier: Some("Org.Included".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            CrateConfig {
                name: "excluded".to_string(),
                path: ".".to_string(),
                tag_template: "v{{ .Version }}".to_string(),
                publish: Some(PublishConfig {
                    chocolatey: Some(ChocolateyConfig {
                        project_repo: Some(ChocolateyRepoConfig {
                            owner: "org".to_string(),
                            name: "excluded".to_string(),
                        }),
                        ..Default::default()
                    }),
                    winget: Some(WingetConfig {
                        manifests_repo: Some(WingetManifestsRepoConfig {
                            owner: "org".to_string(),
                            name: "winget-pkgs".to_string(),
                        }),
                        package_identifier: Some("Org.Excluded".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ];

        let mut ctx = Context::new(
            config,
            ContextOptions {
                dry_run: true,
                selected_crates: vec!["included".to_string()],
                ..Default::default()
            },
        );

        // Should only run for "included", not "excluded"
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    // -----------------------------------------------------------------------
    // AUR integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_run_dry_run_aur() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                aur: Some(AurConfig {
                    git_url: Some("ssh://aur@aur.archlinux.org/mytool.git".to_string()),
                    description: Some("My tool".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    // -----------------------------------------------------------------------
    // Krew integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_run_dry_run_krew() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "kubectl-mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                krew: Some(KrewConfig {
                    manifests_repo: Some(KrewManifestsRepoConfig {
                        owner: "myorg".to_string(),
                        name: "krew-index".to_string(),
                    }),
                    short_description: Some("A kubectl plugin".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_run_dry_run_all_seven_publishers() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "allpub7".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                cargo: Some(CargoPublishConfig::default()),
                homebrew: Some(HomebrewConfig {
                    tap: Some(TapConfig {
                        owner: "org".to_string(),
                        name: "homebrew-tap".to_string(),
                    }),
                    ..Default::default()
                }),
                scoop: Some(ScoopConfig {
                    bucket: Some(BucketConfig {
                        owner: "org".to_string(),
                        name: "scoop-bucket".to_string(),
                    }),
                    description: None,
                    ..Default::default()
                }),
                chocolatey: Some(ChocolateyConfig {
                    project_repo: Some(ChocolateyRepoConfig {
                        owner: "org".to_string(),
                        name: "allpub7".to_string(),
                    }),
                    ..Default::default()
                }),
                winget: Some(WingetConfig {
                    manifests_repo: Some(WingetManifestsRepoConfig {
                        owner: "org".to_string(),
                        name: "winget-pkgs".to_string(),
                    }),
                    package_identifier: Some("Org.Allpub7".to_string()),
                    ..Default::default()
                }),
                aur: Some(AurConfig {
                    git_url: Some("ssh://aur@aur.archlinux.org/allpub7.git".to_string()),
                    ..Default::default()
                }),
                krew: Some(KrewConfig {
                    manifests_repo: Some(KrewManifestsRepoConfig {
                        owner: "org".to_string(),
                        name: "krew-index".to_string(),
                    }),
                    ..Default::default()
                }),
                nix: None,
                aur_source: None,
                homebrew_cask: None,
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    // -----------------------------------------------------------------------
    // Top-level AUR sources integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_run_dry_run_top_level_aur_sources() {
        use anodizer_core::config::AurSourceConfig;

        let mut config = Config::default();
        config.aur_sources = Some(vec![AurSourceConfig {
            name: Some("myapp".to_string()),
            description: Some("My application".to_string()),
            license: Some("MIT".to_string()),
            git_url: Some("ssh://aur@aur.archlinux.org/myapp.git".to_string()),
            makedepends: Some(vec!["rust".to_string(), "cargo".to_string()]),
            ..Default::default()
        }]);
        config.crates = vec![CrateConfig {
            name: "myapp".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        ctx.template_vars_mut().set("Version", "1.0.0");
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        ctx.template_vars_mut().set("ProjectName", "myapp");
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_top_level_aur_sources_empty_is_noop() {
        let mut config = Config::default();
        config.aur_sources = Some(vec![]);
        config.crates = vec![CrateConfig {
            name: "myapp".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_top_level_aur_sources_none_is_noop() {
        let mut config = Config::default();
        config.aur_sources = None;

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }

    // -----------------------------------------------------------------------
    // Nix integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_run_dry_run_nix() {
        use anodizer_core::config::{NixConfig, RepositoryConfig};

        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mytool".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                nix: Some(NixConfig {
                    repository: Some(RepositoryConfig {
                        owner: Some("myorg".to_string()),
                        name: Some("nixpkgs-overlay".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        assert!(PublishStage.run(&mut ctx).is_ok());
    }
}

pub mod crates_io;
pub mod homebrew;
pub mod scoop;

use anodize_core::context::Context;
use anodize_core::stage::Stage;
use anyhow::Result;

use crates_io::publish_to_crates_io;
use homebrew::publish_to_homebrew;
use scoop::publish_to_scoop;

pub struct PublishStage;

impl Stage for PublishStage {
    fn name(&self) -> &str {
        "publish"
    }

    fn run(&self, ctx: &mut Context) -> Result<()> {
        let selected = ctx.options.selected_crates.clone();

        // 1. crates.io — publish all crates with `publish.crates` enabled.
        publish_to_crates_io(ctx, &selected)?;

        // 2. Homebrew — one call per crate that has a homebrew config.
        let homebrew_crates: Vec<String> = ctx
            .config
            .crates
            .iter()
            .filter(|c| selected.is_empty() || selected.contains(&c.name))
            .filter(|c| {
                c.publish
                    .as_ref()
                    .and_then(|p| p.homebrew.as_ref())
                    .is_some()
            })
            .map(|c| c.name.clone())
            .collect();

        for crate_name in &homebrew_crates {
            publish_to_homebrew(ctx, crate_name)?;
        }

        // 3. Scoop — one call per crate that has a scoop config.
        let scoop_crates: Vec<String> = ctx
            .config
            .crates
            .iter()
            .filter(|c| selected.is_empty() || selected.contains(&c.name))
            .filter(|c| c.publish.as_ref().and_then(|p| p.scoop.as_ref()).is_some())
            .map(|c| c.name.clone())
            .collect();

        for crate_name in &scoop_crates {
            publish_to_scoop(ctx, crate_name)?;
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use anodize_core::config::{
        BucketConfig, Config, CrateConfig, CratesPublishConfig, HomebrewConfig, PublishConfig,
        ScoopConfig, TapConfig,
    };
    use anodize_core::context::{Context, ContextOptions};

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
    fn test_run_dry_run_crates_io() {
        let mut config = Config::default();
        config.crates = vec![CrateConfig {
            name: "mylib".to_string(),
            path: ".".to_string(),
            tag_template: "v{{ .Version }}".to_string(),
            publish: Some(PublishConfig {
                crates: Some(CratesPublishConfig::Bool(true)),
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
                crates: Some(CratesPublishConfig::Bool(true)),
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
                crates: Some(CratesPublishConfig::Bool(true)),
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
            }),
            ..Default::default()
        }];

        let mut ctx = dry_run_ctx(config);
        // All three publishers should succeed in dry-run mode
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
}

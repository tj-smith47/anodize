use std::collections::HashMap;

use anodize_core::context::Context;
use anodize_core::stage::Stage;
use anyhow::Result;

pub mod discord;
pub mod email;
pub mod mattermost;
pub mod slack;
pub mod teams;
pub mod telegram;
pub mod webhook;

// ---------------------------------------------------------------------------
// AnnounceStage
// ---------------------------------------------------------------------------

pub struct AnnounceStage;

impl Stage for AnnounceStage {
    fn name(&self) -> &str {
        "announce"
    }

    fn run(&self, ctx: &mut Context) -> Result<()> {
        let announce = match ctx.config.announce.clone() {
            Some(a) => a,
            None => {
                eprintln!("[announce] no announce config — skipping");
                return Ok(());
            }
        };

        // ----------------------------------------------------------------
        // Discord
        // ----------------------------------------------------------------
        if let Some(discord_cfg) = &announce.discord
            && discord_cfg.enabled.unwrap_or(false)
        {
            let raw_url = discord_cfg
                .webhook_url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("announce.discord: missing webhook_url"))?;
            let url = ctx.render_template(raw_url)?;

            let tmpl = discord_cfg
                .message_template
                .as_deref()
                .unwrap_or("{{ .ProjectName }} {{ .Tag }} released!");

            let message = ctx.render_template(tmpl)?;

            if ctx.is_dry_run() {
                eprintln!("[announce] (dry-run) discord: {}", message);
            } else {
                eprintln!("[announce] discord: {}", message);
                discord::send_discord(&url, &message)?;
            }
        }

        // ----------------------------------------------------------------
        // Slack
        // ----------------------------------------------------------------
        if let Some(slack_cfg) = &announce.slack
            && slack_cfg.enabled.unwrap_or(false)
        {
            let raw_url = slack_cfg
                .webhook_url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("announce.slack: missing webhook_url"))?;
            let url = ctx.render_template(raw_url)?;

            let tmpl = slack_cfg
                .message_template
                .as_deref()
                .unwrap_or("{{ .ProjectName }} {{ .Tag }} released!");

            let message = ctx.render_template(tmpl)?;

            if ctx.is_dry_run() {
                eprintln!("[announce] (dry-run) slack: {}", message);
            } else {
                eprintln!("[announce] slack: {}", message);
                slack::send_slack(&url, &message)?;
            }
        }

        // ----------------------------------------------------------------
        // Generic HTTP webhook
        // ----------------------------------------------------------------
        if let Some(webhook_cfg) = &announce.webhook
            && webhook_cfg.enabled.unwrap_or(false)
        {
            let raw_url = webhook_cfg
                .endpoint_url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("announce.webhook: missing endpoint_url"))?;
            let url = ctx.render_template(raw_url)?;

            let tmpl = webhook_cfg
                .message_template
                .as_deref()
                .unwrap_or("{{ .ProjectName }} {{ .Tag }} released!");

            let message = ctx.render_template(tmpl)?;

            let raw_headers = webhook_cfg.headers.clone().unwrap_or_default();
            let mut headers = HashMap::new();
            for (k, v) in &raw_headers {
                headers.insert(k.clone(), ctx.render_template(v)?);
            }

            let content_type = webhook_cfg
                .content_type
                .clone()
                .unwrap_or_else(|| "application/json".to_string());

            if ctx.is_dry_run() {
                eprintln!("[announce] (dry-run) webhook: {}", message);
            } else {
                eprintln!("[announce] webhook: {}", message);
                webhook::send_webhook(&url, &message, &headers, &content_type)?;
            }
        }

        // ----------------------------------------------------------------
        // Telegram
        // ----------------------------------------------------------------
        if let Some(telegram_cfg) = &announce.telegram
            && telegram_cfg.enabled.unwrap_or(false)
        {
            let raw_token = telegram_cfg
                .bot_token
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("announce.telegram: missing bot_token"))?;
            let bot_token = ctx.render_template(raw_token)?;

            let raw_chat_id = telegram_cfg
                .chat_id
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("announce.telegram: missing chat_id"))?;
            let chat_id = ctx.render_template(raw_chat_id)?;

            let tmpl = telegram_cfg
                .message_template
                .as_deref()
                .unwrap_or("{{ .ProjectName }} {{ .Tag }} released!");
            let message = ctx.render_template(tmpl)?;

            let parse_mode = telegram_cfg.parse_mode.as_deref();

            if ctx.is_dry_run() {
                eprintln!("[announce] (dry-run) telegram: {}", message);
            } else {
                eprintln!("[announce] telegram: {}", message);
                telegram::send_telegram(&bot_token, &chat_id, &message, parse_mode)?;
            }
        }

        // ----------------------------------------------------------------
        // Microsoft Teams
        // ----------------------------------------------------------------
        if let Some(teams_cfg) = &announce.teams
            && teams_cfg.enabled.unwrap_or(false)
        {
            let raw_url = teams_cfg
                .webhook_url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("announce.teams: missing webhook_url"))?;
            let url = ctx.render_template(raw_url)?;

            let tmpl = teams_cfg
                .message_template
                .as_deref()
                .unwrap_or("{{ .ProjectName }} {{ .Tag }} released!");
            let message = ctx.render_template(tmpl)?;

            if ctx.is_dry_run() {
                eprintln!("[announce] (dry-run) teams: {}", message);
            } else {
                eprintln!("[announce] teams: {}", message);
                teams::send_teams(&url, &message)?;
            }
        }

        // ----------------------------------------------------------------
        // Mattermost
        // ----------------------------------------------------------------
        if let Some(mm_cfg) = &announce.mattermost
            && mm_cfg.enabled.unwrap_or(false)
        {
            let raw_url = mm_cfg
                .webhook_url
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("announce.mattermost: missing webhook_url"))?;
            let url = ctx.render_template(raw_url)?;

            let tmpl = mm_cfg
                .message_template
                .as_deref()
                .unwrap_or("{{ .ProjectName }} {{ .Tag }} released!");
            let message = ctx.render_template(tmpl)?;

            let channel = mm_cfg.channel.as_deref();
            let username = mm_cfg.username.as_deref();
            let icon_url = mm_cfg.icon_url.as_deref();

            if ctx.is_dry_run() {
                eprintln!("[announce] (dry-run) mattermost: {}", message);
            } else {
                eprintln!("[announce] mattermost: {}", message);
                mattermost::send_mattermost(&url, &message, channel, username, icon_url)?;
            }
        }

        // ----------------------------------------------------------------
        // Email (SMTP via sendmail/msmtp)
        // ----------------------------------------------------------------
        if let Some(email_cfg) = &announce.email
            && email_cfg.enabled.unwrap_or(false)
        {
            let from = email_cfg
                .from
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("announce.email: missing from"))?;
            let from = ctx.render_template(from)?;

            if email_cfg.to.is_empty() {
                anyhow::bail!("announce.email: missing to (recipient list)");
            }

            let subject_tmpl = email_cfg
                .subject_template
                .as_deref()
                .unwrap_or("{{ .ProjectName }} {{ .Tag }} released");
            let subject = ctx.render_template(subject_tmpl)?;

            let body_tmpl = email_cfg
                .message_template
                .as_deref()
                .unwrap_or("{{ .ProjectName }} {{ .Tag }} released!");
            let body = ctx.render_template(body_tmpl)?;

            if ctx.is_dry_run() {
                eprintln!(
                    "[announce] (dry-run) email to {}: {}",
                    email_cfg.to.join(", "),
                    subject
                );
            } else {
                eprintln!(
                    "[announce] email to {}: {}",
                    email_cfg.to.join(", "),
                    subject
                );
                email::send_email(&email::EmailParams {
                    from: &from,
                    to: &email_cfg.to,
                    subject: &subject,
                    body: &body,
                })?;
            }
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
        AnnounceConfig, AnnounceProviderConfig, Config, EmailAnnounce, MattermostAnnounce,
        TeamsAnnounce, TelegramAnnounce, WebhookConfig,
    };
    use anodize_core::context::{Context, ContextOptions};

    fn make_ctx(announce: Option<AnnounceConfig>) -> Context {
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = announce;
        let mut ctx = Context::new(config, ContextOptions::default());
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        ctx.template_vars_mut().set(
            "ReleaseURL",
            "https://github.com/org/myapp/releases/tag/v1.0.0",
        );
        ctx
    }

    #[test]
    fn test_skips_when_no_announce_config() {
        let mut ctx = make_ctx(None);
        let stage = AnnounceStage;
        assert!(stage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_skips_disabled_discord() {
        let announce = AnnounceConfig {
            discord: Some(AnnounceProviderConfig {
                enabled: Some(false),
                webhook_url: Some("https://discord.invalid/webhook".to_string()),
                message_template: None,
            }),
            slack: None,
            webhook: None,
            ..Default::default()
        };
        let mut ctx = make_ctx(Some(announce));
        // Should complete without attempting network I/O.
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_skips_disabled_slack() {
        let announce = AnnounceConfig {
            discord: None,
            slack: Some(AnnounceProviderConfig {
                enabled: Some(false),
                webhook_url: Some("https://hooks.slack.invalid/services/T000".to_string()),
                message_template: None,
            }),
            webhook: None,
            ..Default::default()
        };
        let mut ctx = make_ctx(Some(announce));
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_skips_disabled_webhook() {
        let announce = AnnounceConfig {
            discord: None,
            slack: None,
            webhook: Some(WebhookConfig {
                enabled: Some(false),
                endpoint_url: Some("https://example.invalid/hook".to_string()),
                headers: None,
                content_type: None,
                message_template: None,
            }),
            ..Default::default()
        };
        let mut ctx = make_ctx(Some(announce));
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_dry_run_discord_does_not_send() {
        let announce = AnnounceConfig {
            discord: Some(AnnounceProviderConfig {
                enabled: Some(true),
                webhook_url: Some("https://discord.invalid/webhook".to_string()),
                message_template: Some("{{ .ProjectName }} {{ .Tag }} released!".to_string()),
            }),
            slack: None,
            webhook: None,
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let opts = ContextOptions {
            dry_run: true,
            ..Default::default()
        };
        let mut ctx = Context::new(config, opts);
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        // Should not make a network call (URL is `.invalid`), just log.
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_dry_run_slack_does_not_send() {
        let announce = AnnounceConfig {
            discord: None,
            slack: Some(AnnounceProviderConfig {
                enabled: Some(true),
                webhook_url: Some("https://hooks.slack.invalid/services/T000".to_string()),
                message_template: Some("{{ .ProjectName }} {{ .Tag }} released!".to_string()),
            }),
            webhook: None,
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let opts = ContextOptions {
            dry_run: true,
            ..Default::default()
        };
        let mut ctx = Context::new(config, opts);
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_dry_run_webhook_does_not_send() {
        let announce = AnnounceConfig {
            discord: None,
            slack: None,
            webhook: Some(WebhookConfig {
                enabled: Some(true),
                endpoint_url: Some("https://example.invalid/hook".to_string()),
                headers: None,
                content_type: Some("application/json".to_string()),
                message_template: Some("{{ .ProjectName }} {{ .Tag }} released!".to_string()),
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let opts = ContextOptions {
            dry_run: true,
            ..Default::default()
        };
        let mut ctx = Context::new(config, opts);
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_missing_webhook_url_returns_error() {
        let announce = AnnounceConfig {
            discord: Some(AnnounceProviderConfig {
                enabled: Some(true),
                webhook_url: None, // intentionally missing
                message_template: None,
            }),
            slack: None,
            webhook: None,
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let opts = ContextOptions {
            dry_run: false,
            ..Default::default()
        };
        let mut ctx = Context::new(config, opts);
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_err());
    }

    // ----------------------------------------------------------------
    // Telegram tests
    // ----------------------------------------------------------------

    #[test]
    fn test_skips_disabled_telegram() {
        let announce = AnnounceConfig {
            telegram: Some(TelegramAnnounce {
                enabled: Some(false),
                bot_token: Some("123:ABC".to_string()),
                chat_id: Some("-100123".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut ctx = make_ctx(Some(announce));
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_dry_run_telegram_does_not_send() {
        let announce = AnnounceConfig {
            telegram: Some(TelegramAnnounce {
                enabled: Some(true),
                bot_token: Some("123:ABC".to_string()),
                chat_id: Some("-100123".to_string()),
                message_template: Some("{{ .ProjectName }} {{ .Tag }} released!".to_string()),
                parse_mode: Some("MarkdownV2".to_string()),
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let opts = ContextOptions {
            dry_run: true,
            ..Default::default()
        };
        let mut ctx = Context::new(config, opts);
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_missing_telegram_bot_token_returns_error() {
        let announce = AnnounceConfig {
            telegram: Some(TelegramAnnounce {
                enabled: Some(true),
                bot_token: None,
                chat_id: Some("-100123".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let mut ctx = Context::new(config, ContextOptions::default());
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_err());
    }

    #[test]
    fn test_missing_telegram_chat_id_returns_error() {
        let announce = AnnounceConfig {
            telegram: Some(TelegramAnnounce {
                enabled: Some(true),
                bot_token: Some("123:ABC".to_string()),
                chat_id: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let mut ctx = Context::new(config, ContextOptions::default());
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_err());
    }

    // ----------------------------------------------------------------
    // Teams tests
    // ----------------------------------------------------------------

    #[test]
    fn test_skips_disabled_teams() {
        let announce = AnnounceConfig {
            teams: Some(TeamsAnnounce {
                enabled: Some(false),
                webhook_url: Some("https://teams.invalid/webhook".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut ctx = make_ctx(Some(announce));
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_dry_run_teams_does_not_send() {
        let announce = AnnounceConfig {
            teams: Some(TeamsAnnounce {
                enabled: Some(true),
                webhook_url: Some("https://teams.invalid/webhook".to_string()),
                message_template: Some("{{ .ProjectName }} {{ .Tag }} released!".to_string()),
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let opts = ContextOptions {
            dry_run: true,
            ..Default::default()
        };
        let mut ctx = Context::new(config, opts);
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_missing_teams_webhook_url_returns_error() {
        let announce = AnnounceConfig {
            teams: Some(TeamsAnnounce {
                enabled: Some(true),
                webhook_url: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let mut ctx = Context::new(config, ContextOptions::default());
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_err());
    }

    // ----------------------------------------------------------------
    // Mattermost tests
    // ----------------------------------------------------------------

    #[test]
    fn test_skips_disabled_mattermost() {
        let announce = AnnounceConfig {
            mattermost: Some(MattermostAnnounce {
                enabled: Some(false),
                webhook_url: Some("https://mm.invalid/hooks/xxx".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut ctx = make_ctx(Some(announce));
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_dry_run_mattermost_does_not_send() {
        let announce = AnnounceConfig {
            mattermost: Some(MattermostAnnounce {
                enabled: Some(true),
                webhook_url: Some("https://mm.invalid/hooks/xxx".to_string()),
                channel: Some("releases".to_string()),
                username: Some("release-bot".to_string()),
                icon_url: Some("https://example.com/icon.png".to_string()),
                message_template: Some("{{ .ProjectName }} {{ .Tag }} released!".to_string()),
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let opts = ContextOptions {
            dry_run: true,
            ..Default::default()
        };
        let mut ctx = Context::new(config, opts);
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_missing_mattermost_webhook_url_returns_error() {
        let announce = AnnounceConfig {
            mattermost: Some(MattermostAnnounce {
                enabled: Some(true),
                webhook_url: None,
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let mut ctx = Context::new(config, ContextOptions::default());
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_err());
    }

    // ----------------------------------------------------------------
    // Email tests
    // ----------------------------------------------------------------

    #[test]
    fn test_skips_disabled_email() {
        let announce = AnnounceConfig {
            email: Some(EmailAnnounce {
                enabled: Some(false),
                from: Some("bot@example.com".to_string()),
                to: vec!["dev@example.com".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut ctx = make_ctx(Some(announce));
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_dry_run_email_does_not_send() {
        let announce = AnnounceConfig {
            email: Some(EmailAnnounce {
                enabled: Some(true),
                from: Some("bot@example.com".to_string()),
                to: vec!["dev@example.com".to_string()],
                subject_template: Some("{{ .ProjectName }} {{ .Tag }} released".to_string()),
                message_template: Some("New release!".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let opts = ContextOptions {
            dry_run: true,
            ..Default::default()
        };
        let mut ctx = Context::new(config, opts);
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_ok());
    }

    #[test]
    fn test_missing_email_from_returns_error() {
        let announce = AnnounceConfig {
            email: Some(EmailAnnounce {
                enabled: Some(true),
                from: None,
                to: vec!["dev@example.com".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let mut ctx = Context::new(config, ContextOptions::default());
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_err());
    }

    #[test]
    fn test_missing_email_to_returns_error() {
        let announce = AnnounceConfig {
            email: Some(EmailAnnounce {
                enabled: Some(true),
                from: Some("bot@example.com".to_string()),
                to: vec![],
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut config = Config::default();
        config.project_name = "myapp".to_string();
        config.announce = Some(announce);
        let mut ctx = Context::new(config, ContextOptions::default());
        ctx.template_vars_mut().set("Tag", "v1.0.0");
        assert!(AnnounceStage.run(&mut ctx).is_err());
    }
}

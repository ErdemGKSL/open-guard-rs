use crate::db::entities::module_configs::{self, BotAddingProtectionModuleConfig, ModuleType};
use crate::services::logger::LogLevel;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;
use serenity::model::guild::audit_log::{Action, MemberAction};

pub async fn handle_audit_log(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Error> {
    let config_model = match module_configs::Entity::find_by_id((
        guild_id.get() as i64,
        ModuleType::BotAddingProtection,
    ))
    .one(&data.db)
    .await?
    {
        Some(m) => {
            if !m.enabled {
                return Ok(());
            }
            m
        }
        None => return Ok(()),
    };

    let _config: BotAddingProtectionModuleConfig =
        serde_json::from_value(config_model.config.clone()).unwrap_or_default();

    let user_id = match entry.user_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // Ignore actions by the bot itself
    if user_id == ctx.cache.current_user().id {
        return Ok(());
    }

    // Check whitelist
    let whitelist_level = data
        .whitelist
        .get_whitelist_level(ctx, guild_id, user_id, ModuleType::BotAddingProtection)
        .await?;

    match entry.action {
        Action::Member(MemberAction::BotAdd) => {
            handle_bot_add(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
            )
            .await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_bot_add(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
) -> Result<(), Error> {
    let bot_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    if bot_id == 0 {
        return Ok(());
    }

    let guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(g) => g,
        Err(_) => return Ok(()),
    };
    let l10n = data.l10n.get_proxy(&guild.preferred_locale.to_string());

    let mut status = if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        l10n.t("log-status-whitelisted", Some(&args))
    } else {
        l10n.t("log-status-unauthorized", None)
    };

    if whitelist_level.is_none() {
        // Punishment for the user who added the bot
        let reason = l10n.t("log-bot-add-reason", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::BotAddingProtection,
                &reason,
            )
            .await?;

        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => {
                let mut args = fluent::FluentArgs::new();
                args.set("type", format!("{:?}", p));
                l10n.t("log-status-punished", Some(&args))
            }
            crate::services::punishment::ViolationResult::ViolationRecorded {
                current,
                threshold,
            } => {
                let mut args = fluent::FluentArgs::new();
                args.set("current", current);
                args.set("threshold", threshold);
                l10n.t("log-status-violation", Some(&args))
            }
            crate::services::punishment::ViolationResult::None => {
                l10n.t("log-status-blocked", None)
            }
        };

        // Revert (Kick the added bot)
        if config.revert {
            let revert_reason = l10n.t("log-bot-add-revert-reason", None);
            if guild_id
                .kick(
                    &ctx.http,
                    serenity::UserId::new(bot_id),
                    Some(&revert_reason),
                )
                .await
                .is_ok()
            {
                status += &l10n.t("log-status-reverted", None);
            } else {
                status += &l10n.t("log-status-revert-failed", None);
            }
        }
    } else if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        args.set("punishment", format!("{:?}", config.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-bot-add-title-whitelisted", None)
    } else {
        l10n.t("log-bot-add-title-blocked", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else {
        LogLevel::Warn
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("botId", bot_id.to_string());
    desc_args.set("userId", user_id.get().to_string());
    let desc = l10n.t("log-bot-add-desc", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::BotAddingProtection),
            log_level,
            &title,
            &desc,
            vec![
                (
                    &l10n.t("log-field-acting-user", None),
                    format!("<@{}>", user_id),
                ),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

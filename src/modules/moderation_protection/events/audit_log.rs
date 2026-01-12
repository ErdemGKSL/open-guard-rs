use crate::db::entities::module_configs::{self, ModerationProtectionModuleConfig, ModuleType};
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
    // Check action type first to avoid unnecessary database calls
    if !matches!(
        entry.action,
        Action::Member(MemberAction::BanAdd)
            | Action::Member(MemberAction::BanRemove)
            | Action::Member(MemberAction::Kick)
            | Action::Member(MemberAction::Update)
    ) {
        return Ok(());
    }

    // Fetch module config
    let config_model = module_configs::Entity::find_by_id((
        guild_id.get() as i64,
        ModuleType::ModerationProtection,
    ))
    .one(&data.db)
    .await?;

    let (config_model, config) = match config_model {
        Some(m) => {
            if !m.enabled {
                return Ok(());
            }
            let config: ModerationProtectionModuleConfig =
                serde_json::from_value(m.config.clone()).unwrap_or_default();
            (m, config)
        }
        None => return Ok(()), // Module not configured for this guild
    };

    let user_id = match entry.user_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // Ignore actions by the bot itself (commands bypass)
    if user_id == ctx.cache.current_user().id {
        return Ok(());
    }

    // Check whitelist
    let whitelist_level = data
        .whitelist
        .get_whitelist_level(ctx, guild_id, user_id, ModuleType::ModerationProtection)
        .await?;

    match entry.action {
        Action::Member(MemberAction::BanAdd) => {
            handle_moderation_action(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
                &config,
                "ban",
            )
            .await?;
        }
        Action::Member(MemberAction::Kick) => {
            handle_moderation_action(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
                &config,
                "kick",
            )
            .await?;
        }
        Action::Member(MemberAction::Update) => {
            // Check for timeout changes
            let has_timeout_change = entry.changes.iter().any(|change| {
                matches!(
                    change,
                    serenity::model::guild::audit_log::Change::CommunicationDisabledUntil { .. }
                )
            });
            if has_timeout_change {
                handle_moderation_action(
                    ctx,
                    entry,
                    guild_id,
                    data,
                    &config_model,
                    user_id,
                    whitelist_level,
                    &config,
                    "timeout",
                )
                .await?;
            }
        }
        Action::Member(MemberAction::BanRemove) => {
            handle_moderation_action(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
                &config,
                "unban",
            )
            .await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_moderation_action(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &ModerationProtectionModuleConfig,
    action_type: &str,
) -> Result<(), Error> {
    let target_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    let should_punish =
        config.punish_when.is_empty() || config.punish_when.contains(&action_type.to_string());

    let guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(g) => g,
        Err(_) => return Ok(()),
    };
    let l10n = data.l10n.get_proxy(&guild.preferred_locale.to_string());

    let mut status = if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        l10n.t("log-status-whitelisted", Some(&args))
    } else if !should_punish {
        l10n.t("log-status-not-enabled", None)
    } else {
        l10n.t("log-status-unauthorized", None)
    };

    if whitelist_level.is_none() && should_punish {
        let reason = format!("Moderation Limit Exceeded: {}", action_type);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ModerationProtection,
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

        // DM Warning logic
        match result {
            crate::services::punishment::ViolationResult::ViolationRecorded {
                current,
                threshold,
            } => {
                // threshold is when someone is PUNISHED.
                // so threshold - current - 1 is how many "safe" actions are left.
                let remaining_safe = threshold - current - 1;

                if remaining_safe == 2 {
                    let _ = user_id
                        .direct_message(
                            &ctx.http,
                            serenity::CreateMessage::new()
                                .content(l10n.t("mod-warn-remaining-2", None)),
                        )
                        .await;
                } else if remaining_safe == 1 {
                    let _ = user_id
                        .direct_message(
                            &ctx.http,
                            serenity::CreateMessage::new()
                                .content(l10n.t("mod-warn-remaining-1", None)),
                        )
                        .await;
                } else if remaining_safe == 0 {
                    let mut warn_args = fluent::FluentArgs::new();
                    warn_args.set("punishment", format!("{:?}", config_model.punishment));
                    let msg = l10n.t("mod-warn-limit-reached", Some(&warn_args));
                    let _ = user_id
                        .direct_message(&ctx.http, serenity::CreateMessage::new().content(msg))
                        .await;
                }
            }
            _ => {}
        }

        // Revert ONLY if limit is exceeded (Punished)
        if matches!(
            result,
            crate::services::punishment::ViolationResult::Punished(_)
        ) && config_model.revert
            && target_id != 0
        {
            let revert_reason = format!("Moderation Protection Revert: {}", action_type);
            let revert_success = match action_type {
                "ban" => guild_id
                    .unban(
                        &ctx.http,
                        serenity::UserId::new(target_id),
                        Some(&revert_reason),
                    )
                    .await
                    .is_ok(),
                "timeout" => {
                    if let Ok(mut member) = guild_id
                        .member(&ctx.http, serenity::UserId::new(target_id))
                        .await
                    {
                        member.enable_communication(&ctx.http).await.is_ok()
                    } else {
                        false
                    }
                }
                "kick" => false, // Cannot revert a kick
                _ => false,
            };

            if revert_success {
                status += &l10n.t("log-status-reverted", None);
            } else if action_type != "kick" {
                status += &l10n.t("log-status-revert-failed", None);
            }
        }
    }

    // Logging...
    let is_whitelisted = whitelist_level.is_some();
    let mut title_args = fluent::FluentArgs::new();
    title_args.set("action", action_type);

    let title = if is_whitelisted {
        l10n.t("log-mod-audit-title-whitelisted", Some(&title_args))
    } else if should_punish {
        l10n.t("log-mod-audit-title-limited", Some(&title_args))
    } else {
        l10n.t("log-mod-audit-title-logged", Some(&title_args))
    };

    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else if should_punish {
        LogLevel::Warn
    } else {
        LogLevel::Info
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("userId", user_id.get());
    desc_args.set("action", action_type);
    desc_args.set("targetId", target_id);

    let desc = l10n.t("log-mod-audit-desc", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ModerationProtection),
            log_level,
            &title,
            &desc,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id)),
                (
                    &l10n.t("log-field-target-member", None),
                    format!("<@{}>", target_id),
                ),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

use crate::db::entities::module_configs::{
    self, ChannelPermissionProtectionModuleConfig, ModuleType,
};
use crate::services::logger::LogLevel;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;
use serenity::model::guild::audit_log::{Action, ChannelOverwriteAction};

pub async fn handle_audit_log(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Error> {
    // Fetch module config
    let config_model = module_configs::Entity::find_by_id((
        guild_id.get() as i64,
        ModuleType::ChannelPermissionProtection,
    ))
    .one(&data.db)
    .await?;

    let config_model = match config_model {
        Some(m) => {
            if !m.enabled {
                return Ok(());
            }
            m
        }
        None => return Ok(()), // Module not configured for this guild
    };

    let config: ChannelPermissionProtectionModuleConfig =
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
        .get_whitelist_level(
            ctx,
            guild_id,
            user_id,
            ModuleType::ChannelPermissionProtection,
        )
        .await?;

    // Check if we should ignore private channels (ownership check)
    if config.ignore_private_channels {
        let channel_id = entry.options.as_ref().and_then(|o| o.channel_id);
        if let Some(channel_id) = channel_id {
            if let Ok(serenity::Channel::Guild(channel)) = ctx.http.get_channel(channel_id).await {
                let is_owner = channel.permission_overwrites.iter().any(|overwrite| {
                    if let serenity::PermissionOverwriteType::Member(id) = overwrite.kind {
                        id == user_id
                            && overwrite
                                .allow
                                .contains(serenity::Permissions::MANAGE_ROLES)
                    } else {
                        false
                    }
                });

                if is_owner {
                    return Ok(());
                }
            }
        }
    }

    // Match on the audit log action to triggers variants error
    match entry.action {
        Action::ChannelOverwrite(ChannelOverwriteAction::Create) => {
            handle_overwrite_create(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
                &config,
            )
            .await?;
        }
        Action::ChannelOverwrite(ChannelOverwriteAction::Delete) => {
            handle_overwrite_delete(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
                &config,
            )
            .await?;
        }
        Action::ChannelOverwrite(ChannelOverwriteAction::Update) => {
            handle_overwrite_update(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
                &config,
            )
            .await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_overwrite_create(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &ChannelPermissionProtectionModuleConfig,
) -> Result<(), Error> {
    let channel_id = entry
        .options
        .as_ref()
        .and_then(|o| o.channel_id)
        .map(|id| id.get())
        .unwrap_or(0);
    let target_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    let should_punish =
        config.punish_when.is_empty() || config.punish_when.contains(&"create".to_string());

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
        // Punishment
        let reason = l10n.t("log-chan-perm-reason-create", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelPermissionProtection,
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

        // Revert
        if config_model.revert && channel_id != 0 && target_id != 0 {
            let revert_reason = l10n.t("log-chan-perm-revert-reason", None);
            if ctx
                .http
                .delete_permission(
                    serenity::ChannelId::new(channel_id),
                    serenity::TargetId::new(target_id),
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
        args.set("punishment", format!("{:?}", config_model.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-chan-perm-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-chan-perm-title-blocked", None)
    } else {
        l10n.t("log-chan-perm-title-logged", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else if should_punish {
        LogLevel::Warn
    } else {
        LogLevel::Info
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("channelId", channel_id);
    desc_args.set("userId", user_id.get());
    let desc = l10n.t("log-chan-perm-desc-create", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelPermissionProtection),
            log_level,
            &title,
            &desc,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id)),
                (
                    &l10n.t("log-field-channel", None),
                    format!("<#{}>", channel_id),
                ),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

async fn handle_overwrite_delete(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &ChannelPermissionProtectionModuleConfig,
) -> Result<(), Error> {
    let channel_id = entry
        .options
        .as_ref()
        .and_then(|o| o.channel_id)
        .map(|id| id.get())
        .unwrap_or(0);
    let target_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    let should_punish =
        config.punish_when.is_empty() || config.punish_when.contains(&"delete".to_string());

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
        // Punishment
        let reason = l10n.t("log-chan-perm-reason-delete", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelPermissionProtection,
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

        // Revert
        if config_model.revert && channel_id != 0 && target_id != 0 {
            let mut allow = serenity::Permissions::empty();
            let mut deny = serenity::Permissions::empty();

            for change in &entry.changes {
                match change {
                    serenity::model::guild::audit_log::Change::Allow { old, .. } => {
                        if let Some(p) = old {
                            allow = *p;
                        }
                    }
                    serenity::model::guild::audit_log::Change::Deny { old, .. } => {
                        if let Some(p) = old {
                            deny = *p;
                        }
                    }
                    _ => {}
                }
            }

            let kind_num = if let Some(options) = &entry.options {
                match options.kind.as_ref().map(|s| s.as_str()) {
                    Some("role") => 0,
                    Some("member") => 1,
                    _ => 0,
                }
            } else {
                0
            };

            let map = serde_json::json!({
                "allow": allow.bits(),
                "deny": deny.bits(),
                "type": kind_num,
            });

            let revert_reason = l10n.t("log-chan-perm-revert-reason", None);
            if ctx
                .http
                .create_permission(
                    serenity::ChannelId::new(channel_id),
                    serenity::TargetId::new(target_id),
                    &map,
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
        args.set("punishment", format!("{:?}", config_model.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-chan-perm-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-chan-perm-title-blocked", None)
    } else {
        l10n.t("log-chan-perm-title-logged", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else if should_punish {
        LogLevel::Error
    } else {
        LogLevel::Info
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("channelId", channel_id);
    desc_args.set("userId", user_id.get());
    let desc = l10n.t("log-chan-perm-desc-delete", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelPermissionProtection),
            log_level,
            &title,
            &desc,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id)),
                (
                    &l10n.t("log-field-channel", None),
                    format!("<#{}>", channel_id),
                ),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

async fn handle_overwrite_update(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &ChannelPermissionProtectionModuleConfig,
) -> Result<(), Error> {
    let channel_id = entry
        .options
        .as_ref()
        .and_then(|o| o.channel_id)
        .map(|id| id.get())
        .unwrap_or(0);
    let target_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    let should_punish =
        config.punish_when.is_empty() || config.punish_when.contains(&"update".to_string());

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
        // Punishment
        let reason = l10n.t("log-chan-perm-reason-update", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelPermissionProtection,
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

        // Revert
        if config_model.revert && channel_id != 0 && target_id != 0 {
            let mut allow = None;
            let mut deny = None;

            for change in &entry.changes {
                match change {
                    serenity::model::guild::audit_log::Change::Allow { old, .. } => {
                        allow = *old;
                    }
                    serenity::model::guild::audit_log::Change::Deny { old, .. } => {
                        deny = *old;
                    }
                    _ => {}
                }
            }

            if allow.is_some() || deny.is_some() {
                let channel = ctx
                    .http
                    .get_channel(serenity::GenericChannelId::new(channel_id))
                    .await;
                if let Ok(serenity::Channel::Guild(channel)) = channel {
                    let current_overwrite =
                        channel.permission_overwrites.iter().find(|o| match o.kind {
                            serenity::PermissionOverwriteType::Role(id) => id.get() == target_id,
                            serenity::PermissionOverwriteType::Member(id) => id.get() == target_id,
                            _ => false,
                        });

                    let final_allow = allow.unwrap_or_else(|| {
                        current_overwrite
                            .map(|o| o.allow)
                            .unwrap_or_else(serenity::Permissions::empty)
                    });
                    let final_deny = deny.unwrap_or_else(|| {
                        current_overwrite
                            .map(|o| o.deny)
                            .unwrap_or_else(serenity::Permissions::empty)
                    });

                    let kind_num = if let Some(options) = &entry.options {
                        match options.kind.as_ref().map(|s| s.as_str()) {
                            Some("role") => 0,
                            Some("member") => 1,
                            _ => 0,
                        }
                    } else {
                        0
                    };

                    let map = serde_json::json!({
                        "allow": final_allow.bits(),
                        "deny": final_deny.bits(),
                        "type": kind_num,
                    });

                    let revert_reason = l10n.t("log-chan-perm-revert-reason", None);
                    if ctx
                        .http
                        .create_permission(
                            serenity::ChannelId::new(channel_id),
                            serenity::TargetId::new(target_id),
                            &map,
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
            } else {
                status += &l10n.t("log-status-no-revert", None);
            }
        }
    } else if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        args.set("punishment", format!("{:?}", config_model.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-chan-perm-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-chan-perm-title-blocked", None)
    } else {
        l10n.t("log-chan-perm-title-logged", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else if should_punish {
        LogLevel::Info
    } else {
        LogLevel::Info
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("channelId", channel_id);
    desc_args.set("userId", user_id.get());
    let desc = l10n.t("log-chan-perm-desc-update", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelPermissionProtection),
            log_level,
            &title,
            &desc,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id)),
                (
                    &l10n.t("log-field-channel", None),
                    format!("<#{}>", channel_id),
                ),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

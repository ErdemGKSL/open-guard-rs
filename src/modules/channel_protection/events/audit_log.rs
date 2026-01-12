use crate::db::entities::module_configs::{self, ChannelProtectionModuleConfig, ModuleType};
use crate::services::logger::LogLevel;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;
use serenity::model::guild::audit_log::{Action, ChannelAction};

pub async fn handle_audit_log(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Error> {
    // Fetch module config
    let config_model =
        module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::ChannelProtection))
            .one(&data.db)
            .await?;

    let (config_model, config) = match config_model {
        Some(m) => {
            if !m.enabled {
                return Ok(());
            }
            let config: ChannelProtectionModuleConfig =
                serde_json::from_value(m.config.clone()).unwrap_or_default();
            (m, config)
        }
        None => return Ok(()), // Module not configured for this guild
    };

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
        .get_whitelist_level(ctx, guild_id, user_id, ModuleType::ChannelProtection)
        .await?;

    // Check if we should ignore private channels (ownership check)
    if config.ignore_private_channels {
        if let Some(target_id) = entry.target_id {
            let channel_id = serenity::ChannelId::new(target_id.get());
            // We check the HTTP for the channel to check overwrites.
            if let Ok(serenity::Channel::Guild(channel)) =
                ctx.http.get_channel(channel_id.into()).await
            {
                let is_owner = channel.permission_overwrites.iter().any(|overwrite| {
                    if let serenity::PermissionOverwriteType::Member(id) = overwrite.kind {
                        id == user_id
                            && overwrite
                                .allow
                                .contains(serenity::Permissions::MANAGE_CHANNELS)
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
        Action::Channel(ChannelAction::Create) => {
            handle_channel_create(
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
        Action::Channel(ChannelAction::Delete) => {
            handle_channel_delete(
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
        Action::Channel(ChannelAction::Update) => {
            handle_channel_update(
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

async fn handle_channel_create(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &ChannelProtectionModuleConfig,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
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
        let reason = l10n.t("log-channel-reason-create", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelProtection,
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
        if config_model.revert && channel_id != 0 {
            let revert_reason = l10n.t("log-channel-revert-reason", None);
            if ctx
                .http
                .delete_channel(
                    serenity::GenericChannelId::new(channel_id),
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
        l10n.t("log-channel-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-channel-title-blocked", None)
    } else {
        l10n.t("log-channel-title-logged", None)
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
    let desc = l10n.t("log-channel-desc-create", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
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

async fn handle_channel_delete(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &ChannelProtectionModuleConfig,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
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
        let reason = l10n.t("log-channel-reason-delete", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelProtection,
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
        if config_model.revert {
            // Wait for the channel to be stored in cache
            let mut cached_channel = None;
            for _ in 0..10 {
                if let Some(c) = data
                    .cache
                    .take_channel(guild_id, serenity::ChannelId::new(channel_id))
                {
                    cached_channel = Some(c);
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }

            if let Some(channel) = cached_channel {
                let mut create_channel = serenity::CreateChannel::new(channel.base.name.clone())
                    .kind(channel.base.kind)
                    .permissions(channel.permission_overwrites.clone());

                if let Some(id) = channel.parent_id {
                    create_channel = create_channel.category(id);
                }

                if let Some(ref topic) = channel.topic {
                    create_channel = create_channel.topic(topic);
                }

                create_channel = create_channel.nsfw(channel.nsfw);

                if let Some(bitrate) = channel.bitrate {
                    create_channel = create_channel.bitrate(bitrate.get());
                }

                if let Some(user_limit) = channel.user_limit {
                    create_channel = create_channel.user_limit(user_limit);
                }

                if let Some(ref rate_limit) = channel.base.rate_limit_per_user {
                    create_channel = create_channel.rate_limit_per_user(*rate_limit);
                }

                create_channel = create_channel.position(channel.position as u16);

                if guild_id
                    .create_channel(&ctx.http, create_channel)
                    .await
                    .is_ok()
                {
                    status += &l10n.t("log-status-reverted", None);
                } else {
                    status += &l10n.t("log-status-revert-failed", None);
                }
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
        l10n.t("log-channel-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-channel-title-blocked", None)
    } else {
        l10n.t("log-channel-title-logged", None)
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
    let desc = l10n.t("log-channel-desc-delete", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
            log_level,
            &title,
            &desc,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id)),
                (
                    &l10n.t("log-field-channel-id", None),
                    channel_id.to_string(),
                ),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

async fn handle_channel_update(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &ChannelProtectionModuleConfig,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
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
        let reason = l10n.t("log-channel-reason-update", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelProtection,
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
        if config_model.revert && channel_id != 0 {
            let mut map = serde_json::Map::new();
            for change in &entry.changes {
                match change {
                    serenity::model::guild::audit_log::Change::Name { old, .. } => {
                        if let Some(n) = old {
                            map.insert("name".to_string(), serde_json::json!(n.as_str()));
                        }
                    }
                    serenity::model::guild::audit_log::Change::Topic { old, .. } => {
                        if let Some(t) = old {
                            map.insert("topic".to_string(), serde_json::json!(t.as_str()));
                        }
                    }
                    serenity::model::guild::audit_log::Change::Nsfw { old, .. } => {
                        if let Some(n) = old {
                            map.insert("nsfw".to_string(), serde_json::json!(n));
                        }
                    }
                    serenity::model::guild::audit_log::Change::RateLimitPerUser { old, .. } => {
                        if let Some(r) = old {
                            map.insert("rate_limit_per_user".to_string(), serde_json::json!(r));
                        }
                    }
                    _ => {}
                }
            }

            if !map.is_empty() {
                let revert_reason = l10n.t("log-channel-revert-reason", None);
                if ctx
                    .http
                    .edit_channel(
                        serenity::GenericChannelId::new(channel_id),
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
        l10n.t("log-channel-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-channel-title-blocked", None)
    } else {
        l10n.t("log-channel-title-logged", None)
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
    let desc = l10n.t("log-channel-desc-update", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
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

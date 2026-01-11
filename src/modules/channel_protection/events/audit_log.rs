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
    let is_whitelisted = data.whitelist.get_whitelist_level(ctx, guild_id, user_id, ModuleType::ChannelProtection).await?.is_some();

    // Check if we should ignore private channels (ownership check)
    if config.ignore_private_channels {
        if let Some(target_id) = entry.target_id {
            let channel_id = serenity::ChannelId::new(target_id.get());
            // We check the HTTP for the channel to check overwrites.
            if let Ok(serenity::Channel::Guild(channel)) = ctx.http.get_channel(channel_id.into()).await {
                let is_owner = channel.permission_overwrites.iter().any(|overwrite| {
                    if let serenity::PermissionOverwriteType::Member(id) = overwrite.kind {
                        id == user_id && overwrite.allow.contains(serenity::Permissions::MANAGE_CHANNELS)
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
            if config.punish_when.contains(&"create".to_string()) {
                handle_channel_create(ctx, entry, guild_id, data, &config_model, user_id, is_whitelisted).await?;
            }
        }
        Action::Channel(ChannelAction::Delete) => {
            if config.punish_when.contains(&"delete".to_string()) {
                handle_channel_delete(ctx, entry, guild_id, data, &config_model, user_id, is_whitelisted).await?;
            }
        }
        Action::Channel(ChannelAction::Update) => {
            if config.punish_when.contains(&"update".to_string()) {
                handle_channel_update(ctx, entry, guild_id, data, &config_model, user_id, is_whitelisted).await?;
            }
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
    config: &module_configs::Model,
    user_id: serenity::UserId,
    is_whitelisted: bool,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    let mut status = if is_whitelisted { 
        "✅ Whitelisted (No action taken)".to_string() 
    } else { 
        "🚨 Blocked (Revert Pending)".to_string() 
    };

    if !is_whitelisted {
        // Punishment
        let result = data.punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelProtection,
                "Channel Created",
            )
            .await?;
        
        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => format!("🚨 Blocked & Punished ({:?})", p),
            crate::services::punishment::ViolationResult::ViolationRecorded { current, threshold } => {
                format!("🚨 Blocked & Violation Recorded ({}/{})", current, threshold)
            },
            crate::services::punishment::ViolationResult::None => "🚨 Blocked (No Punishment Configured)".to_string(),
        };

        // Revert
        if config.revert && channel_id != 0 {
            let _ = ctx
                .http
                .delete_channel(serenity::GenericChannelId::new(channel_id), Some("Channel Protection Revert"))
                .await;
        }
    }

    let title = if is_whitelisted { "Channel Created (Whitelisted)" } else { "Channel Created" };
    let log_level = if is_whitelisted { LogLevel::Audit } else { LogLevel::Warn };

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
            log_level,
            title,
            &format!(
                "A new channel (<#{}>) was created by <@{}>.\n\n**Status**: {}",
                channel_id, user_id, status
            ),
            vec![
                ("User", format!("<@{}>", user_id)),
                ("Channel", format!("<#{}>", channel_id)),
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
    config: &module_configs::Model,
    user_id: serenity::UserId,
    is_whitelisted: bool,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    let mut status = if is_whitelisted { 
        "✅ Whitelisted (No action taken)".to_string() 
    } else { 
        "🚨 Blocked (Revert Pending)".to_string() 
    };

    if !is_whitelisted {
        // Punishment
        let result = data.punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelProtection,
                "Channel Deleted",
            )
            .await?;

        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => format!("🚨 Blocked & Punished ({:?})", p),
            crate::services::punishment::ViolationResult::ViolationRecorded { current, threshold } => {
                format!("🚨 Blocked & Violation Recorded ({}/{})", current, threshold)
            },
            crate::services::punishment::ViolationResult::None => "🚨 Blocked (No Punishment Configured)".to_string(),
        };

        // Revert
        if config.revert {
            // Wait for the channel to be stored in cache (it might come slightly after the audit log)
            let mut cached_channel = None;
            for _ in 0..10 {
                if let Some(c) = data.cache.take_channel(guild_id, serenity::ChannelId::new(channel_id)) {
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

                if let Some(rate_limit) = channel.base.rate_limit_per_user {
                    create_channel = create_channel.rate_limit_per_user(rate_limit);
                }

                create_channel = create_channel.position(channel.position as u16);

                let _ = guild_id.create_channel(&ctx.http, create_channel).await;
            }
        }
    }

    let title = if is_whitelisted { "Channel Deleted (Whitelisted)" } else { "Channel Deleted" };
    let log_level = if is_whitelisted { LogLevel::Audit } else { LogLevel::Error };

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
            log_level,
            title,
            &format!(
                "A channel (`{}`) was deleted by <@{}>.\n\n**Status**: {}",
                channel_id, user_id, status
            ),
            vec![
                ("User", format!("<@{}>", user_id)),
                ("Channel ID", channel_id.to_string()),
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
    config: &module_configs::Model,
    user_id: serenity::UserId,
    is_whitelisted: bool,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    let mut status = if is_whitelisted { 
        "✅ Whitelisted (No action taken)".to_string() 
    } else { 
        "🚨 Blocked (Revert Pending)".to_string() 
    };

    if !is_whitelisted {
        // Punishment
        let result = data.punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::ChannelProtection,
                "Channel Updated",
            )
            .await?;

        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => format!("🚨 Blocked & Punished ({:?})", p),
            crate::services::punishment::ViolationResult::ViolationRecorded { current, threshold } => {
                format!("🚨 Blocked & Violation Recorded ({}/{})", current, threshold)
            },
            crate::services::punishment::ViolationResult::None => "🚨 Blocked (No Punishment Configured)".to_string(),
        };

        // Revert
        if config.revert && channel_id != 0 {
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
                let _ = ctx
                    .http
                    .edit_channel(
                        serenity::GenericChannelId::new(channel_id),
                        &map,
                        Some("Channel Protection Revert"),
                    )
                    .await;
            }
        }
    }

    let title = if is_whitelisted { "Channel Updated (Whitelisted)" } else { "Channel Updated" };
    let log_level = if is_whitelisted { LogLevel::Audit } else { LogLevel::Info };

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
            log_level,
            title,
            &format!(
                "A channel (<#{}>) was modified by <@{}>.\n\n**Status**: {}",
                channel_id, user_id, status
            ),
            vec![
                ("User", format!("<@{}>", user_id)),
                ("Channel", format!("<#{}>", channel_id)),
            ],
        )
        .await?;

    Ok(())
}

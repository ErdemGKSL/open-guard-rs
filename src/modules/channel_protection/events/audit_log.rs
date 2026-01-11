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
                handle_channel_create(ctx, entry, guild_id, data, &config_model, user_id).await?;
            }
        }
        Action::Channel(ChannelAction::Delete) => {
            if config.punish_when.contains(&"delete".to_string()) {
                handle_channel_delete(ctx, entry, guild_id, data, &config_model, user_id).await?;
            }
        }
        Action::Channel(ChannelAction::Update) => {
            if config.punish_when.contains(&"update".to_string()) {
                handle_channel_update(ctx, entry, guild_id, data, &config_model, user_id).await?;
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
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    // Punishment
    data.punishment
        .handle_violation(
            &ctx.http,
            guild_id,
            user_id,
            ModuleType::ChannelProtection,
            "Channel Created",
        )
        .await?;

    // Revert
    if config.revert && channel_id != 0 {
        let _ = ctx
            .http
            .delete_channel(serenity::GenericChannelId::new(channel_id), Some("Channel Protection Revert"))
            .await;
    }

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
            LogLevel::Warn,
            "Channel Created",
            &format!(
                "A new channel (<#{}>) was created by <@{}>.",
                channel_id, user_id
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
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    // Punishment
    data.punishment
        .handle_violation(
            &ctx.http,
            guild_id,
            user_id,
            ModuleType::ChannelProtection,
            "Channel Deleted",
        )
        .await?;

    // Revert
    if config.revert {
        let mut map = serde_json::Map::new();
        for change in &entry.changes {
            match change {
                serenity::model::guild::audit_log::Change::Name { old, .. } => {
                    if let Some(n) = old {
                        map.insert("name".to_string(), serde_json::json!(n.as_str()));
                    }
                }
                serenity::model::guild::audit_log::Change::Type { old, .. } => {
                    if let Some(t) = old {
                        let type_num = match t {
                            serenity::model::guild::audit_log::EntityType::Int(i) => *i,
                            serenity::model::guild::audit_log::EntityType::Str(s) => match s.as_str() {
                                "text" => 0,
                                "voice" => 2,
                                "category" => 4,
                                "news" => 5,
                                "stage" => 13,
                                "forum" => 15,
                                _ => 0,
                            },
                            _ => 0,
                        };
                        map.insert("type".to_string(), serde_json::json!(type_num));
                    }
                }
                serenity::model::guild::audit_log::Change::Topic { old, .. } => {
                    if let Some(t) = old {
                        map.insert("topic".to_string(), serde_json::json!(t.as_str()));
                    }
                }
                _ => {}
            }
        }

        if !map.is_empty() {
            let _ = ctx
                .http
                .create_channel(guild_id, &map, Some("Channel Protection Revert"))
                .await;
        }
    }

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
            LogLevel::Error,
            "Channel Deleted",
            &format!(
                "A channel (`{}`) was deleted by <@{}>.",
                channel_id, user_id
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
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    // Punishment
    data.punishment
        .handle_violation(
            &ctx.http,
            guild_id,
            user_id,
            ModuleType::ChannelProtection,
            "Channel Updated",
        )
        .await?;

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

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelProtection),
            LogLevel::Info,
            "Channel Updated",
            &format!(
                "A channel (<#{}>) was modified by <@{}>.",
                channel_id, user_id
            ),
            vec![
                ("User", format!("<@{}>", user_id)),
                ("Channel", format!("<#{}>", channel_id)),
            ],
        )
        .await?;

    Ok(())
}

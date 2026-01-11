use crate::db::entities::module_configs::{self, ChannelPermissionProtectionModuleConfig, ModuleType};
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
    let config_model =
        module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::ChannelPermissionProtection))
            .one(&data.db)
            .await?;

    let config: ChannelPermissionProtectionModuleConfig = match config_model {
        Some(m) => serde_json::from_value(m.config.clone()).unwrap_or_default(),
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
    // Similar logic to skip if the executor has MANAGE_CHANNELS in the channel
    if config.ignore_private_channels {
        if let Some(target_id) = entry.target_id {
            let channel_id = serenity::ChannelId::new(target_id.get());
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
        Action::ChannelOverwrite(ChannelOverwriteAction::Create) => {
            if config.punish_when.contains(&"create".to_string()) {
                handle_overwrite_create(ctx, entry, guild_id, data, &config, user_id).await?;
            }
        }
        Action::ChannelOverwrite(ChannelOverwriteAction::Delete) => {
            if config.punish_when.contains(&"delete".to_string()) {
                handle_overwrite_delete(ctx, entry, guild_id, data, &config, user_id).await?;
            }
        }
        Action::ChannelOverwrite(ChannelOverwriteAction::Update) => {
            if config.punish_when.contains(&"update".to_string()) {
                handle_overwrite_update(ctx, entry, guild_id, data, &config, user_id).await?;
            }
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
    _config: &ChannelPermissionProtectionModuleConfig,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelPermissionProtection),
            LogLevel::Warn,
            "Channel Permission Overwrite Created",
            &format!(
                "A permission overwrite in channel (<#{}>) was created by <@{}>.",
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

async fn handle_overwrite_delete(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    _config: &ChannelPermissionProtectionModuleConfig,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelPermissionProtection),
            LogLevel::Error,
            "Channel Permission Overwrite Deleted",
            &format!(
                "A permission overwrite in channel (<#{}>) was deleted by <@{}>.",
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

async fn handle_overwrite_update(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    _config: &ChannelPermissionProtectionModuleConfig,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::ChannelPermissionProtection),
            LogLevel::Info,
            "Channel Permission Overwrite Updated",
            &format!(
                "A permission overwrite in channel (<#{}>) was modified by <@{}>.",
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

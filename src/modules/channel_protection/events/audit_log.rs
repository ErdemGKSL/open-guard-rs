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

    let config: ChannelProtectionModuleConfig = match config_model {
        Some(m) => serde_json::from_value(m.config.clone())?,
        None => return Ok(()), // Module not configured for this guild
    };

    if !config.lock_new_channels {
        return Ok(());
    }

    let user_id = match entry.user_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // Ignore actions by the bot itself
    if user_id == ctx.cache.current_user().id {
        return Ok(());
    }

    // Match on the audit log action to triggers variants error
    match entry.action {
        Action::Channel(ChannelAction::Create) => {
            handle_channel_create(ctx, entry, guild_id, data, &config, user_id).await?;
        }
        Action::Channel(ChannelAction::Delete) => {
            handle_channel_delete(ctx, entry, guild_id, data, &config, user_id).await?;
        }
        Action::Channel(ChannelAction::Update) => {
            handle_channel_update(ctx, entry, guild_id, data, &config, user_id).await?;
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
    _config: &ChannelProtectionModuleConfig,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

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
    _config: &ChannelProtectionModuleConfig,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

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
    _config: &ChannelProtectionModuleConfig,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

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

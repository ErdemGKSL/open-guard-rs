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

    let config_model = match config_model {
        Some(m) => m,
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

    // Check if we should ignore private channels (ownership check)
    if config.ignore_private_channels {
        let channel_id = entry.options.as_ref().and_then(|o| o.channel_id);
        if let Some(channel_id) = channel_id {
            if let Ok(serenity::Channel::Guild(channel)) = ctx.http.get_channel(channel_id).await {
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
                handle_overwrite_create(ctx, entry, guild_id, data, &config_model, user_id).await?;
            }
        }
        Action::ChannelOverwrite(ChannelOverwriteAction::Delete) => {
            if config.punish_when.contains(&"delete".to_string()) {
                handle_overwrite_delete(ctx, entry, guild_id, data, &config_model, user_id).await?;
            }
        }
        Action::ChannelOverwrite(ChannelOverwriteAction::Update) => {
            if config.punish_when.contains(&"update".to_string()) {
                handle_overwrite_update(ctx, entry, guild_id, data, &config_model, user_id).await?;
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
    config: &module_configs::Model,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.options.as_ref().and_then(|o| o.channel_id).map(|id| id.get()).unwrap_or(0);
    let target_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    // Punishment
    data.punishment
        .handle_violation(
            &ctx.http,
            guild_id,
            user_id,
            ModuleType::ChannelPermissionProtection,
            "Channel Permission Overwrite Created",
        )
        .await?;

    // Revert
    if config.revert && channel_id != 0 && target_id != 0 {
        let _ = ctx
            .http
            .delete_permission(
                serenity::ChannelId::new(channel_id),
                serenity::TargetId::new(target_id),
                Some("Channel Permission Protection Revert"),
            )
            .await;
    }

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
    config: &module_configs::Model,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.options.as_ref().and_then(|o| o.channel_id).map(|id| id.get()).unwrap_or(0);
    let target_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    // Punishment
    data.punishment
        .handle_violation(
            &ctx.http,
            guild_id,
            user_id,
            ModuleType::ChannelPermissionProtection,
            "Channel Permission Overwrite Deleted",
        )
        .await?;

    // Revert
    if config.revert && channel_id != 0 && target_id != 0 {
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

        let _ = ctx
            .http
            .create_permission(
                serenity::ChannelId::new(channel_id),
                serenity::TargetId::new(target_id),
                &map,
                Some("Channel Permission Protection Revert"),
            )
            .await;
    }

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
    config: &module_configs::Model,
    user_id: serenity::UserId,
) -> Result<(), Error> {
    let channel_id = entry.options.as_ref().and_then(|o| o.channel_id).map(|id| id.get()).unwrap_or(0);
    let target_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    // Punishment
    data.punishment
        .handle_violation(
            &ctx.http,
            guild_id,
            user_id,
            ModuleType::ChannelPermissionProtection,
            "Channel Permission Overwrite Updated",
        )
        .await?;

    // Revert
    if config.revert && channel_id != 0 && target_id != 0 {
        // We need to fetch current permissions and apply changes back
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
            // Get current permissions from the channel to fill the missing one
            let channel = ctx.http.get_channel(serenity::GenericChannelId::new(channel_id)).await;
            if let Ok(serenity::Channel::Guild(channel)) = channel {
                let current_overwrite = channel.permission_overwrites.iter().find(|o| match o.kind {
                    serenity::PermissionOverwriteType::Role(id) => id.get() == target_id,
                    serenity::PermissionOverwriteType::Member(id) => id.get() == target_id,
                    _ => false,
                });

                let final_allow = allow.unwrap_or_else(|| {
                    current_overwrite.map(|o| o.allow).unwrap_or_else(serenity::Permissions::empty)
                });
                let final_deny = deny.unwrap_or_else(|| {
                    current_overwrite.map(|o| o.deny).unwrap_or_else(serenity::Permissions::empty)
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

                let _ = ctx
                    .http
                    .create_permission(
                        serenity::ChannelId::new(channel_id),
                        serenity::TargetId::new(target_id),
                        &map,
                        Some("Channel Permission Protection Revert"),
                    )
                    .await;
            }
        }
    }

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


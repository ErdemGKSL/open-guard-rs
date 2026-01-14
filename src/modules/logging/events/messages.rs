use crate::db::entities::module_configs::{self, LoggingModuleConfig, ModuleType};
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;

pub async fn handle_message_delete(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    channel_id: serenity::ChannelId,
    deleted_message_id: serenity::MessageId,
    data: &Data,
) -> Result<(), Error> {
    // 1. Get module config
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::Logging))
        .one(&data.db)
        .await?;

    let config: LoggingModuleConfig = m_config
        .and_then(|m| serde_json::from_value(m.config).ok())
        .unwrap_or_default();

    if !config.log_messages {
        return Ok(());
    }

    let l10n = data.l10n.get_l10n_for_guild(guild_id, &data.db).await;

    // Try to get message from cache and extract data immediately to avoid Send issues
    let cached_data = ctx
        .cache
        .message(channel_id.into(), deleted_message_id)
        .map(|msg| (msg.author.id, msg.content.to_string()));

    let mut args = fluent::FluentArgs::new();
    args.set("channelId", channel_id.get().to_string());
    args.set("userId", 0);

    let mut fields: Vec<(&str, String)> = vec![];
    let content_label = l10n.t("log-msg-delete-content", None);

    if let Some((author_id, content)) = cached_data {
        args.set("userId", author_id.get().to_string());
        fields.push((content_label.as_str(), content));
    }

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::Logging),
            config.message_log_channel_id,
            crate::services::logger::LogLevel::Info,
            &l10n.t("log-msg-delete-title", None),
            &l10n.t("log-msg-delete-desc", Some(&args)),
            fields,
        )
        .await?;

    Ok(())
}

pub async fn handle_message_edit(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    old_if_available: Option<serenity::Message>,
    new: serenity::MessageUpdateEvent,
    data: &Data,
) -> Result<(), Error> {
    // 1. Get module config
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::Logging))
        .one(&data.db)
        .await?;

    let config: LoggingModuleConfig = m_config
        .and_then(|m| serde_json::from_value(m.config).ok())
        .unwrap_or_default();

    if !config.log_messages {
        return Ok(());
    }

    // 2. Check if content changed
    let old_content = old_if_available
        .as_ref()
        .map(|m| m.content.as_str())
        .unwrap_or("");
    let new_content = new.message.content.as_str();

    if old_content == new_content && !old_content.is_empty() {
        return Ok(());
    }

    let l10n = data.l10n.get_l10n_for_guild(guild_id, &data.db).await;
    let author_id = old_if_available
        .as_ref()
        .map(|m| m.author.id.get())
        .unwrap_or_else(|| new.message.author.id.get());

    let mut args = fluent::FluentArgs::new();
    args.set("userId", author_id.to_string());
    args.set("channelId", new.message.channel_id.get().to_string());

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::Logging),
            config.message_log_channel_id,
            crate::services::logger::LogLevel::Info,
            &l10n.t("log-msg-edit-title", None),
            &l10n.t("log-msg-edit-desc", Some(&args)),
            vec![
                (
                    &l10n.t("log-msg-edit-before", None),
                    old_content.to_string(),
                ),
                (&l10n.t("log-msg-edit-after", None), new_content.to_string()),
            ],
        )
        .await?;

    Ok(())
}

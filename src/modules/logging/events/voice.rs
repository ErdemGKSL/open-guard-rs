use crate::db::entities::module_configs::{self, LoggingModuleConfig, ModuleType};
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;

pub async fn handle_voice_state_update(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    old: Option<serenity::VoiceState>,
    new: serenity::VoiceState,
    data: &Data,
) -> Result<(), Error> {
    // 1. Get module config
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::Logging))
        .one(&data.db)
        .await?;

    let config: LoggingModuleConfig = m_config
        .and_then(|m| serde_json::from_value(m.config).ok())
        .unwrap_or_default();

    if !config.log_voice {
        return Ok(());
    }

    let l10n = data.l10n.get_l10n_for_guild(guild_id, &data.db).await;
    let user_id = new.user_id;

    // Detect changes
    if old.as_ref().and_then(|s| s.channel_id) != new.channel_id {
        // Channel change (join, leave, move)
        match (old.as_ref().and_then(|s| s.channel_id), new.channel_id) {
            (None, Some(channel_id)) => {
                // Join
                let mut args = fluent::FluentArgs::new();
                args.set("userId", user_id.get());
                args.set("channelId", channel_id.get());

                data.logger
                    .log_action(
                        &ctx.http,
                        guild_id,
                        Some(ModuleType::Logging),
                        config.voice_log_channel_id,
                        crate::services::logger::LogLevel::Info,
                        &l10n.t("log-voice-join-title", None),
                        &l10n.t("log-voice-join-desc", Some(&args)),
                        vec![],
                    )
                    .await?;
            }
            (Some(channel_id), None) => {
                // Leave
                let mut args = fluent::FluentArgs::new();
                args.set("userId", user_id.get());
                args.set("channelId", channel_id.get());

                data.logger
                    .log_action(
                        &ctx.http,
                        guild_id,
                        Some(ModuleType::Logging),
                        config.voice_log_channel_id,
                        crate::services::logger::LogLevel::Info,
                        &l10n.t("log-voice-leave-title", None),
                        &l10n.t("log-voice-leave-desc", Some(&args)),
                        vec![],
                    )
                    .await?;
            }
            (Some(old_cid), Some(new_cid)) => {
                // Move
                let mut args = fluent::FluentArgs::new();
                args.set("userId", user_id.get());
                args.set("oldChannelId", old_cid.get());
                args.set("newChannelId", new_cid.get());

                data.logger
                    .log_action(
                        &ctx.http,
                        guild_id,
                        Some(ModuleType::Logging),
                        config.voice_log_channel_id,
                        crate::services::logger::LogLevel::Info,
                        &l10n.t("log-voice-move-title", None),
                        &l10n.t("log-voice-move-desc", Some(&args)),
                        vec![],
                    )
                    .await?;
            }
            _ => {}
        }
    } else {
        // Other state change (mute, deaf, etc.)
        let mut changes = vec![];
        if old.as_ref().map(|s| s.self_mute()) != Some(new.self_mute()) {
            changes.push(format!("Mute: {}", new.self_mute()));
        }
        if old.as_ref().map(|s| s.self_deaf()) != Some(new.self_deaf()) {
            changes.push(format!("Deaf: {}", new.self_deaf()));
        }
        if old.as_ref().map(|s| s.self_video()) != Some(new.self_video()) {
            changes.push(format!("Video: {}", new.self_video()));
        }
        if old.as_ref().map(|s| s.self_stream().unwrap_or(false))
            != Some(new.self_stream().unwrap_or(false))
        {
            changes.push(format!("Stream: {}", new.self_stream().unwrap_or(false)));
        }

        if !changes.is_empty() {
            let mut args = fluent::FluentArgs::new();
            args.set("userId", user_id.get());
            args.set("state", changes.join(", "));

            data.logger
                .log_action(
                    &ctx.http,
                    guild_id,
                    Some(ModuleType::Logging),
                    config.voice_log_channel_id,
                    crate::services::logger::LogLevel::Info,
                    &l10n.t("log-voice-state-title", None),
                    &l10n.t("log-voice-state-desc", Some(&args)),
                    vec![],
                )
                .await?;
        }
    }

    Ok(())
}

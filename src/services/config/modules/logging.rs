use crate::Data;
use crate::db::entities::module_configs::{self, LoggingModuleConfig, ModuleType};
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

pub fn build_ui(
    page: u32,
    config: &LoggingModuleConfig,
    l10n: &L10nProxy,
) -> Vec<serenity::CreateContainerComponent<'static>> {
    let mut components = vec![];

    if page == 0 {
        // Sub-log toggles in one row
        let msg_btn = serenity::CreateButton::new("config_log_msg_toggle")
            .label(if config.log_messages {
                l10n.t("config-log-msg-label", None)
            } else {
                format!("~~{}~~", l10n.t("config-log-msg-label", None))
            })
            .style(if config.log_messages {
                serenity::ButtonStyle::Success
            } else {
                serenity::ButtonStyle::Secondary
            });

        let voice_btn = serenity::CreateButton::new("config_log_voice_toggle")
            .label(if config.log_voice {
                l10n.t("config-log-voice-label", None)
            } else {
                format!("~~{}~~", l10n.t("config-log-voice-label", None))
            })
            .style(if config.log_voice {
                serenity::ButtonStyle::Success
            } else {
                serenity::ButtonStyle::Secondary
            });

        let member_btn = serenity::CreateButton::new("config_log_member_toggle")
            .label(if config.log_membership {
                l10n.t("config-log-member-label", None)
            } else {
                format!("~~{}~~", l10n.t("config-log-member-label", None))
            })
            .style(if config.log_membership {
                serenity::ButtonStyle::Success
            } else {
                serenity::ButtonStyle::Secondary
            });

        components.push(serenity::CreateContainerComponent::ActionRow(
            serenity::CreateActionRow::Buttons(vec![msg_btn, voice_btn, member_btn].into()),
        ));
    } else {
        // Page 1: Channels
        // Message Log Channel Select
        components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(l10n.t("config-log-msg-channel-label", None)),
        ));
        components.push(serenity::CreateContainerComponent::ActionRow(
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "config_log_msg_channel",
                    serenity::CreateSelectMenuKind::Channel {
                        channel_types: Some(vec![serenity::ChannelType::Text].into()),
                        default_channels: config
                            .message_log_channel_id
                            .map(|id| vec![serenity::ChannelId::new(id as u64).into()].into()),
                    },
                )
                .placeholder(l10n.t("config-log-msg-channel-placeholder", None)),
            ),
        ));

        // Voice Log Channel Select
        components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(l10n.t("config-log-voice-channel-label", None)),
        ));
        components.push(serenity::CreateContainerComponent::ActionRow(
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "config_log_voice_channel",
                    serenity::CreateSelectMenuKind::Channel {
                        channel_types: Some(vec![serenity::ChannelType::Text].into()),
                        default_channels: config
                            .voice_log_channel_id
                            .map(|id| vec![serenity::ChannelId::new(id as u64).into()].into()),
                    },
                )
                .placeholder(l10n.t("config-log-voice-channel-placeholder", None)),
            ),
        ));

        // Membership Log Channel Select
        components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(l10n.t("config-log-member-channel-label", None)),
        ));
        components.push(serenity::CreateContainerComponent::ActionRow(
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "config_log_member_channel",
                    serenity::CreateSelectMenuKind::Channel {
                        channel_types: Some(vec![serenity::ChannelType::Text].into()),
                        default_channels: config
                            .membership_log_channel_id
                            .map(|id| vec![serenity::ChannelId::new(id as u64).into()].into()),
                    },
                )
                .placeholder(l10n.t("config-log-member-channel-placeholder", None)),
            ),
        ));
    }

    components
}

pub async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<bool, crate::Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id == "config_log_msg_toggle" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.log_messages = !config.log_messages;
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_log_voice_toggle" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.log_voice = !config.log_voice;
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_log_member_toggle" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        let was_enabled = config.log_membership;
        config.log_membership = !config.log_membership;
        save_config(data, config_active, config.clone()).await?;

        // Handle member roles storage based on toggle state
        if config.log_membership && !was_enabled {
            // Enabling: fetch all members and store their roles in background
            let http = ctx.http.clone();
            let db = data.db.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    crate::modules::logging::events::membership::fetch_and_store_all_members(
                        http, guild_id, db,
                    )
                    .await
                {
                    tracing::error!(
                        "Failed to fetch and store members for guild {}: {:?}",
                        guild_id,
                        e
                    );
                }
            });
        } else if !config.log_membership && was_enabled {
            // Disabling: delete all member roles for this guild ONLY IF sticky roles is also disabled
            let is_sticky_enabled =
                crate::modules::sticky_roles::tracking::get_sticky_roles_config(guild_id, data)
                    .await?
                    .is_some();
            if !is_sticky_enabled {
                if let Err(e) =
                    crate::modules::logging::events::membership::delete_all_guild_member_roles(
                        guild_id, data,
                    )
                    .await
                {
                    tracing::error!(
                        "Failed to delete member roles for guild {}: {:?}",
                        guild_id,
                        e
                    );
                }
            }
        }

        return Ok(true);
    }

    // Channel select handlers
    if custom_id == "config_log_msg_channel" {
        if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
            &interaction.data.kind
        {
            let (config_active, mut config) = get_config(data, guild_id).await?;
            config.message_log_channel_id = values.first().map(|c| c.get() as i64);
            save_config(data, config_active, config).await?;
            return Ok(true);
        }
    }

    if custom_id == "config_log_voice_channel" {
        if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
            &interaction.data.kind
        {
            let (config_active, mut config) = get_config(data, guild_id).await?;
            config.voice_log_channel_id = values.first().map(|c| c.get() as i64);
            save_config(data, config_active, config).await?;
            return Ok(true);
        }
    }

    if custom_id == "config_log_member_channel" {
        if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
            &interaction.data.kind
        {
            let (config_active, mut config) = get_config(data, guild_id).await?;
            config.membership_log_channel_id = values.first().map(|c| c.get() as i64);
            save_config(data, config_active, config).await?;
            return Ok(true);
        }
    }

    Ok(false)
}

async fn get_config(
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<(module_configs::ActiveModel, LoggingModuleConfig), crate::Error> {
    let db = &data.db;
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::Logging))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Config not found"))?;

    let config: LoggingModuleConfig =
        serde_json::from_value(m_config.config.clone()).unwrap_or_default();
    Ok((m_config.into(), config))
}

async fn save_config(
    data: &Data,
    mut config_active: module_configs::ActiveModel,
    config: LoggingModuleConfig,
) -> Result<(), crate::Error> {
    config_active.config = Set(serde_json::to_value(config)?);
    config_active.update(&data.db).await?;
    Ok(())
}

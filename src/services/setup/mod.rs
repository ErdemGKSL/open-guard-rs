pub mod state;
pub mod steps;

use crate::{Context, Data, Error};
use crate::db::entities::{module_configs, whitelist_role, whitelist_user, whitelists::WhitelistLevel};
use crate::db::entities::module_configs::ModuleType;
use crate::services::localization::{ContextL10nExt, L10nProxy};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, Iterable, QueryFilter, Set};
use state::SetupStep;

/// Start the fast setup process for the bot.
#[poise::command(slash_command, guild_only, required_permissions = "ADMINISTRATOR")]
pub async fn setup(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let setup_svc = &ctx.data().setup;
    let l10n = ctx.l10n_user();

    match setup_svc.start_setup(guild_id.get()) {
        Ok(setup_id) => {
            let (content, components) = steps::systems::build_systems_step(&setup_id, &l10n);
            ctx.send(poise::CreateReply::default()
                .content(content)
                .components(components)
                .ephemeral(true))
                .await?;
        }
        Err(e) => {
            ctx.send(poise::CreateReply::default()
                .content(format!("❌ {}", e))
                .ephemeral(true))
                .await?;
        }
    }

    Ok(())
}

pub async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;
    let guild_id = interaction.guild_id.unwrap();
    let l10n = L10nProxy {
        manager: data.l10n.clone(),
        locale: interaction.locale.to_string(),
    };

    if let Some(setup_id) = custom_id.strip_prefix("setup_systems_") {
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };

        let selected_modules: Vec<ModuleType> = values
            .iter()
            .map(|v| match v.as_str() {
                "ChannelProtection" => ModuleType::ChannelProtection,
                "ChannelPermissionProtection" => ModuleType::ChannelPermissionProtection,
                "RoleProtection" => ModuleType::RoleProtection,
                "RolePermissionProtection" => ModuleType::RolePermissionProtection,
                "MemberPermissionProtection" => ModuleType::MemberPermissionProtection,
                "BotAddingProtection" => ModuleType::BotAddingProtection,
                "ModerationProtection" => ModuleType::ModerationProtection,
                "Logging" => ModuleType::Logging,
                "StickyRoles" => ModuleType::StickyRoles,
                _ => unreachable!(),
            })
            .collect();

        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                state.enabled_modules = selected_modules;
                state.current_step = SetupStep::Logging;
            }
        });

        let (content, components) = steps::logging::build_logging_step(setup_id, &l10n, false);
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(content)
                        .components(components),
                ),
            )
            .await?;
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_logging_select_") {
        let channel_id = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::ChannelSelect { values } => {
                values.first().map(|v| v.get())
            }
            _ => return Ok(()),
        };

        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                state.fallback_log_channel = channel_id;
                state.current_step = SetupStep::Whitelist;
            }
        });

        let (content, components) = steps::whitelist::build_whitelist_step(setup_id, &l10n);
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(content)
                        .components(components),
                ),
            )
            .await?;
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_logging_skip_") {
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                state.current_step = SetupStep::Whitelist;
            }
        });

        let (content, components) = steps::whitelist::build_whitelist_step(setup_id, &l10n);
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(content)
                        .components(components),
                ),
            )
            .await?;
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_whitelist_users_") {
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::UserSelect { values } => values,
            _ => return Ok(()),
        };
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                state.whitelist_users = values.iter().map(|v| v.get()).collect();
            }
        });
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_whitelist_roles_") {
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::RoleSelect { values } => values,
            _ => return Ok(()),
        };
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                state.whitelist_roles = values.iter().map(|v| v.get()).collect();
            }
        });
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_whitelist_next_") {
        let mut next_step = None;
        let mut state_opt = None;
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let enabled = state.enabled_modules.clone();
                state.pending_modules = enabled.clone();
                if !state.pending_modules.is_empty() {
                    let first = state.pending_modules.remove(0);
                    state.current_step = SetupStep::ModuleConfig(first.clone());
                    next_step = Some(state.current_step.clone());
                } else {
                    state.current_step = SetupStep::Summary;
                    next_step = Some(SetupStep::Summary);
                }
                state_opt = Some(state.clone());
            }
        });

        match next_step {
            Some(SetupStep::ModuleConfig(m)) => {
                let has_log_channel = state_opt.as_ref().map(|s| s.fallback_log_channel.is_some()).unwrap_or(false);
                let (content, components) = steps::module_config::build_module_config_step(setup_id, &l10n, m, has_log_channel);
                interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(content)
                                .components(components),
                        ),
                    )
                    .await?;
            }
            Some(SetupStep::Summary) => {
                let (content, components) = steps::summary::build_summary_step(setup_id, &l10n, &state_opt.unwrap());
                 interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(content)
                                .components(components),
                        ),
                    )
                    .await?;
            }
            _ => {}
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_logging_types_") {
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };

        let mut config = crate::db::entities::module_configs::LoggingModuleConfig::default();
        for val in values {
            match val.as_str() {
                "messages" => config.log_messages = true,
                "voice" => config.log_voice = true,
                "membership" => config.log_membership = true,
                _ => {}
            }
        }

        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                state.module_configs.insert(
                    ModuleType::Logging,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_cp_ignore_private_toggle_") {
        // Channel Protection: Toggle ignore_private_channels
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
        
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let mut config = crate::db::entities::module_configs::ChannelProtectionModuleConfig::default();
                if let Some(existing) = state.module_configs.get(&ModuleType::ChannelProtection) {
                    config = serde_json::from_value(existing.clone()).unwrap_or_default();
                }
                config.ignore_private_channels = !config.ignore_private_channels;
                state.module_configs.insert(
                    ModuleType::ChannelProtection,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        
        // Rebuild UI with updated state
        if let Some(state) = data.setup.get_state(guild_id.get()) {
            let config: crate::db::entities::module_configs::ChannelProtectionModuleConfig = state.module_configs
                .get(&ModuleType::ChannelProtection)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let (content, components) = steps::module_config::channel_protection::build_ui_with_config(
                setup_id, 
                &l10n,
                &config
            );
            
            interaction.edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content(content)
                    .components(components),
            ).await?;
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_cp_punish_when_") {
        // Channel Protection: Update punish_when
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
        
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };
        
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let mut config = crate::db::entities::module_configs::ChannelProtectionModuleConfig::default();
                if let Some(existing) = state.module_configs.get(&ModuleType::ChannelProtection) {
                    config = serde_json::from_value(existing.clone()).unwrap_or_default();
                }
                config.punish_when = values.to_vec();
                state.module_configs.insert(
                    ModuleType::ChannelProtection,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        
        // Rebuild UI with updated state
        if let Some(state) = data.setup.get_state(guild_id.get()) {
            let config: crate::db::entities::module_configs::ChannelProtectionModuleConfig = state.module_configs
                .get(&ModuleType::ChannelProtection)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let (content, components) = steps::module_config::channel_protection::build_ui_with_config(
                setup_id, 
                &l10n,
                &config
            );
            
            interaction.edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content(content)
                    .components(components),
            ).await?;
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_cpp_ignore_private_toggle_") {
        // Channel Permission Protection: Toggle ignore_private_channels
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
        
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let mut config = crate::db::entities::module_configs::ChannelPermissionProtectionModuleConfig::default();
                if let Some(existing) = state.module_configs.get(&ModuleType::ChannelPermissionProtection) {
                    config = serde_json::from_value(existing.clone()).unwrap_or_default();
                }
                config.ignore_private_channels = !config.ignore_private_channels;
                state.module_configs.insert(
                    ModuleType::ChannelPermissionProtection,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        
        // Rebuild UI with updated state
        if let Some(state) = data.setup.get_state(guild_id.get()) {
            let config: crate::db::entities::module_configs::ChannelPermissionProtectionModuleConfig = state.module_configs
                .get(&ModuleType::ChannelPermissionProtection)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let (content, components) = steps::module_config::channel_permission_protection::build_ui_with_config(
                setup_id, 
                &l10n,
                &config
            );
            
            interaction.edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content(content)
                    .components(components),
            ).await?;
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_cpp_punish_when_") {
        // Channel Permission Protection: Update punish_when
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
        
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };
        
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let mut config = crate::db::entities::module_configs::ChannelPermissionProtectionModuleConfig::default();
                if let Some(existing) = state.module_configs.get(&ModuleType::ChannelPermissionProtection) {
                    config = serde_json::from_value(existing.clone()).unwrap_or_default();
                }
                config.punish_when = values.to_vec();
                state.module_configs.insert(
                    ModuleType::ChannelPermissionProtection,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        
        // Rebuild UI with updated state
        if let Some(state) = data.setup.get_state(guild_id.get()) {
            let config: crate::db::entities::module_configs::ChannelPermissionProtectionModuleConfig = state.module_configs
                .get(&ModuleType::ChannelPermissionProtection)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let (content, components) = steps::module_config::channel_permission_protection::build_ui_with_config(
                setup_id, 
                &l10n,
                &config
            );
            
            interaction.edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content(content)
                    .components(components),
            ).await?;
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_rp_punish_when_") {
        // Role Protection: Update punish_when
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
        
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };
        
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let mut config = crate::db::entities::module_configs::RoleProtectionModuleConfig::default();
                if let Some(existing) = state.module_configs.get(&ModuleType::RoleProtection) {
                    config = serde_json::from_value(existing.clone()).unwrap_or_default();
                }
                config.punish_when = values.to_vec();
                state.module_configs.insert(
                    ModuleType::RoleProtection,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        
        // Rebuild UI with updated state
        if let Some(state) = data.setup.get_state(guild_id.get()) {
            let config: crate::db::entities::module_configs::RoleProtectionModuleConfig = state.module_configs
                .get(&ModuleType::RoleProtection)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let (content, components) = steps::module_config::role_protection::build_ui_with_config(
                setup_id, 
                &l10n,
                &config
            );
            
            interaction.edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content(content)
                    .components(components),
            ).await?;
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_mp_punish_when_") {
        // Moderation Protection: Update punish_when
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
        
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };
        
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let mut config = crate::db::entities::module_configs::ModerationProtectionModuleConfig::default();
                if let Some(existing) = state.module_configs.get(&ModuleType::ModerationProtection) {
                    config = serde_json::from_value(existing.clone()).unwrap_or_default();
                }
                config.punish_when = values.to_vec();
                state.module_configs.insert(
                    ModuleType::ModerationProtection,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        
        // Rebuild UI with updated state
        if let Some(state) = data.setup.get_state(guild_id.get()) {
            let config: crate::db::entities::module_configs::ModerationProtectionModuleConfig = state.module_configs
                .get(&ModuleType::ModerationProtection)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let (content, components) = steps::module_config::moderation_protection::build_ui_with_config(
                setup_id, 
                &l10n,
                &config
            );
            
            interaction.edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content(content)
                    .components(components),
            ).await?;
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_it_vanity_toggle_") {
        // Invite Tracking: Toggle track_vanity
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
        
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let mut config = crate::db::entities::module_configs::InviteTrackingModuleConfig::default();
                if let Some(existing) = state.module_configs.get(&ModuleType::InviteTracking) {
                    config = serde_json::from_value(existing.clone()).unwrap_or_default();
                }
                config.track_vanity = !config.track_vanity;
                state.module_configs.insert(
                    ModuleType::InviteTracking,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        
        // Rebuild UI with updated state
        if let Some(state) = data.setup.get_state(guild_id.get()) {
            let config: crate::db::entities::module_configs::InviteTrackingModuleConfig = state.module_configs
                .get(&ModuleType::InviteTracking)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let (content, components) = steps::module_config::invite_tracking::build_ui_with_config(
                setup_id, 
                &l10n,
                &config
            );
            
            interaction.edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content(content)
                    .components(components),
            ).await?;
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_module_it_ignore_bots_toggle_") {
        // Invite Tracking: Toggle ignore_bots
        interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge).await?;
        
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                let mut config = crate::db::entities::module_configs::InviteTrackingModuleConfig::default();
                if let Some(existing) = state.module_configs.get(&ModuleType::InviteTracking) {
                    config = serde_json::from_value(existing.clone()).unwrap_or_default();
                }
                config.ignore_bots = !config.ignore_bots;
                state.module_configs.insert(
                    ModuleType::InviteTracking,
                    serde_json::to_value(config).unwrap_or_default(),
                );
            }
        });
        
        // Rebuild UI with updated state
        if let Some(state) = data.setup.get_state(guild_id.get()) {
            let config: crate::db::entities::module_configs::InviteTrackingModuleConfig = state.module_configs
                .get(&ModuleType::InviteTracking)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();
            let (content, components) = steps::module_config::invite_tracking::build_ui_with_config(
                setup_id, 
                &l10n,
                &config
            );
            
            interaction.edit_response(
                &ctx.http,
                serenity::EditInteractionResponse::new()
                    .content(content)
                    .components(components),
            ).await?;
        }
    } else if let Some(rest) = custom_id.strip_prefix("setup_module_next_") {
        let parts: Vec<&str> = rest.split('_').collect();
        if parts.len() < 2 { return Ok(()); }
        let setup_id = parts[0];

        let mut next_step = None;
        let mut state_opt = None;
        data.setup.update_state(guild_id.get(), |state| {
            if state.id == setup_id {
                if !state.pending_modules.is_empty() {
                    let first = state.pending_modules.remove(0);
                    state.current_step = SetupStep::ModuleConfig(first.clone());
                    next_step = Some(state.current_step.clone());
                } else {
                    state.current_step = SetupStep::Summary;
                    next_step = Some(SetupStep::Summary);
                }
                state_opt = Some(state.clone());
            }
        });

        match next_step {
            Some(SetupStep::ModuleConfig(m)) => {
                let has_log_channel = state_opt.as_ref().map(|s| s.fallback_log_channel.is_some()).unwrap_or(false);
                let (content, components) = steps::module_config::build_module_config_step(setup_id, &l10n, m, has_log_channel);
                interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(content)
                                .components(components),
                        ),
                    )
                    .await?;
            }
            Some(SetupStep::Summary) => {
                let (content, components) = steps::summary::build_summary_step(setup_id, &l10n, &state_opt.unwrap());
                 interaction
                    .create_response(
                        &ctx.http,
                        serenity::CreateInteractionResponse::UpdateMessage(
                            serenity::CreateInteractionResponseMessage::new()
                                .content(content)
                                .components(components),
                        ),
                    )
                    .await?;
            }
            _ => {}
        }
    } else if let Some(setup_id) = custom_id.strip_prefix("setup_apply_") {
        interaction.defer(&ctx.http).await?;
        handle_apply(ctx, interaction, data, setup_id).await?;
    } else if let Some(_setup_id) = custom_id.strip_prefix("setup_cancel_") {
        data.setup.cancel_setup(guild_id.get());
        interaction
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content(l10n.t("setup-cancelled", None))
                        .components(vec![]),
                ),
            )
            .await?;
    }

    Ok(())
}

async fn handle_apply(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
    setup_id: &str,
) -> Result<(), Error> {
    let guild_id = interaction.guild_id.unwrap();
    let state = match data.setup.get_state(guild_id.get()) {
        Some(s) if s.id == setup_id => s,
        _ => return Ok(()),
    };

    // 1. Update modules
    for module_type in ModuleType::iter() {
        let is_enabled = state.enabled_modules.contains(&module_type);
        
        let existing = module_configs::Entity::find()
            .filter(module_configs::Column::GuildId.eq(guild_id.get() as i64))
            .filter(module_configs::Column::ModuleType.eq(module_type))
            .one(&data.db)
            .await?;

        let mut am: module_configs::ActiveModel = existing
            .map(|m| m.into())
            .unwrap_or_else(|| module_configs::ActiveModel {
                guild_id: Set(guild_id.get() as i64),
                module_type: Set(module_type),
                ..Default::default()
            });

        am.enabled = Set(is_enabled);
        
        if module_type == ModuleType::Logging {
             if let Some(log_id) = state.fallback_log_channel {
                 am.log_channel_id = Set(Some(log_id as i64));
             }
        }
        
        if let Some(config) = state.module_configs.get(&module_type) {
            am.config = Set(config.clone());
        }

        module_configs::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    module_configs::Column::GuildId,
                    module_configs::Column::ModuleType,
                ])
                .update_columns([
                    module_configs::Column::Enabled,
                    module_configs::Column::LogChannelId,
                    module_configs::Column::Config,
                ])
                .to_owned(),
            )
            .exec(&data.db)
            .await?;
    }

    // 2. Update whitelist - check if exists first, then insert or update
    for user_id in &state.whitelist_users {
        let existing = whitelist_user::Entity::find()
            .filter(whitelist_user::Column::GuildId.eq(guild_id.get() as i64))
            .filter(whitelist_user::Column::UserId.eq(*user_id as i64))
            .filter(whitelist_user::Column::ModuleType.is_null())
            .one(&data.db)
            .await?;

        if let Some(existing) = existing {
            // Update existing entry
            let mut active: whitelist_user::ActiveModel = existing.into();
            active.level = Set(WhitelistLevel::Invulnerable);
            active.update(&data.db).await?;
        } else {
            // Insert new entry
            let am = whitelist_user::ActiveModel {
                guild_id: Set(guild_id.get() as i64),
                user_id: Set(*user_id as i64),
                module_type: Set(None),
                level: Set(WhitelistLevel::Invulnerable),
                ..Default::default()
            };
            am.insert(&data.db).await?;
        }
    }

    for role_id in &state.whitelist_roles {
        let existing = whitelist_role::Entity::find()
            .filter(whitelist_role::Column::GuildId.eq(guild_id.get() as i64))
            .filter(whitelist_role::Column::RoleId.eq(*role_id as i64))
            .filter(whitelist_role::Column::ModuleType.is_null())
            .one(&data.db)
            .await?;

        if let Some(existing) = existing {
            // Update existing entry
            let mut active: whitelist_role::ActiveModel = existing.into();
            active.level = Set(WhitelistLevel::Invulnerable);
            active.update(&data.db).await?;
        } else {
            // Insert new entry
            let am = whitelist_role::ActiveModel {
                guild_id: Set(guild_id.get() as i64),
                role_id: Set(*role_id as i64),
                module_type: Set(None),
                level: Set(WhitelistLevel::Invulnerable),
                ..Default::default()
            };
            am.insert(&data.db).await?;
        }
    }

    data.setup.cancel_setup(guild_id.get());

    let l10n = L10nProxy {
        manager: data.l10n.clone(),
        locale: interaction.locale.to_string(),
    };

    let mut details = format!("## {}\n\n", l10n.t("setup-apply-success", None));

    details.push_str(&format!("**{}**\n", l10n.t("setup-summary-enabled-modules", None)));
    if state.enabled_modules.is_empty() {
        details.push_str(&format!("- {}\n", l10n.t("setup-summary-none", None)));
    } else {
        for module in &state.enabled_modules {
            let name_key = match module {
                ModuleType::ChannelProtection => "module-channel-protection-name",
                ModuleType::ChannelPermissionProtection => "module-channel-permission-protection-name",
                ModuleType::RoleProtection => "module-role-protection-name",
                ModuleType::RolePermissionProtection => "module-role-permission-protection-name",
                ModuleType::MemberPermissionProtection => "module-member-permission-protection-name",
                ModuleType::BotAddingProtection => "module-bot-adding-protection-name",
                ModuleType::ModerationProtection => "module-moderation-protection-name",
                ModuleType::Logging => "module-logging-name",
                ModuleType::StickyRoles => "module-sticky-roles-name",
                ModuleType::InviteTracking => "module-invite-tracking-name",
            };
            details.push_str(&format!("- ✅ {}\n", l10n.t(name_key, None)));
        }
    }

    details.push_str(&format!("\n**{}**\n", l10n.t("setup-summary-fallback-log", None)));
    if let Some(channel_id) = state.fallback_log_channel {
        details.push_str(&format!("- <#{}>\n", channel_id));
    } else {
        details.push_str(&format!("- {}\n", l10n.t("config-punishment-type-none", None)));
    }

    details.push_str(&format!("\n**{}**\n", l10n.t("setup-summary-whitelist", None)));

    let mut users_args = FluentArgs::new();
    users_args.set("count", state.whitelist_users.len());
    details.push_str(&format!(
        "- {}\n",
        l10n.t("setup-summary-users", Some(&users_args))
    ));

    let mut roles_args = FluentArgs::new();
    roles_args.set("count", state.whitelist_roles.len());
    details.push_str(&format!(
        "- {}\n",
        l10n.t("setup-summary-roles", Some(&roles_args))
    ));

    interaction
        .edit_response(
            &ctx.http,
            serenity::EditInteractionResponse::new()
                .content(details)
                .components(vec![]),
        )
        .await?;

    Ok(())
}

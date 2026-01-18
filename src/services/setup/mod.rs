pub mod state;
pub mod steps;

use crate::{Context, Data, Error};
use crate::db::entities::{module_configs, whitelist_role, whitelist_user, whitelists::WhitelistLevel};
use crate::db::entities::module_configs::ModuleType;
use crate::services::localization::{ContextL10nExt, L10nProxy};
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, EntityTrait, Iterable, QueryFilter, Set};
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

        let (content, components) = steps::logging::build_logging_step(setup_id, &l10n);
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
                let (content, components) = steps::module_config::build_module_config_step(setup_id, &l10n, m);
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
                let (content, components) = steps::module_config::build_module_config_step(setup_id, &l10n, m);
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

    // 2. Update whitelist
    for user_id in state.whitelist_users {
        let am = whitelist_user::ActiveModel {
            guild_id: Set(guild_id.get() as i64),
            user_id: Set(user_id as i64),
            module_type: Set(None),
            level: Set(WhitelistLevel::Admin),
            ..Default::default()
        };
        whitelist_user::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    whitelist_user::Column::GuildId,
                    whitelist_user::Column::UserId,
                    whitelist_user::Column::ModuleType,
                ])
                .update_column(whitelist_user::Column::Level)
                .to_owned()
            )
            .exec(&data.db)
            .await?;
    }

    for role_id in state.whitelist_roles {
        let am = whitelist_role::ActiveModel {
            guild_id: Set(guild_id.get() as i64),
            role_id: Set(role_id as i64),
            module_type: Set(None),
            level: Set(WhitelistLevel::Admin),
            ..Default::default()
        };
        whitelist_role::Entity::insert(am)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    whitelist_role::Column::GuildId,
                    whitelist_role::Column::RoleId,
                    whitelist_role::Column::ModuleType,
                ])
                .update_column(whitelist_role::Column::Level)
                .to_owned()
            )
            .exec(&data.db)
            .await?;
    }

    data.setup.cancel_setup(guild_id.get());

    let l10n = L10nProxy {
        manager: data.l10n.clone(),
        locale: interaction.locale.to_string(),
    };

    interaction
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content(l10n.t("setup-apply-success", None))
                    .components(vec![]),
            ),
        )
        .await?;

    Ok(())
}

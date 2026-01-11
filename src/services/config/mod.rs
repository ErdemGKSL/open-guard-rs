pub mod builders;
pub mod modules;

use crate::db::entities::{guild_configs, module_configs};
use crate::db::entities::module_configs::{ModuleType, ChannelProtectionModuleConfig};
use crate::services::localization::{L10nProxy, ContextL10nExt};
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::{EntityTrait, ActiveModelTrait, Set};

pub use builders::*;

/// Global configuration command
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn config(ctx: crate::Context<'_>) -> Result<(), Error> {
    let l10n = ctx.l10n_guild();
    ctx.defer_ephemeral().await?;
    let components = build_main_menu(&ctx.data(), ctx.guild_id().unwrap(), &l10n).await?;
    
    ctx.send(
        poise::CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2 | serenity::MessageFlags::EPHEMERAL)
            .components(components),
    )
    .await?;
    
    Ok(())
}

/// Builds the main configuration menu
pub async fn build_main_menu(
    data: &Data,
    guild_id: serenity::GuildId,
    l10n: &L10nProxy,
) -> Result<Vec<serenity::CreateComponent<'static>>, Error> {
    let db = &data.db;

    // Fetch current general config
    let g_config = guild_configs::Entity::find_by_id(guild_id.get() as i64)
        .one(db)
        .await?;

    let log_channel_id = g_config.as_ref().and_then(|c| c.log_channel_id);
    let jail_role_id = g_config.as_ref().and_then(|c| c.jail_role_id);

    let mut inner_components = vec![];

    // General Configuration Header
    inner_components.extend(create_header(l10n.t("config-general-header", None), true));

    // Log Channel Section
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-log-channel-label", None)),
    ));

    inner_components.push(create_select_menu_row(
        "config_general_log_channel".to_string(),
        serenity::CreateSelectMenuKind::Channel {
            channel_types: None,
            default_channels: log_channel_id
                .map(|id| vec![serenity::ChannelId::new(id as u64).into()].into()),
        },
        l10n.t("config-select-log-channel-placeholder", None),
    ));

    // Jail Role Section
    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-jail-role-label", None)),
    ));

    inner_components.push(create_select_menu_row(
        "config_jail_role".to_string(),
        serenity::CreateSelectMenuKind::Role {
            default_roles: jail_role_id
                .map(|id| vec![serenity::RoleId::new(id as u64).into()].into()),
        },
        l10n.t("config-select-jail-role-placeholder", None),
    ));

    // Modules Section
    inner_components.extend(create_header(l10n.t("config-modules-header", None), true));

    let options = vec![
        serenity::CreateSelectMenuOption::new(
            l10n.t("config-channel-protection-label", None),
            "ChannelProtection",
        )
        .description(l10n.t("config-channel-protection-desc", None)),
    ];

    inner_components.push(create_select_menu_row(
        "config_navigate_modules".to_string(),
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
        l10n.t("config-select-module-placeholder", None),
    ));

    Ok(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components)
    )])
}

/// Builds the module-specific configuration menu
pub async fn build_module_menu(
    data: &Data,
    guild_id: serenity::GuildId,
    module: ModuleType,
    l10n: &L10nProxy,
) -> Result<Vec<serenity::CreateComponent<'static>>, Error> {
    let m_config = match module_configs::Entity::find_by_id((guild_id.get() as i64, module))
        .one(&data.db)
        .await?
    {
        Some(m) => m,
        None => {
            // Create default config if not exists
            let m = module_configs::ActiveModel {
                guild_id: Set(guild_id.get() as i64),
                module_type: Set(module),
                ..Default::default()
            };
            m.insert(&data.db).await?
        }
    };

    let name = match module {
        ModuleType::ChannelProtection => l10n.t("config-channel-protection-label", None),
        ModuleType::ChannelPermissionProtection => l10n.t("config-channel-permission-protection-label", None),
    };

    let mut inner_components = create_module_config_payload(
        name,
        module,
        m_config.log_channel_id,
        m_config.punishment,
        m_config.punishment_at,
        m_config.punishment_at_interval,
        m_config.revert,
        l10n,
    );

    // Append module-specific UI
    if module == ModuleType::ChannelProtection {
        let cp_config: ChannelProtectionModuleConfig = serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(serenity::CreateSeparator::new(true)));
        inner_components.extend(modules::channel_protection::build_ui(&cp_config, l10n));
    }

    // Add back button at the very end
    inner_components.push(serenity::CreateContainerComponent::Separator(serenity::CreateSeparator::new(false)));
    inner_components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new("** **"),
            )],
            serenity::CreateSectionAccessory::Button(
                serenity::CreateButton::new("config_back_to_main")
                    .label(l10n.t("config-back-label", None))
                    .style(serenity::ButtonStyle::Secondary)
            ),
        )
    ));

    Ok(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components)
    )])
}

/// Handle interactions from the config menus
pub async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => return Ok(()),
    };

    let l10n_manager = &data.l10n;
    let l10n = L10nProxy {
        manager: l10n_manager.clone(),
        locale: interaction.locale.to_string(),
    };

    let custom_id = &interaction.data.custom_id;

    // Acknowledge the interaction immediately to prevent "Interaction Failed"
    interaction
        .create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge)
        .await?;

    let mut updated_reply = None;

    // Try module-specific handlers first
    if modules::channel_protection::handle_interaction(ctx, interaction, data, guild_id).await? {
        updated_reply = Some(build_module_menu(data, guild_id, ModuleType::ChannelProtection, &l10n).await?);
    } else if custom_id == "config_general_log_channel" {
        if let serenity::ComponentInteractionDataKind::ChannelSelect { values } = &interaction.data.kind {
            if let Some(channel_id) = values.first() {
                guild_configs::Entity::insert(guild_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    log_channel_id: Set(Some(channel_id.get() as i64)),
                    ..Default::default()
                })
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(guild_configs::Column::GuildId)
                        .update_column(guild_configs::Column::LogChannelId)
                        .to_owned(),
                )
                .exec(&data.db)
                .await?;

                updated_reply = Some(build_main_menu(data, guild_id, &l10n).await?);
            }
        }
    } else if custom_id == "config_jail_role" {
        if let serenity::ComponentInteractionDataKind::RoleSelect { values } = &interaction.data.kind {
            if let Some(role_id) = values.first() {
                guild_configs::Entity::insert(guild_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    jail_role_id: Set(Some(role_id.get() as i64)),
                    ..Default::default()
                })
                .on_conflict(
                    sea_orm::sea_query::OnConflict::column(guild_configs::Column::GuildId)
                        .update_column(guild_configs::Column::JailRoleId)
                        .to_owned(),
                )
                .exec(&data.db)
                .await?;

                updated_reply = Some(build_main_menu(data, guild_id, &l10n).await?);
            }
        }
    } else if custom_id == "config_navigate_modules" {
        if let serenity::ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
            if let Some(module_str) = values.first() {
                let module_type = match module_str.as_str() {
                    "ChannelProtection" => ModuleType::ChannelProtection,
                    _ => return Ok(()),
                };
                updated_reply = Some(build_module_menu(data, guild_id, module_type, &l10n).await?);
            }
        }
    } else if custom_id.starts_with("config_module_log_channel_") {
        if let serenity::ComponentInteractionDataKind::ChannelSelect { values } = &interaction.data.kind {
            if let Some(channel_id) = values.first() {
                let module_str = custom_id.trim_start_matches("config_module_log_channel_");
                let module_type = match module_str {
                    "ChannelProtection" => ModuleType::ChannelProtection,
                    _ => return Ok(()),
                };

                module_configs::Entity::insert(module_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    module_type: Set(module_type),
                    log_channel_id: Set(Some(channel_id.get() as i64)),
                    ..Default::default()
                })
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        module_configs::Column::GuildId,
                        module_configs::Column::ModuleType,
                    ])
                    .update_column(module_configs::Column::LogChannelId)
                    .to_owned(),
                )
                .exec(&data.db)
                .await?;

                updated_reply = Some(build_module_menu(data, guild_id, module_type, &l10n).await?);
            }
        }
    } else if custom_id.starts_with("config_module_punishment_") {
        if let serenity::ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
            if let Some(p_str) = values.first() {
                let module_str = custom_id.trim_start_matches("config_module_punishment_");
                let module_type = match module_str {
                    "ChannelProtection" => ModuleType::ChannelProtection,
                    _ => return Ok(()),
                };

                use crate::db::entities::module_configs::PunishmentType;
                let punishment = match p_str.as_str() {
                    "none" => PunishmentType::None,
                    "unperm" => PunishmentType::Unperm,
                    "ban" => PunishmentType::Ban,
                    "kick" => PunishmentType::Kick,
                    "jail" => PunishmentType::Jail,
                    _ => return Ok(()),
                };

                module_configs::Entity::insert(module_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    module_type: Set(module_type),
                    punishment: Set(punishment),
                    ..Default::default()
                })
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        module_configs::Column::GuildId,
                        module_configs::Column::ModuleType,
                    ])
                    .update_column(module_configs::Column::Punishment)
                    .to_owned(),
                )
                .exec(&data.db)
                .await?;

                updated_reply = Some(build_module_menu(data, guild_id, module_type, &l10n).await?);
            }
        }
    } else if custom_id.starts_with("config_module_revert_") {
        let module_str = custom_id.trim_start_matches("config_module_revert_");
        let module_type = match module_str {
            "ChannelProtection" => ModuleType::ChannelProtection,
            _ => return Ok(()),
        };

        let module_configs::Model { revert, .. } = match module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
            .one(&data.db)
            .await?
        {
            Some(m) => m,
            None => {
                // If no config exists, create it with default values
                module_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    module_type: Set(module_type),
                    ..Default::default()
                }
                .insert(&data.db)
                .await?
            }
        };

        module_configs::Entity::update(module_configs::ActiveModel {
            guild_id: Set(guild_id.get() as i64),
            module_type: Set(module_type),
            revert: Set(!revert),
            ..Default::default()
        })
        .exec(&data.db)
        .await?;

        updated_reply = Some(build_module_menu(data, guild_id, module_type, &l10n).await?);
    } else if custom_id == "config_back_to_main" {
        updated_reply = Some(build_main_menu(data, guild_id, &l10n).await?);
    }

    if let Some(components) = updated_reply {
        let edit = serenity::EditInteractionResponse::new().components(components);
        interaction.edit_response(&ctx.http, edit).await?;
    }

    Ok(())
}

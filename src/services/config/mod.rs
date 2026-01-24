use crate::db::entities::module_configs::{ChannelProtectionModuleConfig, ModuleType};
use crate::db::entities::{guild_configs, module_configs};
use crate::services::localization::{ContextL10nExt, L10nProxy};
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

pub mod builders;
pub mod modules;
pub mod whitelist;

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

    // Whitelists Section
    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));
    inner_components.push(create_whitelist_section(
        "config_whitelist_view_global".to_string(),
        l10n,
    ));

    // Modules Section
    inner_components.extend(create_header(l10n.t("config-modules-header", None), true));

    let m_configs: Vec<module_configs::Model> = module_configs::Entity::find()
        .filter(module_configs::Column::GuildId.eq(guild_id.get() as i64))
        .all(db)
        .await?;

    let get_status = |m: ModuleType| {
        let enabled = m_configs
            .iter()
            .find(|c| c.module_type == m)
            .map(|c| c.enabled)
            .unwrap_or(false);
        if enabled {
            format!("🟢 {}", l10n.t("config-btn-enabled", None))
        } else {
            format!("🔴 {}", l10n.t("config-btn-disabled", None))
        }
    };

    let options = vec![
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-channel-protection-label", None),
                get_status(ModuleType::ChannelProtection)
            ),
            "ChannelProtection",
        )
        .description(l10n.t("config-channel-protection-desc", None)),
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-channel-permission-protection-label", None),
                get_status(ModuleType::ChannelPermissionProtection)
            ),
            "ChannelPermissionProtection",
        )
        .description(l10n.t("config-channel-permission-protection-desc", None)),
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-role-protection-label", None),
                get_status(ModuleType::RoleProtection)
            ),
            "RoleProtection",
        )
        .description(l10n.t("config-role-protection-desc", None)),
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-role-permission-protection-label", None),
                get_status(ModuleType::RolePermissionProtection)
            ),
            "RolePermissionProtection",
        )
        .description(l10n.t("config-role-permission-protection-desc", None)),
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-member-permission-protection-label", None),
                get_status(ModuleType::MemberPermissionProtection)
            ),
            "MemberPermissionProtection",
        )
        .description(l10n.t("config-member-permission-protection-desc", None)),
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-bot-adding-protection-label", None),
                get_status(ModuleType::BotAddingProtection)
            ),
            "BotAddingProtection",
        )
        .description(l10n.t("config-bot-adding-protection-desc", None)),
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-moderation-protection-label", None),
                get_status(ModuleType::ModerationProtection)
            ),
            "ModerationProtection",
        )
        .description(l10n.t("config-moderation-protection-desc", None)),
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-logging-label", None),
                get_status(ModuleType::Logging)
            ),
            "Logging",
        )
        .description(l10n.t("config-logging-desc", None)),
        serenity::CreateSelectMenuOption::new(
            format!(
                "{} - {}",
                l10n.t("config-sticky-roles-label", None),
                get_status(ModuleType::StickyRoles)
            ),
            "StickyRoles",
        )
        .description(l10n.t("config-sticky-roles-desc", None)),
    ];

    inner_components.push(create_select_menu_row(
        "config_navigate_modules".to_string(),
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
        l10n.t("config-select-module-placeholder", None),
    ));

    Ok(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components),
    )])
}

/// Builds the module-specific configuration menu
pub async fn build_module_menu(
    data: &Data,
    guild_id: serenity::GuildId,
    module: ModuleType,
    page: u32,
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
        ModuleType::ChannelPermissionProtection => {
            l10n.t("config-channel-permission-protection-label", None)
        }
        ModuleType::RoleProtection => l10n.t("config-role-protection-label", None),
        ModuleType::RolePermissionProtection => {
            l10n.t("config-role-permission-protection-label", None)
        }
        ModuleType::MemberPermissionProtection => {
            l10n.t("config-member-permission-protection-label", None)
        }
        ModuleType::BotAddingProtection => l10n.t("config-bot-adding-protection-label", None),
        ModuleType::ModerationProtection => l10n.t("config-moderation-protection-label", None),
        ModuleType::Logging => l10n.t("config-logging-label", None),
        ModuleType::StickyRoles => l10n.t("config-sticky-roles-label", None),
        ModuleType::InviteTracking => l10n.t("config-invite-tracking-label", None),
    };

    let mut inner_components = if module == ModuleType::Logging && page == 1 {
        create_header(name, true)
    } else if module == ModuleType::Logging {
        // Page 0: General - Header and Global toggle
        let mut components = vec![];
        let status_label = if m_config.enabled {
            l10n.t("config-btn-enabled", None)
        } else {
            l10n.t("config-btn-disabled", None)
        };

        let toggle_btn = serenity::CreateButton::new(format!("config_module_toggle_{}", module))
            .label(status_label)
            .style(if m_config.enabled {
                serenity::ButtonStyle::Success
            } else {
                serenity::ButtonStyle::Danger
            });

        components.push(serenity::CreateContainerComponent::Section(
            serenity::CreateSection::new(
                vec![serenity::CreateSectionComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!("⚙️ **{}**", name)),
                )],
                serenity::CreateSectionAccessory::Button(toggle_btn),
            ),
        ));

        // Sub-log toggles
        let logging_config: crate::db::entities::module_configs::LoggingModuleConfig =
            serde_json::from_value(m_config.config.clone()).unwrap_or_default();
        components.extend(modules::logging::build_ui(0, &logging_config, l10n));

        components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));

        // Whitelist
        components.push(create_whitelist_section(
            format!("config_whitelist_view_module_{}", module),
            l10n,
        ));

        components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));

        // General Log Channel
        components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(l10n.t("config-module-log-channel-label", None)),
        ));

        components.push(create_select_menu_row(
            format!("config_module_log_channel_{:?}", module),
            serenity::CreateSelectMenuKind::Channel {
                channel_types: None,
                default_channels: m_config
                    .log_channel_id
                    .map(|id| vec![serenity::ChannelId::new(id as u64).into()].into()),
            },
            l10n.t("config-select-module-log-channel-placeholder", None),
        ));

        components
    } else {
        create_module_config_payload(
            name,
            module,
            m_config.log_channel_id,
            m_config.punishment,
            m_config.punishment_at,
            m_config.punishment_at_interval,
            m_config.enabled,
            m_config.revert,
            l10n,
        )
    };

    if module == ModuleType::ChannelProtection {
        let cp_config: ChannelProtectionModuleConfig =
            serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::channel_protection::build_ui(&cp_config, l10n));
    } else if module == ModuleType::ChannelPermissionProtection {
        let cpp_config: crate::db::entities::module_configs::ChannelPermissionProtectionModuleConfig = serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::channel_permission_protection::build_ui(
            &cpp_config,
            l10n,
        ));
    } else if module == ModuleType::RoleProtection {
        let rp_config: crate::db::entities::module_configs::RoleProtectionModuleConfig =
            serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::role_protection::build_ui(&rp_config, l10n));
    } else if module == ModuleType::RolePermissionProtection {
        let rpp_config: crate::db::entities::module_configs::RolePermissionProtectionModuleConfig =
            serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::role_permission_protection::build_ui(
            &rpp_config,
            l10n,
        ));
    } else if module == ModuleType::MemberPermissionProtection {
        let mpp_config: crate::db::entities::module_configs::MemberPermissionProtectionModuleConfig = serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::member_permission_protection::build_ui(
            &mpp_config,
            l10n,
        ));
    } else if module == ModuleType::BotAddingProtection {
        let bap_config: crate::db::entities::module_configs::BotAddingProtectionModuleConfig =
            serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::bot_adding_protection::build_ui(&bap_config, l10n));
    } else if module == ModuleType::ModerationProtection {
        let mp_config: crate::db::entities::module_configs::ModerationProtectionModuleConfig =
            serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::moderation_protection::build_ui(&mp_config, l10n));
    } else if module == ModuleType::Logging {
        if page == 1 {
            let logging_config: crate::db::entities::module_configs::LoggingModuleConfig =
                serde_json::from_value(m_config.config).unwrap_or_default();
            inner_components.extend(modules::logging::build_ui(page, &logging_config, l10n));
        }
    } else if module == ModuleType::StickyRoles {
        let sr_config: crate::db::entities::module_configs::StickyRolesModuleConfig =
            serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::sticky_roles::build_ui(&sr_config, l10n));
    } else if module == ModuleType::InviteTracking {
        let it_config: crate::db::entities::module_configs::InviteTrackingModuleConfig =
            serde_json::from_value(m_config.config).unwrap_or_default();
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        inner_components.extend(modules::invite_tracking::build_ui(&it_config, l10n));
    }

    // Add pagination row for Logging
    if module == ModuleType::Logging {
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
        let buttons = vec![
            serenity::CreateButton::new(format!("config_page_{}_0", module))
                .label(l10n.t("config-page-general", None))
                .style(if page == 0 {
                    serenity::ButtonStyle::Primary
                } else {
                    serenity::ButtonStyle::Secondary
                }),
            serenity::CreateButton::new(format!("config_page_{}_1", module))
                .label(l10n.t("config-page-channels", None))
                .style(if page == 1 {
                    serenity::ButtonStyle::Primary
                } else {
                    serenity::ButtonStyle::Secondary
                }),
        ];
        inner_components.push(serenity::CreateContainerComponent::ActionRow(
            serenity::CreateActionRow::Buttons(buttons.into()),
        ));
    }

    // Add back button at the very end
    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));
    inner_components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new("** **"),
            )],
            serenity::CreateSectionAccessory::Button(
                serenity::CreateButton::new("config_back_to_main")
                    .label(l10n.t("config-back-label", None))
                    .style(serenity::ButtonStyle::Secondary),
            ),
        ),
    ));

    Ok(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components),
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

    // Check for whitelist modal actions FIRST - modals must be the direct response
    // We cannot acknowledge before showing a modal
    if custom_id.starts_with("config_whitelist_add_")
        || custom_id.starts_with("config_whitelist_manage_")
    {
        match whitelist::handle_interaction(ctx, interaction, data).await? {
            whitelist::WhitelistInteractionResult::ShowModal(modal) => {
                interaction
                    .create_response(&ctx.http, serenity::CreateInteractionResponse::Modal(modal))
                    .await?;
                return Ok(());
            }
            whitelist::WhitelistInteractionResult::Components(components) => {
                // This shouldn't happen for add/manage buttons, but handle it
                interaction
                    .create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge)
                    .await?;
                let edit = serenity::EditInteractionResponse::new()
                    .components(components)
                    .allowed_mentions(serenity::CreateAllowedMentions::new().empty_users().empty_roles());
                interaction.edit_response(&ctx.http, edit).await?;
                return Ok(());
            }
            whitelist::WhitelistInteractionResult::None => {
                // Continue with normal flow
            }
        }
    }

    // Acknowledge the interaction for non-modal responses
    interaction
        .create_response(&ctx.http, serenity::CreateInteractionResponse::Acknowledge)
        .await?;

    let mut updated_reply = None;

    let mut page = 0;
    if custom_id.ends_with("_channel") && custom_id.starts_with("config_log_") {
        page = 1;
    } else if custom_id.ends_with("_toggle") && custom_id.starts_with("config_log_") {
        page = 0;
    } else if let Some(rest) = custom_id.strip_prefix("config_page_") {
        let parts: Vec<&str> = rest.split('_').collect();
        if parts.len() >= 2 {
            page = parts[1].parse().unwrap_or(0);
        }
    }

    // Try module-specific handlers first
    if modules::channel_protection::handle_interaction(ctx, interaction, data, guild_id).await? {
        updated_reply = Some(
            build_module_menu(data, guild_id, ModuleType::ChannelProtection, page, &l10n).await?,
        );
    } else if modules::channel_permission_protection::handle_interaction(
        ctx,
        interaction,
        data,
        guild_id,
    )
    .await?
    {
        updated_reply = Some(
            build_module_menu(
                data,
                guild_id,
                ModuleType::ChannelPermissionProtection,
                page,
                &l10n,
            )
            .await?,
        );
    } else if modules::role_protection::handle_interaction(ctx, interaction, data, guild_id).await?
    {
        updated_reply =
            Some(build_module_menu(data, guild_id, ModuleType::RoleProtection, page, &l10n).await?);
    } else if modules::role_permission_protection::handle_interaction(
        ctx,
        interaction,
        data,
        guild_id,
    )
    .await?
    {
        updated_reply = Some(
            build_module_menu(
                data,
                guild_id,
                ModuleType::RolePermissionProtection,
                page,
                &l10n,
            )
            .await?,
        );
    } else if modules::member_permission_protection::handle_interaction(
        ctx,
        interaction,
        data,
        guild_id,
    )
    .await?
    {
        updated_reply = Some(
            build_module_menu(
                data,
                guild_id,
                ModuleType::MemberPermissionProtection,
                page,
                &l10n,
            )
            .await?,
        );
    } else if modules::bot_adding_protection::handle_interaction(ctx, interaction, data, guild_id)
        .await?
    {
        updated_reply = Some(
            build_module_menu(data, guild_id, ModuleType::BotAddingProtection, page, &l10n).await?,
        );
    } else if modules::moderation_protection::handle_interaction(ctx, interaction, data, guild_id)
        .await?
    {
        updated_reply = Some(
            build_module_menu(
                data,
                guild_id,
                ModuleType::ModerationProtection,
                page,
                &l10n,
            )
            .await?,
        );
    } else if modules::logging::handle_interaction(ctx, interaction, data, guild_id).await? {
        updated_reply =
            Some(build_module_menu(data, guild_id, ModuleType::Logging, page, &l10n).await?);
    } else if modules::sticky_roles::handle_interaction(ctx, interaction, data, guild_id).await? {
        updated_reply =
            Some(build_module_menu(data, guild_id, ModuleType::StickyRoles, page, &l10n).await?);
    } else if modules::invite_tracking::handle_interaction(ctx, interaction, data, guild_id).await? {
        updated_reply =
            Some(build_module_menu(data, guild_id, ModuleType::InviteTracking, page, &l10n).await?);
    } else {
        // Handle remaining whitelist interactions (non-modal ones like delete, navigation)
        match whitelist::handle_interaction(ctx, interaction, data).await? {
            whitelist::WhitelistInteractionResult::Components(components) => {
                updated_reply = Some(components);
            }
            whitelist::WhitelistInteractionResult::ShowModal(_) => {
                // Modal actions should have been handled before acknowledgment
                // This should not happen, but ignore if it does
            }
            whitelist::WhitelistInteractionResult::None => {
                // Whitelist didn't handle this, continue to other handlers
            }
        }
    }

    // Other handlers (if whitelist didn't handle it)
    if updated_reply.is_none() {
        if custom_id == "config_back_to_main" {
            updated_reply = Some(build_main_menu(data, guild_id, &l10n).await?);
        } else if let Some(module_str) = custom_id.strip_prefix("config_module_menu_") {
            let module_type = match module_str {
                "channel_protection" => ModuleType::ChannelProtection,
                "channel_permission_protection" => ModuleType::ChannelPermissionProtection,
                "role_protection" => ModuleType::RoleProtection,
                "role_permission_protection" => ModuleType::RolePermissionProtection,
                "member_permission_protection" => ModuleType::MemberPermissionProtection,
                "bot_adding_protection" => ModuleType::BotAddingProtection,
                "moderation_protection" => ModuleType::ModerationProtection,
                "logging" => ModuleType::Logging,
                "sticky_roles" => ModuleType::StickyRoles,
                _ => return Ok(()),
            };
            updated_reply = Some(build_module_menu(data, guild_id, module_type, 0, &l10n).await?);
        } else if custom_id == "config_general_log_channel" {
            if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                &interaction.data.kind
            {
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
            if let serenity::ComponentInteractionDataKind::RoleSelect { values } =
                &interaction.data.kind
            {
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
            if let serenity::ComponentInteractionDataKind::StringSelect { values } =
                &interaction.data.kind
            {
                if let Some(module_str) = values.first() {
                    let module_type = match module_str.as_str() {
                        "ChannelProtection" => ModuleType::ChannelProtection,
                        "ChannelPermissionProtection" => ModuleType::ChannelPermissionProtection,
                        "RoleProtection" => ModuleType::RoleProtection,
                        "RolePermissionProtection" => ModuleType::RolePermissionProtection,
                        "MemberPermissionProtection" => ModuleType::MemberPermissionProtection,
                        "BotAddingProtection" => ModuleType::BotAddingProtection,
                        "ModerationProtection" => ModuleType::ModerationProtection,
                        "Logging" => ModuleType::Logging,
                        "StickyRoles" => ModuleType::StickyRoles,
                        _ => return Ok(()),
                    };
                    updated_reply =
                        Some(build_module_menu(data, guild_id, module_type, 0, &l10n).await?);
                }
            }
        } else if let Some(rest) = custom_id.strip_prefix("config_page_") {
            let parts: Vec<&str> = rest.split('_').collect();
            if parts.len() >= 2 {
                let module_str = parts[0];
                let page_num: u32 = parts[1].parse().unwrap_or(0);
                let module_type = match module_str {
                    "channel_protection" | "ChannelProtection" => ModuleType::ChannelProtection,
                    "channel_permission_protection" | "ChannelPermissionProtection" => {
                        ModuleType::ChannelPermissionProtection
                    }
                    "role_protection" | "RoleProtection" => ModuleType::RoleProtection,
                    "role_permission_protection" | "RolePermissionProtection" => {
                        ModuleType::RolePermissionProtection
                    }
                    "member_permission_protection" | "MemberPermissionProtection" => {
                        ModuleType::MemberPermissionProtection
                    }
                    "bot_adding_protection" | "BotAddingProtection" => {
                        ModuleType::BotAddingProtection
                    }
                    "moderation_protection" | "ModerationProtection" => {
                        ModuleType::ModerationProtection
                    }
                    "logging" | "Logging" => ModuleType::Logging,
                    "sticky_roles" | "StickyRoles" => ModuleType::StickyRoles,
                    _ => return Ok(()),
                };
                updated_reply =
                    Some(build_module_menu(data, guild_id, module_type, page_num, &l10n).await?);
            }
        } else if custom_id.starts_with("config_module_log_channel_") {
            if let serenity::ComponentInteractionDataKind::ChannelSelect { values } =
                &interaction.data.kind
            {
                if let Some(channel_id) = values.first() {
                    let module_str = custom_id.trim_start_matches("config_module_log_channel_");
                    let module_type = match module_str {
                        "ChannelProtection" => ModuleType::ChannelProtection,
                        "ChannelPermissionProtection" => ModuleType::ChannelPermissionProtection,
                        "RoleProtection" => ModuleType::RoleProtection,
                        "RolePermissionProtection" => ModuleType::RolePermissionProtection,
                        "MemberPermissionProtection" => ModuleType::MemberPermissionProtection,
                        "BotAddingProtection" => ModuleType::BotAddingProtection,
                        "ModerationProtection" => ModuleType::ModerationProtection,
                        "InviteTracking" => ModuleType::InviteTracking,
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

                    updated_reply =
                        Some(build_module_menu(data, guild_id, module_type, page, &l10n).await?);
                }
            }
        } else if custom_id.starts_with("config_module_punishment_") {
            if let serenity::ComponentInteractionDataKind::StringSelect { values } =
                &interaction.data.kind
            {
                if let Some(p_str) = values.first() {
                    let module_str = custom_id.trim_start_matches("config_module_punishment_");
                    let module_type = match module_str {
                        "ChannelProtection" => ModuleType::ChannelProtection,
                        "ChannelPermissionProtection" => ModuleType::ChannelPermissionProtection,
                        "RoleProtection" => ModuleType::RoleProtection,
                        "RolePermissionProtection" => ModuleType::RolePermissionProtection,
                        "MemberPermissionProtection" => ModuleType::MemberPermissionProtection,
                        "BotAddingProtection" => ModuleType::BotAddingProtection,
                        "ModerationProtection" => ModuleType::ModerationProtection,
                        "InviteTracking" => ModuleType::InviteTracking,
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

                    updated_reply =
                        Some(build_module_menu(data, guild_id, module_type, page, &l10n).await?);
                }
            }
        } else if let Some(module_str) = custom_id.strip_prefix("config_module_revert_") {
            let module_type = match module_str {
                "channel_protection" | "ChannelProtection" => ModuleType::ChannelProtection,
                "channel_permission_protection" | "ChannelPermissionProtection" => {
                    ModuleType::ChannelPermissionProtection
                }
                "role_protection" | "RoleProtection" => ModuleType::RoleProtection,
                "role_permission_protection" | "RolePermissionProtection" => {
                    ModuleType::RolePermissionProtection
                }
                "member_permission_protection" | "MemberPermissionProtection" => {
                    ModuleType::MemberPermissionProtection
                }
                "bot_adding_protection" | "BotAddingProtection" => ModuleType::BotAddingProtection,
                "moderation_protection" | "ModerationProtection" => {
                    ModuleType::ModerationProtection
                }
                "logging" | "Logging" => ModuleType::Logging,
                _ => return Ok(()),
            };

            let module_configs::Model { revert, .. } =
                match module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
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

            updated_reply =
                Some(build_module_menu(data, guild_id, module_type, page, &l10n).await?);
        } else if custom_id.contains("_punish_at_") || custom_id.contains("_punish_interval_") {
            let module_type = if custom_id.contains("ChannelProtection") {
                ModuleType::ChannelProtection
            } else if custom_id.contains("ChannelPermissionProtection") {
                ModuleType::ChannelPermissionProtection
            } else if custom_id.contains("RoleProtection") {
                ModuleType::RoleProtection
            } else if custom_id.contains("RolePermissionProtection") {
                ModuleType::RolePermissionProtection
            } else if custom_id.contains("MemberPermissionProtection") {
                ModuleType::MemberPermissionProtection
            } else if custom_id.contains("BotAddingProtection") {
                ModuleType::BotAddingProtection
            } else if custom_id.contains("ModerationProtection") {
                ModuleType::ModerationProtection
            } else if custom_id.contains("Logging") {
                ModuleType::Logging
            } else {
                return Ok(());
            };

            let config = module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
                .one(&data.db)
                .await?;

            let (mut am, current_at, current_interval) = match config.as_ref() {
                Some(m) => (m.clone().into(), m.punishment_at, m.punishment_at_interval),
                None => {
                    let am = module_configs::ActiveModel {
                        guild_id: Set(guild_id.get() as i64),
                        module_type: Set(module_type),
                        punishment_at: Set(1),
                        punishment_at_interval: Set(10),
                        ..Default::default()
                    };
                    (am, 1, 10)
                }
            };

            if custom_id.contains("punish_at_inc") {
                am.punishment_at = Set(current_at + 1);
            } else if custom_id.contains("punish_at_dec") {
                am.punishment_at = Set((current_at - 1).max(1));
            } else if custom_id.contains("punish_interval_inc") {
                am.punishment_at_interval = Set(current_interval + 5);
            } else if custom_id.contains("punish_interval_dec") {
                am.punishment_at_interval = Set((current_interval - 5).max(1));
            }

            if config.is_some() {
                am.update(&data.db).await?;
            } else {
                am.insert(&data.db).await?;
            }
            updated_reply =
                Some(build_module_menu(data, guild_id, module_type, page, &l10n).await?);
        } else if custom_id.starts_with("config_module_toggle_") {
            let module_str = custom_id.trim_start_matches("config_module_toggle_");
            let module_type = match module_str {
                "channel_protection" | "ChannelProtection" => ModuleType::ChannelProtection,
                "channel_permission_protection" | "ChannelPermissionProtection" => {
                    ModuleType::ChannelPermissionProtection
                }
                "role_protection" | "RoleProtection" => ModuleType::RoleProtection,
                "role_permission_protection" | "RolePermissionProtection" => {
                    ModuleType::RolePermissionProtection
                }
                "member_permission_protection" | "MemberPermissionProtection" => {
                    ModuleType::MemberPermissionProtection
                }
                "bot_adding_protection" | "BotAddingProtection" => ModuleType::BotAddingProtection,
                "moderation_protection" | "ModerationProtection" => {
                    ModuleType::ModerationProtection
                }
                "logging" | "Logging" => ModuleType::Logging,
                "invite_tracking" | "InviteTracking" => ModuleType::InviteTracking,
                _ => return Ok(()),
            };

            let config = module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
                .one(&data.db)
                .await?;

            let (mut am, current_enabled): (module_configs::ActiveModel, bool) = match config {
                Some(m) => {
                    let enabled = m.enabled;
                    (m.into(), enabled)
                }
                None => {
                    let am = module_configs::ActiveModel {
                        guild_id: Set(guild_id.get() as i64),
                        module_type: Set(module_type),
                        enabled: Set(false), // Start disabled by default if not exist
                        ..Default::default()
                    };
                    let entry = am.insert(&data.db).await?;
                    (entry.into(), false)
                }
            };

            let new_enabled = !current_enabled;
            am.enabled = Set(new_enabled);
            am.update(&data.db).await?;

            // If invite tracking is enabled, sync invites
            if module_type == ModuleType::InviteTracking && new_enabled {
                if let Err(e) = crate::modules::invite_tracking::tracking::sync_all_guild_invites(
                    ctx, guild_id, data,
                )
                .await
                {
                    tracing::error!("Failed to sync invites on module enable: {:?}", e);
                }
            }

            updated_reply =
                Some(build_module_menu(data, guild_id, module_type, page, &l10n).await?);
        }
    }

    if let Some(components) = updated_reply {
        let edit = serenity::EditInteractionResponse::new()
            .components(components)
            .allowed_mentions(serenity::CreateAllowedMentions::new().empty_users().empty_roles());
        interaction.edit_response(&ctx.http, edit).await?;
    }

    Ok(())
}

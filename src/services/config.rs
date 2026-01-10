use crate::db::entities::guild_configs;
use crate::db::entities::module_configs::ModuleType;
use crate::services::localization::L10nProxy;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;

/// Reusable UI: Create a header with a separator
pub fn create_header(text: String, with_separator: bool) -> Vec<serenity::CreateComponent<'static>> {
    let mut components = vec![serenity::CreateComponent::TextDisplay(
        serenity::CreateTextDisplay::new(text),
    )];
    if with_separator {
        components.push(serenity::CreateComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
    }
    components
}

/// Reusable UI: Create an ActionRow with a SelectMenu
pub fn create_select_menu_row(
    id: String,
    kind: serenity::CreateSelectMenuKind<'static>,
    placeholder: String,
) -> serenity::CreateComponent<'static> {
    serenity::CreateComponent::ActionRow(serenity::CreateActionRow::SelectMenu(
        serenity::CreateSelectMenu::new(id, kind).placeholder(placeholder),
    ))
}

/// Reusable UI: Create a standard module configuration payload
pub fn create_module_config_payload(
    name: String,
    module_type: ModuleType,
    log_channel_id: Option<i64>,
    punishment: crate::db::entities::module_configs::PunishmentType,
    punishment_at: i32,
    punishment_at_interval: i32,
    revert: bool,
    l10n: &L10nProxy,
) -> Vec<serenity::CreateComponent<'static>> {
    let mut components = vec![];

    // Module Header
    components.extend(create_header(format!("⚙️ **{}**", name), true));

    // Log Channel Section
    components.push(serenity::CreateComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-log-channel-label", None)),
    ));

    components.push(create_select_menu_row(
        format!("config_module_log_channel_{:?}", module_type),
        serenity::CreateSelectMenuKind::Channel {
            channel_types: None,
            default_channels: log_channel_id
                .map(|id| vec![serenity::ChannelId::new(id as u64).into()].into()),
        },
        l10n.t("config-select-log-channel-placeholder", None),
    ));

    components.push(serenity::CreateComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));

    // Punishment Section
    components.push(serenity::CreateComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!("**{}**", l10n.t("config-punishment-label", None))),
    ));

    use crate::db::entities::module_configs::PunishmentType;
    let punishment_options = vec![
        PunishmentType::None,
        PunishmentType::Unperm,
        PunishmentType::Ban,
        PunishmentType::Kick,
        PunishmentType::Jail,
    ]
    .into_iter()
    .map(|p| {
        let is_default = p == punishment;
        let p_str = format!("{:?}", p).to_lowercase();
        serenity::CreateSelectMenuOption::new(
            l10n.t(&format!("config-punishment-type-{}", p_str), None),
            p_str,
        )
        .default_selection(is_default)
    })
    .collect::<Vec<_>>();

    components.push(create_select_menu_row(
        format!("config_module_punishment_{:?}", module_type),
        serenity::CreateSelectMenuKind::String {
            options: punishment_options.into(),
        },
        l10n.t("config-select-punishment-placeholder", None),
    ));

    // Revert and Navigation Buttons
    let revert_btn = serenity::CreateButton::new(format!("config_module_revert_{:?}", module_type))
        .label(if revert {
            l10n.t("config-revert-enabled", None)
        } else {
            l10n.t("config-revert-disabled", None)
        })
        .style(if revert {
            serenity::ButtonStyle::Success
        } else {
            serenity::ButtonStyle::Secondary
        });

    let back_btn = serenity::CreateButton::new("config_back_to_main")
        .label(l10n.t("config-back-label", None))
        .style(serenity::ButtonStyle::Secondary);

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![revert_btn, back_btn].into()),
    ));

    // Display Current Repetition Settings
    components.push(serenity::CreateComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));
    components.push(serenity::CreateComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "**At Repetition:** {}\n**Interval:** {} min",
            punishment_at, punishment_at_interval
        )),
    ));

    components
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

    let mut components = vec![];

    // General Configuration Header
    components.extend(create_header(l10n.t("config-general-header", None), true));

    // Log Channel Section
    components.push(serenity::CreateComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-log-channel-label", None)),
    ));

    components.push(create_select_menu_row(
        "config_general_log_channel".to_string(),
        serenity::CreateSelectMenuKind::Channel {
            channel_types: None,
            default_channels: log_channel_id
                .map(|id| vec![serenity::ChannelId::new(id as u64).into()].into()),
        },
        l10n.t("config-select-log-channel-placeholder", None),
    ));

    // Jail Role Section
    components.push(serenity::CreateComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));
    components.push(serenity::CreateComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-jail-role-label", None)),
    ));

    components.push(create_select_menu_row(
        "config_jail_role".to_string(),
        serenity::CreateSelectMenuKind::Role {
            default_roles: jail_role_id
                .map(|id| vec![serenity::RoleId::new(id as u64).into()].into()),
        },
        l10n.t("config-select-jail-role-placeholder", None),
    ));

    // Modules Section
    components.extend(create_header(l10n.t("config-modules-header", None), true));

    let channel_prot_btn = serenity::CreateButton::new("config_navigate_module_ChannelProtection")
        .label(l10n.t("config-channel-protection-label", None))
        .style(serenity::ButtonStyle::Secondary);

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![channel_prot_btn].into()),
    ));

    Ok(components)
}

/// Builds the module-specific configuration menu
pub async fn build_module_menu(
    data: &Data,
    guild_id: serenity::GuildId,
    module: ModuleType,
    l10n: &L10nProxy,
) -> Result<Vec<serenity::CreateComponent<'static>>, Error> {
    use crate::db::entities::module_configs;

    let m_config = match module_configs::Entity::find_by_id((guild_id.get() as i64, module))
        .one(&data.db)
        .await?
    {
        Some(m) => m,
        None => {
            // Create default config if not exists
            use sea_orm::{ActiveModelTrait, Set};
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
        _ => format!("{:?}", module),
    };

    Ok(create_module_config_payload(
        name,
        module,
        m_config.log_channel_id,
        m_config.punishment,
        m_config.punishment_at,
        m_config.punishment_at_interval,
        m_config.revert,
        l10n,
    ))
}

/// Global configuration command
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn config(ctx: crate::Context<'_>) -> Result<(), Error> {
    let l10n = crate::services::localization::ContextL10nExt::l10n_guild(&ctx);
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

    if custom_id == "config_general_log_channel" {
        if let serenity::ComponentInteractionDataKind::ChannelSelect { values } = &interaction.data.kind {
            if let Some(channel_id) = values.first() {
                // Update DB
                use crate::db::entities::guild_configs;
                use sea_orm::{ActiveModelTrait, Set};

                let mut config: guild_configs::ActiveModel =
                    match guild_configs::Entity::find_by_id(guild_id.get() as i64)
                        .one(&data.db)
                        .await?
                    {
                        Some(m) => m.into(),
                        None => guild_configs::ActiveModel {
                            guild_id: Set(guild_id.get() as i64),
                            ..Default::default()
                        },
                    };

                config.log_channel_id = Set(Some(channel_id.get() as i64));
                config.save(&data.db).await?;

                updated_reply = Some(build_main_menu(data, guild_id, &l10n).await?);
            }
        }
    } else if custom_id == "config_jail_role" {
        if let serenity::ComponentInteractionDataKind::RoleSelect { values } = &interaction.data.kind {
            if let Some(role_id) = values.first() {
                // Update DB
                use crate::db::entities::guild_configs;
                use sea_orm::{ActiveModelTrait, Set};

                let mut config: guild_configs::ActiveModel =
                    match guild_configs::Entity::find_by_id(guild_id.get() as i64)
                        .one(&data.db)
                        .await?
                    {
                        Some(m) => m.into(),
                        None => guild_configs::ActiveModel {
                            guild_id: Set(guild_id.get() as i64),
                            ..Default::default()
                        },
                    };

                config.jail_role_id = Set(Some(role_id.get() as i64));
                config.save(&data.db).await?;

                updated_reply = Some(build_main_menu(data, guild_id, &l10n).await?);
            }
        }
    } else if custom_id.starts_with("config_navigate_module_") {
        let module_str = custom_id.trim_start_matches("config_navigate_module_");
        let module_type = match module_str {
            "ChannelProtection" => ModuleType::ChannelProtection,
            _ => return Ok(()),
        };

        updated_reply = Some(build_module_menu(data, guild_id, module_type, &l10n).await?);
    } else if custom_id.starts_with("config_module_log_channel_") {
        if let serenity::ComponentInteractionDataKind::ChannelSelect { values } = &interaction.data.kind {
            if let Some(channel_id) = values.first() {
                let module_str = custom_id.trim_start_matches("config_module_log_channel_");
                let module_type = match module_str {
                    "ChannelProtection" => ModuleType::ChannelProtection,
                    _ => return Ok(()),
                };

                use crate::db::entities::module_configs;
                use sea_orm::{ActiveModelTrait, Set};

                let mut config: module_configs::ActiveModel =
                    match module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
                        .one(&data.db)
                        .await?
                    {
                        Some(m) => m.into(),
                        None => module_configs::ActiveModel {
                            guild_id: Set(guild_id.get() as i64),
                            module_type: Set(module_type),
                            ..Default::default()
                        },
                    };

                config.log_channel_id = Set(Some(channel_id.get() as i64));
                config.save(&data.db).await?;

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

                use crate::db::entities::module_configs::{self, PunishmentType};
                use sea_orm::{ActiveModelTrait, Set};

                let punishment = match p_str.as_str() {
                    "none" => PunishmentType::None,
                    "unperm" => PunishmentType::Unperm,
                    "ban" => PunishmentType::Ban,
                    "kick" => PunishmentType::Kick,
                    "jail" => PunishmentType::Jail,
                    _ => return Ok(()),
                };

                let mut config: module_configs::ActiveModel =
                    match module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
                        .one(&data.db)
                        .await?
                    {
                        Some(m) => m.into(),
                        None => module_configs::ActiveModel {
                            guild_id: Set(guild_id.get() as i64),
                            module_type: Set(module_type),
                            ..Default::default()
                        },
                    };

                config.punishment = Set(punishment);
                config.save(&data.db).await?;

                updated_reply = Some(build_module_menu(data, guild_id, module_type, &l10n).await?);
            }
        }
    } else if custom_id.starts_with("config_module_revert_") {
        let module_str = custom_id.trim_start_matches("config_module_revert_");
        let module_type = match module_str {
            "ChannelProtection" => ModuleType::ChannelProtection,
            _ => return Ok(()),
        };

        use crate::db::entities::module_configs;
        use sea_orm::{ActiveModelTrait, Set, ActiveValue};

        let mut config: module_configs::ActiveModel =
            match module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
                .one(&data.db)
                .await?
            {
                Some(m) => m.into(),
                None => module_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    module_type: Set(module_type),
                    ..Default::default()
                },
            };

        let current_revert = match &config.revert {
            ActiveValue::Set(v) | ActiveValue::Unchanged(v) => *v,
            _ => true,
        };

        config.revert = Set(!current_revert);
        config.save(&data.db).await?;

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

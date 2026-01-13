use crate::Data;
use crate::db::entities::module_configs::{
    self, ChannelPermissionProtectionModuleConfig, ModuleType,
};
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

pub fn build_ui(
    config: &ChannelPermissionProtectionModuleConfig,
    l10n: &L10nProxy,
) -> Vec<serenity::CreateContainerComponent<'static>> {
    let mut components = vec![];

    // Ignore Private Channels Section
    let ignore_btn_label = if config.ignore_private_channels {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let ignore_btn = serenity::CreateButton::new("config_cpp_ignore_private_toggle")
        .label(ignore_btn_label)
        .style(if config.ignore_private_channels {
            serenity::ButtonStyle::Success
        } else {
            serenity::ButtonStyle::Secondary
        });

    components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new(l10n.t("config-cpp-ignore-private-label", None)),
            )],
            serenity::CreateSectionAccessory::Button(ignore_btn),
        ),
    ));

    // Punish When Multi-Select
    let options = vec![
        serenity::CreateSelectMenuOption::new(l10n.t("config-cpp-punish-create", None), "create")
            .default_selection(config.punish_when.contains(&"create".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("config-cpp-punish-update", None), "update")
            .default_selection(config.punish_when.contains(&"update".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("config-cpp-punish-delete", None), "delete")
            .default_selection(config.punish_when.contains(&"delete".to_string())),
    ];

    let select_menu = serenity::CreateSelectMenu::new(
        "config_cpp_punish_when",
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder(l10n.t("config-cpp-punish-when-placeholder", None))
    .min_values(0)
    .max_values(3);

    components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::SelectMenu(select_menu),
    ));

    components
}

pub async fn handle_interaction(
    _ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<bool, crate::Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id == "config_cpp_ignore_private_toggle" {
        let (config_active, mut config) = get_config(data, guild_id).await?;
        config.ignore_private_channels = !config.ignore_private_channels;
        save_config(data, config_active, config).await?;
        return Ok(true);
    }

    if custom_id == "config_cpp_punish_when" {
        if let serenity::ComponentInteractionDataKind::StringSelect { values } =
            &interaction.data.kind
        {
            let (config_active, mut config) = get_config(data, guild_id).await?;
            config.punish_when = values.to_vec();
            save_config(data, config_active, config).await?;
            return Ok(true);
        }
    }

    Ok(false)
}

async fn get_config(
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<
    (
        module_configs::ActiveModel,
        ChannelPermissionProtectionModuleConfig,
    ),
    crate::Error,
> {
    let db = &data.db;
    let m_config = module_configs::Entity::find_by_id((
        guild_id.get() as i64,
        ModuleType::ChannelPermissionProtection,
    ))
    .one(db)
    .await?
    .ok_or_else(|| anyhow::anyhow!("Config not found"))?;

    let config: ChannelPermissionProtectionModuleConfig =
        serde_json::from_value(m_config.config.clone()).unwrap_or_default();
    Ok((m_config.into(), config))
}

async fn save_config(
    data: &Data,
    mut config_active: module_configs::ActiveModel,
    config: ChannelPermissionProtectionModuleConfig,
) -> Result<(), crate::Error> {
    config_active.config = Set(serde_json::to_value(config)?);
    config_active.update(&data.db).await?;
    Ok(())
}

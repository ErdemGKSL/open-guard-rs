use crate::db::entities::module_configs::{self, RoleProtectionModuleConfig, ModuleType};
use crate::services::localization::L10nProxy;
use crate::Data;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

pub fn build_ui(
    config: &RoleProtectionModuleConfig,
    l10n: &L10nProxy,
) -> Vec<serenity::CreateContainerComponent<'static>> {
    let mut components = vec![];

    // Punish When Multi-Select
    let options = vec![
        serenity::CreateSelectMenuOption::new(l10n.t("config-rp-punish-create", None), "create")
            .default_selection(config.punish_when.contains(&"create".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("config-rp-punish-update", None), "update")
            .default_selection(config.punish_when.contains(&"update".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("config-rp-punish-delete", None), "delete")
            .default_selection(config.punish_when.contains(&"delete".to_string())),
    ];

    let select_menu = serenity::CreateSelectMenu::new(
        "config_rp_punish_when",
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder(l10n.t("config-rp-punish-when-placeholder", None))
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

    if custom_id == "config_rp_punish_when" {
        if let serenity::ComponentInteractionDataKind::StringSelect { values } = &interaction.data.kind {
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
) -> Result<(module_configs::ActiveModel, RoleProtectionModuleConfig), crate::Error> {
    let db = &data.db;
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::RoleProtection))
        .one(db)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Config not found"))?;

    let config: RoleProtectionModuleConfig = serde_json::from_value(m_config.config.clone()).unwrap_or_default();
    Ok((m_config.into(), config))
}

async fn save_config(
    data: &Data,
    mut config_active: module_configs::ActiveModel,
    config: RoleProtectionModuleConfig,
) -> Result<(), crate::Error> {
    config_active.config = Set(serde_json::to_value(config)?);
    config_active.update(&data.db).await?;
    Ok(())
}


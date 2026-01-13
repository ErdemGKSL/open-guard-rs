use crate::Data;
use crate::db::entities::module_configs::{self, ModuleType, StickyRolesModuleConfig};
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

pub fn build_ui(
    _config: &StickyRolesModuleConfig,
    _l10n: &L10nProxy,
) -> Vec<serenity::CreateContainerComponent<'static>> {
    // Currently no specific options for sticky roles
    vec![]
}

pub async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<bool, crate::Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id.starts_with("config_module_toggle_sticky_roles")
        || custom_id == "config_module_toggle_StickyRoles"
    {
        let (config_active, enabled) = get_config_status(data, guild_id).await?;
        let new_enabled = !enabled;

        let mut active = config_active;
        active.enabled = Set(new_enabled);
        active.update(&data.db).await?;

        if new_enabled {
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
                        "Failed to fetch and store members for guild {} (StickyRoles): {:?}",
                        guild_id,
                        e
                    );
                }
            });
        } else {
            // Disabling: check if logging also disabled
            if !is_logging_membership_enabled(data, guild_id).await? {
                crate::modules::logging::events::membership::delete_all_guild_member_roles(
                    guild_id, data,
                )
                .await?;
            }
        }

        return Ok(true);
    }

    Ok(false)
}

async fn get_config_status(
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<(module_configs::ActiveModel, bool), crate::Error> {
    let db = &data.db;
    let m_config =
        module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::StickyRoles))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Config not found"))?;

    Ok((m_config.clone().into(), m_config.enabled))
}

async fn is_logging_membership_enabled(
    data: &Data,
    guild_id: serenity::GuildId,
) -> Result<bool, crate::Error> {
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::Logging))
        .one(&data.db)
        .await?;

    if let Some(m) = m_config {
        if !m.enabled {
            return Ok(false);
        }
        let config: crate::db::entities::module_configs::LoggingModuleConfig =
            serde_json::from_value(m.config).unwrap_or_default();
        return Ok(config.log_membership);
    }
    Ok(false)
}

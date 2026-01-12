use crate::db::entities::module_configs::RolePermissionProtectionModuleConfig;
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

pub fn build_ui(
    _config: &RolePermissionProtectionModuleConfig,
    _l10n: &L10nProxy,
) -> Vec<serenity::CreateContainerComponent<'static>> {
    // Role Permission Protection currently has no specific configuration options
    // besides the common ones (punishment, log channel, etc.)
    vec![]
}

pub async fn handle_interaction(
    _ctx: &serenity::Context,
    _interaction: &serenity::ComponentInteraction,
    _data: &crate::Data,
    _guild_id: serenity::GuildId,
) -> Result<bool, crate::Error> {
    Ok(false)
}


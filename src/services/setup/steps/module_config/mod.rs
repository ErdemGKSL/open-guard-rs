use crate::services::localization::L10nProxy;
use crate::db::entities::module_configs::ModuleType;
use poise::serenity_prelude as serenity;

pub mod logging;

pub fn build_module_config_step(
    setup_id: &str,
    l10n: &L10nProxy,
    module: ModuleType,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    match module {
        ModuleType::Logging => logging::build_ui(setup_id, l10n),
        _ => build_generic_ui(setup_id, l10n, module),
    }
}

fn build_generic_ui(
    setup_id: &str,
    l10n: &L10nProxy,
    module: ModuleType,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    let label = match module {
        ModuleType::ChannelProtection => l10n.t("config-channel-protection-label", None),
        ModuleType::ChannelPermissionProtection => l10n.t("config-channel-permission-protection-label", None),
        ModuleType::RoleProtection => l10n.t("config-role-protection-label", None),
        ModuleType::RolePermissionProtection => l10n.t("config-role-permission-protection-label", None),
        ModuleType::MemberPermissionProtection => l10n.t("config-member-permission-protection-label", None),
        ModuleType::BotAddingProtection => l10n.t("config-bot-adding-protection-label", None),
        ModuleType::ModerationProtection => l10n.t("config-moderation-protection-label", None),
        ModuleType::Logging => l10n.t("config-logging-label", None),
        ModuleType::StickyRoles => l10n.t("config-sticky-roles-label", None),
    };

    let next_button = serenity::CreateButton::new(format!("setup_module_next_{}_{:?}", setup_id, module))
        .label(l10n.t("setup-next", None))
        .style(serenity::ButtonStyle::Primary);

    let mut args = fluent::FluentArgs::new();
    args.set("label", label);

    (
        format!("{}\n{}", l10n.t("setup-step4-title", Some(&args)), l10n.t("setup-step4-generic-desc", None)),
        vec![serenity::CreateComponent::ActionRow(serenity::CreateActionRow::buttons(vec![next_button]))],
    )
}

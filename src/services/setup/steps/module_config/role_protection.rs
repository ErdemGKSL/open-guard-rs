use crate::db::entities::module_configs::RoleProtectionModuleConfig;
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

/// Initial UI builder (uses defaults)
pub fn build_ui(
    setup_id: &str,
    l10n: &L10nProxy,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    build_ui_with_config(setup_id, l10n, &Default::default())
}

/// UI builder with current config state
pub fn build_ui_with_config(
    setup_id: &str,
    l10n: &L10nProxy,
    config: &RoleProtectionModuleConfig,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    let mut components = vec![];

    // Punish When Multi-Select
    let options = vec![
        serenity::CreateSelectMenuOption::new(l10n.t("setup-rp-punish-create", None), "create")
            .default_selection(config.punish_when.contains(&"create".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("setup-rp-punish-update", None), "update")
            .default_selection(config.punish_when.contains(&"update".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("setup-rp-punish-delete", None), "delete")
            .default_selection(config.punish_when.contains(&"delete".to_string())),
    ];

    let select_menu = serenity::CreateSelectMenu::new(
        format!("setup_module_rp_punish_when_{}", setup_id),
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder(l10n.t("setup-rp-punish-when-placeholder", None))
    .min_values(0)
    .max_values(3);

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::select_menu(select_menu),
    ));

    // Next Button
    let next_button =
        serenity::CreateButton::new(format!("setup_module_next_{}_RoleProtection", setup_id))
            .label(l10n.t("setup-next", None))
            .style(serenity::ButtonStyle::Primary);

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::buttons(vec![next_button]),
    ));

    // Build content
    let mut args = fluent::FluentArgs::new();
    args.set("label", l10n.t("config-role-protection-label", None));

    let content = format!(
        "{}\n{}",
        l10n.t("setup-step4-title", Some(&args)),
        l10n.t("setup-rp-desc", None)
    );

    (content, components)
}

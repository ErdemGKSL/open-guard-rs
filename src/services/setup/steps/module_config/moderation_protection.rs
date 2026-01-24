use crate::db::entities::module_configs::ModerationProtectionModuleConfig;
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

/// Initial UI builder (uses defaults)
pub fn build_ui(setup_id: &str, l10n: &L10nProxy) -> Vec<serenity::CreateComponent<'static>> {
    build_ui_with_config(setup_id, l10n, &Default::default())
}

/// UI builder with current config state
pub fn build_ui_with_config(
    setup_id: &str,
    l10n: &L10nProxy,
    config: &ModerationProtectionModuleConfig,
) -> Vec<serenity::CreateComponent<'static>> {
    let mut inner_components = vec![];

    // Build title and description
    let mut args = fluent::FluentArgs::new();
    args.set("label", l10n.t("config-moderation-protection-label", None));

    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "## {}\n{}",
            l10n.t("setup-step4-title", Some(&args)),
            l10n.t("setup-mp-desc", None)
        )),
    ));

    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Punish When Multi-Select
    let options = vec![
        serenity::CreateSelectMenuOption::new(l10n.t("setup-mp-punish-ban", None), "ban")
            .default_selection(config.punish_when.contains(&"ban".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("setup-mp-punish-kick", None), "kick")
            .default_selection(config.punish_when.contains(&"kick".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("setup-mp-punish-timeout", None), "timeout")
            .default_selection(config.punish_when.contains(&"timeout".to_string())),
    ];

    let select_menu = serenity::CreateSelectMenu::new(
        format!("setup_module_mp_punish_when_{}", setup_id),
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder(l10n.t("setup-mp-punish-when-placeholder", None))
    .min_values(0)
    .max_values(3);

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::SelectMenu(select_menu),
    ));

    // Next Button
    let next_button = serenity::CreateButton::new(format!(
        "setup_module_next_{}_ModerationProtection",
        setup_id
    ))
    .label(l10n.t("setup-next", None))
    .style(serenity::ButtonStyle::Primary);

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![next_button].into()),
    ));

    vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components),
    )]
}

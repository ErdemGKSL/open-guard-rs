use crate::db::entities::module_configs::ChannelProtectionModuleConfig;
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
    config: &ChannelProtectionModuleConfig,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    let mut components = vec![];

    // Ignore Private Channels Toggle Button
    let toggle_label = if config.ignore_private_channels {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let toggle_button = serenity::CreateButton::new(format!(
        "setup_module_cp_ignore_private_toggle_{}",
        setup_id
    ))
    .label(toggle_label)
    .style(if config.ignore_private_channels {
        serenity::ButtonStyle::Success
    } else {
        serenity::ButtonStyle::Secondary
    });

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::buttons(vec![toggle_button]),
    ));

    // Punish When Multi-Select
    let options = vec![
        serenity::CreateSelectMenuOption::new(l10n.t("setup-cp-punish-create", None), "create")
            .default_selection(config.punish_when.contains(&"create".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("setup-cp-punish-update", None), "update")
            .default_selection(config.punish_when.contains(&"update".to_string())),
        serenity::CreateSelectMenuOption::new(l10n.t("setup-cp-punish-delete", None), "delete")
            .default_selection(config.punish_when.contains(&"delete".to_string())),
    ];

    let select_menu = serenity::CreateSelectMenu::new(
        format!("setup_module_cp_punish_when_{}", setup_id),
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .placeholder(l10n.t("setup-cp-punish-when-placeholder", None))
    .min_values(0)
    .max_values(3);

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::select_menu(select_menu),
    ));

    // Next Button
    let next_button =
        serenity::CreateButton::new(format!("setup_module_next_{}_ChannelProtection", setup_id))
            .label(l10n.t("setup-next", None))
            .style(serenity::ButtonStyle::Primary);

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::buttons(vec![next_button]),
    ));

    // Build content
    let mut args = fluent::FluentArgs::new();
    args.set("label", l10n.t("config-channel-protection-label", None));

    let content = format!(
        "{}\n{}",
        l10n.t("setup-step4-title", Some(&args)),
        l10n.t("setup-cp-desc", None)
    );

    (content, components)
}

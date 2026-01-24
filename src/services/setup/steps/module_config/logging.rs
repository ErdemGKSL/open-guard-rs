use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

pub fn build_ui(setup_id: &str, l10n: &L10nProxy) -> Vec<serenity::CreateComponent<'static>> {
    let options = vec![
        serenity::CreateSelectMenuOption::new(l10n.t("setup-log-type-messages", None), "messages")
            .description(l10n.t("setup-log-type-messages-desc", None)),
        serenity::CreateSelectMenuOption::new(l10n.t("setup-log-type-voice", None), "voice")
            .description(l10n.t("setup-log-type-voice-desc", None)),
        serenity::CreateSelectMenuOption::new(
            l10n.t("setup-log-type-membership", None),
            "membership",
        )
        .description(l10n.t("setup-log-type-membership-desc", None)),
    ];

    let select = serenity::CreateSelectMenu::new(
        format!("setup_module_logging_types_{}", setup_id),
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .min_values(0)
    .max_values(3)
    .placeholder(l10n.t("setup-logging-types-placeholder", None));

    let next_button =
        serenity::CreateButton::new(format!("setup_module_next_{}_Logging", setup_id))
            .label(l10n.t("setup-next", None))
            .style(serenity::ButtonStyle::Primary);

    let mut inner_components = vec![];

    // Add title and description
    let mut title_args = fluent::FluentArgs::new();
    title_args.set("label", l10n.t("config-logging-label", None));

    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "## {}\n{}",
            l10n.t("setup-step4-title", Some(&title_args)),
            l10n.t("setup-step4-logging-desc", None)
        )),
    ));

    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Add select menu
    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::SelectMenu(select),
    ));

    // Add next button
    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![next_button].into()),
    ));

    vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components),
    )]
}

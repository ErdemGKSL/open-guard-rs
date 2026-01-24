use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

pub fn build_whitelist_step(
    setup_id: &str,
    l10n: &L10nProxy,
) -> Vec<serenity::CreateComponent<'static>> {
    let user_select = serenity::CreateSelectMenu::new(
        format!("setup_whitelist_users_{}", setup_id),
        serenity::CreateSelectMenuKind::User {
            default_users: None,
        },
    )
    .placeholder(l10n.t("setup-whitelist-users-placeholder", None))
    .min_values(0)
    .max_values(25);

    let role_select = serenity::CreateSelectMenu::new(
        format!("setup_whitelist_roles_{}", setup_id),
        serenity::CreateSelectMenuKind::Role {
            default_roles: None,
        },
    )
    .placeholder(l10n.t("setup-whitelist-roles-placeholder", None))
    .min_values(0)
    .max_values(25);

    let next_button = serenity::CreateButton::new(format!("setup_whitelist_next_{}", setup_id))
        .label(l10n.t("setup-next", None))
        .style(serenity::ButtonStyle::Primary);

    let mut inner_components = vec![];

    // Add title and description
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "## {}\n{}",
            l10n.t("setup-step3-title", None),
            l10n.t("setup-step3-desc", None)
        )),
    ));

    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Add select menus
    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::SelectMenu(user_select),
    ));

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::SelectMenu(role_select),
    ));

    // Add next button
    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![next_button].into()),
    ));

    vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components),
    )]
}

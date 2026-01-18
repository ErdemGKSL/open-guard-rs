use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

pub fn build_whitelist_step(
    setup_id: &str,
    l10n: &L10nProxy,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    let user_select = serenity::CreateSelectMenu::new(
        format!("setup_whitelist_users_{}", setup_id),
        serenity::CreateSelectMenuKind::User { default_users: None },
    )
    .placeholder(l10n.t("setup-whitelist-users-placeholder", None))
    .min_values(0)
    .max_values(25);

    let role_select = serenity::CreateSelectMenu::new(
        format!("setup_whitelist_roles_{}", setup_id),
        serenity::CreateSelectMenuKind::Role { default_roles: None },
    )
    .placeholder(l10n.t("setup-whitelist-roles-placeholder", None))
    .min_values(0)
    .max_values(25);

    let next_button = serenity::CreateButton::new(format!("setup_whitelist_next_{}", setup_id))
        .label(l10n.t("setup-next", None))
        .style(serenity::ButtonStyle::Primary);

    (
        format!("{}\n{}", l10n.t("setup-step3-title", None), l10n.t("setup-step3-desc", None)),
        vec![
            serenity::CreateComponent::ActionRow(serenity::CreateActionRow::select_menu(user_select)),
            serenity::CreateComponent::ActionRow(serenity::CreateActionRow::select_menu(role_select)),
            serenity::CreateComponent::ActionRow(serenity::CreateActionRow::buttons(vec![next_button])),
        ],
    )
}

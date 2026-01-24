use crate::db::entities::module_configs::InviteTrackingModuleConfig;
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
    config: &InviteTrackingModuleConfig,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    let mut components = vec![];

    // Track Vanity URL Toggle
    let vanity_label = if config.track_vanity {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let vanity_button =
        serenity::CreateButton::new(format!("setup_module_it_vanity_toggle_{}", setup_id))
            .label(vanity_label)
            .style(if config.track_vanity {
                serenity::ButtonStyle::Success
            } else {
                serenity::ButtonStyle::Secondary
            });

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::buttons(vec![vanity_button]),
    ));

    // Ignore Bots Toggle
    let bots_label = if config.ignore_bots {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let bots_button =
        serenity::CreateButton::new(format!("setup_module_it_ignore_bots_toggle_{}", setup_id))
            .label(bots_label)
            .style(if config.ignore_bots {
                serenity::ButtonStyle::Success
            } else {
                serenity::ButtonStyle::Secondary
            });

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::buttons(vec![bots_button]),
    ));

    // Next Button
    let next_button =
        serenity::CreateButton::new(format!("setup_module_next_{}_InviteTracking", setup_id))
            .label(l10n.t("setup-next", None))
            .style(serenity::ButtonStyle::Primary);

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::buttons(vec![next_button]),
    ));

    // Build content
    let mut args = fluent::FluentArgs::new();
    args.set("label", l10n.t("config-invite-tracking-label", None));

    let content = format!(
        "{}\n{}",
        l10n.t("setup-step4-title", Some(&args)),
        l10n.t("setup-it-desc", None)
    );

    (content, components)
}

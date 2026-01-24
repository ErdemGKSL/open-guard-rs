use crate::db::entities::module_configs::InviteTrackingModuleConfig;
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
    config: &InviteTrackingModuleConfig,
) -> Vec<serenity::CreateComponent<'static>> {
    let mut inner_components = vec![];

    // Build title and description
    let mut args = fluent::FluentArgs::new();
    args.set("label", l10n.t("config-invite-tracking-label", None));

    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "## {}\n{}",
            l10n.t("setup-step4-title", Some(&args)),
            l10n.t("setup-it-desc", None)
        )),
    ));

    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Track Vanity URL Toggle
    let vanity_label = if config.track_vanity {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let vanity_button =
        serenity::CreateButton::new(format!("setup_module_it_vanity_toggle_{}", setup_id))
            .label(format!("Vanity: {}", vanity_label))
            .style(if config.track_vanity {
                serenity::ButtonStyle::Success
            } else {
                serenity::ButtonStyle::Secondary
            });

    // Ignore Bots Toggle
    let bots_label = if config.ignore_bots {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let bots_button =
        serenity::CreateButton::new(format!("setup_module_it_ignore_bots_toggle_{}", setup_id))
            .label(format!("Bots: {}", bots_label))
            .style(if config.ignore_bots {
                serenity::ButtonStyle::Success
            } else {
                serenity::ButtonStyle::Secondary
            });

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![vanity_button, bots_button].into()),
    ));

    // Minimum Account Age
    let mut age_args = fluent_bundle::FluentArgs::new();
    age_args.set("count", config.minimum_account_age_days);
    let age_label = l10n.t("config-it-min-age-label", Some(&age_args));

    let age_dec = serenity::CreateButton::new(format!("setup_module_it_min_age_dec_{}", setup_id))
        .label("-")
        .style(serenity::ButtonStyle::Secondary);
    let age_display =
        serenity::CreateButton::new(format!("setup_module_it_min_age_val_{}", setup_id))
            .label(age_label)
            .style(serenity::ButtonStyle::Secondary)
            .disabled(true);
    let age_inc = serenity::CreateButton::new(format!("setup_module_it_min_age_inc_{}", setup_id))
        .label("+")
        .style(serenity::ButtonStyle::Secondary);

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![age_dec, age_display, age_inc].into()),
    ));

    // Fake Threshold
    let mut fake_args = fluent_bundle::FluentArgs::new();
    fake_args.set("count", config.fake_threshold_hours);
    let fake_label = l10n.t("config-it-fake-threshold-label", Some(&fake_args));

    let fake_dec = serenity::CreateButton::new(format!("setup_module_it_fake_dec_{}", setup_id))
        .label("-")
        .style(serenity::ButtonStyle::Secondary);
    let fake_display =
        serenity::CreateButton::new(format!("setup_module_it_fake_val_{}", setup_id))
            .label(fake_label)
            .style(serenity::ButtonStyle::Secondary)
            .disabled(true);
    let fake_inc = serenity::CreateButton::new(format!("setup_module_it_fake_inc_{}", setup_id))
        .label("+")
        .style(serenity::ButtonStyle::Secondary);

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![fake_dec, fake_display, fake_inc].into()),
    ));

    // Leaderboard Limit
    let mut limit_args = fluent_bundle::FluentArgs::new();
    limit_args.set("count", config.leaderboard_limit);
    let limit_label = l10n.t("config-it-leaderboard-limit-label", Some(&limit_args));

    let limit_dec = serenity::CreateButton::new(format!("setup_module_it_limit_dec_{}", setup_id))
        .label("-")
        .style(serenity::ButtonStyle::Secondary);
    let limit_display =
        serenity::CreateButton::new(format!("setup_module_it_limit_val_{}", setup_id))
            .label(limit_label)
            .style(serenity::ButtonStyle::Secondary)
            .disabled(true);
    let limit_inc = serenity::CreateButton::new(format!("setup_module_it_limit_inc_{}", setup_id))
        .label("+")
        .style(serenity::ButtonStyle::Secondary);

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![limit_dec, limit_display, limit_inc].into()),
    ));

    // Next Button
    let next_button =
        serenity::CreateButton::new(format!("setup_module_next_{}_InviteTracking", setup_id))
            .label(l10n.t("setup-next", None))
            .style(serenity::ButtonStyle::Primary);

    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(vec![next_button].into()),
    ));

    vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components),
    )]
}

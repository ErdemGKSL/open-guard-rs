use super::super::state::SetupState;
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

pub fn build_summary_step(
    setup_id: &str,
    l10n: &L10nProxy,
    state: &SetupState,
) -> Vec<serenity::CreateComponent<'static>> {
    let mut summary = format!(
        "## {}\n{}\n\n",
        l10n.t("setup-summary-title", None),
        l10n.t("setup-summary-desc", None)
    );

    summary.push_str(&format!(
        "**{}**\n",
        l10n.t("setup-summary-enabled-modules", None)
    ));
    if state.enabled_modules.is_empty() {
        summary.push_str(&format!("- {}\n", l10n.t("setup-summary-none", None)));
    } else {
        for module in &state.enabled_modules {
            summary.push_str(&format!("- {:?}\n", module));
        }
    }

    summary.push_str(&format!(
        "\n**{}**\n",
        l10n.t("setup-summary-fallback-log", None)
    ));
    if let Some(channel_id) = state.fallback_log_channel {
        summary.push_str(&format!("- <#{}>\n", channel_id));
    } else {
        summary.push_str(&format!(
            "- {}\n",
            l10n.t("config-punishment-type-none", None)
        ));
    }

    summary.push_str(&format!(
        "\n**{}**\n",
        l10n.t("setup-summary-whitelist", None)
    ));

    let mut user_args = fluent::FluentArgs::new();
    user_args.set("count", state.whitelist_users.len());
    summary.push_str(&format!(
        "- {}\n",
        l10n.t("setup-summary-users", Some(&user_args))
    ));

    let mut role_args = fluent::FluentArgs::new();
    role_args.set("count", state.whitelist_roles.len());
    summary.push_str(&format!(
        "- {}\n",
        l10n.t("setup-summary-roles", Some(&role_args))
    ));

    let mut inner_components = vec![];

    // Add summary text
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(summary),
    ));

    inner_components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Add action buttons
    inner_components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::Buttons(
            vec![
                serenity::CreateButton::new(format!("setup_apply_{}", setup_id))
                    .label(l10n.t("setup-apply", None))
                    .style(serenity::ButtonStyle::Success),
                serenity::CreateButton::new(format!("setup_cancel_{}", setup_id))
                    .label(l10n.t("setup-cancel", None))
                    .style(serenity::ButtonStyle::Danger),
            ]
            .into(),
        ),
    ));

    vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components),
    )]
}

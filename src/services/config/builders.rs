use crate::db::entities::module_configs::ModuleType;
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

/// Reusable UI: Create a header with a separator
pub fn create_header(text: String, with_separator: bool) -> Vec<serenity::CreateContainerComponent<'static>> {
    let mut components = vec![serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(text),
    )];
    if with_separator {
        components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(true),
        ));
    }
    components
}

/// Reusable UI: Create an ActionRow with a SelectMenu
pub fn create_select_menu_row(
    id: String,
    kind: serenity::CreateSelectMenuKind<'static>,
    placeholder: String,
) -> serenity::CreateContainerComponent<'static> {
    serenity::CreateContainerComponent::ActionRow(serenity::CreateActionRow::SelectMenu(
        serenity::CreateSelectMenu::new(id, kind).placeholder(placeholder),
    ))
}

/// Reusable UI: Create a standard module configuration payload
pub fn create_module_config_payload(
    name: String,
    module_type: ModuleType,
    log_channel_id: Option<i64>,
    punishment: crate::db::entities::module_configs::PunishmentType,
    punishment_at: i32,
    punishment_at_interval: i32,
    enabled: bool,
    revert: bool,
    l10n: &L10nProxy,
) -> Vec<serenity::CreateContainerComponent<'static>> {
    let mut components = vec![];

    // Module Header
    let status_label = if enabled {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let toggle_btn = serenity::CreateButton::new(format!("config_module_toggle_{}", module_type))
        .label(status_label)
        .style(if enabled {
            serenity::ButtonStyle::Success
        } else {
            serenity::ButtonStyle::Danger
        });

    components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!("⚙️ **{}**", name)),
            )],
            serenity::CreateSectionAccessory::Button(toggle_btn),
        )
    ));

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Whitelist Section (Moved to top)
    components.push(create_whitelist_section(format!("config_whitelist_view_module_{}", module_type), l10n));
    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Log Channel Section
    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-module-log-channel-label", None)),
    ));

    components.push(create_select_menu_row(
        format!("config_module_log_channel_{:?}", module_type),
        serenity::CreateSelectMenuKind::Channel {
            channel_types: None,
            default_channels: log_channel_id
                .map(|id| vec![serenity::ChannelId::new(id as u64).into()].into()),
        },
        l10n.t("config-select-module-log-channel-placeholder", None),
    ));

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(false),
    ));

    // Punishment Section
    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!("**{}**", l10n.t("config-punishment-label", None))),
    ));

    use crate::db::entities::module_configs::PunishmentType;
    let punishment_options = vec![
        PunishmentType::None,
        PunishmentType::Unperm,
        PunishmentType::Ban,
        PunishmentType::Kick,
        PunishmentType::Jail,
    ]
    .into_iter()
    .map(|p| {
        let is_default = p == punishment;
        let p_str = format!("{:?}", p).to_lowercase();
        serenity::CreateSelectMenuOption::new(
            l10n.t(&format!("config-punishment-type-{}", p_str), None),
            p_str,
        )
        .default_selection(is_default)
    })
    .collect::<Vec<_>>();

    components.push(create_select_menu_row(
        format!("config_module_punishment_{:?}", module_type),
        serenity::CreateSelectMenuKind::String {
            options: punishment_options.into(),
        },
        l10n.t("config-select-punishment-placeholder", None),
    ));

    // Revert Toggle Section
    let revert_btn_label = if revert {
        l10n.t("config-btn-enabled", None)
    } else {
        l10n.t("config-btn-disabled", None)
    };

    let revert_btn = serenity::CreateButton::new(format!("config_module_revert_{:?}", module_type))
        .label(revert_btn_label)
        .style(if revert {
            serenity::ButtonStyle::Success
        } else {
            serenity::ButtonStyle::Secondary
        });

    components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new(l10n.t("config-revert-label", None)),
            )],
            serenity::CreateSectionAccessory::Button(revert_btn),
        )
    ));

    // Display Current Repetition Settings
    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Repetition Section
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("count", punishment_at);
    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-repetition-at-label", Some(&args))),
    ));

    components.push(serenity::CreateContainerComponent::ActionRow(serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("config_module_punish_at_dec_{:?}", module_type))
            .label("-")
            .style(serenity::ButtonStyle::Secondary),
        serenity::CreateButton::new(format!("config_module_punish_at_inc_{:?}", module_type))
            .label("+")
            .style(serenity::ButtonStyle::Secondary),
    ].into())));

    // Interval Section
    components.push(serenity::CreateContainerComponent::Separator(serenity::CreateSeparator::new(false)));
    let mut args = fluent_bundle::FluentArgs::new();
    args.set("count", punishment_at_interval);
    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(l10n.t("config-repetition-interval-label", Some(&args))),
    ));

    components.push(serenity::CreateContainerComponent::ActionRow(serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new(format!("config_module_punish_interval_dec_{:?}", module_type))
            .label("-")
            .style(serenity::ButtonStyle::Secondary),
        serenity::CreateButton::new(format!("config_module_punish_interval_inc_{:?}", module_type))
            .label("+")
            .style(serenity::ButtonStyle::Secondary),
    ].into())));

    components
}

/// Reusable UI: Create a whitelist navigation section
pub fn create_whitelist_section(
    id: String,
    l10n: &L10nProxy,
) -> serenity::CreateContainerComponent<'static> {
    serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new(format!("🛡️ **{}**", l10n.t("config-whitelists-btn", None))),
            )],
            serenity::CreateSectionAccessory::Button(
                serenity::CreateButton::new(id)
                    .label(l10n.t("config-whitelists-view-btn", None))
                    .style(serenity::ButtonStyle::Primary),
            ),
        )
    )
}

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
    revert: bool,
    l10n: &L10nProxy,
) -> Vec<serenity::CreateContainerComponent<'static>> {
    let mut components = vec![];

    // Module Header
    components.extend(create_header(format!("⚙️ **{}**", name), true));

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
    components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new(format!(
            "**At Repetition:** {}\n**Interval:** {} min",
            punishment_at, punishment_at_interval
        )),
    ));

    components
}

use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

pub fn build_logging_step(
    setup_id: &str,
    l10n: &L10nProxy,
    has_channel: bool,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    let select = serenity::CreateSelectMenu::new(
        format!("setup_logging_select_{}", setup_id),
        serenity::CreateSelectMenuKind::Channel {
            channel_types: Some(vec![serenity::ChannelType::Text].into()),
            default_channels: None,
        },
    )
    .placeholder(l10n.t("setup-logging-placeholder", None))
    .min_values(0)
    .max_values(1);

    let mut components = vec![
        serenity::CreateComponent::ActionRow(serenity::CreateActionRow::select_menu(select)),
    ];

    // Only show create channel section if no channel is set
    if !has_channel {
        let create_section = serenity::CreateComponent::Container(
            serenity::CreateContainer::new(vec![
                serenity::CreateContainerComponent::Section(
                    serenity::CreateSection::new(
                        vec![serenity::CreateSectionComponent::TextDisplay(
                            serenity::CreateTextDisplay::new(l10n.t("setup-or-create", None)),
                        )],
                        serenity::CreateSectionAccessory::Button(
                            serenity::CreateButton::new(format!("setup_create_fallback_channel_{}", setup_id))
                                .emoji('📝')
                                .style(serenity::ButtonStyle::Secondary),
                        ),
                    ),
                ),
            ]),
        );
        components.push(create_section);
    }

    let skip_button = serenity::CreateButton::new(format!("setup_logging_skip_{}", setup_id))
        .label(l10n.t("setup-logging-skip", None))
        .style(serenity::ButtonStyle::Secondary);

    components.push(serenity::CreateComponent::ActionRow(serenity::CreateActionRow::buttons(vec![skip_button])));

    (
        format!("{}\n{}", l10n.t("setup-step2-title", None), l10n.t("setup-step2-desc", None)),
        components,
    )
}

use crate::db::entities::module_configs::ModuleType;
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;

pub mod channel_permission_protection;
pub mod channel_protection;
pub mod invite_tracking;
pub mod logging;
pub mod moderation_protection;
pub mod role_protection;

pub fn build_module_config_step(
    setup_id: &str,
    l10n: &L10nProxy,
    module: ModuleType,
    has_log_channel: bool,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    match module {
        ModuleType::Logging => logging::build_ui(setup_id, l10n),
        ModuleType::ChannelProtection => channel_protection::build_ui(setup_id, l10n),
        ModuleType::ChannelPermissionProtection => {
            channel_permission_protection::build_ui(setup_id, l10n)
        }
        ModuleType::RoleProtection => role_protection::build_ui(setup_id, l10n),
        ModuleType::ModerationProtection => moderation_protection::build_ui(setup_id, l10n),
        ModuleType::InviteTracking => invite_tracking::build_ui(setup_id, l10n),
        _ => build_generic_ui(setup_id, l10n, module, has_log_channel),
    }
}

fn get_module_label(module: ModuleType, l10n: &L10nProxy) -> String {
    match module {
        ModuleType::ChannelProtection => l10n.t("config-channel-protection-label", None),
        ModuleType::ChannelPermissionProtection => {
            l10n.t("config-channel-permission-protection-label", None)
        }
        ModuleType::RoleProtection => l10n.t("config-role-protection-label", None),
        ModuleType::RolePermissionProtection => {
            l10n.t("config-role-permission-protection-label", None)
        }
        ModuleType::MemberPermissionProtection => {
            l10n.t("config-member-permission-protection-label", None)
        }
        ModuleType::BotAddingProtection => l10n.t("config-bot-adding-protection-label", None),
        ModuleType::ModerationProtection => l10n.t("config-moderation-protection-label", None),
        ModuleType::Logging => l10n.t("config-logging-label", None),
        ModuleType::StickyRoles => l10n.t("config-sticky-roles-label", None),
        ModuleType::InviteTracking => l10n.t("config-invite-tracking-label", None),
    }
}

fn build_generic_ui(
    setup_id: &str,
    l10n: &L10nProxy,
    module: ModuleType,
    has_log_channel: bool,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    let label = get_module_label(module, l10n);

    let mut components = vec![];

    // Log channel selection (for non-Logging, non-StickyRoles modules)
    if module != ModuleType::StickyRoles {
        let mut args = fluent::FluentArgs::new();
        args.set("label", label.clone());

        // Log channel select
        let log_select = serenity::CreateSelectMenu::new(
            format!("setup_module_log_channel_{}_{:?}", setup_id, module),
            serenity::CreateSelectMenuKind::Channel {
                channel_types: Some(vec![serenity::ChannelType::Text].into()),
                default_channels: None,
            },
        )
        .placeholder(l10n.t("setup-module-log-channel-placeholder", None))
        .min_values(0)
        .max_values(1);

        components.push(serenity::CreateComponent::ActionRow(
            serenity::CreateActionRow::select_menu(log_select),
        ));

        // Create channel section (only if no channel set)
        if !has_log_channel {
            components.push(serenity::CreateComponent::Container(
                serenity::CreateContainer::new(vec![serenity::CreateContainerComponent::Section(
                    serenity::CreateSection::new(
                        vec![serenity::CreateSectionComponent::TextDisplay(
                            serenity::CreateTextDisplay::new(l10n.t("setup-or-create", None)),
                        )],
                        serenity::CreateSectionAccessory::Button(
                            serenity::CreateButton::new(format!(
                                "setup_create_module_channel_{}_{:?}",
                                setup_id, module
                            ))
                            .emoji(serenity::ReactionType::Unicode('📝'.into()))
                            .style(serenity::ButtonStyle::Secondary),
                        ),
                    ),
                )]),
            ));
        }
    }

    let next_button =
        serenity::CreateButton::new(format!("setup_module_next_{}_{:?}", setup_id, module))
            .label(l10n.t("setup-next", None))
            .style(serenity::ButtonStyle::Primary);

    components.push(serenity::CreateComponent::ActionRow(
        serenity::CreateActionRow::buttons(vec![next_button]),
    ));

    let mut args = fluent::FluentArgs::new();
    args.set("label", label);

    let desc = if module == ModuleType::StickyRoles {
        l10n.t("setup-step4-generic-desc", None)
    } else {
        l10n.t("setup-module-log-channel-desc", None)
    };

    (
        format!("{}\n{}", l10n.t("setup-step4-title", Some(&args)), desc),
        components,
    )
}

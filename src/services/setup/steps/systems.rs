use crate::db::entities::module_configs::ModuleType;
use crate::services::localization::L10nProxy;
use poise::serenity_prelude as serenity;
use sea_orm::Iterable;

pub fn build_systems_step(
    setup_id: &str,
    l10n: &L10nProxy,
) -> (String, Vec<serenity::CreateComponent<'static>>) {
    let mut options = Vec::new();
    for module in ModuleType::iter() {
        let label = match module {
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
        };

        let desc = match module {
            ModuleType::ChannelProtection => l10n.t("config-channel-protection-desc", None),
            ModuleType::ChannelPermissionProtection => {
                l10n.t("config-channel-permission-protection-desc", None)
            }
            ModuleType::RoleProtection => l10n.t("config-role-protection-desc", None),
            ModuleType::RolePermissionProtection => {
                l10n.t("config-role-permission-protection-desc", None)
            }
            ModuleType::MemberPermissionProtection => {
                l10n.t("config-member-permission-protection-desc", None)
            }
            ModuleType::BotAddingProtection => l10n.t("config-bot-adding-protection-desc", None),
            ModuleType::ModerationProtection => l10n.t("config-moderation-protection-desc", None),
            ModuleType::Logging => l10n.t("config-logging-desc", None),
            ModuleType::StickyRoles => l10n.t("config-sticky-roles-desc", None),
            ModuleType::InviteTracking => l10n.t("module-invite-tracking-desc", None),
        };

        options.push(
            serenity::CreateSelectMenuOption::new(label, format!("{:?}", module)).description(desc),
        );
    }

    let select = serenity::CreateSelectMenu::new(
        format!("setup_systems_{}", setup_id),
        serenity::CreateSelectMenuKind::String {
            options: options.into(),
        },
    )
    .min_values(0)
    .max_values(9)
    .placeholder(l10n.t("setup-systems-placeholder", None));

    (
        format!(
            "{}\n{}",
            l10n.t("setup-step1-title", None),
            l10n.t("setup-step1-desc", None)
        ),
        vec![serenity::CreateComponent::ActionRow(
            serenity::CreateActionRow::select_menu(select),
        )],
    )
}

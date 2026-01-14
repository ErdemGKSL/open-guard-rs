use crate::db::entities::{
    module_configs::ModuleType, whitelist_role, whitelist_user, whitelists::WhitelistLevel,
};
use crate::services::localization::L10nProxy;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, Iterable, QueryFilter, Set};

/// Checks if a member has "Head" level permissions for a specific module or globally.
pub async fn check_permission(
    ctx: &serenity::Context,
    data: &Data,
    guild_id: serenity::GuildId,
    member: &serenity::Member,
    module: Option<ModuleType>,
) -> Result<bool, Error> {
    // 1. Server owner is always "Head"
    if let Some(guild) = ctx.cache.guild(guild_id) {
        if guild.owner_id == member.user.id {
            return Ok(true);
        }
    } else {
        let guild = ctx.http.get_guild(guild_id).await?;
        if guild.owner_id == member.user.id {
            return Ok(true);
        }
    }

    // 2. Check individual user whitelist
    let cond = Condition::any().add(whitelist_user::Column::ModuleType.is_null());

    let cond = if let Some(m) = module {
        cond.add(whitelist_user::Column::ModuleType.eq(Some(m)))
    } else {
        cond
    };

    let user_head = whitelist_user::Entity::find()
        .filter(whitelist_user::Column::GuildId.eq(guild_id.get() as i64))
        .filter(whitelist_user::Column::UserId.eq(member.user.id.get() as i64))
        .filter(whitelist_user::Column::Level.eq(WhitelistLevel::Head))
        .filter(cond)
        .one(&data.db)
        .await?;

    if user_head.is_some() {
        return Ok(true);
    }

    // 3. Check role-based whitelist
    let role_ids: Vec<i64> = member.roles.iter().map(|id| id.get() as i64).collect();
    if !role_ids.is_empty() {
        let cond = Condition::any().add(whitelist_role::Column::ModuleType.is_null());

        let cond = if let Some(m) = module {
            cond.add(whitelist_role::Column::ModuleType.eq(Some(m)))
        } else {
            cond
        };

        let role_head = whitelist_role::Entity::find()
            .filter(whitelist_role::Column::GuildId.eq(guild_id.get() as i64))
            .filter(whitelist_role::Column::RoleId.is_in(role_ids))
            .filter(whitelist_role::Column::Level.eq(WhitelistLevel::Head))
            .filter(cond)
            .one(&data.db)
            .await?;

        if role_head.is_some() {
            return Ok(true);
        }
    }

    Ok(false)
}

const ENTRIES_PER_PAGE: usize = 10;

#[derive(Debug, Clone)]
pub enum WhitelistItem {
    User(whitelist_user::Model),
    Role(whitelist_role::Model),
}

impl WhitelistItem {
    fn id(&self) -> i32 {
        match self {
            WhitelistItem::User(u) => u.id,
            WhitelistItem::Role(r) => r.id,
        }
    }

    fn is_user(&self) -> bool {
        matches!(self, WhitelistItem::User(_))
    }

    fn mention(&self) -> String {
        match self {
            WhitelistItem::User(u) => format!("<@{}>", u.user_id),
            WhitelistItem::Role(r) => format!("<@&{}>", r.role_id),
        }
    }
}

/// Builds the whitelist configuration menu with pagination
pub async fn build_whitelist_menu(
    data: &Data,
    guild_id: serenity::GuildId,
    module: Option<ModuleType>,
    page: usize,
    is_head: bool,
    l10n: &L10nProxy,
) -> Result<Vec<serenity::CreateComponent<'static>>, Error> {
    let mut components = vec![];

    // Header
    let title = if let Some(m) = module {
        match m {
            ModuleType::ChannelProtection => {
                l10n.t("config-whitelist-channel-protection-header", None)
            }
            ModuleType::ChannelPermissionProtection => l10n.t(
                "config-whitelist-channel-permission-protection-header",
                None,
            ),
            ModuleType::RoleProtection => l10n.t("config-whitelist-role-protection-header", None),
            ModuleType::RolePermissionProtection => {
                l10n.t("config-whitelist-role-permission-protection-header", None)
            }
            ModuleType::MemberPermissionProtection => {
                l10n.t("config-whitelist-member-permission-protection-header", None)
            }
            ModuleType::BotAddingProtection => {
                l10n.t("config-whitelist-bot-adding-protection-header", None)
            }
            ModuleType::ModerationProtection => {
                l10n.t("config-whitelist-moderation-protection-header", None)
            }
            ModuleType::Logging => l10n.t("config-whitelist-logging-header", None),
            ModuleType::StickyRoles => l10n.t("config-whitelist-sticky-roles-header", None),
        }
    } else {
        l10n.t("config-whitelist-global-header", None)
    };

    let back_id = if let Some(m) = module {
        format!("config_module_menu_{}", m)
    } else {
        "config_back_to_main".to_string()
    };

    components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new(title),
            )],
            serenity::CreateSectionAccessory::Button(
                serenity::CreateButton::new(back_id)
                    .label(l10n.t("config-back-label", None))
                    .style(serenity::ButtonStyle::Secondary),
            ),
        ),
    ));

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Fetch entries
    let users = whitelist_user::Entity::find()
        .filter(whitelist_user::Column::GuildId.eq(guild_id.get() as i64))
        .filter(match module {
            Some(m) => whitelist_user::Column::ModuleType.eq(Some(m)),
            None => whitelist_user::Column::ModuleType.is_null(),
        })
        .all(&data.db)
        .await?;

    let roles = whitelist_role::Entity::find()
        .filter(whitelist_role::Column::GuildId.eq(guild_id.get() as i64))
        .filter(match module {
            Some(m) => whitelist_role::Column::ModuleType.eq(Some(m)),
            None => whitelist_role::Column::ModuleType.is_null(),
        })
        .all(&data.db)
        .await?;

    let mut all_items = vec![];
    for u in users {
        all_items.push(WhitelistItem::User(u));
    }
    for r in roles {
        all_items.push(WhitelistItem::Role(r));
    }

    let total_pages = (all_items.len() + ENTRIES_PER_PAGE - 1) / ENTRIES_PER_PAGE;
    let start = page * ENTRIES_PER_PAGE;
    let end = (start + ENTRIES_PER_PAGE).min(all_items.len());

    if !all_items.is_empty() && start < all_items.len() {
        for item in &all_items[start..end] {
            let item_type = if item.is_user() { "user" } else { "role" };
            let suffix = module
                .map(|m| m.to_string())
                .unwrap_or_else(|| "global".to_string());
            components.push(serenity::CreateContainerComponent::Section(
                serenity::CreateSection::new(
                    vec![serenity::CreateSectionComponent::TextDisplay(
                        serenity::CreateTextDisplay::new(item.mention()),
                    )],
                    serenity::CreateSectionAccessory::Button(
                        serenity::CreateButton::new(format!(
                            "config_whitelist_manage_{}_{}_{}",
                            item_type,
                            item.id(),
                            suffix
                        ))
                        .label(l10n.t("config-whitelist-manage-btn", None))
                        .style(serenity::ButtonStyle::Secondary)
                        .disabled(!is_head),
                    ),
                ),
            ));
        }
    }

    // Pagination
    if total_pages > 1 {
        let mut nav_buttons = vec![];
        let suffix = module
            .map(|m| m.to_string())
            .unwrap_or_else(|| "global".to_string());

        if page > 0 {
            nav_buttons.push(
                serenity::CreateButton::new(format!(
                    "config_whitelist_page_{}_{}",
                    suffix,
                    page - 1
                ))
                .label(l10n.t("config-whitelist-prev-page", None))
                .style(serenity::ButtonStyle::Secondary),
            );
        }

        if page + 1 < total_pages {
            nav_buttons.push(
                serenity::CreateButton::new(format!(
                    "config_whitelist_page_{}_{}",
                    suffix,
                    page + 1
                ))
                .label(l10n.t("config-whitelist-next-page", None))
                .style(serenity::ButtonStyle::Secondary),
            );
        }

        if !nav_buttons.is_empty() {
            components.push(serenity::CreateContainerComponent::ActionRow(
                serenity::CreateActionRow::buttons(nav_buttons),
            ));
        }
    }

    components.push(serenity::CreateContainerComponent::Separator(
        serenity::CreateSeparator::new(true),
    ));

    // Add buttons
    let suffix = module
        .map(|m| m.to_string())
        .unwrap_or_else(|| "global".to_string());
    components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::buttons(vec![
            serenity::CreateButton::new(format!("config_whitelist_add_user_page_{}", suffix))
                .label(l10n.t("config-whitelist-add-user-btn", None))
                .style(serenity::ButtonStyle::Primary)
                .disabled(!is_head),
            serenity::CreateButton::new(format!("config_whitelist_add_role_page_{}", suffix))
                .label(l10n.t("config-whitelist-add-role-btn", None))
                .style(serenity::ButtonStyle::Primary)
                .disabled(!is_head),
        ]),
    ));

    Ok(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(components),
    )])
}

/// Creates a modal for adding/editing a whitelist user entry
pub fn build_whitelist_user_modal<'a>(
    entry_id: Option<i32>,
    module: Option<ModuleType>,
    current_user_id: Option<serenity::UserId>,
    current_level: Option<WhitelistLevel>,
    l10n: &L10nProxy,
) -> serenity::CreateModal<'a> {
    let suffix = module
        .map(|m| m.to_string())
        .unwrap_or_else(|| "global".to_string());

    let custom_id = if let Some(id) = entry_id {
        format!("whitelist_modal_user_edit_{}_{}", id, suffix)
    } else {
        format!("whitelist_modal_user_new_{}", suffix)
    };

    let title = if entry_id.is_some() {
        l10n.t("config-whitelist-edit-user-modal-title", None)
    } else {
        l10n.t("config-whitelist-add-user-modal-title", None)
    };

    // User select with Label wrapper
    let user_select = serenity::CreateSelectMenu::new(
        "whitelist_modal_user_select",
        serenity::CreateSelectMenuKind::User {
            default_users: current_user_id.map(|id| vec![id].into()),
        },
    )
    .min_values(1)
    .max_values(1)
    .required(true);

    let user_label = serenity::CreateLabel::select_menu(
        l10n.t("config-whitelist-modal-user-label", None),
        user_select,
    )
    .description(l10n.t("config-whitelist-modal-user-description", None));

    // Level select with Label wrapper
    let mut level_options = vec![];
    for level in WhitelistLevel::iter() {
        let label = match level {
            WhitelistLevel::Head => l10n.t("config-level-head", None),
            WhitelistLevel::Admin => l10n.t("config-level-admin", None),
            WhitelistLevel::Invulnerable => l10n.t("config-level-invulnerable", None),
        };
        let mut opt =
            serenity::CreateSelectMenuOption::new(label, level.to_string().to_lowercase());
        if let Some(l) = current_level {
            if l == level {
                opt = opt.default_selection(true);
            }
        }
        level_options.push(opt);
    }

    let level_select = serenity::CreateSelectMenu::new(
        "whitelist_modal_level_select",
        serenity::CreateSelectMenuKind::String {
            options: level_options.into(),
        },
    )
    .required(true);

    let level_label = serenity::CreateLabel::select_menu(
        l10n.t("config-whitelist-modal-level-label", None),
        level_select,
    )
    .description(l10n.t("config-whitelist-modal-level-description", None));

    serenity::CreateModal::new(custom_id, title).components(vec![
        serenity::CreateModalComponent::Label(user_label),
        serenity::CreateModalComponent::Label(level_label),
    ])
}

/// Creates a modal for adding/editing a whitelist role entry
pub fn build_whitelist_role_modal<'a>(
    entry_id: Option<i32>,
    module: Option<ModuleType>,
    current_role_id: Option<serenity::RoleId>,
    current_level: Option<WhitelistLevel>,
    l10n: &L10nProxy,
) -> serenity::CreateModal<'a> {
    let suffix = module
        .map(|m| m.to_string())
        .unwrap_or_else(|| "global".to_string());

    let custom_id = if let Some(id) = entry_id {
        format!("whitelist_modal_role_edit_{}_{}", id, suffix)
    } else {
        format!("whitelist_modal_role_new_{}", suffix)
    };

    let title = if entry_id.is_some() {
        l10n.t("config-whitelist-edit-role-modal-title", None)
    } else {
        l10n.t("config-whitelist-add-role-modal-title", None)
    };

    // Role select with Label wrapper
    let role_select = serenity::CreateSelectMenu::new(
        "whitelist_modal_role_select",
        serenity::CreateSelectMenuKind::Role {
            default_roles: current_role_id.map(|id| vec![id].into()),
        },
    )
    .min_values(1)
    .max_values(1)
    .required(true);

    let role_label = serenity::CreateLabel::select_menu(
        l10n.t("config-whitelist-modal-role-label", None),
        role_select,
    )
    .description(l10n.t("config-whitelist-modal-role-description", None));

    // Level select with Label wrapper
    let mut level_options = vec![];
    for level in WhitelistLevel::iter() {
        let label = match level {
            WhitelistLevel::Head => l10n.t("config-level-head", None),
            WhitelistLevel::Admin => l10n.t("config-level-admin", None),
            WhitelistLevel::Invulnerable => l10n.t("config-level-invulnerable", None),
        };
        let mut opt =
            serenity::CreateSelectMenuOption::new(label, level.to_string().to_lowercase());
        if let Some(l) = current_level {
            if l == level {
                opt = opt.default_selection(true);
            }
        }
        level_options.push(opt);
    }

    let level_select = serenity::CreateSelectMenu::new(
        "whitelist_modal_level_select",
        serenity::CreateSelectMenuKind::String {
            options: level_options.into(),
        },
    )
    .required(true);

    let level_label = serenity::CreateLabel::select_menu(
        l10n.t("config-whitelist-modal-level-label", None),
        level_select,
    )
    .description(l10n.t("config-whitelist-modal-level-description", None));

    serenity::CreateModal::new(custom_id, title).components(vec![
        serenity::CreateModalComponent::Label(role_label),
        serenity::CreateModalComponent::Label(level_label),
    ])
}

/// Builds the individual whitelist entry management page
pub async fn build_manage_entry(
    data: &Data,
    guild_id: serenity::GuildId,
    entry_id: Option<i32>,
    is_user: bool,
    module: Option<ModuleType>,
    is_head: bool,
    l10n: &L10nProxy,
) -> Result<Vec<serenity::CreateComponent<'static>>, Error> {
    let mut components = vec![];
    let suffix = module
        .map(|m| m.to_string())
        .unwrap_or_else(|| "global".to_string());

    // Header
    components.push(serenity::CreateContainerComponent::Section(
        serenity::CreateSection::new(
            vec![serenity::CreateSectionComponent::TextDisplay(
                serenity::CreateTextDisplay::new(l10n.t("config-whitelist-manage-title", None)),
            )],
            serenity::CreateSectionAccessory::Button(
                serenity::CreateButton::new(format!("config_whitelist_view_{}", suffix))
                    .label(l10n.t("config-back-label", None))
                    .style(serenity::ButtonStyle::Secondary),
            ),
        ),
    ));

    // Fetch existing data if any
    let (current_target_id, current_level) = if let Some(id) = entry_id {
        if is_user {
            whitelist_user::Entity::find_by_id(id)
                .filter(whitelist_user::Column::GuildId.eq(guild_id.get() as i64))
                .one(&data.db)
                .await?
                .map(|u| (Some(u.user_id as u64), Some(u.level)))
                .unwrap_or((None, None))
        } else {
            whitelist_role::Entity::find_by_id(id)
                .filter(whitelist_role::Column::GuildId.eq(guild_id.get() as i64))
                .one(&data.db)
                .await?
                .map(|r| (Some(r.role_id as u64), Some(r.level)))
                .unwrap_or((None, None))
        }
    } else {
        (None, None)
    };

    // User or Role Select
    let select_id = if let Some(id) = entry_id {
        format!(
            "config_whitelist_entry_target_{}_{}",
            if is_user { "user" } else { "role" },
            id
        )
    } else {
        format!(
            "config_whitelist_entry_target_new_{}_{}",
            if is_user { "user" } else { "role" },
            suffix
        )
    };

    if is_user {
        components.push(serenity::CreateContainerComponent::ActionRow(
            serenity::CreateActionRow::select_menu(
                serenity::CreateSelectMenu::new(
                    select_id,
                    serenity::CreateSelectMenuKind::User {
                        default_users: current_target_id
                            .map(|id| vec![serenity::UserId::new(id)].into()),
                    },
                )
                .min_values(1)
                .max_values(1)
                .placeholder(l10n.t("config-select-user-placeholder", None))
                .disabled(!is_head),
            ),
        ));
    } else {
        components.push(serenity::CreateContainerComponent::ActionRow(
            serenity::CreateActionRow::select_menu(
                serenity::CreateSelectMenu::new(
                    select_id,
                    serenity::CreateSelectMenuKind::Role {
                        default_roles: current_target_id
                            .map(|id| vec![serenity::RoleId::new(id)].into()),
                    },
                )
                .min_values(1)
                .max_values(1)
                .placeholder(l10n.t("config-select-role-placeholder", None))
                .disabled(!is_head),
            ),
        ));
    }

    // Level Select
    let level_id = if let Some(id) = entry_id {
        format!(
            "config_whitelist_entry_level_{}_{}",
            if is_user { "user" } else { "role" },
            id
        )
    } else {
        format!(
            "config_whitelist_entry_level_new_{}_{}",
            if is_user { "user" } else { "role" },
            suffix
        )
    };

    let mut level_options = vec![];
    for level in WhitelistLevel::iter() {
        let label = match level {
            WhitelistLevel::Head => l10n.t("config-level-head", None),
            WhitelistLevel::Admin => l10n.t("config-level-admin", None),
            WhitelistLevel::Invulnerable => l10n.t("config-level-invulnerable", None),
        };
        let mut opt =
            serenity::CreateSelectMenuOption::new(label, level.to_string().to_lowercase());
        if let Some(l) = current_level {
            if l == level {
                opt = opt.default_selection(true);
            }
        }
        level_options.push(opt);
    }

    components.push(serenity::CreateContainerComponent::ActionRow(
        serenity::CreateActionRow::select_menu(
            serenity::CreateSelectMenu::new(
                level_id,
                serenity::CreateSelectMenuKind::String {
                    options: level_options.into(),
                },
            )
            .placeholder(l10n.t("config-select-level-placeholder", None))
            .disabled(!is_head),
        ),
    ));

    // Delete Button (only if editing)
    if let Some(id) = entry_id {
        components.push(serenity::CreateContainerComponent::Section(
            serenity::CreateSection::new(
                vec![serenity::CreateSectionComponent::TextDisplay(
                    serenity::CreateTextDisplay::new("** **"),
                )],
                serenity::CreateSectionAccessory::Button(
                    serenity::CreateButton::new(format!(
                        "config_whitelist_entry_delete_{}_{}_{}",
                        if is_user { "user" } else { "role" },
                        id,
                        suffix
                    ))
                    .label(l10n.t("config-whitelist-delete-btn", None))
                    .style(serenity::ButtonStyle::Danger)
                    .disabled(!is_head),
                ),
            ),
        ));
    }

    Ok(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(components),
    )])
}

/// Result type for whitelist interaction handling
pub enum WhitelistInteractionResult {
    /// Update the message with new components
    Components(Vec<serenity::CreateComponent<'static>>),
    /// Show a modal to the user (the caller should NOT acknowledge the interaction)
    ShowModal(serenity::CreateModal<'static>),
    /// No action needed
    None,
}

pub async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<WhitelistInteractionResult, Error> {
    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => return Ok(WhitelistInteractionResult::None),
    };

    let member = interaction
        .member
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Interaction must be in a guild"))?;
    let custom_id = &interaction.data.custom_id;

    let parse_module = |suffix: &str| -> Option<ModuleType> {
        if suffix == "global" {
            None
        } else {
            match suffix {
                "channel_protection" => Some(ModuleType::ChannelProtection),
                "channel_permission_protection" => Some(ModuleType::ChannelPermissionProtection),
                "role_protection" => Some(ModuleType::RoleProtection),
                "role_permission_protection" => Some(ModuleType::RolePermissionProtection),
                "member_permission_protection" => Some(ModuleType::MemberPermissionProtection),
                "bot_adding_protection" => Some(ModuleType::BotAddingProtection),
                "moderation_protection" => Some(ModuleType::ModerationProtection),
                "logging" => Some(ModuleType::Logging),
                "sticky_roles" => Some(ModuleType::StickyRoles),
                _ => None,
            }
        }
    };

    let l10n_manager = &data.l10n;
    let l10n = L10nProxy {
        manager: l10n_manager.clone(),
        locale: interaction.locale.to_string(),
    };

    // Helper for permission check
    let check_perm = |module: Option<ModuleType>| async move {
        check_permission(ctx, data, guild_id, member, module).await
    };

    // View Whitelist Menu
    if custom_id == "config_whitelist_view_global" {
        let is_head = check_perm(None).await?;
        return Ok(WhitelistInteractionResult::Components(
            build_whitelist_menu(data, guild_id, None, 0, is_head, &l10n).await?,
        ));
    }
    if let Some(module_str) = custom_id.strip_prefix("config_whitelist_view_module_") {
        let module = parse_module(module_str);
        let is_head = check_perm(module).await?;
        return Ok(WhitelistInteractionResult::Components(
            build_whitelist_menu(data, guild_id, module, 0, is_head, &l10n).await?,
        ));
    }
    if let Some(rest) = custom_id.strip_prefix("config_whitelist_view_") {
        let module = parse_module(rest);
        let is_head = check_perm(module).await?;
        return Ok(WhitelistInteractionResult::Components(
            build_whitelist_menu(data, guild_id, module, 0, is_head, &l10n).await?,
        ));
    }

    // Pagination
    if let Some(rest) = custom_id.strip_prefix("config_whitelist_page_") {
        let parts: Vec<&str> = rest.splitn(2, '_').collect();
        if parts.len() == 2 {
            let module = parse_module(parts[0]);
            let page: usize = parts[1].parse().unwrap_or(0);
            let is_head = check_perm(module).await?;
            return Ok(WhitelistInteractionResult::Components(
                build_whitelist_menu(data, guild_id, module, page, is_head, &l10n).await?,
            ));
        }
    }

    // Manage Page Navigation -> Show edit modal
    if let Some(rest) = custom_id.strip_prefix("config_whitelist_manage_") {
        // format: user_{id}_{suffix} or role_{id}_{suffix}
        let parts: Vec<&str> = rest.splitn(3, '_').collect();
        if parts.len() == 3 {
            let is_user = parts[0] == "user";
            let id: i32 = parts[1].parse().unwrap_or(0);
            let module = parse_module(parts[2]);
            if !check_perm(module).await? {
                return Ok(WhitelistInteractionResult::None);
            }

            // Fetch existing entry data and show edit modal
            if is_user {
                let entry = whitelist_user::Entity::find_by_id(id)
                    .filter(whitelist_user::Column::GuildId.eq(guild_id.get() as i64))
                    .one(&data.db)
                    .await?;
                if let Some(entry) = entry {
                    let modal = build_whitelist_user_modal(
                        Some(id),
                        module,
                        Some(serenity::UserId::new(entry.user_id as u64)),
                        Some(entry.level),
                        &l10n,
                    );
                    return Ok(WhitelistInteractionResult::ShowModal(modal));
                }
            } else {
                let entry = whitelist_role::Entity::find_by_id(id)
                    .filter(whitelist_role::Column::GuildId.eq(guild_id.get() as i64))
                    .one(&data.db)
                    .await?;
                if let Some(entry) = entry {
                    let modal = build_whitelist_role_modal(
                        Some(id),
                        module,
                        Some(serenity::RoleId::new(entry.role_id as u64)),
                        Some(entry.level),
                        &l10n,
                    );
                    return Ok(WhitelistInteractionResult::ShowModal(modal));
                }
            }
        }
    }

    // Add New Entry -> Show create modal
    if let Some(rest) = custom_id.strip_prefix("config_whitelist_add_user_page_") {
        let module = parse_module(rest);
        if !check_perm(module).await? {
            return Ok(WhitelistInteractionResult::None);
        }
        let modal = build_whitelist_user_modal(None, module, None, None, &l10n);
        return Ok(WhitelistInteractionResult::ShowModal(modal));
    }
    if let Some(rest) = custom_id.strip_prefix("config_whitelist_add_role_page_") {
        let module = parse_module(rest);
        if !check_perm(module).await? {
            return Ok(WhitelistInteractionResult::None);
        }
        let modal = build_whitelist_role_modal(None, module, None, None, &l10n);
        return Ok(WhitelistInteractionResult::ShowModal(modal));
    }

    // Delete Entry (keep this - no modal needed)
    if let Some(rest) = custom_id.strip_prefix("config_whitelist_entry_delete_") {
        let parts: Vec<&str> = rest.splitn(3, '_').collect();
        if parts.len() == 3 {
            let is_user = parts[0] == "user";
            let id: i32 = parts[1].parse().unwrap_or(0);
            let module = parse_module(parts[2]);

            if !check_perm(module).await? {
                return Ok(WhitelistInteractionResult::None);
            }

            if is_user {
                whitelist_user::Entity::delete_by_id(id)
                    .exec(&data.db)
                    .await?;
            } else {
                whitelist_role::Entity::delete_by_id(id)
                    .exec(&data.db)
                    .await?;
            }

            return Ok(WhitelistInteractionResult::Components(
                build_whitelist_menu(data, guild_id, module, 0, true, &l10n).await?,
            ));
        }
    }

    Ok(WhitelistInteractionResult::None)
}

/// Handle modal submissions for whitelist entries
pub async fn handle_modal_submit(
    ctx: &serenity::Context,
    interaction: &serenity::ModalInteraction,
    data: &Data,
) -> Result<Option<Vec<serenity::CreateComponent<'static>>>, Error> {
    let guild_id = match interaction.guild_id {
        Some(id) => id,
        None => return Ok(None),
    };

    let member = interaction
        .member
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Interaction must be in a guild"))?;
    let custom_id = &interaction.data.custom_id;

    let l10n_manager = &data.l10n;
    let l10n = L10nProxy {
        manager: l10n_manager.clone(),
        locale: interaction.locale.to_string(),
    };

    let parse_module = |suffix: &str| -> Option<ModuleType> {
        if suffix == "global" {
            None
        } else {
            match suffix {
                "channel_protection" => Some(ModuleType::ChannelProtection),
                "channel_permission_protection" => Some(ModuleType::ChannelPermissionProtection),
                "role_protection" => Some(ModuleType::RoleProtection),
                "role_permission_protection" => Some(ModuleType::RolePermissionProtection),
                "member_permission_protection" => Some(ModuleType::MemberPermissionProtection),
                "bot_adding_protection" => Some(ModuleType::BotAddingProtection),
                "moderation_protection" => Some(ModuleType::ModerationProtection),
                "logging" => Some(ModuleType::Logging),
                "sticky_roles" => Some(ModuleType::StickyRoles),
                _ => None,
            }
        }
    };

    // Helper to extract string select values from modal components
    fn extract_string_select_value(
        components: &[serenity::Component],
        target_custom_id: &str,
    ) -> Option<String> {
        for component in components {
            if let serenity::Component::Label(label) = component {
                if let serenity::LabelComponent::SelectMenu(menu) = &label.component {
                    // Compare custom_id with target
                    if &*menu.custom_id == target_custom_id {
                        return menu.values.first().map(|s| s.to_string());
                    }
                }
            }
        }
        None
    }

    // For user/role selects, use the resolved data directly (not Option)
    let resolved = &interaction.data.resolved;

    // Handle new user modal submission
    if let Some(suffix) = custom_id.strip_prefix("whitelist_modal_user_new_") {
        let module = parse_module(suffix);
        if !check_permission(ctx, data, guild_id, member, module).await? {
            return Ok(None);
        }

        // Get selected user from resolved data
        let selected_user_id = resolved.users.iter().next().map(|user| user.id);

        // Get level from string select
        let level_str = extract_string_select_value(
            &interaction.data.components,
            "whitelist_modal_level_select",
        );
        let selected_level = match level_str.as_deref() {
            Some("head") => WhitelistLevel::Head,
            Some("admin") => WhitelistLevel::Admin,
            _ => WhitelistLevel::Invulnerable,
        };

        if let Some(user_id) = selected_user_id {
            let model = whitelist_user::ActiveModel {
                guild_id: Set(guild_id.get() as i64),
                user_id: Set(user_id.get() as i64),
                level: Set(selected_level),
                module_type: Set(module),
                ..Default::default()
            };
            model.insert(&data.db).await?;

            return Ok(Some(
                build_whitelist_menu(data, guild_id, module, 0, true, &l10n).await?,
            ));
        }
    }

    // Handle edit user modal submission
    if let Some(rest) = custom_id.strip_prefix("whitelist_modal_user_edit_") {
        let parts: Vec<&str> = rest.splitn(2, '_').collect();
        if parts.len() == 2 {
            let id: i32 = parts[0].parse().unwrap_or(0);
            let module = parse_module(parts[1]);

            if !check_permission(ctx, data, guild_id, member, module).await? {
                return Ok(None);
            }

            let existing = whitelist_user::Entity::find_by_id(id)
                .filter(whitelist_user::Column::GuildId.eq(guild_id.get() as i64))
                .one(&data.db)
                .await?;

            if let Some(existing) = existing {
                let mut active: whitelist_user::ActiveModel = existing.into();

                // Get selected user from resolved data
                if let Some(user) = resolved.users.iter().next() {
                    active.user_id = Set(user.id.get() as i64);
                }

                // Get level from string select
                let level_str = extract_string_select_value(
                    &interaction.data.components,
                    "whitelist_modal_level_select",
                );
                if let Some(level) = match level_str.as_deref() {
                    Some("head") => Some(WhitelistLevel::Head),
                    Some("admin") => Some(WhitelistLevel::Admin),
                    Some("invulnerable") => Some(WhitelistLevel::Invulnerable),
                    _ => None,
                } {
                    active.level = Set(level);
                }

                active.update(&data.db).await?;

                return Ok(Some(
                    build_whitelist_menu(data, guild_id, module, 0, true, &l10n).await?,
                ));
            }
        }
    }

    // Handle new role modal submission
    if let Some(suffix) = custom_id.strip_prefix("whitelist_modal_role_new_") {
        let module = parse_module(suffix);
        if !check_permission(ctx, data, guild_id, member, module).await? {
            return Ok(None);
        }

        // Get selected role from resolved data
        let selected_role_id = resolved.roles.iter().next().map(|role| role.id);

        // Get level from string select
        let level_str = extract_string_select_value(
            &interaction.data.components,
            "whitelist_modal_level_select",
        );
        let selected_level = match level_str.as_deref() {
            Some("head") => WhitelistLevel::Head,
            Some("admin") => WhitelistLevel::Admin,
            _ => WhitelistLevel::Invulnerable,
        };

        if let Some(role_id) = selected_role_id {
            let model = whitelist_role::ActiveModel {
                guild_id: Set(guild_id.get() as i64),
                role_id: Set(role_id.get() as i64),
                level: Set(selected_level),
                module_type: Set(module),
                ..Default::default()
            };
            model.insert(&data.db).await?;

            return Ok(Some(
                build_whitelist_menu(data, guild_id, module, 0, true, &l10n).await?,
            ));
        }
    }

    // Handle edit role modal submission
    if let Some(rest) = custom_id.strip_prefix("whitelist_modal_role_edit_") {
        let parts: Vec<&str> = rest.splitn(2, '_').collect();
        if parts.len() == 2 {
            let id: i32 = parts[0].parse().unwrap_or(0);
            let module = parse_module(parts[1]);

            if !check_permission(ctx, data, guild_id, member, module).await? {
                return Ok(None);
            }

            let existing = whitelist_role::Entity::find_by_id(id)
                .filter(whitelist_role::Column::GuildId.eq(guild_id.get() as i64))
                .one(&data.db)
                .await?;

            if let Some(existing) = existing {
                let mut active: whitelist_role::ActiveModel = existing.into();

                // Get selected role from resolved data
                if let Some(role) = resolved.roles.iter().next() {
                    active.role_id = Set(role.id.get() as i64);
                }

                // Get level from string select
                let level_str = extract_string_select_value(
                    &interaction.data.components,
                    "whitelist_modal_level_select",
                );
                if let Some(level) = match level_str.as_deref() {
                    Some("head") => Some(WhitelistLevel::Head),
                    Some("admin") => Some(WhitelistLevel::Admin),
                    Some("invulnerable") => Some(WhitelistLevel::Invulnerable),
                    _ => None,
                } {
                    active.level = Set(level);
                }

                active.update(&data.db).await?;

                return Ok(Some(
                    build_whitelist_menu(data, guild_id, module, 0, true, &l10n).await?,
                ));
            }
        }
    }

    Ok(None)
}

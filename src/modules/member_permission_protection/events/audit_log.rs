use crate::db::entities::module_configs::{self, ModuleType};
use crate::services::logger::LogLevel;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;
use serenity::model::guild::audit_log::{Action, Change, MemberAction};

pub async fn handle_audit_log(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Error> {
    // Check action type first to avoid unnecessary database calls
    if !matches!(entry.action, Action::Member(MemberAction::RoleUpdate)) {
        return Ok(());
    }

    let config_model = match module_configs::Entity::find_by_id((
        guild_id.get() as i64,
        ModuleType::MemberPermissionProtection,
    ))
    .one(&data.db)
    .await?
    {
        Some(m) => {
            if !m.enabled {
                return Ok(());
            }
            m
        }
        None => return Ok(()),
    };

    let user_id = match entry.user_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // Ignore actions by the bot itself
    if user_id == ctx.cache.current_user().id {
        return Ok(());
    }

    // Check whitelist
    let whitelist_level = data
        .whitelist
        .get_whitelist_level(
            ctx,
            guild_id,
            user_id,
            ModuleType::MemberPermissionProtection,
        )
        .await?;

    match entry.action {
        Action::Member(MemberAction::RoleUpdate) => {
            handle_member_role_update(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
            )
            .await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_member_role_update(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
) -> Result<(), Error> {
    let target_user_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    if target_user_id == 0 {
        return Ok(());
    }

    // Extract added roles from audit log changes
    let mut added_role_ids = Vec::new();
    let mut removed_role_ids = Vec::new();

    for change in &entry.changes {
        match change {
            Change::RolesAdded { new, .. } => {
                if let Some(roles) = new {
                    let ids: Vec<serenity::RoleId> = roles.iter().map(|r| r.id).collect();
                    added_role_ids.extend(ids);
                }
            }
            Change::RolesRemove { old, .. } => {
                if let Some(roles) = old {
                    let ids: Vec<serenity::RoleId> = roles.iter().map(|r| r.id).collect();
                    removed_role_ids.extend(ids);
                }
            }
            _ => {}
        }
    }

    if added_role_ids.is_empty() {
        return Ok(());
    }

    // Get guild to check role permissions
    let guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(g) => g,
        Err(_) => return Ok(()),
    };

    // Get target member current state
    let member = match guild_id
        .member(&ctx.http, serenity::UserId::new(target_user_id))
        .await
    {
        Ok(m) => m,
        Err(_) => return Ok(()),
    };

    let dangerous_permissions = serenity::Permissions::ADMINISTRATOR
        | serenity::Permissions::MANAGE_GUILD
        | serenity::Permissions::MANAGE_ROLES
        | serenity::Permissions::MANAGE_CHANNELS
        | serenity::Permissions::KICK_MEMBERS
        | serenity::Permissions::BAN_MEMBERS
        | serenity::Permissions::MANAGE_WEBHOOKS
        | serenity::Permissions::MANAGE_GUILD_EXPRESSIONS
        | serenity::Permissions::MANAGE_THREADS
        | serenity::Permissions::MANAGE_MESSAGES
        | serenity::Permissions::MANAGE_EVENTS
        | serenity::Permissions::MODERATE_MEMBERS;

    // We compare "Before" and "After" permissions
    // Before = Current member roles - Added roles + Removed roles
    // After = Current member roles

    let current_roles: Vec<serenity::RoleId> = member.roles.iter().cloned().collect();

    let mut roles_before = current_roles.clone();
    roles_before.retain(|r| !added_role_ids.contains(r));
    roles_before.extend(removed_role_ids.clone());

    let mut perms_before = serenity::Permissions::empty();
    // Everyone role
    if let Some(everyone_role) = guild.roles.get(&serenity::RoleId::new(guild_id.get())) {
        perms_before |= everyone_role.permissions;
    }
    for role_id in &roles_before {
        if let Some(role) = guild.roles.get(role_id) {
            perms_before |= role.permissions;
        }
    }

    let mut perms_after = serenity::Permissions::empty();
    if let Some(everyone_role) = guild.roles.get(&serenity::RoleId::new(guild_id.get())) {
        perms_after |= everyone_role.permissions;
    }
    for role_id in &current_roles {
        if let Some(role) = guild.roles.get(role_id) {
            perms_after |= role.permissions;
        }
    }

    let added_perms = perms_after & !perms_before;
    let dangerous_added = added_perms.intersects(dangerous_permissions);

    if !dangerous_added {
        return Ok(());
    }

    let l10n = data.l10n.get_proxy(&guild.preferred_locale.to_string());

    let mut status = if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        l10n.t("log-status-whitelisted", Some(&args))
    } else {
        l10n.t("log-status-unauthorized", None)
    };

    if whitelist_level.is_none() {
        let reason = l10n.t("log-member-perm-reason-update", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::MemberPermissionProtection,
                &reason,
            )
            .await?;

        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => {
                let mut args = fluent::FluentArgs::new();
                args.set("type", format!("{:?}", p));
                l10n.t("log-status-punished", Some(&args))
            }
            crate::services::punishment::ViolationResult::ViolationRecorded {
                current,
                threshold,
            } => {
                let mut args = fluent::FluentArgs::new();
                args.set("current", current);
                args.set("threshold", threshold);
                l10n.t("log-status-violation", Some(&args))
            }
            crate::services::punishment::ViolationResult::None => {
                l10n.t("log-status-blocked", None)
            }
        };

        if config.revert {
            let mut roles_to_set = current_roles.clone();
            roles_to_set.retain(|r| !added_role_ids.contains(r));

            let revert_reason = l10n.t("log-member-perm-revert-reason", None);
            if guild_id
                .edit_member(
                    &ctx.http,
                    serenity::UserId::new(target_user_id),
                    serenity::EditMember::default()
                        .roles(roles_to_set)
                        .audit_log_reason(&revert_reason),
                )
                .await
                .is_ok()
            {
                status += &l10n.t("log-status-reverted", None);
            } else {
                status += &l10n.t("log-status-revert-failed", None);
            }
        }
    } else if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        args.set("punishment", format!("{:?}", config.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-member-perm-title-whitelisted", None)
    } else {
        l10n.t("log-member-perm-title-blocked", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else {
        LogLevel::Warn
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("userId", user_id.get());
    desc_args.set("targetId", target_user_id);
    let desc = l10n.t("log-member-perm-desc", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::MemberPermissionProtection),
            log_level,
            &title,
            &desc,
            vec![
                (
                    &l10n.t("log-field-acting-user", None),
                    format!("<@{}>", user_id),
                ),
                (
                    &l10n.t("log-field-target-member", None),
                    format!("<@{}>", target_user_id),
                ),
                (
                    &l10n.t("log-field-added-perms", None),
                    format!("`{:?}`", added_perms & dangerous_permissions),
                ),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

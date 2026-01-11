use crate::db::entities::module_configs::{self, ModuleType, RolePermissionProtectionModuleConfig};
use crate::services::logger::LogLevel;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use serenity::model::guild::audit_log::{Action, RoleAction, Change};
use sea_orm::EntityTrait;

pub async fn handle_audit_log(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Error> {
    let config_model = match module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::RolePermissionProtection))
        .one(&data.db)
        .await?
    {
        Some(m) => {
            if !m.enabled {
                return Ok(());
            }
            m
        },
        None => return Ok(()),
    };

    let _config: RolePermissionProtectionModuleConfig = serde_json::from_value(config_model.config.clone()).unwrap_or_default();

    let user_id = match entry.user_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // Ignore actions by the bot itself
    if user_id == ctx.cache.current_user().id {
        return Ok(());
    }

    // Check whitelist
    let whitelist_level = data.whitelist.get_whitelist_level(ctx, guild_id, user_id, ModuleType::RolePermissionProtection).await?;

    match entry.action {
        Action::Role(RoleAction::Update) => {
            // Check if permissions changed
            let has_perm_change = entry.changes.iter().any(|c| matches!(c, Change::Permissions { .. }));
            if has_perm_change {
                handle_role_permission_update(ctx, entry, guild_id, data, &config_model, user_id, whitelist_level).await?;
            }
        }
        _ => {}
    }

    Ok(())
}

async fn handle_role_permission_update(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
) -> Result<(), Error> {
    let role_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    let guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(g) => g,
        Err(_) => return Ok(()),
    };
    let l10n = data.l10n.get_proxy(&guild.preferred_locale.to_string());

    let mut status = if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        l10n.t("log-status-whitelisted", Some(&args))
    } else {
        l10n.t("log-status-unauthorized", None)
    };

    if whitelist_level.is_none() {
        // Punishment
        let reason = l10n.t("log-role-perm-reason-update", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::RolePermissionProtection,
                &reason,
            )
            .await?;

        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => {
                let mut args = fluent::FluentArgs::new();
                args.set("type", format!("{:?}", p));
                l10n.t("log-status-punished", Some(&args))
            }
            crate::services::punishment::ViolationResult::ViolationRecorded { current, threshold } => {
                let mut args = fluent::FluentArgs::new();
                args.set("current", current);
                args.set("threshold", threshold);
                l10n.t("log-status-violation", Some(&args))
            }
            crate::services::punishment::ViolationResult::None => l10n.t("log-status-blocked", None),
        };

        // Revert
        if config.revert && role_id != 0 {
            let mut old_permissions = None;
            for change in &entry.changes {
                if let Change::Permissions { old, .. } = change {
                    old_permissions = *old;
                    break;
                }
            }

            if let Some(p) = old_permissions {
                let revert_reason = l10n.t("log-role-perm-revert-reason", None);
                if guild_id
                    .edit_role(
                        &ctx.http,
                        serenity::RoleId::new(role_id),
                        serenity::EditRole::default()
                            .permissions(p)
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
        }
    } else if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        args.set("punishment", format!("{:?}", config.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-role-perm-title-whitelisted", None)
    } else {
        l10n.t("log-role-perm-title-blocked", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else {
        LogLevel::Warn
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("roleId", role_id);
    desc_args.set("userId", user_id.get());
    let description = l10n.t("log-role-perm-desc-update", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::RolePermissionProtection),
            log_level,
            &title,
            &description,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id)),
                (&l10n.t("log-field-role", None), format!("<@&{}>", role_id)),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}


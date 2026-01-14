use crate::db::entities::module_configs::{self, ModuleType, RoleProtectionModuleConfig};
use crate::services::logger::LogLevel;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;
use serenity::model::guild::audit_log::{Action, RoleAction};

pub async fn handle_audit_log(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Error> {
    // Check action type first to avoid unnecessary database calls
    if !matches!(
        entry.action,
        Action::Role(RoleAction::Create)
            | Action::Role(RoleAction::Delete)
            | Action::Role(RoleAction::Update)
    ) {
        return Ok(());
    }

    let config_model = match module_configs::Entity::find_by_id((
        guild_id.get() as i64,
        ModuleType::RoleProtection,
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

    let config: RoleProtectionModuleConfig =
        serde_json::from_value(config_model.config.clone()).unwrap_or_default();

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
        .get_whitelist_level(ctx, guild_id, user_id, ModuleType::RoleProtection)
        .await?;

    // Match on the audit log action to triggers variants error
    match entry.action {
        Action::Role(RoleAction::Create) => {
            handle_role_create(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
                &config,
            )
            .await?;
        }
        Action::Role(RoleAction::Delete) => {
            handle_role_delete(
                ctx,
                entry,
                guild_id,
                data,
                &config_model,
                user_id,
                whitelist_level,
                &config,
            )
            .await?;
        }
        Action::Role(RoleAction::Update) => {
            // Only process if there are non-permission changes to avoid double handling with RolePermissionProtection
            let has_other_changes = entry.changes.iter().any(|c| {
                !matches!(
                    c,
                    serenity::model::guild::audit_log::Change::Permissions { .. }
                )
            });
            if has_other_changes {
                handle_role_update(
                    ctx,
                    entry,
                    guild_id,
                    data,
                    &config_model,
                    user_id,
                    whitelist_level,
                    &config,
                )
                .await?;
            }
        }
        _ => {}
    }

    Ok(())
}

async fn handle_role_create(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &RoleProtectionModuleConfig,
) -> Result<(), Error> {
    let role_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    let should_punish =
        config.punish_when.is_empty() || config.punish_when.contains(&"create".to_string());

    let guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(g) => g,
        Err(_) => return Ok(()),
    };
    let l10n = data.l10n.get_proxy(&guild.preferred_locale.to_string());

    let mut status = if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        l10n.t("log-status-whitelisted", Some(&args))
    } else if !should_punish {
        l10n.t("log-status-not-enabled", None)
    } else {
        l10n.t("log-status-unauthorized", None)
    };

    if whitelist_level.is_none() && should_punish {
        // Punishment
        let reason = l10n.t("log-role-reason-create", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::RoleProtection,
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
                args.set("current", current.to_string());
                args.set("threshold", threshold.to_string());
                l10n.t("log-status-violation", Some(&args))
            }
            crate::services::punishment::ViolationResult::None => {
                l10n.t("log-status-blocked", None)
            }
        };

        // Revert
        if config_model.revert && role_id != 0 {
            let revert_reason = l10n.t("log-role-revert-reason", None);
            if ctx
                .http
                .delete_role(
                    guild_id,
                    serenity::RoleId::new(role_id),
                    Some(&revert_reason),
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
        args.set("punishment", format!("{:?}", config_model.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-role-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-role-title-blocked", None)
    } else {
        l10n.t("log-role-title-logged", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else if should_punish {
        LogLevel::Warn
    } else {
        LogLevel::Info
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("roleId", role_id.to_string());
    desc_args.set("userId", user_id.get().to_string());
    let desc = l10n.t("log-role-desc-create", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::RoleProtection),
            None,
            log_level,
            &title,
            &desc,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id.get())),
                (&l10n.t("log-field-role-id", None), role_id.to_string()),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

async fn handle_role_delete(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &RoleProtectionModuleConfig,
) -> Result<(), Error> {
    let role_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    let should_punish =
        config.punish_when.is_empty() || config.punish_when.contains(&"delete".to_string());

    let guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(g) => g,
        Err(_) => return Ok(()),
    };
    let l10n = data.l10n.get_proxy(&guild.preferred_locale.to_string());

    let mut status = if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        l10n.t("log-status-whitelisted", Some(&args))
    } else if !should_punish {
        l10n.t("log-status-not-enabled", None)
    } else {
        l10n.t("log-status-unauthorized", None)
    };

    if whitelist_level.is_none() && should_punish {
        // Punishment
        let reason = l10n.t("log-role-reason-delete", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::RoleProtection,
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
                args.set("current", current.to_string());
                args.set("threshold", threshold.to_string());
                l10n.t("log-status-violation", Some(&args))
            }
            crate::services::punishment::ViolationResult::None => {
                l10n.t("log-status-blocked", None)
            }
        };

        // Revert
        if config_model.revert {
            // Wait for the role to be stored in cache
            let mut cached_role = None;
            for _ in 0..10 {
                if let Some(r) = data
                    .cache
                    .take_role(guild_id, serenity::RoleId::new(role_id))
                {
                    cached_role = Some(r);
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }

            if let Some(role) = cached_role {
                let edit_role = serenity::EditRole::default()
                    .name(role.name.clone())
                    .colour(role.colour)
                    .hoist(role.hoist())
                    .mentionable(role.mentionable())
                    .permissions(role.permissions);

                if guild_id.create_role(&ctx.http, edit_role).await.is_ok() {
                    status += &l10n.t("log-status-reverted", None);
                } else {
                    status += &l10n.t("log-status-revert-failed", None);
                }
            } else {
                status += &l10n.t("log-status-revert-failed", None);
            }
        }
    } else if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        args.set("punishment", format!("{:?}", config_model.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-role-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-role-title-blocked", None)
    } else {
        l10n.t("log-role-title-logged", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else if should_punish {
        LogLevel::Error
    } else {
        LogLevel::Info
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("roleId", role_id.to_string());
    desc_args.set("userId", user_id.get().to_string());
    let desc = l10n.t("log-role-desc-delete", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::RoleProtection),
            None,
            log_level,
            &title,
            &desc,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id.get())),
                (&l10n.t("log-field-role-id", None), role_id.to_string()),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

async fn handle_role_update(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
    config_model: &module_configs::Model,
    user_id: serenity::UserId,
    whitelist_level: Option<crate::db::entities::whitelists::WhitelistLevel>,
    config: &RoleProtectionModuleConfig,
) -> Result<(), Error> {
    let role_id = entry.target_id.map(|id| id.get()).unwrap_or(0);
    let should_punish =
        config.punish_when.is_empty() || config.punish_when.contains(&"update".to_string());

    let guild = match guild_id.to_partial_guild(&ctx.http).await {
        Ok(g) => g,
        Err(_) => return Ok(()),
    };
    let l10n = data.l10n.get_proxy(&guild.preferred_locale.to_string());

    let mut status = if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        l10n.t("log-status-whitelisted", Some(&args))
    } else if !should_punish {
        l10n.t("log-status-not-enabled", None)
    } else {
        l10n.t("log-status-unauthorized", None)
    };

    if whitelist_level.is_none() && should_punish {
        // Punishment
        let reason = l10n.t("log-role-reason-update", None);
        let result = data
            .punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::RoleProtection,
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
                args.set("current", current.to_string());
                args.set("threshold", threshold.to_string());
                l10n.t("log-status-violation", Some(&args))
            }
            crate::services::punishment::ViolationResult::None => {
                l10n.t("log-status-blocked", None)
            }
        };

        // Revert
        if config_model.revert && role_id != 0 {
            let mut edit_role = serenity::EditRole::default();
            let mut change_count = 0;
            for change in &entry.changes {
                match change {
                    serenity::model::guild::audit_log::Change::Name { old, .. } => {
                        if let Some(n) = old {
                            edit_role = edit_role.name(n);
                            change_count += 1;
                        }
                    }
                    serenity::model::guild::audit_log::Change::Color { old, .. } => {
                        if let Some(c) = old {
                            edit_role = edit_role.colour(*c as u32);
                            change_count += 1;
                        }
                    }
                    serenity::model::guild::audit_log::Change::Hoist { old, .. } => {
                        if let Some(h) = old {
                            edit_role = edit_role.hoist(*h);
                            change_count += 1;
                        }
                    }
                    serenity::model::guild::audit_log::Change::Mentionable { old, .. } => {
                        if let Some(m) = old {
                            edit_role = edit_role.mentionable(*m);
                            change_count += 1;
                        }
                    }
                    _ => {}
                }
            }

            if change_count > 0 {
                let revert_reason = l10n.t("log-role-revert-reason", None);
                if guild_id
                    .edit_role(
                        &ctx.http,
                        serenity::RoleId::new(role_id),
                        edit_role.audit_log_reason(&revert_reason),
                    )
                    .await
                    .is_ok()
                {
                    status += &l10n.t("log-status-reverted", None);
                } else {
                    status += &l10n.t("log-status-revert-failed", None);
                }
            } else {
                status += &l10n.t("log-status-no-revert", None);
            }
        }
    } else if let Some(level) = whitelist_level {
        let mut args = fluent::FluentArgs::new();
        args.set("level", format!("{:?}", level));
        args.set("punishment", format!("{:?}", config_model.punishment));
        status += &l10n.t("log-status-skipped", Some(&args));
    }

    let is_whitelisted = whitelist_level.is_some();
    let title = if is_whitelisted {
        l10n.t("log-role-title-whitelisted", None)
    } else if should_punish {
        l10n.t("log-role-title-blocked", None)
    } else {
        l10n.t("log-role-title-logged", None)
    };
    let log_level = if is_whitelisted {
        LogLevel::Audit
    } else if should_punish {
        LogLevel::Info
    } else {
        LogLevel::Info
    };

    let mut desc_args = fluent::FluentArgs::new();
    desc_args.set("roleId", role_id.to_string());
    desc_args.set("userId", user_id.get().to_string());
    let desc = l10n.t("log-role-desc-update", Some(&desc_args));

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::RoleProtection),
            None,
            log_level,
            &title,
            &desc,
            vec![
                (&l10n.t("log-field-user", None), format!("<@{}>", user_id.get())),
                (&l10n.t("log-field-role", None), format!("<@&{}>", role_id)),
                (&l10n.t("log-field-action-status", None), status),
            ],
        )
        .await?;

    Ok(())
}

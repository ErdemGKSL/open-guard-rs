use crate::db::entities::module_configs::{self, ModuleType, RoleProtectionModuleConfig};
use crate::services::logger::LogLevel;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use serenity::model::guild::audit_log::{Action, RoleAction};
use sea_orm::EntityTrait;

pub async fn handle_audit_log(
    ctx: &serenity::Context,
    entry: &serenity::AuditLogEntry,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Error> {
    let config_model = match module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::RoleProtection))
        .one(&data.db)
        .await?
    {
        Some(m) => m,
        None => return Ok(()),
    };

    let config: RoleProtectionModuleConfig = serde_json::from_value(config_model.config.clone()).unwrap_or_default();

    let user_id = match entry.user_id {
        Some(id) => id,
        None => return Ok(()),
    };

    // Ignore actions by the bot itself
    if user_id == ctx.cache.current_user().id {
        return Ok(());
    }

    // Check whitelist
    let is_whitelisted = data.whitelist.get_whitelist_level(ctx, guild_id, user_id, ModuleType::RoleProtection).await?.is_some();

    match entry.action {
        Action::Role(RoleAction::Create) => {
            if config.punish_when.contains(&"create".to_string()) {
                handle_role_create(ctx, entry, guild_id, data, &config_model, user_id, is_whitelisted).await?;
            }
        }
        Action::Role(RoleAction::Delete) => {
            if config.punish_when.contains(&"delete".to_string()) {
                handle_role_delete(ctx, entry, guild_id, data, &config_model, user_id, is_whitelisted).await?;
            }
        }
        Action::Role(RoleAction::Update) => {
            // Only process if there are non-permission changes to avoid double handling with RolePermissionProtection
            let has_other_changes = entry.changes.iter().any(|c| !matches!(c, serenity::model::guild::audit_log::Change::Permissions { .. }));
            if has_other_changes && config.punish_when.contains(&"update".to_string()) {
                handle_role_update(ctx, entry, guild_id, data, &config_model, user_id, is_whitelisted).await?;
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
    config: &module_configs::Model,
    user_id: serenity::UserId,
    is_whitelisted: bool,
) -> Result<(), Error> {
    let role_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    let mut status = if is_whitelisted { 
        "✅ Whitelisted (No action taken)".to_string() 
    } else { 
        "🚨 Blocked (Revert Pending)".to_string() 
    };

    if !is_whitelisted {
        // Punishment
        let result = data.punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::RoleProtection,
                "Role Created",
            )
            .await?;

        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => format!("🚨 Blocked & Punished ({:?})", p),
            crate::services::punishment::ViolationResult::ViolationRecorded { current, threshold } => {
                format!("🚨 Blocked & Violation Recorded ({}/{})", current, threshold)
            },
            crate::services::punishment::ViolationResult::None => "🚨 Blocked (No Punishment Configured)".to_string(),
        };

        // Revert
        if config.revert && role_id != 0 {
            let _ = ctx
                .http
                .delete_role(guild_id, serenity::RoleId::new(role_id), Some("Role Protection Revert"))
                .await;
        }
    }

    let title = if is_whitelisted { "Role Created (Whitelisted)" } else { "Role Created" };
    let log_level = if is_whitelisted { LogLevel::Audit } else { LogLevel::Warn };

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::RoleProtection),
            log_level,
            title,
            &format!(
                "A new role (`{}`) was created by <@{}>.\n\n**Status**: {}",
                role_id, user_id, status
            ),
            vec![
                ("User", format!("<@{}>", user_id)),
                ("Role ID", role_id.to_string()),
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
    config: &module_configs::Model,
    user_id: serenity::UserId,
    is_whitelisted: bool,
) -> Result<(), Error> {
    let role_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    let mut status = if is_whitelisted { 
        "✅ Whitelisted (No action taken)".to_string() 
    } else { 
        "🚨 Blocked (Revert Pending)".to_string() 
    };

    if !is_whitelisted {
        // Punishment
        let result = data.punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::RoleProtection,
                "Role Deleted",
            )
            .await?;

        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => format!("🚨 Blocked & Punished ({:?})", p),
            crate::services::punishment::ViolationResult::ViolationRecorded { current, threshold } => {
                format!("🚨 Blocked & Violation Recorded ({}/{})", current, threshold)
            },
            crate::services::punishment::ViolationResult::None => "🚨 Blocked (No Punishment Configured)".to_string(),
        };

        // Revert
        if config.revert {
            // Wait for the role to be stored in cache
            let mut cached_role = None;
            for _ in 0..10 {
                if let Some(r) = data.cache.take_role(guild_id, serenity::RoleId::new(role_id)) {
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
                
                // Position in create_role might be tricky as it depends on hierarchy, 
                // but we try to set it if EditRole supports it.
                // Actually EditRole::position exists.

                let _ = guild_id.create_role(&ctx.http, edit_role).await;
            }
        }
    }

    let title = if is_whitelisted { "Role Deleted (Whitelisted)" } else { "Role Deleted" };
    let log_level = if is_whitelisted { LogLevel::Audit } else { LogLevel::Error };

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::RoleProtection),
            log_level,
            title,
            &format!(
                "A role (`{}`) was deleted by <@{}>.\n\n**Status**: {}",
                role_id, user_id, status
            ),
            vec![
                ("User", format!("<@{}>", user_id)),
                ("Role ID", role_id.to_string()),
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
    config: &module_configs::Model,
    user_id: serenity::UserId,
    is_whitelisted: bool,
) -> Result<(), Error> {
    let role_id = entry.target_id.map(|id| id.get()).unwrap_or(0);

    let mut status = if is_whitelisted { 
        "✅ Whitelisted (No action taken)".to_string() 
    } else { 
        "🚨 Blocked (Revert Pending)".to_string() 
    };

    if !is_whitelisted {
        // Punishment
        let result = data.punishment
            .handle_violation(
                &ctx.http,
                guild_id,
                user_id,
                ModuleType::RoleProtection,
                "Role Updated",
            )
            .await?;

        status = match result {
            crate::services::punishment::ViolationResult::Punished(p) => format!("🚨 Blocked & Punished ({:?})", p),
            crate::services::punishment::ViolationResult::ViolationRecorded { current, threshold } => {
                format!("🚨 Blocked & Violation Recorded ({}/{})", current, threshold)
            },
            crate::services::punishment::ViolationResult::None => "🚨 Blocked (No Punishment Configured)".to_string(),
        };

        // Revert
        if config.revert && role_id != 0 {
            let mut edit_role = serenity::EditRole::default();
            for change in &entry.changes {
                match change {
                    serenity::model::guild::audit_log::Change::Name { old, .. } => {
                        if let Some(n) = old {
                            edit_role = edit_role.name(n);
                        }
                    }
                    serenity::model::guild::audit_log::Change::Color { old, .. } => {
                        if let Some(c) = old {
                            edit_role = edit_role.colour(*c as u32);
                        }
                    }
                    serenity::model::guild::audit_log::Change::Hoist { old, .. } => {
                        if let Some(h) = old {
                            edit_role = edit_role.hoist(*h);
                        }
                    }
                    serenity::model::guild::audit_log::Change::Mentionable { old, .. } => {
                        if let Some(m) = old {
                            edit_role = edit_role.mentionable(*m);
                        }
                    }
                    _ => {}
                }
            }

            let _ = guild_id.edit_role(&ctx.http, serenity::RoleId::new(role_id), edit_role).await;
        }
    }

    let title = if is_whitelisted { "Role Updated (Whitelisted)" } else { "Role Updated" };
    let log_level = if is_whitelisted { LogLevel::Audit } else { LogLevel::Info };

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::RoleProtection),
            log_level,
            title,
            &format!(
                "A role (<@&{}>) was modified by <@{}>.\n\n**Status**: {}",
                role_id, user_id, status
            ),
            vec![
                ("User", format!("<@{}>", user_id)),
                ("Role", format!("<@&{}>", role_id)),
            ],
        )
        .await?;

    Ok(())
}

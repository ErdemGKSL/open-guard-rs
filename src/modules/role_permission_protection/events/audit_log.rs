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
    let is_whitelisted = data.whitelist.get_whitelist_level(ctx, guild_id, user_id, ModuleType::RolePermissionProtection).await?.is_some();

    match entry.action {
        Action::Role(RoleAction::Update) => {
            // Check if permissions changed
            let has_perm_change = entry.changes.iter().any(|c| matches!(c, Change::Permissions { .. }));
            if has_perm_change {
                handle_role_permission_update(ctx, entry, guild_id, data, &config_model, user_id, is_whitelisted).await?;
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
                ModuleType::RolePermissionProtection,
                "Role Permissions Updated",
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
            let mut old_permissions = None;
            for change in &entry.changes {
                if let Change::Permissions { old, .. } = change {
                    old_permissions = *old;
                    break;
                }
            }

            if let Some(p) = old_permissions {
                let _ = guild_id.edit_role(&ctx.http, serenity::RoleId::new(role_id), serenity::EditRole::default().permissions(p)).await;
            }
        }
    }

    let title = if is_whitelisted { "Role Permissions Updated (Whitelisted)" } else { "Role Permissions Updated" };
    let log_level = if is_whitelisted { LogLevel::Audit } else { LogLevel::Info };

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::RolePermissionProtection),
            log_level,
            title,
            &format!(
                "Permissions for role (<@&{}>) were modified by <@{}>.\n\n**Status**: {}",
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

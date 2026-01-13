use crate::db::entities::module_configs::{self, ModuleType, StickyRolesModuleConfig};
use crate::modules::logging::events::membership::{get_member_roles, touch_logging_guild};
use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use sea_orm::EntityTrait;
use tracing::{error, info};

/// Check if the sticky roles module is enabled.
/// Returns Some(config) if enabled, None if disabled.
pub async fn get_sticky_roles_config(
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<Option<StickyRolesModuleConfig>, Error> {
    let m_config =
        module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::StickyRoles))
            .one(&data.db)
            .await?;

    match m_config {
        Some(m) => {
            if !m.enabled {
                return Ok(None);
            }
            let config: StickyRolesModuleConfig =
                serde_json::from_value(m.config).unwrap_or_default();
            Ok(Some(config))
        }
        None => Ok(None),
    }
}

pub async fn handle_guild_member_add(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    member: &serenity::Member,
    data: &Data,
) -> Result<(), Error> {
    // Check if sticky roles module is enabled
    if get_sticky_roles_config(guild_id, data).await?.is_none() {
        return Ok(());
    }

    // Get stored roles
    let stored_roles = match get_member_roles(guild_id, member.user.id, data).await? {
        Some(roles) => roles,
        None => return Ok(()),
    };

    if stored_roles.is_empty() {
        return Ok(());
    }

    // Touch the logging guild (since it shares the tracking)
    touch_logging_guild(guild_id, data).await?;

    info!(
        "Restoring sticky roles for member {} in guild {}",
        member.user.id, guild_id
    );

    // Fetch bot's highest role position
    let current_user_id = ctx.cache.current_user().id;
    let bot_member = guild_id.member(&ctx.http, current_user_id).await?;
    let guild_roles = guild_id.roles(&ctx.http).await?;

    let bot_highest_role = bot_member
        .roles
        .iter()
        .filter_map(|r| guild_roles.get(r))
        .map(|r| r.position)
        .max()
        .unwrap_or(0);

    let mut roles_to_add = vec![];
    for role_id in stored_roles {
        if member.roles.contains(&role_id) {
            continue;
        }

        if let Some(role) = guild_roles.get(&role_id) {
            if role.position < bot_highest_role && !role.managed() {
                roles_to_add.push(role_id);
            }
        }
    }

    if !roles_to_add.is_empty() {
        let mut final_roles: Vec<serenity::RoleId> = member.roles.iter().cloned().collect();
        for r in roles_to_add {
            if !final_roles.contains(&r) {
                final_roles.push(r);
            }
        }

        let mut member_editable = member.clone();
        if let Err(e) = member_editable
            .edit(&ctx.http, serenity::EditMember::new().roles(final_roles))
            .await
        {
            error!(
                "Failed to restore sticky roles for member {} in guild {}: {:?}",
                member.user.id, guild_id, e
            );
        }
    }

    Ok(())
}

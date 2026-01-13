use crate::db::entities::logging_guilds;
use crate::db::entities::member_old_roles;
use crate::db::entities::module_configs::{self, LoggingModuleConfig, ModuleType};
use crate::{Data, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::json;
use tracing::error;

/// Helper function to touch logging guild stats.
/// This prevents cleanup of data for active guilds.
pub async fn touch_guild_stats(guild_id: serenity::GuildId, data: &Data) -> Result<(), Error> {
    let now = Utc::now();
    let model = logging_guilds::ActiveModel {
        guild_id: Set(guild_id.get() as i64),
        last_accessed_at: Set(now.into()),
    };

    logging_guilds::Entity::insert(model)
        .on_conflict(
            sea_orm::sea_query::OnConflict::column(logging_guilds::Column::GuildId)
                .update_column(logging_guilds::Column::LastAccessedAt)
                .to_owned(),
        )
        .exec(&data.db)
        .await?;

    Ok(())
}

/// Store member roles in the database.
/// This function verifies if EITHER Logging (Membership) OR Sticky Roles is enabled.
/// If neither is enabled, it does nothing.
pub async fn store_member_roles(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    roles: &[serenity::RoleId],
    data: &Data,
) -> Result<(), Error> {
    // Check if we should store roles
    // We store roles if either Logging (with membership logging enabled) or Sticky Roles is enabled
    let configs = module_configs::Entity::find()
        .filter(
            module_configs::Column::GuildId
                .eq(guild_id.get() as i64)
                .and(module_configs::Column::Enabled.eq(true))
                .and(
                    module_configs::Column::ModuleType
                        .eq(ModuleType::Logging)
                        .or(module_configs::Column::ModuleType.eq(ModuleType::StickyRoles)),
                ),
        )
        .all(&data.db)
        .await?;

    let should_store = configs.iter().any(|config| match config.module_type {
        ModuleType::StickyRoles => true,
        ModuleType::Logging => {
            let log_config: LoggingModuleConfig =
                serde_json::from_value(config.config.clone()).unwrap_or_default();
            log_config.log_membership
        }
        _ => false,
    });

    if !should_store {
        return Ok(());
    }

    // Touch guild stats (for cleanup purposes)
    touch_guild_stats(guild_id, data).await?;

    // Store attributes
    let role_ids: Vec<u64> = roles.iter().map(|r| r.get()).collect();
    let now = Utc::now();

    let model = member_old_roles::ActiveModel {
        guild_id: Set(guild_id.get() as i64),
        user_id: Set(user_id.get() as i64),
        role_ids: Set(json!(role_ids)),
        updated_at: Set(now.into()),
    };

    member_old_roles::Entity::insert(model)
        .on_conflict(
            sea_orm::sea_query::OnConflict::columns([
                member_old_roles::Column::GuildId,
                member_old_roles::Column::UserId,
            ])
            .update_columns([
                member_old_roles::Column::RoleIds,
                member_old_roles::Column::UpdatedAt,
            ])
            .to_owned(),
        )
        .exec(&data.db)
        .await?;

    Ok(())
}

/// Retrieve stored roles for a member.
pub async fn get_stored_member_roles(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    data: &Data,
) -> Result<Option<Vec<serenity::RoleId>>, Error> {
    let result =
        member_old_roles::Entity::find_by_id((guild_id.get() as i64, user_id.get() as i64))
            .one(&data.db)
            .await?;

    if let Some(model) = result {
        let role_ids: Vec<u64> = serde_json::from_value(model.role_ids)?;
        let roles: Vec<serenity::RoleId> =
            role_ids.into_iter().map(serenity::RoleId::new).collect();
        Ok(Some(roles))
    } else {
        Ok(None)
    }
}

pub async fn handle_guild_member_add(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    member: serenity::Member,
    data: &Data,
) -> Result<(), Error> {
    // Touch stats
    touch_guild_stats(guild_id, data).await?;

    // 2. Handle Sticky Roles (Restore)
    if let Err(e) = crate::modules::sticky_roles::tracking::handle_guild_member_add(
        ctx, guild_id, &member, data,
    )
    .await
    {
        error!("Error restoring sticky roles: {:?}", e);
    }

    // 3. Handle Logging (Join Message)
    if let Err(e) = crate::modules::logging::events::membership::handle_guild_member_add(
        ctx,
        guild_id,
        member.clone(),
        data,
    )
    .await
    {
        error!("Error logging member join: {:?}", e);
    }

    Ok(())
}

pub async fn handle_guild_member_update(
    _ctx: &serenity::Context,
    old_if_available: Option<serenity::Member>,
    new: Option<serenity::Member>,
    event: serenity::GuildMemberUpdateEvent,
    data: &Data,
) -> Result<(), Error> {
    // Determine if roles changed
    let roles_changed = if let Some(old) = old_if_available {
        // If we have old member, compare roles
        let old_roles = &old.roles;
        let new_roles = &event.roles;

        if old_roles.len() != new_roles.len() {
            true
        } else {
            // Check if all old roles are in new roles
            !old_roles.iter().all(|r| new_roles.contains(r))
        }
    } else if let Some(_new_member) = new {
        // If we only have new member (unlikely without old, but possible)
        // Check event roles vs new_member roles (they should be same)
        true // Assume changed if we don't know old state
    } else {
        // If we have neither old nor new member, we just have the event
        true
    };

    if roles_changed {
        // Store new roles
        store_member_roles(event.guild_id, event.user.id, &event.roles, data).await?;
    }

    Ok(())
}

pub async fn handle_guild_member_remove(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    user: serenity::User,
    member_data_if_available: Option<serenity::Member>,
    data: &Data,
) -> Result<(), Error> {
    // Store roles before they leave (if possible)
    if let Some(member) = member_data_if_available.clone() {
        store_member_roles(guild_id, user.id, &member.roles, data).await?;
    }

    // Handle Logging (Leave Message)
    if let Err(e) = crate::modules::logging::events::membership::handle_guild_member_remove(
        ctx,
        guild_id,
        user,
        member_data_if_available,
        data,
    )
    .await
    {
        error!("Error logging member leave: {:?}", e);
    }

    Ok(())
}

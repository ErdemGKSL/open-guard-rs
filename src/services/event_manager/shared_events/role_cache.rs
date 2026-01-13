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
    // We store roles if:
    // 1. Membership logging is enabled
    // 2. Sticky roles is enabled

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

    let mut should_store = false;

    for config in configs {
        match config.module_type {
            ModuleType::StickyRoles => {
                should_store = true;
            }
            ModuleType::Logging => {
                let log_config: LoggingModuleConfig =
                    serde_json::from_value(config.config).unwrap_or_default();
                if log_config.log_membership {
                    should_store = true;
                }
            }
            _ => {}
        }
    }

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
    // 1. Store member roles (if either Logging or Sticky Roles is enabled)
    // We pass the roles from the new member object.
    // If it's a new join, roles are likely empty or auto-assigned.
    // This store is important to initialize the record or update it.
    // Wait, if they just joined, they usually have NO roles (unless bots added them immediately).
    // Storing "empty roles" might wipe sticky roles if we are not careful?
    // Sticky roles logic relies on `get_stored_member_roles`.
    // If we overwrite with current (empty) roles BEFORE restoring, we lose data!
    // CRITICAL: We must NOT store roles here blindly.
    // Sticky roles flow:
    // 1. User Joins (Roles empty)
    // 2. We fetch OLD stored roles.
    // 3. We re-apply them.
    // 4. THEN we might store the new state (which matches old state).
    //
    // So, `handle_guild_member_add` should:
    // A. RESTORE roles first (if Sticky enabled).
    // B. LOG the join (if Logging enabled).
    // C. Store roles? Only if they change?
    // Actually, `store_member_roles` logic is mostly for `GuildMemberUpdate`.
    // For `GuildMemberAdd`, we only want to *possibly* update the timestamp (touch stats).
    // But `store_member_roles` also inserts roles.
    // If we call `store_member_roles` with empty list, it might wipe previous data if the user had data?
    // Let's check `store_member_roles` implementation in `tracking.rs`.
    // It uses `Entity::insert` with `on_conflict(...).update_columns(...)`.
    // YES, it will overwrite `role_ids` with the passed `roles`.
    // IF the user rejoined, they have 0 roles. If we overwrite, we delete their sticky roles!
    //
    // THEREFORE: We should NOT call `store_member_roles` immediately on join.
    // We should only "Touch" the guild stats.
    // The `Sticky Roles` module needs to fetch the *existing* data.
    //
    // So the previous `logging` implementation was:
    // `store_member_roles_internal(guild_id, member.user.id, &member.roles, data).await?;`
    // This was DESTRUCTIVE if the user had sticky roles!
    // Good catch. The user said "handle_guild_member_add is also should be handled on shared events".
    //
    // Revised Logic for `handle_guild_member_add`:
    // 1. Touch Stats (always, if any module active).
    // 2. Restore Sticky Roles (if enabled). This will fetch old roles and apply them.
    //    Note: If we restore roles, `GuildMemberUpdate` events will trigger?
    //    Serenity might trigger `GuildMemberUpdate` when we `edit` the member.
    //    If so, that will trigger data storage, which is fine (it will store the restored roles).
    // 3. Log Join (if enabled).
    //
    // We should NOT store roles here.

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

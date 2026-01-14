use crate::db::entities::logging_guilds;
use crate::db::entities::member_old_roles;
use crate::db::entities::module_configs::{self, LoggingModuleConfig, ModuleType};
use crate::{Data, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;
use sea_orm::{EntityTrait, Set};
use serde_json::json;

/// Check if the logging module is enabled for membership logging.
/// Returns Some(config) if enabled, None if disabled.
pub async fn get_membership_logging_config(
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<Option<LoggingModuleConfig>, Error> {
    let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::Logging))
        .one(&data.db)
        .await?;

    match m_config {
        Some(m) => {
            // Check if module is enabled
            if !m.enabled {
                return Ok(None);
            }
            let config: LoggingModuleConfig = serde_json::from_value(m.config).unwrap_or_default();
            // Check if membership logging is enabled
            if !config.log_membership {
                return Ok(None);
            }
            Ok(Some(config))
        }
        None => Ok(None),
    }
}

use crate::services::event_manager::shared_events;

/// Upsert the logging guild entry to track last access time.
/// This is used for cleanup of stale guilds.
pub async fn touch_logging_guild(guild_id: serenity::GuildId, data: &Data) -> Result<(), Error> {
    shared_events::touch_guild_stats(guild_id, data).await
}

/// Get the member's stored roles from the database.
pub async fn get_member_roles(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    data: &Data,
) -> Result<Option<Vec<serenity::RoleId>>, Error> {
    shared_events::get_stored_member_roles(guild_id, user_id, data).await
}

/// Delete the member's stored roles from the database.

/// Delete all member role records for a guild.
/// Used when membership logging is disabled.
pub async fn delete_all_guild_member_roles(
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<u64, Error> {
    use sea_orm::ColumnTrait;
    use sea_orm::QueryFilter;

    let result = member_old_roles::Entity::delete_many()
        .filter(member_old_roles::Column::GuildId.eq(guild_id.get() as i64))
        .exec(&data.db)
        .await?;

    // Also delete the logging guild entry
    logging_guilds::Entity::delete_by_id(guild_id.get() as i64)
        .exec(&data.db)
        .await
        .ok();

    Ok(result.rows_affected)
}

/// Internal function to touch logging guild with db connection directly.
async fn touch_logging_guild_with_db(
    guild_id: serenity::GuildId,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), Error> {
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
        .exec(db)
        .await?;

    Ok(())
}

/// Internal function to store member roles with db connection directly.
async fn store_member_roles_with_db(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    roles: &[serenity::RoleId],
    db: &sea_orm::DatabaseConnection,
) -> Result<(), Error> {
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
        .exec(db)
        .await?;

    Ok(())
}

/// Fetch all members from the guild and store their roles in the database.
/// Used when membership logging is enabled.
/// This is run in the background to not block the interaction.
/// Takes DatabaseConnection directly so it can be used in spawned tasks.
/// Filters out managed roles (bot/integration roles) as they can't be restored.
pub async fn fetch_and_store_all_members(
    http: std::sync::Arc<serenity::Http>,
    guild_id: serenity::GuildId,
    db: sea_orm::DatabaseConnection,
) -> Result<usize, Error> {
    use serenity::nonmax::NonMaxU16;
    use std::collections::HashSet;
    use tracing::{error, info};

    // Touch the logging guild first
    touch_logging_guild_with_db(guild_id, &db).await?;

    // Fetch guild roles to identify managed roles
    let guild_roles = guild_id.roles(&http).await.unwrap_or_default();
    let managed_role_ids: HashSet<serenity::RoleId> = guild_roles
        .into_iter()
        .filter(|role| role.managed())
        .map(|role| role.id)
        .collect();

    let mut last_id: Option<serenity::UserId> = None;
    let mut total_members = 0;

    loop {
        match guild_id
            .members(&http, Some(NonMaxU16::new(1000).unwrap()), last_id)
            .await
        {
            Ok(members) => {
                let count = members.len();
                total_members += count;

                // Store each member's roles (excluding managed roles)
                for member in &members {
                    let roles: Vec<serenity::RoleId> = member
                        .roles
                        .iter()
                        .filter(|r| !managed_role_ids.contains(r))
                        .cloned()
                        .collect();

                    if let Err(e) =
                        store_member_roles_with_db(guild_id, member.user.id, &roles, &db).await
                    {
                        error!(
                            "Failed to store roles for member {} in guild {}: {:?}",
                            member.user.id.get(), guild_id.get(), e
                        );
                    }
                }

                info!(
                    "Stored roles for {} members (total: {}) in guild {}",
                    count, total_members, guild_id.get()
                );

                if count < 1000 {
                    break;
                }

                if let Some(last_member) = members.last() {
                    last_id = Some(last_member.user.id);
                }
            }
            Err(e) => {
                error!(
                    "Failed to fetch members for guild {} at offset {:?}: {:?}",
                    guild_id, last_id, e
                );
                break;
            }
        }
    }

    info!(
        "Finished storing roles for {} members in guild {}",
        total_members, guild_id.get()
    );

    Ok(total_members)
}

pub async fn handle_guild_member_add(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    member: serenity::Member,
    data: &Data,
) -> Result<(), Error> {
    // Check if logging module is enabled for membership
    let config = match get_membership_logging_config(guild_id, data).await? {
        Some(c) => c,
        None => return Ok(()),
    };

    // We do NOT store roles here anymore, as it is handled by shared_events::membership.

    let l10n = data.l10n.get_l10n_for_guild(guild_id, &data.db).await;

    let mut args = fluent::FluentArgs::new();
    args.set("userId", member.user.id.get().to_string());

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::Logging),
            config.membership_log_channel_id,
            crate::services::logger::LogLevel::Info,
            &l10n.t("log-member-join-title", None),
            &l10n.t("log-member-join-desc", Some(&args)),
            vec![],
        )
        .await?;

    Ok(())
}

pub async fn handle_guild_member_remove(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    user: serenity::User,
    member_data_if_available: Option<serenity::Member>,
    data: &Data,
) -> Result<(), Error> {
    // Check if logging module is enabled for membership
    let config = match get_membership_logging_config(guild_id, data).await? {
        Some(c) => c,
        None => return Ok(()),
    };

    // Touch the logging guild to update last access time
    touch_logging_guild(guild_id, data).await?;

    let l10n = data.l10n.get_l10n_for_guild(guild_id, &data.db).await;

    let mut args = fluent::FluentArgs::new();
    args.set("userId", user.id.get().to_string());

    let mut fields = vec![];

    // Try to get roles from cache first, then fall back to database
    let roles: Vec<serenity::RoleId> = if let Some(ref member) = member_data_if_available {
        member.roles.iter().cloned().collect()
    } else {
        // Fall back to database
        get_member_roles(guild_id, user.id, data)
            .await?
            .unwrap_or_default()
    };

    if !roles.is_empty() {
        let role_mentions = roles
            .iter()
            .map(|id| format!("<@&{}>", id.get()))
            .collect::<Vec<_>>()
            .join(", ");

        let label = l10n.t("log-member-leave-roles", None);
        fields.push((Box::leak(label.into_boxed_str()) as &str, role_mentions));
    }

    // We don't clean up the stored roles here anymore because Sticky Roles needs them.
    // Stale data is handled by the LoggingCleanupService (30 day inactivity).

    data.logger
        .log_action(
            &ctx.http,
            guild_id,
            Some(ModuleType::Logging),
            config.membership_log_channel_id,
            crate::services::logger::LogLevel::Info,
            &l10n.t("log-member-leave-title", None),
            &l10n.t("log-member-leave-desc", Some(&args)),
            fields,
        )
        .await?;

    Ok(())
}

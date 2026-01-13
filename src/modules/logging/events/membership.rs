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

/// Upsert the logging guild entry to track last access time.
/// This is used for cleanup of stale guilds.
async fn touch_logging_guild(guild_id: serenity::GuildId, data: &Data) -> Result<(), Error> {
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

/// Internal function to store member roles without config check.
async fn store_member_roles_internal(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    roles: &[serenity::RoleId],
    data: &Data,
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
        .exec(&data.db)
        .await?;

    Ok(())
}

/// Store or update the member's roles in the database.
/// Only stores if the logging module is enabled.
pub async fn store_member_roles(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    roles: &[serenity::RoleId],
    data: &Data,
) -> Result<(), Error> {
    // Check if logging module is enabled for membership
    if get_membership_logging_config(guild_id, data)
        .await?
        .is_none()
    {
        return Ok(());
    }

    // Touch the logging guild to update last access time
    touch_logging_guild(guild_id, data).await?;

    // Store the roles
    store_member_roles_internal(guild_id, user_id, roles, data).await
}

/// Get the member's stored roles from the database.
pub async fn get_member_roles(
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

/// Delete the member's stored roles from the database.
pub async fn delete_member_roles(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    data: &Data,
) -> Result<(), Error> {
    member_old_roles::Entity::delete_by_id((guild_id.get() as i64, user_id.get() as i64))
        .exec(&data.db)
        .await?;
    Ok(())
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

    // Touch the logging guild to update last access time
    touch_logging_guild(guild_id, data).await?;

    // Store the member's initial roles (usually empty on join, but could have auto-roles)
    store_member_roles_internal(guild_id, member.user.id, &member.roles, data).await?;

    let l10n = data.l10n.get_l10n_for_guild(guild_id, &data.db).await;

    let mut args = fluent::FluentArgs::new();
    args.set("userId", member.user.id.get());

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
    args.set("userId", user.id.get());

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

    // Clean up the stored roles after logging
    delete_member_roles(guild_id, user.id, data).await?;

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

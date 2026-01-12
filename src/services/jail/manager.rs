use crate::db::entities::module_configs::ModuleType;
use crate::db::entities::{guild_configs, jails};
use crate::services::logger::{LogLevel, LoggerService};
use chrono::{Duration, Utc};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

pub struct JailService {
    pub(crate) db: DatabaseConnection,
    pub(crate) logger: Arc<LoggerService>,
    pub(crate) l10n: Arc<crate::services::localization::LocalizationManager>,
}

impl JailService {
    pub fn new(
        db: DatabaseConnection,
        logger: Arc<LoggerService>,
        l10n: Arc<crate::services::localization::LocalizationManager>,
    ) -> Self {
        Self { db, logger, l10n }
    }

    pub async fn jail_user(
        &self,
        http: &serenity::Http,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
        duration: Option<Duration>,
        reason: &str,
    ) -> Result<(), crate::Error> {
        let guild = guild_id.to_partial_guild(http).await?;
        let mut member = guild_id.member(http, user_id).await?;

        // Get guild config for jail role
        let g_config = guild_configs::Entity::find_by_id(guild_id.get() as i64)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Guild configuration not found"))?;

        let jail_role_id = g_config
            .jail_role_id
            .ok_or_else(|| anyhow::anyhow!("Jail role not configured"))?;
        let jail_role = serenity::RoleId::new(jail_role_id as u64);

        // Find bot's highest role to avoid trying to remove roles we can't reach
        let current_user = http.get_current_user().await?;
        let bot_member = guild_id.member(http, current_user.id).await?;
        let bot_highest_role_pos = bot_member
            .roles
            .iter()
            .filter_map(|r| guild.roles.get(r).map(|role| role.position))
            .max()
            .unwrap_or(0);

        let mut old_roles_to_store = Vec::new();
        let mut new_roles_to_apply = Vec::new();

        for role_id in &member.roles {
            if let Some(role) = guild.roles.get(role_id) {
                // Keep the role if it's managed or above bot's reach
                if role.managed() || role.position >= bot_highest_role_pos {
                    new_roles_to_apply.push(*role_id);
                } else {
                    // This is a role we will remove and store
                    old_roles_to_store.push(role_id.get().to_string());
                }
            }
        }

        // Add jail role to new roles
        if !new_roles_to_apply.contains(&jail_role) {
            new_roles_to_apply.push(jail_role);
        }

        // Store in DB
        let expires_at = duration.map(|d| (Utc::now() + d).naive_utc());
        let old_roles_json = serde_json::to_value(&old_roles_to_store)?;

        let model = jails::ActiveModel {
            guild_id: Set(guild_id.get() as i64),
            user_id: Set(user_id.get() as i64),
            old_roles: Set(old_roles_json),
            expires_at: Set(expires_at),
            reason: Set(Some(reason.to_string())),
            ..Default::default()
        };

        model.insert(&self.db).await?;

        // Apply role changes
        member
            .edit(
                http,
                serenity::EditMember::new()
                    .roles(new_roles_to_apply)
                    .audit_log_reason(reason),
            )
            .await?;

        // Log action
        let guild_locale = guild.preferred_locale.to_string();
        let l10n = self.l10n.get_proxy(&guild_locale);

        let mut user_args = FluentArgs::new();
        user_args.set("userId", user_id.get());

        let duration_str = duration
            .map(|d| format!("{:?}", d))
            .unwrap_or_else(|| l10n.t("log-val-permanent", None));

        self.logger
            .log_action(
                http,
                guild_id,
                Some(ModuleType::ModerationProtection),
                None,
                LogLevel::Audit,
                &l10n.t("log-mod-jail-title", None),
                &l10n.t("log-mod-jail-desc", Some(&user_args)),
                vec![
                    (&l10n.t("log-field-user", None), format!("<@{}>", user_id)),
                    (&l10n.t("log-field-duration", None), duration_str),
                    (&l10n.t("log-field-reason", None), reason.to_string()),
                ],
            )
            .await?;

        Ok(())
    }

    pub async fn unjail_user(
        &self,
        http: &serenity::Http,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
    ) -> Result<(), crate::Error> {
        let jail_record = jails::Entity::find()
            .filter(jails::Column::GuildId.eq(guild_id.get() as i64))
            .filter(jails::Column::UserId.eq(user_id.get() as i64))
            .one(&self.db)
            .await?;

        let record = match jail_record {
            Some(r) => r,
            None => return Ok(()), // Not jailed
        };

        let old_roles_ids: Vec<String> = serde_json::from_value(record.old_roles)?;

        let mut member = guild_id.member(http, user_id).await?;
        let guild = guild_id.to_partial_guild(http).await?;

        // Get jail role id
        let g_config = guild_configs::Entity::find_by_id(guild_id.get() as i64)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Guild configuration not found"))?;

        let jail_role_id = g_config
            .jail_role_id
            .map(|id| serenity::RoleId::new(id as u64));

        let mut new_roles = Vec::new();

        // Keep current managed roles or roles above bot (though they should have been kept anyway)
        let current_user = http.get_current_user().await?;
        let bot_member = guild_id.member(http, current_user.id).await?;
        let bot_highest_role_pos = bot_member
            .roles
            .iter()
            .filter_map(|r| guild.roles.get(r).map(|role| role.position))
            .max()
            .unwrap_or(0);

        for role_id in &member.roles {
            if let Some(role) = guild.roles.get(role_id) {
                if (role.managed() || role.position >= bot_highest_role_pos)
                    && Some(*role_id) != jail_role_id
                {
                    new_roles.push(*role_id);
                }
            }
        }

        // Restore old roles
        for role_str in old_roles_ids {
            if let Ok(id) = role_str.parse::<u64>() {
                let role_id = serenity::RoleId::new(id);
                if !new_roles.contains(&role_id) {
                    new_roles.push(role_id);
                }
            }
        }

        // Apply changes
        member
            .edit(
                http,
                serenity::EditMember::new()
                    .roles(new_roles)
                    .audit_log_reason("Jail expired or removed"),
            )
            .await?;

        // Delete record
        jails::Entity::delete_by_id(record.id)
            .exec(&self.db)
            .await?;

        // Log action
        let guild_locale = guild.preferred_locale.to_string();
        let l10n = self.l10n.get_proxy(&guild_locale);

        let mut user_args = FluentArgs::new();
        user_args.set("userId", user_id.get());

        self.logger
            .log_action(
                http,
                guild_id,
                Some(ModuleType::ModerationProtection),
                None,
                LogLevel::Audit,
                &l10n.t("log-mod-unjail-title", None),
                &l10n.t("log-mod-unjail-desc", Some(&user_args)),
                vec![(&l10n.t("log-field-user", None), format!("<@{}>", user_id))],
            )
            .await?;

        Ok(())
    }
}

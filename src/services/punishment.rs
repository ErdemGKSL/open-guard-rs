use crate::Error;
use crate::db::entities::{
    guild_configs,
    module_configs::{self, ModuleType, PunishmentType},
    violations,
};
use chrono::Utc;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub struct PunishmentService {
    db: DatabaseConnection,
}

impl PunishmentService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Handles a violation by incrementing the counter and applying punishment if threshold is reached.
    pub async fn handle_violation(
        &self,
        http: &serenity::Http,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
        module_type: ModuleType,
        reason: &str,
    ) -> Result<(), Error> {
        let config = module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Module config not found"))?;

        if config.punishment == PunishmentType::None {
            return Ok(());
        }

        // Find or create violation record
        let violation = violations::Entity::find()
            .filter(violations::Column::GuildId.eq(guild_id.get() as i64))
            .filter(violations::Column::UserId.eq(user_id.get() as i64))
            .filter(violations::Column::ModuleType.eq(module_type))
            .one(&self.db)
            .await?;

        let now = Utc::now().naive_utc();
        let active_violation: violations::ActiveModel = match violation {
            Some(v) => {
                let last = v.last_violation_at;
                let diff = (now - last).num_minutes();

                let mut am: violations::ActiveModel = v.clone().into();
                if diff > config.punishment_at_interval as i64 {
                    // Reset count if interval has passed
                    am.count = Set(1);
                } else {
                    am.count = Set(v.count + 1);
                }
                am.last_violation_at = Set(now);
                am
            }
            None => violations::ActiveModel {
                guild_id: Set(guild_id.get() as i64),
                user_id: Set(user_id.get() as i64),
                module_type: Set(module_type),
                count: Set(1),
                last_violation_at: Set(now),
                ..Default::default()
            },
        };

        let updated_violation = active_violation.save(&self.db).await?;
        let current_count = updated_violation.count.as_ref();

        // If punishment_at is 0 or 1, it's immediate.
        // Otherwise, wait until the count hits the threshold.
        let threshold = if config.punishment_at <= 0 {
            1
        } else {
            config.punishment_at
        };

        if *current_count >= threshold {
            self.punish(http, guild_id, user_id, config.punishment, reason)
                .await?;

            // Reset count after punishment? User didn't specify, but usually it's better to reset
            // so they don't get punished on EVERY subsequent action if punishment_at > 1.
            let mut reset_am: violations::ActiveModel = updated_violation;
            reset_am.count = Set(0);
            reset_am.save(&self.db).await?;
        }

        Ok(())
    }

    pub async fn punish(
        &self,
        http: &serenity::Http,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
        punishment: PunishmentType,
        reason: &str,
    ) -> Result<(), Error> {
        match punishment {
            PunishmentType::None => Ok(()),
            PunishmentType::Unperm => {
                let member = guild_id.member(http, user_id).await?;
                for role_id in &member.roles {
                    // Ignore errors for individual roles (e.g. managed roles)
                    let _ = member.remove_role(http, *role_id, Some(reason)).await;
                }
                Ok(())
            }
            PunishmentType::Ban => {
                guild_id.ban(http, user_id, 0, Some(reason)).await?;
                Ok(())
            }
            PunishmentType::Kick => {
                guild_id.kick(http, user_id, Some(reason)).await?;
                Ok(())
            }
            PunishmentType::Jail => {
                let g_config = guild_configs::Entity::find_by_id(guild_id.get() as i64)
                    .one(&self.db)
                    .await?;

                if let Some(config) = g_config {
                    if let Some(role_id) = config.jail_role_id {
                        let member = guild_id.member(http, user_id).await?;
                        member
                            .add_role(http, serenity::RoleId::new(role_id as u64), Some(reason))
                            .await?;
                    }
                }
                Ok(())
            }
        }
    }
}

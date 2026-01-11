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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViolationResult {
    Punished(PunishmentType),
    ViolationRecorded { current: i32, threshold: i32 },
    None,
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
    ) -> Result<ViolationResult, Error> {
        let config = module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Module config not found"))?;

        if config.punishment == PunishmentType::None {
            return Ok(ViolationResult::None);
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
        let current_count = *updated_violation.count.as_ref();

        // If punishment_at is 0 or 1, it's immediate.
        let threshold = if config.punishment_at <= 0 {
            1
        } else {
            config.punishment_at
        };

        if current_count >= threshold {
            if let Err(e) = self.punish(http, guild_id, user_id, config.punishment, reason).await {
                tracing::error!("Failed to punish user {} in guild {}: {:?}", user_id, guild_id, e);
            }

            // Reset count after punishment
            let mut reset_am: violations::ActiveModel = updated_violation;
            reset_am.count = Set(0);
            reset_am.save(&self.db).await?;

            return Ok(ViolationResult::Punished(config.punishment));
        }

        Ok(ViolationResult::ViolationRecorded {
            current: current_count,
            threshold,
        })
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
                let guild = guild_id.to_partial_guild(http).await?;
                let bot_id = http.get_current_user().await?.id;
                let bot_member = guild_id.member(http, bot_id).await?;
                
                // Get bot's highest role position
                let bot_highest_role_pos = bot_member.roles.iter()
                    .filter_map(|role_id| guild.roles.get(role_id))
                    .map(|role| role.position)
                    .max()
                    .unwrap_or(0);

                let dangerous_permissions = serenity::Permissions::ADMINISTRATOR 
                    | serenity::Permissions::MANAGE_GUILD
                    | serenity::Permissions::MANAGE_ROLES
                    | serenity::Permissions::MANAGE_CHANNELS
                    | serenity::Permissions::KICK_MEMBERS
                    | serenity::Permissions::BAN_MEMBERS
                    | serenity::Permissions::MANAGE_WEBHOOKS
                    | serenity::Permissions::MANAGE_GUILD_EXPRESSIONS
                    | serenity::Permissions::MANAGE_THREADS
                    | serenity::Permissions::MANAGE_MESSAGES
                    | serenity::Permissions::MANAGE_EVENTS
                    | serenity::Permissions::MODERATE_MEMBERS;

                let mut new_roles = Vec::new();

                for role_id in &member.roles {
                    if let Some(role) = guild.roles.get(role_id) {
                        // Keep the role if:
                        // 1. It's managed (integration/booster role)
                        // 2. It's higher than or equal to bot's highest role (outside our reach/safety)
                        // 3. It DOES NOT have dangerous permissions
                        if role.managed() || role.position >= bot_highest_role_pos || !role.permissions.intersects(dangerous_permissions) {
                            new_roles.push(*role_id);
                        }
                    } else {
                        // If role is not found in guild roles map, keep it just in case?
                        // Usually this means it's @everyone or something went wrong.
                        new_roles.push(*role_id);
                    }
                }

                // If roles changed, apply it
                if new_roles.len() != member.roles.len() as usize {
                    guild_id.edit_member(http, user_id, serenity::EditMember::default().roles(new_roles).audit_log_reason(reason)).await?;
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

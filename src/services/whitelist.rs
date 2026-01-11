use crate::db::entities::{
    module_configs::ModuleType,
    whitelist_role, whitelist_user,
    whitelists::WhitelistLevel,
};
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

pub struct WhitelistService {
    db: DatabaseConnection,
}

impl WhitelistService {
    pub fn new(db: DatabaseConnection) -> self::WhitelistService {
        Self { db }
    }

    /// Determines the whitelist level for a user in a guild context.
    /// Checks both implicit (role hierarchy) and explicit (database) whitelists.
    /// Returns the highest level found, or None if not whitelisted.
    pub async fn get_whitelist_level(
        &self,
        ctx: &serenity::Context,
        guild_id: serenity::GuildId,
        user_id: serenity::UserId,
        module: ModuleType,
    ) -> Result<Option<WhitelistLevel>, crate::Error> {
        let mut level = None;

        // 0. Server Owner Check
        let cached_owner = ctx.cache.guild(guild_id).map(|g| g.owner_id);
        let is_owner = if let Some(owner_id) = cached_owner {
            owner_id == user_id
        } else {
            match ctx.http.get_guild(guild_id).await {
                Ok(guild) => guild.owner_id == user_id,
                Err(_) => false,
            }
        };

        if is_owner {
            return Ok(Some(WhitelistLevel::Head));
        }

        // 1. Implicit Checks (Hierarchy)
        if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
            let bot_id = ctx.cache.current_user().id;
            if let Ok(bot_member) = guild_id.member(&ctx.http, bot_id).await {
                // We need to resolve role positions.
                // Assuming we can get roles from cache or we need to fetch guild roles.
                // For simplicity, let's try cache first, then http.
                let roles = if let Some(g) = ctx.cache.guild(guild_id) {
                    g.roles.clone()
                } else {
                    guild_id.roles(&ctx.http).await.unwrap_or_default()
                };

                let get_position = |member: &serenity::Member| -> i16 {
                    member.roles.iter()
                        .filter_map(|r| roles.get(r).map(|role| role.position))
                        .max()
                        .unwrap_or(0)
                        // If cached member has no roles but is owner? Owner check requires Guild object.
                };
                
                let user_position = get_position(&member);
                let bot_position = get_position(&bot_member);

                if user_position > bot_position {
                    // User is above bot
                    let has_admin = member.permissions.map_or(false, |p| p.contains(serenity::Permissions::ADMINISTRATOR));
                    
                    let implicit_level = if has_admin {
                        WhitelistLevel::Admin
                    } else {
                        WhitelistLevel::Invulnerable
                    };
                    
                    level = self.merge_levels(level, Some(implicit_level));
                }
            }
        }

        // 2. Explicit User Whitelist
        // Check for specific module OR global (null)
        let user_whitelists = whitelist_user::Entity::find()
            .filter(whitelist_user::Column::GuildId.eq(guild_id.get() as i64))
            .filter(whitelist_user::Column::UserId.eq(user_id.get() as i64))
            .all(&self.db)
            .await?;

        for w in user_whitelists {
            if w.module_type.is_none() || w.module_type == Some(module) {
                level = self.merge_levels(level, Some(w.level));
            }
        }

        // 3. Explicit Role Whitelist
        if let Ok(member) = guild_id.member(&ctx.http, user_id).await {
            let role_ids: Vec<i64> = member.roles.iter().map(|r| r.get() as i64).collect();

            if !role_ids.is_empty() {
                let role_whitelists = whitelist_role::Entity::find()
                    .filter(whitelist_role::Column::GuildId.eq(guild_id.get() as i64))
                    .filter(whitelist_role::Column::RoleId.is_in(role_ids))
                    .all(&self.db)
                    .await?;

                for w in role_whitelists {
                    if w.module_type.is_none() || w.module_type == Some(module) {
                        level = self.merge_levels(level, Some(w.level));
                    }
                }
            }
        }

        Ok(level)
    }

    /// Helper to merge levels, keeping the highest privilege.
    /// Head > Admin > Invulnerable
    fn merge_levels(&self, current: Option<WhitelistLevel>, new: Option<WhitelistLevel>) -> Option<WhitelistLevel> {
        match (current, new) {
            (Some(c), Some(n)) => {
                Some(if c == WhitelistLevel::Head || n == WhitelistLevel::Head {
                    WhitelistLevel::Head
                } else if c == WhitelistLevel::Admin || n == WhitelistLevel::Admin {
                    WhitelistLevel::Admin
                } else {
                    WhitelistLevel::Invulnerable
                })
            }
            (Some(c), None) => Some(c),
            (None, Some(n)) => Some(n),
            (None, None) => None,
        }
    }
}

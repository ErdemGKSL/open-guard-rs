use crate::db::entities::temp_bans;
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

pub struct TempBanService {
    db: DatabaseConnection,
}

impl TempBanService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Starts the background task that checks for expired temporary bans and unbans them.
    pub fn start_unban_runner(&self, http: Arc<serenity::Http>) {
        let db = self.db.clone();
        tokio::spawn(async move {
            info!("Temp-ban unban runner started.");
            loop {
                sleep(Duration::from_secs(60)).await;
                let now = chrono::Utc::now().naive_utc();

                match temp_bans::Entity::find()
                    .filter(temp_bans::Column::ExpiresAt.lt(now))
                    .all(&db)
                    .await
                {
                    Ok(expired_bans) => {
                        for ban in expired_bans {
                            let guild_id = serenity::GuildId::new(ban.guild_id as u64);
                            let user_id = serenity::UserId::new(ban.user_id as u64);

                            info!(
                                "Unbanning user {} in guild {} (ban expired)",
                                user_id, guild_id
                            );

                            if let Err(e) = guild_id
                                .unban(&http, user_id, Some("Temporary ban expired"))
                                .await
                            {
                                error!("Failed to unban user {}: {:?}", user_id, e);
                            }

                            // Delete from DB after unbanning (or attempting to unban)
                            if let Err(e) = temp_bans::Entity::delete_by_id(ban.id).exec(&db).await
                            {
                                error!("Failed to delete expired ban record {}: {:?}", ban.id, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch expired bans: {:?}", e);
                    }
                }
            }
        });
    }
}

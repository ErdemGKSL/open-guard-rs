use super::manager::JailService;
use crate::db::entities::jails;
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

impl JailService {
    pub fn start_unjail_runner(&self, http: Arc<serenity::Http>) {
        let db = self.db.clone();
        let logger = self.logger.clone();
        let l10n = self.l10n.clone();

        tokio::spawn(async move {
            info!("Jail unjail runner started.");
            let jail_service = JailService::new(db.clone(), logger.clone(), l10n.clone());

            loop {
                sleep(Duration::from_secs(60)).await;
                let now = chrono::Utc::now().naive_utc();

                match jails::Entity::find()
                    .filter(jails::Column::ExpiresAt.lt(now))
                    .all(&db)
                    .await
                {
                    Ok(expired_jails) => {
                        for jail in expired_jails {
                            let guild_id = serenity::GuildId::new(jail.guild_id as u64);
                            let user_id = serenity::UserId::new(jail.user_id as u64);

                            info!(
                                "Unjailing user {} in guild {} (jail expired)",
                                user_id, guild_id
                            );

                            if let Err(e) = jail_service.unjail_user(&http, guild_id, user_id).await
                            {
                                error!("Failed to unjail user {}: {:?}", user_id, e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to fetch expired jails: {:?}", e);
                    }
                }
            }
        });
    }
}

use crate::db::entities::module_configs::ModuleType;
use crate::db::entities::temp_bans;
use crate::services::logger::{LogLevel, LoggerService};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tracing::{error, info};

pub struct TempBanService {
    db: DatabaseConnection,
    logger: Arc<LoggerService>,
    l10n: Arc<crate::services::localization::LocalizationManager>,
}

impl TempBanService {
    pub fn new(
        db: DatabaseConnection,
        logger: Arc<LoggerService>,
        l10n: Arc<crate::services::localization::LocalizationManager>,
    ) -> Self {
        Self { db, logger, l10n }
    }

    /// Starts the background task that checks for expired temporary bans and unbans them.
    pub fn start_unban_runner(&self, http: Arc<serenity::Http>) {
        let db = self.db.clone();
        let logger = self.logger.clone();
        let l10n_service = self.l10n.clone();
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
                            } else {
                                // Log successful unban
                                let guild = guild_id.to_partial_guild(&http).await;
                                let locale = guild
                                    .map(|g| g.preferred_locale.to_string())
                                    .unwrap_or_else(|_| "en-US".to_string());
                                let l10n = l10n_service.get_proxy(&locale);

                                let mut user_args = FluentArgs::new();
                                user_args.set("userId", user_id.get());

                                let _ = logger
                                    .log_action(
                                        &http,
                                        guild_id,
                                        Some(ModuleType::ModerationProtection),
                                        LogLevel::Audit,
                                        &l10n.t("log-mod-unban-title", None),
                                        &l10n.t("log-mod-unban-desc", Some(&user_args)),
                                        vec![
                                            (
                                                &l10n.t("log-field-user", None),
                                                format!("<@{}>", user_id),
                                            ),
                                            (
                                                &l10n.t("log-field-reason", None),
                                                l10n.t("log-val-temp-ban-expired", None),
                                            ),
                                        ],
                                    )
                                    .await;
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

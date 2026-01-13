use crate::db::entities::logging_guilds;
use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use std::sync::Arc;
use tokio::time::sleep;
use tracing::{error, info};

/// Service to clean up stale logging guild records.
/// Guilds that haven't accessed logging features in 1 month will have
/// their member_old_roles data deleted (via cascade) to save storage.
pub struct LoggingCleanupService {
    db: DatabaseConnection,
}

impl LoggingCleanupService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Starts the background cleanup task.
    /// Runs every hour and deletes logging_guilds that haven't been accessed in 30 days.
    /// Due to the foreign key cascade, this also deletes related member_old_roles records.
    pub fn start_cleanup_runner(self: Arc<Self>) {
        let db = self.db.clone();
        tokio::spawn(async move {
            info!("Logging cleanup runner started.");
            loop {
                // Sleep for 1 hour between cleanup cycles
                sleep(std::time::Duration::from_secs(3600)).await;

                let cutoff = Utc::now() - Duration::days(30);
                info!(
                    "Running logging cleanup for guilds not accessed since: {}",
                    cutoff
                );

                match logging_guilds::Entity::delete_many()
                    .filter(logging_guilds::Column::LastAccessedAt.lt(cutoff))
                    .exec(&db)
                    .await
                {
                    Ok(result) => {
                        if result.rows_affected > 0 {
                            info!(
                                "Cleaned up {} stale logging guild(s) and their member role data",
                                result.rows_affected
                            );
                        }
                    }
                    Err(e) => {
                        error!("Failed to clean up stale logging guilds: {:?}", e);
                    }
                }
            }
        });
    }
}

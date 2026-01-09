use sea_orm::{Database, DatabaseConnection};
use std::time::Duration;
use tracing::info;

pub mod entities;
pub mod migrations;

pub async fn establish_connection() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    
    let mut opt = sea_orm::ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Info);

    info!("Connecting to database...");
    let db = Database::connect(opt).await?;
    info!("Database connection established");

    Ok(db)
}

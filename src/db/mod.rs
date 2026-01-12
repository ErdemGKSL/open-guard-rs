use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};
use std::time::Duration;
use tracing::{error, info, warn};

pub mod entities;
pub mod migrations;

pub async fn establish_connection() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    ensure_database_exists(&database_url)
        .await
        .map_err(|e| sea_orm::DbErr::Custom(format!("Failed to ensure database exists: {}", e)))?;

    let mut opt = sea_orm::ConnectOptions::new(database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(tracing::log::LevelFilter::Debug);

    info!("Connecting to database...");
    let db = Database::connect(opt).await?;
    info!("Database connection established");

    Ok(db)
}

pub async fn ensure_database_exists(database_url: &str) -> anyhow::Result<()> {
    if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
        return Ok(());
    }

    let (base_url, db_name) = match database_url.rsplit_once('/') {
        Some((base, name)) if !name.is_empty() => (base, name),
        _ => return Ok(()), // Could not parse DB name, let connection fail naturally later
    };

    // Remove query parameters if any, but save them for later
    let (db_name, query_params) = if let Some((name, params)) = db_name.split_once('?') {
        (name, Some(params))
    } else {
        (db_name, None)
    };

    // Connect to 'postgres' database to create the target database
    // Preserve query parameters (like ?host=/var/run/postgresql for Unix sockets)
    let root_url = if let Some(params) = query_params {
        format!("{}/postgres?{}", base_url, params)
    } else {
        format!("{}/postgres", base_url)
    };

    info!("Ensuring database '{}' exists...", db_name);

    let db = match Database::connect(&root_url).await {
        Ok(db) => db,
        Err(e) => {
            warn!(
                "Could not connect to 'postgres' database to check for existence of '{}': {}. Attempting to proceed...",
                db_name, e
            );
            return Ok(());
        }
    };

    let backend = db.get_database_backend();

    // Check if database exists
    let check_sql = format!("SELECT 1 FROM pg_database WHERE datname = '{}'", db_name);
    let results = db
        .query_one(Statement::from_string(backend, check_sql))
        .await?;

    if results.is_none() {
        info!("Database '{}' does not exist. Creating...", db_name);
        let create_sql = format!("CREATE DATABASE \"{}\"", db_name);
        match db
            .execute(Statement::from_string(backend, create_sql))
            .await
        {
            Ok(_) => info!("Database '{}' created successfully", db_name),
            Err(e) => {
                error!("Failed to create database '{}': {}", db_name, e);
                return Err(e.into());
            }
        }
    } else {
        info!("Database '{}' already exists", db_name);
    }

    Ok(())
}


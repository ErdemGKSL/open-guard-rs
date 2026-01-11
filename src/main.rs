use anyhow::Context as _;
use clap::Parser as _;
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tracing::{error, info};

mod db;
mod modules;
mod services;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Publish commands. If no guild ID is provided, publish globally.
    #[arg(long, num_args = 0..=1)]
    publish: Option<Vec<u64>>,
}

// Custom user data passed to all command functions
#[derive(Clone)]
pub struct Data {
    pub db: DatabaseConnection,
    pub l10n: Arc<services::localization::LocalizationManager>,
    pub logger: Arc<services::logger::LoggerService>,
    pub punishment: Arc<services::punishment::PunishmentService>,
    pub whitelist: Arc<services::whitelist::WhitelistService>,
    pub cache: Arc<services::cache::ObjectCacheService>,
    pub module_definitions: Vec<modules::ModuleDefinition>,
}

pub type Error = anyhow::Error;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Discord Guard Bot...");

    // Establish database connection
    let db = db::establish_connection()
        .await
        .context("Failed to connect to database")?;

    // Run migrations
    use sea_orm_migration::MigratorTrait;
    db::migrations::Migrator::up(&db, None)
        .await
        .context("Failed to run migrations")?;

    let token = serenity::Token::from_env("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::AUTO_MODERATION_CONFIGURATION
        | serenity::GatewayIntents::GUILD_MODERATION
        | serenity::GatewayIntents::AUTO_MODERATION_EXECUTION;

    // Initialize localization manager
    let l10n = Arc::new(services::localization::LocalizationManager::new());

    // Initialize logger service
    let logger = Arc::new(services::logger::LoggerService::new(db.clone()));

    // Initialize punishment service
    let punishment = Arc::new(services::punishment::PunishmentService::new(db.clone()));

    // Initialize whitelist service
    let whitelist = Arc::new(services::whitelist::WhitelistService::new(db.clone()));

    // Initialize object cache service
    let cache = Arc::new(services::cache::ObjectCacheService::new());

    // Load and translate commands
    let mut commands = modules::commands();
    l10n.apply_translations(&mut commands);

    let framework_options = poise::FrameworkOptions {
        commands,
        ..Default::default()
    };

    // Handle command registration if requested
    if let Some(publish_args) = args.publish {
        let http = serenity::HttpBuilder::new(token.clone())
            .application_id(serenity::ApplicationId::new(
                std::env::var("APPLICATION_ID")
                    .expect("missing APPLICATION_ID")
                    .parse()
                    .expect("invalid APPLICATION_ID"),
            ))
            .build();
        let commands = &framework_options.commands;

        if publish_args.is_empty() {
            info!("Registering commands globally...");
            if let Err(e) = poise::builtins::register_globally(&http, commands).await {
                error!("Failed to register commands globally: {}", e);
            } else {
                info!("Commands registered globally");
            }
        } else {
            for guild_id in publish_args {
                info!("Registering commands in guild {}...", guild_id);
                if let Err(e) = poise::builtins::register_in_guild(
                    &http,
                    commands,
                    serenity::GuildId::new(guild_id),
                )
                .await
                {
                    error!("Failed to register commands in guild {}: {}", guild_id, e);
                } else {
                    info!("Commands registered in guild {}", guild_id);
                }
            }
        }
        std::process::exit(0);
    }

    // Create the poise framework
    let framework = poise::Framework::new(framework_options);

    // Build the client with both poise framework and custom event handler
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(Box::new(framework))
        .event_handler(Arc::new(services::event_manager::Handler))
        .data(Arc::new(Data {
            db,
            l10n,
            logger,
            punishment,
            whitelist,
            cache,
            module_definitions: modules::definitions(),
        }) as _)
        .await
        .context("Failed to create client")?;

    info!("Bot is ready!");
    client.start_autosharded().await.context("Client error")?;

    Ok(())
}

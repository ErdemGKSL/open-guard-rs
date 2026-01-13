use anyhow::Context as _;
use clap::Parser as _;
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tracing::{error, info};

mod db;
mod modules;
mod services;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Publish commands. If no guild ID is provided, publish globally.
    #[arg(long, num_args = 0..)]
    publish: Option<Vec<u64>>,

    /// Clear all commands instead of publishing them.
    #[arg(long)]
    clear: bool,

    /// Rollback the specified number of migrations and run all migrations again.
    #[arg(long, num_args = 0..=1, default_missing_value = "1")]
    refresh_migrations: Option<u32>,
}

// Custom user data passed to all command functions
pub struct Data {
    pub db: DatabaseConnection,
    pub l10n: Arc<services::localization::LocalizationManager>,
    pub logger: Arc<services::logger::LoggerService>,
    pub punishment: Arc<services::punishment::PunishmentService>,
    pub whitelist: Arc<services::whitelist::WhitelistService>,
    pub cache: Arc<services::cache::ObjectCacheService>,
    pub module_definitions: Vec<modules::ModuleDefinition>,
    pub temp_ban: Arc<services::temp_ban::TempBanService>,
    pub jail: Arc<services::jail::JailService>,
    pub shard_count: AtomicU32,
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
    if let Some(depth) = args.refresh_migrations {
        info!("Refreshing migrations (down {}, then up)...", depth);
        db::migrations::Migrator::down(&db, Some(depth))
            .await
            .context("Failed to rollback migration")?;
    }

    db::migrations::Migrator::up(&db, None)
        .await
        .context("Failed to run migrations")?;

    if args.refresh_migrations.is_some() {
        info!("Migrations refreshed successfully.");
        std::process::exit(0);
    }

    let token = serenity::Token::from_env("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::AUTO_MODERATION_CONFIGURATION
        | serenity::GatewayIntents::GUILD_MODERATION
        | serenity::GatewayIntents::AUTO_MODERATION_EXECUTION
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILD_VOICE_STATES;

    // Initialize localization manager
    let l10n = Arc::new(services::localization::LocalizationManager::new());

    // Initialize logger service
    let logger = Arc::new(services::logger::LoggerService::new(db.clone()));

    // Initialize whitelist service
    let whitelist = Arc::new(services::whitelist::WhitelistService::new(db.clone()));

    // Initialize object cache service
    let cache = Arc::new(services::cache::ObjectCacheService::new());

    // Initialize temp ban service
    let temp_ban = Arc::new(services::temp_ban::TempBanService::new(
        db.clone(),
        logger.clone(),
        l10n.clone(),
    ));

    // Initialize jail service
    let jail = Arc::new(services::jail::JailService::new(
        db.clone(),
        logger.clone(),
        l10n.clone(),
    ));

    // Initialize punishment service
    let mut punishment_svc =
        services::punishment::PunishmentService::new(db.clone(), logger.clone(), l10n.clone());
    punishment_svc.set_jail_service(jail.clone());
    let punishment = Arc::new(punishment_svc);

    // Load and translate commands
    let mut commands = modules::commands();
    l10n.apply_translations(&mut commands);

    let framework_options = poise::FrameworkOptions {
        commands,
        ..Default::default()
    };

    // Handle command registration if requested
    if let Some(publish_args) = args.publish {
        let http = serenity::HttpBuilder::new(token.clone()).build();
        let bot_user = http
            .get_current_user()
            .await
            .context("Failed to fetch bot user info")?;
        let application_id = bot_user.id;

        info!("Fetched Application ID: {}", application_id);

        let http = serenity::HttpBuilder::new(token.clone())
            .application_id(serenity::ApplicationId::new(application_id.get()))
            .build();

        let empty_commands = vec![];
        let commands = if args.clear {
            &empty_commands
        } else {
            &framework_options.commands
        };

        if publish_args.is_empty() {
            if args.clear {
                info!("Clearing commands globally...");
            } else {
                info!("Registering commands globally...");
            }

            if let Err(e) = poise::builtins::register_globally(&http, commands).await {
                error!("Failed to register commands globally: {}", e);
            } else {
                info!("Global command operation successful");
            }
        } else {
            for guild_id in publish_args {
                if args.clear {
                    info!("Clearing commands in guild {}...", guild_id);
                } else {
                    info!("Registering commands in guild {}...", guild_id);
                }

                if let Err(e) = poise::builtins::register_in_guild(
                    &http,
                    commands,
                    serenity::GuildId::new(guild_id),
                )
                .await
                {
                    error!("Failed to register commands in guild {}: {}", guild_id, e);
                } else {
                    info!("Guild command operation successful for guild {}", guild_id);
                }
            }
        }
        std::process::exit(0);
    }

    let http = serenity::HttpBuilder::new(token.clone()).build();
    let initial_shard_count = http
        .get_bot_gateway()
        .await
        .context("Failed to get bot gateway info")?
        .shards
        .get() as u32;

    let shard_count = Arc::new(AtomicU32::new(initial_shard_count));

    // Spawn background task to refresh shard count every 2 minutes
    {
        let shard_count = shard_count.clone();
        let token = token.clone();
        tokio::spawn(async move {
            let http = serenity::HttpBuilder::new(token).build();
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;
                match http.get_bot_gateway().await {
                    Ok(gateway) => {
                        let new_count = gateway.shards.get() as u32;
                        shard_count.store(new_count, Ordering::Relaxed);
                        info!("Shard count refreshed: {}", new_count);
                    }
                    Err(e) => {
                        error!("Failed to refresh shard count: {:?}", e);
                    }
                }
            }
        });
    }

    // Create the poise framework
    let framework = poise::Framework::new(framework_options);

    let mut cache_settings = serenity::cache::Settings::default();
    cache_settings.max_messages = 2048;
    cache_settings.cache_users = true;
    cache_settings.cache_guilds = true;
    cache_settings.cache_channels = true;

    // Build the client with both poise framework and custom event handler
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(Box::new(framework))
        .event_handler(Arc::new(services::event_manager::Handler))
        .cache_settings(cache_settings)
        .data(Arc::new(Data {
            db: db.clone(),
            l10n,
            logger,
            punishment,
            whitelist,
            cache,
            module_definitions: modules::definitions(),
            temp_ban: temp_ban.clone(),
            jail: jail.clone(),
            shard_count: AtomicU32::new(shard_count.load(Ordering::Relaxed)),
        }) as _)
        .await
        .context("Failed to create client")?;

    // Start unban runner
    temp_ban.start_unban_runner(client.http.clone());

    // Start unjail runner
    jail.start_unjail_runner(client.http.clone());

    // Start logging cleanup runner
    let logging_cleanup = Arc::new(services::logging_cleanup::LoggingCleanupService::new(db));
    logging_cleanup.start_cleanup_runner();

    info!("Bot is ready!");
    client.start_autosharded().await.context("Client error")?;

    Ok(())
}

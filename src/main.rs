use anyhow::Context as _;
use clap::Parser as _;
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use sea_orm::DatabaseConnection;
use tracing::info;

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
pub struct Data {
    pub db: DatabaseConnection,
    pub prefix_cache: papaya::HashMap<u64, String>,
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

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: modules::commands(),
            event_handler: |ctx, event, framework, data| {
                Box::pin(services::event_manager::event_handler(
                    ctx, event, framework, data,
                ))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("o!".to_string()),
                dynamic_prefix: Some(|ctx| Box::pin(services::prefix::dynamic_prefix(ctx))),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                if let Some(publish_args) = args.publish {
                    if publish_args.is_empty() {
                        info!("Registering commands globally...");
                        poise::builtins::register_globally(ctx, &framework.options().commands)
                            .await?;
                    } else {
                        for guild_id in publish_args {
                            info!("Registering commands in guild {}...", guild_id);
                            poise::builtins::register_in_guild(
                                ctx,
                                &framework.options().commands,
                                serenity::GuildId::new(guild_id),
                            )
                            .await?;
                        }
                    }
                }

                Ok(Data {
                    db,
                    prefix_cache: papaya::HashMap::new(),
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("Failed to create client")?;

    info!("Bot is ready!");
    client.start_autosharded().await.context("Client error")?;

    Ok(())
}

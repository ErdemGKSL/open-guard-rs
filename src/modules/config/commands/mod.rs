use crate::db::entities::guild_configs;
use crate::db::entities::module_configs::{self, ModuleType};
use crate::{Context, Error, services::logger::LogLevel};
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

/// Configuration commands
#[poise::command(
    slash_command,
    subcommands("set_log_channel", "set_module_log_channel"),
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn config(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the general channel where logs will be sent
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn set_log_channel(
    ctx: Context<'_>,
    #[description = "The channel to send logs to"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().unwrap();

    let mut config: guild_configs::ActiveModel =
        match guild_configs::Entity::find_by_id(guild_id.get() as i64)
            .one(db)
            .await?
        {
            Some(m) => m.into(),
            None => {
                let m = guild_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    ..Default::default()
                };
                m.insert(db).await?.into()
            }
        };

    config.log_channel_id = Set(Some(channel.id.get() as i64));
    config.update(db).await?;

    ctx.say(format!(
        "General log channel has been set to <#{}>",
        channel.id
    ))
    .await?;

    // Log the configuration change
    ctx.data()
        .logger
        .log_context(
            &ctx,
            Some(ModuleType::Config),
            LogLevel::Audit,
            "General Log Channel Updated",
            &format!(
                "The general logging channel has been changed to <#{}>",
                channel.id
            ),
            vec![("New Channel", format!("<#{}>", channel.id))],
        )
        .await?;

    Ok(())
}

/// Set a module-specific channel where logs will be sent
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn set_module_log_channel(
    ctx: Context<'_>,
    #[description = "The module to configure"] module_type: ModuleType,
    #[description = "The channel to send logs to"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().unwrap();

    let mut config: module_configs::ActiveModel =
        match module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
            .one(db)
            .await?
        {
            Some(m) => m.into(),
            None => {
                let m = module_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    module_type: Set(module_type),
                    config: Set(serde_json::json!({})),
                    ..Default::default()
                };
                m.insert(db).await?.into()
            }
        };

    config.log_channel_id = Set(Some(channel.id.get() as i64));
    config.update(db).await?;

    ctx.say(format!(
        "Log channel for module `{:?}` has been set to <#{}>",
        module_type, channel.id
    ))
    .await?;

    // Log the configuration change
    ctx.data()
        .logger
        .log_context(
            &ctx,
            Some(ModuleType::Config),
            LogLevel::Audit,
            "Module Log Channel Updated",
            &format!(
                "The logging channel for module `{:?}` has been changed to <#{}>",
                module_type, channel.id
            ),
            vec![
                ("Module", format!("{:?}", module_type)),
                ("New Channel", format!("<#{}>", channel.id)),
            ],
        )
        .await?;

    Ok(())
}

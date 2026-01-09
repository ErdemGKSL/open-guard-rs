use crate::{
    Context, Error,
    db::entities::module_configs::{self, ChannelProtectionModuleConfig, ModuleType},
    services::logger::LogLevel,
};
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

/// Channel protection commands
#[poise::command(
    slash_command,
    subcommands("toggle", "set_audit_log"),
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn channel_protection(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Enable or disable channel creation locking
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn toggle(
    ctx: Context<'_>,
    #[description = "Whether to lock new channels"] enabled: bool,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().unwrap();

    let (mut config_model, config_json): (module_configs::ActiveModel, serde_json::Value) =
        match module_configs::Entity::find_by_id((
            guild_id.get() as i64,
            ModuleType::ChannelProtection,
        ))
        .one(db)
        .await?
        {
            Some(m) => (m.clone().into(), m.config),
            None => {
                let m = module_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    module_type: Set(ModuleType::ChannelProtection),
                    config: Set(serde_json::to_value(
                        ChannelProtectionModuleConfig::default(),
                    )?),
                    ..Default::default()
                };
                let inserted = m.insert(db).await?;
                (inserted.clone().into(), inserted.config)
            }
        };

    let mut config: ChannelProtectionModuleConfig = serde_json::from_value(config_json)?;
    config.lock_new_channels = enabled;

    config_model.config = Set(serde_json::to_value(config)?);
    config_model.update(db).await?;

    let status = if enabled { "enabled" } else { "disabled" };
    ctx.say(format!(
        "Channel creation protection has been **{}**.",
        status
    ))
    .await?;

    ctx.data()
        .logger
        .log_context(
            &ctx,
            Some(ModuleType::ChannelProtection),
            LogLevel::Audit,
            "Channel Protection Toggled",
            &format!("Channel creation locking has been set to `{}`", enabled),
            vec![("Status", enabled.to_string())],
        )
        .await?;

    Ok(())
}

/// Set a specific channel for protection-related audit logs
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild",
    default_member_permissions = "ADMINISTRATOR"
)]
pub async fn set_audit_log(
    ctx: Context<'_>,
    #[description = "The channel to send protection logs to"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    let db = &ctx.data().db;
    let guild_id = ctx.guild_id().unwrap();

    let (mut config_model, config_json): (module_configs::ActiveModel, serde_json::Value) =
        match module_configs::Entity::find_by_id((
            guild_id.get() as i64,
            ModuleType::ChannelProtection,
        ))
        .one(db)
        .await?
        {
            Some(m) => (m.clone().into(), m.config),
            None => {
                let m = module_configs::ActiveModel {
                    guild_id: Set(guild_id.get() as i64),
                    module_type: Set(ModuleType::ChannelProtection),
                    config: Set(serde_json::to_value(
                        ChannelProtectionModuleConfig::default(),
                    )?),
                    ..Default::default()
                };
                let inserted = m.insert(db).await?;
                (inserted.clone().into(), inserted.config)
            }
        };

    let mut config: ChannelProtectionModuleConfig = serde_json::from_value(config_json)?;
    config.audit_log_channel_id = Some(channel.id.get());

    config_model.config = Set(serde_json::to_value(config)?);
    config_model.log_channel_id = Set(Some(channel.id.get() as i64));
    config_model.update(db).await?;

    ctx.say(format!(
        "Channel protection audit logs will now be sent to <#{}>.",
        channel.id
    ))
    .await?;

    ctx.data()
        .logger
        .log_context(
            &ctx,
            Some(ModuleType::ChannelProtection),
            LogLevel::Audit,
            "Channel Protection Audit Log Set",
            &format!(
                "Audit log channel for channel protection has been set to <#{}>",
                channel.id
            ),
            vec![("Channel", format!("<#{}>", channel.id))],
        )
        .await?;

    Ok(())
}

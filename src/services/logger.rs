use crate::Error;
use crate::db::entities::{
    guild_configs,
    module_configs::{self, ModuleType},
};
use poise::serenity_prelude as serenity;
use sea_orm::{DatabaseConnection, EntityTrait};

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Audit,
}

impl LogLevel {
    pub fn icon(&self) -> &'static str {
        match self {
            LogLevel::Info => "ℹ️",
            LogLevel::Warn => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Audit => "📝",
        }
    }

    pub fn color(&self) -> u32 {
        match self {
            LogLevel::Info => 0x3498db, // Blue
            LogLevel::Warn => 0xf1c40f, // Yellow
            LogLevel::Error => 0xe74c3c, // Red
            LogLevel::Audit => 0x95a5a6, // Gray
        }
    }
}

pub struct LoggerService {
    db: DatabaseConnection,
}

impl LoggerService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Logs a structured message to the configured log channel for a guild or module.
    /// If no log channel is configured for the module, it falls back to the general guild log channel.
    /// If neither is configured, it does nothing.
    pub async fn log_action(
        &self,
        http: &serenity::Http,
        guild_id: serenity::GuildId,
        module: Option<ModuleType>,
        level: LogLevel,
        title: &str,
        description: &str,
        fields: Vec<(&str, String)>,
    ) -> Result<(), Error> {
        let mut target_channel_id = None;

        // 1. Try module-specific channel
        if let Some(module_type) = module {
            let m_config = module_configs::Entity::find_by_id((guild_id.get() as i64, module_type))
                .one(&self.db)
                .await?;

            target_channel_id = m_config.and_then(|c| c.log_channel_id);
        }

        // 2. Fallback to general guild channel if not found
        if target_channel_id.is_none() {
            let g_config = guild_configs::Entity::find_by_id(guild_id.get() as i64)
                .one(&self.db)
                .await?;

            target_channel_id = g_config.and_then(|c| c.log_channel_id);
        }

        let channel_id = match target_channel_id {
            Some(id) => serenity::ChannelId::new(id as u64),
            None => return Ok(()), // No log channel configured
        };

        let mut inner_components = vec![];

        // Header component
        inner_components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(format!("{} **{}**", level.icon(), title)),
        ));

        // Separator
        inner_components.push(serenity::CreateContainerComponent::Separator(
            serenity::CreateSeparator::new(false),
        ));

        // Description component
        inner_components.push(serenity::CreateContainerComponent::TextDisplay(
            serenity::CreateTextDisplay::new(description),
        ));

        // Optional fields
        if !fields.is_empty() {
            inner_components.push(serenity::CreateContainerComponent::Separator(
                serenity::CreateSeparator::new(true),
            ));

            for (name, value) in fields {
                inner_components.push(serenity::CreateContainerComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(format!("**{}**: {}", name, value)),
                ));
            }
        }

        http.send_message(
            channel_id.into(),
            Vec::new(),
            &serenity::CreateMessage::new()
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
                .components(vec![serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(inner_components).accent_color(level.color()),
                )]),
        )
        .await?;

        Ok(())
    }

    /// Helper to log an event from a command context
    pub async fn log_context<U, E>(
        &self,
        ctx: &poise::Context<'_, U, E>,
        module: Option<ModuleType>,
        level: LogLevel,
        title: &str,
        description: &str,
        additional_fields: Vec<(&str, String)>,
    ) -> Result<(), Error>
    where
        U: Send + Sync + 'static,
        E: 'static,
    {
        let guild_id = ctx
            .guild_id()
            .ok_or_else(|| anyhow::anyhow!("Audit logs are only available in guilds"))?;

        let mut fields = vec![
            ("User", format!("<@{}>", ctx.author().id)),
            ("Channel", format!("<#{}>", ctx.channel_id())),
        ];

        fields.extend(additional_fields);

        self.log_action(
            ctx.serenity_context().http.as_ref(),
            guild_id,
            module,
            level,
            title,
            description,
            fields,
        )
        .await
    }
}

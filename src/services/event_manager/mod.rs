use crate::Data;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tracing::{error, info};

pub mod shared_events;

/// Custom event handler for non-command Discord events
pub struct Handler {
    modules: Vec<crate::modules::Module>,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            modules: crate::modules::get_modules(),
        }
    }
}

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn dispatch(&self, ctx: &serenity::Context, event: &serenity::FullEvent) {
        // 1. Core / Service handling
        match event {
            serenity::FullEvent::Ready { data_about_bot, .. } => {
                info!("Logged in as {}", data_about_bot.user.name);
            }
            serenity::FullEvent::GuildCreate { guild, is_new, .. } => {
                if is_new.unwrap_or(false) {
                    info!("Joined new guild: {} ({})", guild.name, guild.id);
                }
            }
            serenity::FullEvent::GuildDelete { incomplete, .. } => {
                info!("Left guild: {}", incomplete.id);
            }
            serenity::FullEvent::ChannelDelete { channel, .. } => {
                let data = ctx.data::<Data>();
                data.cache
                    .store_channel(channel.base.guild_id, channel.clone());
            }
            serenity::FullEvent::GuildRoleDelete {
                guild_id,
                removed_role_data_if_available,
                ..
            } => {
                if let Some(role) = removed_role_data_if_available {
                    let data = ctx.data::<Data>();
                    data.cache.store_role(*guild_id, role.clone());
                }
            }
            // Shared Logic Dispatch
            serenity::FullEvent::GuildMemberAddition { new_member, .. } => {
                let data = ctx.data::<Data>();
                if let Err(e) = shared_events::role_cache::handle_guild_member_add(
                    ctx,
                    new_member.guild_id,
                    new_member.clone(),
                    &data,
                )
                .await
                {
                    error!("Error handling shared guild member add: {:?}", e);
                }
            }
            serenity::FullEvent::InteractionCreate { interaction, .. } => {
                handle_interactions(ctx, interaction).await;
            }
            _ => {}
        }

        // 2. Systematic Module Dispatch
        // We clone the event once to put it in an Arc, allowing multiple spawned tasks to access it cheaply.
        let event_arc = Arc::new(event.clone());
        let data = ctx.data::<Data>();

        for module in &self.modules {
            for handler in &module.event_handlers {
                let ctx = ctx.clone();
                let event_arc = event_arc.clone();
                let data = data.clone();
                let handler = *handler;
                let module_id = module.definition.id;

                tokio::spawn(async move {
                    if let Err(e) = handler(&ctx, &event_arc, &data).await {
                        error!("Error in event handler for module {}: {:?}", module_id, e);
                    }
                });
            }
        }
    }
}

async fn handle_interactions(ctx: &serenity::Context, interaction: &serenity::Interaction) {
    if let serenity::Interaction::Component(component_interaction) = interaction {
        let data = ctx.data::<Data>().clone();
        let ctx = ctx.clone();
        let component_interaction = component_interaction.clone();

        tokio::spawn(async move {
            let custom_id = &component_interaction.data.custom_id;

            if custom_id.starts_with("config_") {
                if let Err(e) =
                    crate::services::config::handle_interaction(&ctx, &component_interaction, &data)
                        .await
                {
                    error!("Error handling config interaction: {:?}", e);
                }
            } else if custom_id.starts_with("help-") {
                if let Err(e) =
                    crate::services::help::handle_interaction(&ctx, &component_interaction, &data)
                        .await
                {
                    error!("Error handling help interaction: {:?}", e);
                }
            } else if custom_id.starts_with("status-") {
                if let Err(e) =
                    crate::services::status::handle_interaction(&ctx, &component_interaction, &data)
                        .await
                {
                    error!("Error handling status interaction: {:?}", e);
                }
            } else if custom_id.starts_with("whitelist_") {
                if let Err(e) = crate::services::config::whitelist::handle_interaction(
                    &ctx,
                    &component_interaction,
                    &data,
                )
                .await
                {
                    error!("Error handling whitelist interaction: {:?}", e);
                }
            }
        });
    }
}

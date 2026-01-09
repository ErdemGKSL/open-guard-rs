use crate::Data;
use crate::modules::channel_protection;
use poise::serenity_prelude as serenity;
use tracing::{error, info};

/// Custom event handler for non-command Discord events
pub struct Handler;

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn dispatch(&self, ctx: &serenity::Context, event: &serenity::FullEvent) {
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
            serenity::FullEvent::GuildAuditLogEntryCreate {
                entry, guild_id, ..
            } => {
                // Get our data from the context
                let data = ctx.data::<Data>();

                if let Err(e) = channel_protection::events::audit_log::handle_audit_log(
                    ctx, entry, *guild_id, &data,
                )
                .await
                {
                    error!("Error handling audit log for channel protection: {:?}", e);
                }
            }
            _ => {}
        }
    }
}

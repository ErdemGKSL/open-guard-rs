use poise::serenity_prelude as serenity;
use tracing::info;

/// Custom event handler for non-command Discord events
pub struct Handler;

#[serenity::async_trait]
impl serenity::EventHandler for Handler {
    async fn dispatch(&self, _ctx: &serenity::Context, event: &serenity::FullEvent) {
        match event {
            serenity::FullEvent::Ready { data_about_bot, .. } => {
                info!("Logged in as {}", data_about_bot.user.name);
            }
            serenity::FullEvent::GuildCreate { guild, is_new, .. } => {
                if is_new.unwrap_or(false) {
                    info!("Joined new guild: {} ({})", guild.name, guild.id);
                    // You can access Data here if needed:
                    // let data = ctx.data::<Data>();
                }
            }
            serenity::FullEvent::GuildDelete { incomplete, .. } => {
                info!("Left guild: {}", incomplete.id);
            }
            _ => {}
        }
    }
}

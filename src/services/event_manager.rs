use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use tracing::info;

pub async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            info!("Logged in as {}", data_about_bot.user.name);
        }
        _ => {}
    }
    Ok(())
}

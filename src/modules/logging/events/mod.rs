pub mod membership;
pub mod messages;
pub mod voice;

use crate::{Data, Error};
use poise::serenity_prelude as serenity;

pub fn handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a serenity::FullEvent,
    data: &'a Data,
) -> poise::BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        match event {
            serenity::FullEvent::MessageUpdate {
                old_if_available,
                event,
                ..
            } => {
                if let Some(guild_id) = event.message.guild_id {
                    messages::handle_message_edit(
                        ctx,
                        guild_id,
                        old_if_available.clone(),
                        event.clone(),
                        data,
                    )
                    .await?;
                }
            }
            serenity::FullEvent::MessageDelete {
                channel_id,
                deleted_message_id,
                guild_id,
                ..
            } => {
                if let Some(guild_id) = guild_id {
                    messages::handle_message_delete(
                        ctx,
                        *guild_id,
                        serenity::ChannelId::new(channel_id.get()),
                        *deleted_message_id,
                        data,
                    )
                    .await?;
                }
            }
            serenity::FullEvent::VoiceStateUpdate { old, new, .. } => {
                if let Some(guild_id) = new.guild_id {
                    voice::handle_voice_state_update(ctx, guild_id, old.clone(), new.clone(), data)
                        .await?;
                }
            }
            _ => {}
        }
        Ok(())
    })
}

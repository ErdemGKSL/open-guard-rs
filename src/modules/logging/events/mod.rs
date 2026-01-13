pub mod membership;
pub mod messages;
pub mod voice;

use crate::{Data, Error};
use poise::serenity_prelude as serenity;
use tracing::error;

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
            serenity::FullEvent::GuildMemberRemoval {
                guild_id,
                user,
                member_data_if_available,
                ..
            } => {
                membership::handle_guild_member_remove(
                    ctx,
                    *guild_id,
                    user.clone(),
                    member_data_if_available.clone(),
                    data,
                )
                .await?;
            }
            serenity::FullEvent::GuildMemberUpdate {
                old_if_available,
                new,
                event,
                ..
            } => {
                handle_member_update(ctx, old_if_available.as_ref(), new.as_ref(), event, data)
                    .await?;
            }
            _ => {}
        }
        Ok(())
    })
}

async fn handle_member_update(
    _ctx: &serenity::Context,
    old_if_available: Option<&serenity::Member>,
    new: Option<&serenity::Member>,
    event: &serenity::GuildMemberUpdateEvent,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = event.guild_id;
    let user_id = event.user.id;

    // Check if logging module is enabled for membership first
    match membership::get_membership_logging_config(guild_id, data).await {
        Ok(Some(_)) => {}
        Ok(None) => return Ok(()), // Logging is disabled, skip
        Err(e) => {
            error!("Error checking logging config: {:?}", e);
            return Ok(());
        }
    }

    // Determine if we need to store roles and what roles to store
    let roles_to_store: Option<Vec<serenity::RoleId>> = match (old_if_available, new) {
        // Both old and new exist - only store if roles changed
        (Some(old_member), Some(new_member)) => {
            let old_roles: Vec<_> = old_member.roles.iter().collect();
            let new_roles: Vec<_> = new_member.roles.iter().collect();
            if old_roles != new_roles {
                Some(new_member.roles.iter().cloned().collect())
            } else {
                None // Roles didn't change, no need to store
            }
        }
        // use event roles directly if cache miss
        _ => Some(event.roles.iter().cloned().collect()),
    };

    // Store roles if we have them
    if let Some(roles) = roles_to_store {
        membership::store_member_roles(guild_id, user_id, &roles, data).await?;
    }

    Ok(())
}

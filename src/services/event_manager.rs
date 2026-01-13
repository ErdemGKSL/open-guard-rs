use crate::Data;
use crate::modules::{
    channel_permission_protection, channel_protection, logging, member_permission_protection,
    moderation_protection, role_permission_protection, role_protection,
};
use ::serenity::nonmax::NonMaxU16;
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

                info!("Caching members for guild: {} ({})", guild.name, guild.id);
                let ctx = ctx.clone();
                let guild_id = guild.id;
                tokio::spawn(async move {
                    let mut last_id: Option<serenity::UserId> = None;
                    let mut total_members = 0;
                    loop {
                        match guild_id
                            .members(&ctx.http, Some(NonMaxU16::new(1000).unwrap()), last_id)
                            .await
                        {
                            Ok(members) => {
                                let count = members.len();
                                total_members += count;
                                info!(
                                    "Fetched {} members (total: {}) for guild: {}",
                                    count, total_members, guild_id
                                );
                                if count < 1000 {
                                    break;
                                }
                                if let Some(last_member) = members.last() {
                                    last_id = Some(last_member.user.id.clone());
                                }
                            }
                            Err(e) => {
                                error!("Failed to cache members for guild {}: {:?}", guild_id, e);
                                break;
                            }
                        }
                    }
                    info!(
                        "Finished caching members for guild: {} (total: {})",
                        guild_id, total_members
                    );
                });
            }
            serenity::FullEvent::GuildDelete { incomplete, .. } => {
                info!("Left guild: {}", incomplete.id);
            }
            serenity::FullEvent::GuildAuditLogEntryCreate {
                entry, guild_id, ..
            } => {
                let data = ctx.data::<Data>().clone();
                let entry = entry.clone();
                let guild_id = *guild_id;
                let ctx = ctx.clone();

                // Channel Protection
                {
                    let ctx = ctx.clone();
                    let entry = entry.clone();
                    let data = data.clone();
                    tokio::spawn(async move {
                        if let Err(e) = channel_protection::events::audit_log::handle_audit_log(
                            &ctx, &entry, guild_id, &data,
                        )
                        .await
                        {
                            error!("Error handling audit log for channel protection: {:?}", e);
                        }
                    });
                }

                // Channel Permission Protection
                {
                    let ctx = ctx.clone();
                    let entry = entry.clone();
                    let data = data.clone();
                    tokio::spawn(async move {
                        if let Err(e) =
                            channel_permission_protection::events::audit_log::handle_audit_log(
                                &ctx, &entry, guild_id, &data,
                            )
                            .await
                        {
                            error!(
                                "Error handling audit log for channel permission protection: {:?}",
                                e
                            );
                        }
                    });
                }

                // Role Protection
                {
                    let ctx = ctx.clone();
                    let entry = entry.clone();
                    let data = data.clone();
                    tokio::spawn(async move {
                        if let Err(e) = role_protection::events::audit_log::handle_audit_log(
                            &ctx, &entry, guild_id, &data,
                        )
                        .await
                        {
                            error!("Error handling audit log for role protection: {:?}", e);
                        }
                    });
                }

                // Role Permission Protection
                {
                    let ctx = ctx.clone();
                    let entry = entry.clone();
                    let data = data.clone();
                    tokio::spawn(async move {
                        if let Err(e) =
                            role_permission_protection::events::audit_log::handle_audit_log(
                                &ctx, &entry, guild_id, &data,
                            )
                            .await
                        {
                            error!(
                                "Error handling audit log for role permission protection: {:?}",
                                e
                            );
                        }
                    });
                }

                // Member Permission Protection
                {
                    let ctx = ctx.clone();
                    let entry = entry.clone();
                    let data = data.clone();
                    tokio::spawn(async move {
                        if let Err(e) =
                            member_permission_protection::events::audit_log::handle_audit_log(
                                &ctx, &entry, guild_id, &data,
                            )
                            .await
                        {
                            error!(
                                "Error handling audit log for member permission protection: {:?}",
                                e
                            );
                        }
                    });
                }

                // Bot Adding Protection
                {
                    let ctx = ctx.clone();
                    let entry = entry.clone();
                    let data = data.clone();
                    tokio::spawn(async move {
                        if let Err(e) = crate::modules::bot_adding_protection::events::audit_log::handle_audit_log(
                            &ctx, &entry, guild_id, &data,
                        )
                        .await
                        {
                            error!("Error handling audit log for bot adding protection: {:?}", e);
                        }
                    });
                }

                // Moderation Protection
                {
                    let ctx = ctx.clone();
                    let entry = entry.clone();
                    let data = data.clone();
                    tokio::spawn(async move {
                        if let Err(e) = moderation_protection::events::audit_log::handle_audit_log(
                            &ctx, &entry, guild_id, &data,
                        )
                        .await
                        {
                            error!(
                                "Error handling audit log for moderation protection: {:?}",
                                e
                            );
                        }
                    });
                }
            }
            serenity::FullEvent::InteractionCreate { interaction, .. } => {
                if let serenity::Interaction::Component(component_interaction) = interaction {
                    let data = ctx.data::<Data>().clone();
                    let ctx = ctx.clone();
                    let component_interaction = component_interaction.clone();

                    tokio::spawn(async move {
                        let custom_id = &component_interaction.data.custom_id;

                        if custom_id.starts_with("config_") {
                            if let Err(e) = crate::services::config::handle_interaction(
                                &ctx,
                                &component_interaction,
                                &data,
                            )
                            .await
                            {
                                error!("Error handling config interaction: {:?}", e);
                            }
                        } else if custom_id.starts_with("help-") {
                            if let Err(e) = crate::services::help::handle_interaction(
                                &ctx,
                                &component_interaction,
                                &data,
                            )
                            .await
                            {
                                error!("Error handling help interaction: {:?}", e);
                            }
                        } else if custom_id.starts_with("status-") {
                            if let Err(e) = crate::services::status::handle_interaction(
                                &ctx,
                                &component_interaction,
                                &data,
                            )
                            .await
                            {
                                error!("Error handling status interaction: {:?}", e);
                            }
                        }
                    });
                }
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
            serenity::FullEvent::MessageUpdate {
                old_if_available,
                event,
                ..
            } => {
                let data = ctx.data::<Data>().clone();
                let ctx = ctx.clone();
                let old_if_available = old_if_available.clone();
                let event = event.clone();
                let guild_id = event.message.guild_id;

                if let Some(guild_id) = guild_id {
                    tokio::spawn(async move {
                        if let Err(e) = logging::events::messages::handle_message_edit(
                            &ctx,
                            guild_id,
                            old_if_available,
                            event,
                            &data,
                        )
                        .await
                        {
                            error!("Error handling message edit log: {:?}", e);
                        }
                    });
                }
            }
            serenity::FullEvent::MessageDelete {
                channel_id,
                deleted_message_id,
                guild_id,
                ..
            } => {
                let data = ctx.data::<Data>().clone();
                let ctx = ctx.clone();
                let channel_id = serenity::ChannelId::new(channel_id.get());
                let deleted_message_id = *deleted_message_id;

                if let Some(guild_id) = guild_id {
                    let guild_id = *guild_id;
                    tokio::spawn(async move {
                        if let Err(e) = logging::events::messages::handle_message_delete(
                            &ctx,
                            guild_id,
                            channel_id,
                            deleted_message_id,
                            &data,
                        )
                        .await
                        {
                            error!("Error handling message delete log: {:?}", e);
                        }
                    });
                }
            }
            serenity::FullEvent::VoiceStateUpdate { old, new, .. } => {
                let data = ctx.data::<Data>().clone();
                let ctx = ctx.clone();
                let old = old.clone();
                let new = new.clone();

                if let Some(guild_id) = new.guild_id {
                    tokio::spawn(async move {
                        if let Err(e) = logging::events::voice::handle_voice_state_update(
                            &ctx, guild_id, old, new, &data,
                        )
                        .await
                        {
                            error!("Error handling voice state update log: {:?}", e);
                        }
                    });
                }
            }
            serenity::FullEvent::GuildMemberAddition {
                new_member: member, ..
            } => {
                let data = ctx.data::<Data>().clone();
                let ctx = ctx.clone();
                let member = member.clone();
                let guild_id = member.guild_id;

                tokio::spawn(async move {
                    if let Err(e) = logging::events::membership::handle_guild_member_add(
                        &ctx, guild_id, member, &data,
                    )
                    .await
                    {
                        error!("Error handling guild member add log: {:?}", e);
                    }
                });
            }
            serenity::FullEvent::GuildMemberRemoval {
                guild_id,
                user,
                member_data_if_available,
                ..
            } => {
                let data = ctx.data::<Data>().clone();
                let ctx = ctx.clone();
                let guild_id = *guild_id;
                let user = user.clone();
                let member_data_if_available = member_data_if_available.clone();

                tokio::spawn(async move {
                    if let Err(e) = logging::events::membership::handle_guild_member_remove(
                        &ctx,
                        guild_id,
                        user,
                        member_data_if_available,
                        &data,
                    )
                    .await
                    {
                        error!("Error handling guild member remove log: {:?}", e);
                    }
                });
            }
            _ => {}
        }
    }
}

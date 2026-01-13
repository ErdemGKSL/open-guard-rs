use crate::Data;
use crate::modules::{
    channel_permission_protection, channel_protection, logging, member_permission_protection,
    moderation_protection, role_permission_protection, role_protection,
};
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
            serenity::FullEvent::GuildMemberUpdate {
                old_if_available,
                new,
                event,
                ..
            } => {
                let data = ctx.data::<Data>().clone();
                let old_if_available = old_if_available.clone();
                let new = new.clone();
                let event = event.clone();

                // Always get guild_id and user_id from event (it's always available)
                let guild_id = event.guild_id;
                let user_id = event.user.id;

                tokio::spawn(async move {
                    // Check if logging module is enabled for membership first
                    match logging::events::membership::get_membership_logging_config(
                        guild_id, &data,
                    )
                    .await
                    {
                        Ok(Some(_)) => {}
                        Ok(None) => return, // Logging is disabled, skip
                        Err(e) => {
                            error!("Error checking logging config: {:?}", e);
                            return;
                        }
                    }

                    // Determine if we need to store roles and what roles to store
                    let roles_to_store: Option<Vec<serenity::RoleId>> =
                        match (&old_if_available, &new) {
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
                            // use event roles directly
                            _ => Some(event.roles.iter().cloned().collect()),
                        };

                    // Store roles if we have them
                    if let Some(roles) = roles_to_store {
                        if let Err(e) = logging::events::membership::store_member_roles(
                            guild_id, user_id, &roles, &data,
                        )
                        .await
                        {
                            error!("Error storing member roles on update: {:?}", e);
                        }
                    }
                });
            }
            _ => {}
        }
    }
}

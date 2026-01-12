use crate::Data;
use crate::modules::{
    channel_permission_protection, channel_protection, member_permission_protection,
    role_permission_protection, role_protection,
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
            _ => {}
        }
    }
}

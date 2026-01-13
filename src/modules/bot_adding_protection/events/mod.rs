pub mod audit_log;

use crate::{Data, Error};
use poise::serenity_prelude as serenity;

pub fn handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a serenity::FullEvent,
    data: &'a Data,
) -> poise::BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        match event {
            serenity::FullEvent::GuildAuditLogEntryCreate {
                entry, guild_id, ..
            } => {
                audit_log::handle_audit_log(ctx, entry, *guild_id, data).await?;
            }
            _ => {}
        }
        Ok(())
    })
}

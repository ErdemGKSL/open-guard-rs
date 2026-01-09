use crate::db::entities::prelude::GuildConfigs;
use crate::{Data, Error};
use sea_orm::EntityTrait;

pub async fn dynamic_prefix(
    ctx: poise::PartialContext<'_, Data, Error>,
) -> Result<Option<String>, Error> {
    let guild_id = match ctx.guild_id {
        Some(id) => id.get(),
        None => return Ok(Some("o!".to_string())), // Default for DMs
    };

    // 1. Try to read from cache
    {
        let pin = ctx.data.prefix_cache.pin();
        if let Some(prefix) = pin.get(&guild_id) {
            return Ok(Some(prefix.clone()));
        }
    }

    // 2. Not in cache, fetch from DB
    let db = &ctx.data.db;
    let config = GuildConfigs::find_by_id(guild_id as i64).one(db).await?;

    let prefix = config.map(|c| c.prefix).unwrap_or_else(|| "o!".to_string());

    // 3. Update cache
    {
        let pin = ctx.data.prefix_cache.pin();
        pin.insert(guild_id, prefix.clone());
    }

    Ok(Some(prefix))
}

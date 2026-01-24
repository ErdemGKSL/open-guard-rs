use crate::db::entities::invite_stats;
use crate::{Data, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

#[derive(Debug, Clone, Copy)]
pub enum StatsUpdate {
    Join,
    Leave,
    FakeLeave,
}

/// Update stats for an inviter
pub async fn update_inviter_stats(
    guild_id: serenity::GuildId,
    inviter_id: Option<serenity::UserId>,
    update_type: StatsUpdate,
    data: &Data,
) -> Result<(), Error> {
    let Some(user_id) = inviter_id else {
        return Ok(()); // No inviter (vanity/widget/etc)
    };

    let existing = invite_stats::Entity::find()
        .filter(invite_stats::Column::GuildId.eq(guild_id.get() as i64))
        .filter(invite_stats::Column::UserId.eq(user_id.get() as i64))
        .one(&data.db)
        .await?;

    let now = Utc::now();

    match existing {
        Some(stats) => {
            // Update existing stats
            let (total, current, left, fake) = match update_type {
                StatsUpdate::Join => {
                    (stats.total_invites + 1, stats.current_members + 1, stats.left_members, stats.fake_members)
                }
                StatsUpdate::Leave => {
                    (stats.total_invites, (stats.current_members - 1).max(0), stats.left_members + 1, stats.fake_members)
                }
                StatsUpdate::FakeLeave => {
                    (stats.total_invites, (stats.current_members - 1).max(0), stats.left_members, stats.fake_members + 1)
                }
            };
            
            let mut active: invite_stats::ActiveModel = stats.into();
            active.total_invites = Set(total);
            active.current_members = Set(current);
            active.left_members = Set(left);
            active.fake_members = Set(fake);
            active.updated_at = Set(now.into());
            
            active.update(&data.db).await?;
        }
        None => {
            // Only create new stats entry for Join events
            // If we get Leave/FakeLeave without existing stats, the user wasn't tracked
            // so we shouldn't create a stats entry
            match update_type {
                StatsUpdate::Join => {
                    let new_stats = invite_stats::ActiveModel {
                        guild_id: Set(guild_id.get() as i64),
                        user_id: Set(user_id.get() as i64),
                        total_invites: Set(1),
                        current_members: Set(1),
                        left_members: Set(0),
                        fake_members: Set(0),
                        updated_at: Set(now.into()),
                    };

                    new_stats.insert(&data.db).await?;
                }
                StatsUpdate::Leave | StatsUpdate::FakeLeave => {
                    // User left but was never tracked joining, skip stats update
                    tracing::debug!(
                        "Skipping stats update for untracked member leave: guild={}, inviter={:?}",
                        guild_id,
                        user_id
                    );
                }
            }
        }
    }

    Ok(())
}

/// Get stats for a user
pub async fn get_user_stats(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    data: &Data,
) -> Result<Option<invite_stats::Model>, Error> {
    let stats = invite_stats::Entity::find()
        .filter(invite_stats::Column::GuildId.eq(guild_id.get() as i64))
        .filter(invite_stats::Column::UserId.eq(user_id.get() as i64))
        .one(&data.db)
        .await?;

    Ok(stats)
}

/// Get top inviters for a guild
pub async fn get_top_inviters(
    guild_id: serenity::GuildId,
    limit: u32,
    data: &Data,
) -> Result<Vec<invite_stats::Model>, Error> {
    use sea_orm::{QueryOrder, QuerySelect};

    let stats = invite_stats::Entity::find()
        .filter(invite_stats::Column::GuildId.eq(guild_id.get() as i64))
        .order_by_desc(invite_stats::Column::CurrentMembers)
        .limit(limit as u64)
        .all(&data.db)
        .await?;

    Ok(stats)
}

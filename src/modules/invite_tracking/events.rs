use crate::db::entities::invite_events;
use crate::modules::invite_tracking::{stats, tracking};
use crate::{Data, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;
use sea_orm::{ActiveModelTrait, Set};

pub fn handler<'a>(
    ctx: &'a serenity::Context,
    event: &'a serenity::FullEvent,
    data: &'a Data,
) -> poise::BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        handle_event(ctx, event, data).await
    })
}

async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::InviteCreate { data: invite_data, .. } => {
            handle_invite_create(ctx, invite_data, data).await?;
        }
        serenity::FullEvent::InviteDelete { data: invite_data, .. } => {
            handle_invite_delete(ctx, invite_data, data).await?;
        }
        serenity::FullEvent::GuildMemberAddition { new_member, .. } => {
            handle_member_join(ctx, new_member, data).await?;
        }
        serenity::FullEvent::GuildMemberRemoval { guild_id, user, .. } => {
            handle_member_leave(ctx, *guild_id, user, data).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_invite_create(
    ctx: &serenity::Context,
    invite_event: &serenity::InviteCreateEvent,
    data: &Data,
) -> Result<(), Error> {
    // guild_id is Option<GuildId>, handle None case
    let guild_id = match invite_event.guild_id {
        Some(id) => id,
        None => {
            tracing::warn!("Invite created without guild_id");
            return Ok(());
        }
    };

    // Check if module is enabled
    if tracking::get_config(guild_id, data).await?.is_none() {
        return Ok(());
    }

    tracing::info!("Invite created: {} in guild {}", invite_event.code, guild_id);

    // Fetch full invite data from Discord
    let invites = match tracking::fetch_guild_invites(ctx, guild_id).await {
        Ok(inv) => inv,
        Err(e) => {
            tracing::error!("Failed to fetch invites after create: {:?}", e);
            return Ok(());
        }
    };

    // Sync invites to snapshots
    tracking::sync_invites_to_snapshots(guild_id, invites, data).await?;

    // Log event
    log_invite_event(
        guild_id,
        "invite_create",
        Some(&invite_event.code),
        invite_event.inviter.as_ref().map(|u| u.id),
        None,
        None,
        None,
        data,
    )
    .await?;

    Ok(())
}

async fn handle_invite_delete(
    _ctx: &serenity::Context,
    invite_event: &serenity::InviteDeleteEvent,
    data: &Data,
) -> Result<(), Error> {
    // guild_id is Option<GuildId>, handle None case
    let guild_id = match invite_event.guild_id {
        Some(id) => id,
        None => {
            tracing::warn!("Invite deleted without guild_id");
            return Ok(());
        }
    };

    // Check if module is enabled
    if tracking::get_config(guild_id, data).await?.is_none() {
        return Ok(());
    }

    tracing::info!("Invite deleted: {} in guild {}", invite_event.code, guild_id);

    // Remove from snapshots
    tracking::delete_invite_snapshot(guild_id, &invite_event.code, data).await?;

    // Log event
    log_invite_event(
        guild_id,
        "invite_delete",
        Some(&invite_event.code),
        None,
        None,
        None,
        None,
        data,
    )
    .await?;

    Ok(())
}

async fn handle_member_join(
    ctx: &serenity::Context,
    member: &serenity::Member,
    data: &Data,
) -> Result<(), Error> {
    let guild_id = member.guild_id;

    // Check if module is enabled
    let config = match tracking::get_config(guild_id, data).await? {
        Some(c) => c,
        None => return Ok(()),
    };

    // Ignore bots if configured
    if config.ignore_bots && member.user.bot() {
        return Ok(());
    }

    tracing::info!("Member joined: {} in guild {}", member.user.id, guild_id);

    // Fetch current invites
    let current_invites = match tracking::fetch_guild_invites(ctx, guild_id).await {
        Ok(invites) => invites,
        Err(e) => {
            tracing::error!("Failed to fetch invites: {:?}", e);
            return Ok(());
        }
    };

    // Get snapshots from database
    let snapshots = tracking::get_snapshots(guild_id, data).await?;

    // Find which invite was used
    let (inviter_id, join_type, invite_code) =
        if let Some(used_invite) = tracking::find_used_invite(&current_invites, &snapshots) {
            (
                used_invite.inviter_id,
                used_invite.invite_type,
                Some(used_invite.code),
            )
        } else {
            // Could be vanity, widget, or discovery
            let (inviter, join_type, code) =
                tracking::determine_special_join_type(guild_id, ctx).await?;
            
            // If vanity URL and not tracking vanity, skip
            if join_type == "vanity" && !config.track_vanity {
                tracing::debug!("Skipping vanity join for guild {}", guild_id);
                return Ok(());
            }
            
            (inviter, join_type, code)
        };

    // Log event
    log_invite_event(
        guild_id,
        "member_join",
        invite_code.as_deref(),
        inviter_id,
        Some(member.user.id),
        Some(&join_type),
        None,
        data,
    )
    .await?;

    // Update stats
    stats::update_inviter_stats(guild_id, inviter_id, stats::StatsUpdate::Join, data).await?;

    // Sync invites again to update uses
    tracking::sync_invites_to_snapshots(guild_id, current_invites, data).await?;

    Ok(())
}

async fn handle_member_leave(
    _ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    user: &serenity::User,
    data: &Data,
) -> Result<(), Error> {
    // Check if module is enabled
    let config = match tracking::get_config(guild_id, data).await? {
        Some(c) => c,
        None => return Ok(()),
    };

    // Ignore bots if configured
    if config.ignore_bots && user.bot() {
        return Ok(());
    }

    tracing::info!("Member left: {} from guild {}", user.id, guild_id);

    // Find who invited this user from join events
    let inviter_id = find_inviter_from_events(guild_id, user.id, data).await?;

    // Determine if this is a fake leave (joined recently)
    let is_fake = is_fake_member(guild_id, user.id, config.fake_threshold_hours, data).await?;

    // Log event
    log_invite_event(
        guild_id,
        "member_leave",
        None,
        inviter_id,
        Some(user.id),
        None,
        Some(serde_json::json!({"is_fake": is_fake})),
        data,
    )
    .await?;

    // Update stats
    let update_type = if is_fake {
        stats::StatsUpdate::FakeLeave
    } else {
        stats::StatsUpdate::Leave
    };
    stats::update_inviter_stats(guild_id, inviter_id, update_type, data).await?;

    Ok(())
}

/// Log an invite event to the database
async fn log_invite_event(
    guild_id: serenity::GuildId,
    event_type: &str,
    invite_code: Option<&str>,
    inviter_id: Option<serenity::UserId>,
    target_user_id: Option<serenity::UserId>,
    join_type: Option<&str>,
    metadata: Option<serde_json::Value>,
    data: &Data,
) -> Result<(), Error> {
    let event = invite_events::ActiveModel {
        guild_id: Set(guild_id.get() as i64),
        event_type: Set(event_type.to_string()),
        invite_code: Set(invite_code.map(|s| s.to_string())),
        inviter_id: Set(inviter_id.map(|id| id.get() as i64)),
        target_user_id: Set(target_user_id.map(|id| id.get() as i64)),
        join_type: Set(join_type.map(|s| s.to_string())),
        metadata: Set(metadata),
        ..Default::default()
    };

    event.insert(&data.db).await?;
    Ok(())
}

/// Find who invited a user by looking at join events
async fn find_inviter_from_events(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    data: &Data,
) -> Result<Option<serenity::UserId>, Error> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    let event = invite_events::Entity::find()
        .filter(invite_events::Column::GuildId.eq(guild_id.get() as i64))
        .filter(invite_events::Column::TargetUserId.eq(user_id.get() as i64))
        .filter(invite_events::Column::EventType.eq("member_join"))
        .order_by_desc(invite_events::Column::CreatedAt)
        .one(&data.db)
        .await?;

    Ok(event.and_then(|e| e.inviter_id).map(|id| serenity::UserId::new(id as u64)))
}

/// Check if a member is considered "fake" (joined and left quickly)
async fn is_fake_member(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    threshold_hours: u32,
    data: &Data,
) -> Result<bool, Error> {
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

    let join_event = invite_events::Entity::find()
        .filter(invite_events::Column::GuildId.eq(guild_id.get() as i64))
        .filter(invite_events::Column::TargetUserId.eq(user_id.get() as i64))
        .filter(invite_events::Column::EventType.eq("member_join"))
        .order_by_desc(invite_events::Column::CreatedAt)
        .one(&data.db)
        .await?;

    if let Some(join_event) = join_event {
        let now = Utc::now();
        let join_time: chrono::DateTime<Utc> = join_event.created_at.into();
        let duration = now - join_time;
        
        return Ok(duration.num_hours() < threshold_hours as i64);
    }

    Ok(false)
}

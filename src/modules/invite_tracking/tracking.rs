use crate::db::entities::{invite_snapshots, module_configs::{self, InviteTrackingModuleConfig, ModuleType}};
use crate::{Data, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Set};
use std::collections::HashMap;

/// Check if invite tracking module is enabled and get config
pub async fn get_config(
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<Option<InviteTrackingModuleConfig>, Error> {
    let m_config =
        module_configs::Entity::find_by_id((guild_id.get() as i64, ModuleType::InviteTracking))
            .one(&data.db)
            .await?;

    match m_config {
        Some(m) => {
            if !m.enabled {
                return Ok(None);
            }
            let config: InviteTrackingModuleConfig =
                serde_json::from_value(m.config).unwrap_or_default();
            Ok(Some(config))
        }
        None => Ok(None),
    }
}

/// Fetch all current invites from Discord API
pub async fn fetch_guild_invites(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
) -> Result<Vec<serenity::RichInvite>, Error> {
    Ok(guild_id.invites(&ctx.http).await?)
}

/// Sync Discord invites to database snapshots
pub async fn sync_invites_to_snapshots(
    guild_id: serenity::GuildId,
    invites: Vec<serenity::RichInvite>,
    data: &Data,
) -> Result<(), Error> {
    let now = Utc::now();

    for invite in invites {
        let invite_type = determine_invite_type(&invite);
        let inviter_id = invite.inviter.as_ref().map(|u| u.id.get() as i64);
        let channel_id = invite.channel.id.get() as i64;
        
        let expires_at = if invite.max_age > 0 {
            // Convert Timestamp to DateTime, add duration
            let created_timestamp = invite.created_at.unix_timestamp();
            let created = chrono::DateTime::from_timestamp(created_timestamp, 0).unwrap();
            Some(created + chrono::Duration::seconds(invite.max_age as i64))
        } else {
            None
        };

        let snapshot = invite_snapshots::ActiveModel {
            guild_id: Set(guild_id.get() as i64),
            code: Set(invite.code.to_string()),
            inviter_id: Set(inviter_id),
            channel_id: Set(Some(channel_id)),
            uses: Set(invite.uses as i32),
            max_uses: Set(Some(invite.max_uses as i32)),
            max_age: Set(Some(invite.max_age as i32)),
            temporary: Set(invite.temporary),
            created_at: Set(chrono::DateTime::from_timestamp(invite.created_at.unix_timestamp(), 0).unwrap().into()),
            expires_at: Set(expires_at.map(Into::into)),
            invite_type: Set(invite_type),
            last_synced_at: Set(now.into()),
        };

        invite_snapshots::Entity::insert(snapshot)
            .on_conflict(
                sea_orm::sea_query::OnConflict::columns([
                    invite_snapshots::Column::GuildId,
                    invite_snapshots::Column::Code,
                ])
                .update_columns([
                    invite_snapshots::Column::Uses,
                    invite_snapshots::Column::LastSyncedAt,
                ])
                .to_owned(),
            )
            .exec(&data.db)
            .await?;
    }

    Ok(())
}

/// Get snapshots from database
pub async fn get_snapshots(
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<Vec<invite_snapshots::Model>, Error> {
    let snapshots = invite_snapshots::Entity::find()
        .filter(invite_snapshots::Column::GuildId.eq(guild_id.get() as i64))
        .all(&data.db)
        .await?;

    Ok(snapshots)
}

/// Compare current invites with snapshots to find which invite was used
pub fn find_used_invite(
    current_invites: &[serenity::RichInvite],
    snapshots: &[invite_snapshots::Model],
) -> Option<UsedInviteInfo> {
    // Create a map of code -> old uses
    let snapshot_map: HashMap<String, i32> = snapshots
        .iter()
        .map(|s| (s.code.clone(), s.uses))
        .collect();

    // Find invite where uses increased
    for invite in current_invites {
        let invite_code_str = invite.code.to_string();
        if let Some(&old_uses) = snapshot_map.get(&invite_code_str) {
            let current_uses = invite.uses as i32;
            if current_uses > old_uses {
                return Some(UsedInviteInfo {
                    code: invite.code.to_string(),
                    inviter_id: invite.inviter.as_ref().map(|u| u.id),
                    invite_type: determine_invite_type(invite),
                });
            }
        }
    }

    None
}

/// Information about which invite was used
#[derive(Debug, Clone)]
pub struct UsedInviteInfo {
    pub code: String,
    pub inviter_id: Option<serenity::UserId>,
    pub invite_type: String,
}

/// Determine the type of invite (normal, vanity, etc.)
fn determine_invite_type(invite: &serenity::RichInvite) -> String {
    // Check if it's a vanity URL
    if invite.code.len() < 6 {
        return "vanity".to_string();
    }
    
    // Default to normal invite
    "normal".to_string()
}

/// Determine join type for unknown invites (vanity, widget, discovery)
pub async fn determine_special_join_type(
    guild_id: serenity::GuildId,
    ctx: &serenity::Context,
) -> Result<(Option<serenity::UserId>, String, Option<String>), Error> {
    // Try to get vanity URL
    if let Ok(guild) = guild_id.to_partial_guild(&ctx.http).await {
        if let Some(vanity_code) = guild.vanity_url_code {
            return Ok((None, "vanity".to_string(), Some(vanity_code.to_string())));
        }
    }

    // If we can't determine, mark as unknown
    Ok((None, "unknown".to_string(), None))
}

pub fn format_join_type(join_type: &str) -> String {
    match join_type {
        "normal" => "Regular Invite".to_string(),
        "vanity" => "Vanity URL".to_string(),
        "widget" => "Server Widget".to_string(),
        "discovery" => "Server Discovery".to_string(),
        "unknown" => "Unknown".to_string(),
        _ => join_type.to_string(),
    }
}

/// Delete invite snapshot
pub async fn delete_invite_snapshot(
    guild_id: serenity::GuildId,
    code: &str,
    data: &Data,
) -> Result<(), Error> {
    invite_snapshots::Entity::delete_many()
        .filter(invite_snapshots::Column::GuildId.eq(guild_id.get() as i64))
        .filter(invite_snapshots::Column::Code.eq(code))
        .exec(&data.db)
        .await?;

    Ok(())
}

/// Sync all invites for a guild (used on bot startup or module enable)
pub async fn sync_all_guild_invites(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    data: &Data,
) -> Result<(), Error> {
    match fetch_guild_invites(ctx, guild_id).await {
        Ok(invites) => {
            sync_invites_to_snapshots(guild_id, invites, data).await?;
            tracing::info!("Synced invites for guild {}", guild_id);
        }
        Err(e) => {
            tracing::error!("Failed to fetch invites for guild {}: {:?}", guild_id, e);
        }
    }

    Ok(())
}

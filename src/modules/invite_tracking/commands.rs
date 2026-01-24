use crate::db::entities::invite_events;
use crate::modules::invite_tracking::{stats, tracking};
use crate::services::localization::ContextL10nExt;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

/// View invite statistics
#[poise::command(
    slash_command,
    guild_only,
    subcommands("stats", "leaderboard", "codes")
)]
pub async fn invites(ctx: Context<'_>) -> Result<(), Error> {
    // This is the parent command, subcommands will handle actual functionality
    ctx.send(
        poise::CreateReply::default()
            .content("Please use a subcommand: `/invites stats`, `/invites leaderboard`, or `/invites codes`")
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// View invite statistics for a user
#[poise::command(slash_command, guild_only)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "User to check (defaults to you)"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let target = user.as_ref().unwrap_or_else(|| ctx.author());
    let l10n = ctx.l10n_guild();

    // Check if module is enabled
    if tracking::get_config(guild_id, &ctx.data()).await?.is_none() {
        ctx.send(
            poise::CreateReply::default()
                .content("Invite tracking is not enabled on this server.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    ctx.defer().await?;

    // Get user stats
    let user_stats = stats::get_user_stats(guild_id, target.id, &ctx.data()).await?;

    let mut response = format!("📊 **Invite Statistics for {}**\n\n", target.name);

    if let Some(stats) = user_stats {
        response.push_str(&format!("📈 Total Invites: **{}**\n", stats.total_invites));
        response.push_str(&format!("✅ Current Members: **{}**\n", stats.current_members));
        response.push_str(&format!("❌ Left Server: **{}**\n", stats.left_members));
        response.push_str(&format!("⚠️ Suspicious/Fake: **{}**\n", stats.fake_members));
    } else {
        response.push_str(&l10n.t("invites-no-stats", None));
    }

    // Find how this user joined
    if target.id != ctx.author().id || user.is_some() {
        response.push_str("\n**How they joined:**\n");
        let join_info = find_user_join_info(guild_id, target.id, &ctx.data()).await?;
        if let Some(info) = join_info {
            if let Some(inviter_id) = info.inviter_id {
                response.push_str(&format!("👤 Invited by: <@{}>\n", inviter_id));
            }
            response.push_str(&format!("🔗 Join type: {}\n", tracking::format_join_type(&info.join_type)));
            if let Some(code) = info.invite_code {
                response.push_str(&format!("🎫 Invite code: `{}`\n", code));
            }
        } else {
            response.push_str(&l10n.t("invites-not-tracked", None));
            response.push_str("\n");
        }
    }

    ctx.send(poise::CreateReply::default().content(response).ephemeral(false))
        .await?;

    Ok(())
}

/// View server invite leaderboard
#[poise::command(slash_command, guild_only)]
pub async fn leaderboard(
    ctx: Context<'_>,
    #[description = "Number of users to show (default: 10)"]
    #[min = 1]
    #[max = 50]
    limit: Option<u32>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    // Check if module is enabled and get config
    let config = match tracking::get_config(guild_id, &ctx.data()).await? {
        Some(c) => c,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("Invite tracking is not enabled on this server.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let limit = limit.unwrap_or(config.leaderboard_limit);

    ctx.defer().await?;

    let top_inviters = stats::get_top_inviters(guild_id, limit, &ctx.data()).await?;

    let mut response = format!("🏆 **Top {} Inviters**\n\n", limit);

    if top_inviters.is_empty() {
        response.push_str("No invite data available yet.");
    } else {
        for (idx, inviter) in top_inviters.iter().enumerate() {
            let medal = match idx {
                0 => "🥇",
                1 => "🥈",
                2 => "🥉",
                _ => "  ",
            };
            response.push_str(&format!(
                "{} **#{}** <@{}> - {} invites ({} current, {} left, {} fake)\n",
                medal,
                idx + 1,
                inviter.user_id,
                inviter.total_invites,
                inviter.current_members,
                inviter.left_members,
                inviter.fake_members
            ));
        }
    }

    ctx.send(poise::CreateReply::default().content(response).ephemeral(false))
        .await?;

    Ok(())
}

/// View your active invite codes (Admin only)
#[poise::command(slash_command, guild_only, required_permissions = "MANAGE_GUILD")]
pub async fn codes(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    // Check if module is enabled
    if tracking::get_config(guild_id, &ctx.data()).await?.is_none() {
        ctx.send(
            poise::CreateReply::default()
                .content("Invite tracking is not enabled on this server.")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    ctx.defer_ephemeral().await?;

    // Fetch current guild invites
    let invites = match tracking::fetch_guild_invites(&ctx.serenity_context(), guild_id).await {
        Ok(inv) => inv,
        Err(e) => {
            ctx.send(
                poise::CreateReply::default()
                    .content(format!("Failed to fetch invites: {:?}", e))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    // Filter invites created by the command user
    let user_invites: Vec<_> = invites
        .iter()
        .filter(|inv| {
            inv.inviter
                .as_ref()
                .map(|u| u.id == ctx.author().id)
                .unwrap_or(false)
        })
        .collect();

    let mut response = format!("🔗 **Your Active Invites**\n\n");

    if user_invites.is_empty() {
        response.push_str("You don't have any active invites.");
    } else {
        for invite in user_invites {
            response.push_str(&format!("**Code:** `{}`\n", invite.code));
            response.push_str(&format!("  📊 Uses: {}", invite.uses));
            if invite.max_uses > 0 {
                response.push_str(&format!("/{}", invite.max_uses));
            } else {
                response.push_str("/∞");
            }
            response.push_str("\n");

            if invite.temporary {
                response.push_str("  ⏱️ Temporary membership\n");
            }

            if invite.max_age > 0 {
                let hours = invite.max_age / 3600;
                response.push_str(&format!("  ⏰ Expires in {} hours\n", hours));
            } else {
                response.push_str("  ♾️ Never expires\n");
            }

            response.push_str("\n");
        }
    }

    ctx.send(poise::CreateReply::default().content(response).ephemeral(true))
        .await?;

    Ok(())
}

pub fn commands() -> Vec<poise::Command<crate::Data, Error>> {
    vec![invites()]
}

// Helper structures and functions

#[derive(Debug)]
struct JoinInfo {
    inviter_id: Option<u64>,
    join_type: String,
    invite_code: Option<String>,
}

async fn find_user_join_info(
    guild_id: serenity::GuildId,
    user_id: serenity::UserId,
    data: &crate::Data,
) -> Result<Option<JoinInfo>, Error> {
    let event = invite_events::Entity::find()
        .filter(invite_events::Column::GuildId.eq(guild_id.get() as i64))
        .filter(invite_events::Column::TargetUserId.eq(user_id.get() as i64))
        .filter(invite_events::Column::EventType.eq("member_join"))
        .order_by_desc(invite_events::Column::CreatedAt)
        .one(&data.db)
        .await?;

    Ok(event.map(|e| JoinInfo {
        inviter_id: e.inviter_id.map(|id| id as u64),
        join_type: e.join_type.unwrap_or_else(|| "unknown".to_string()),
        invite_code: e.invite_code,
    }))
}

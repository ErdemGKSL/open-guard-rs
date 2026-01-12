use crate::modules::moderation_protection::duration_parser::parse_duration;
use crate::services::localization::ContextL10nExt;
use crate::{Context, Error};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;

/// Ban a user from the server
#[poise::command(slash_command, guild_only, required_permissions = "BAN_MEMBERS")]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Duration of the ban (e.g. 1d, 1h, 10m30s)"] duration: Option<String>,
    #[description = "Reason for the ban"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let l10n = ctx.l10n_user();

    let duration_parsed = if let Some(d) = duration {
        match parse_duration(&d) {
            Some(dur) => Some(dur),
            None => {
                ctx.say(l10n.t("mod-error-invalid-duration", None)).await?;
                return Ok(());
            }
        }
    } else {
        None
    };

    let ban_reason = reason
        .clone()
        .unwrap_or_else(|| "No reason provided".to_string());

    // Perform the ban
    guild_id
        .ban(ctx.http(), user.id, 0, Some(&ban_reason))
        .await?;

    if let Some(dur) = duration_parsed {
        // Store temp ban in DB
        let expires_at = chrono::Utc::now() + dur;
        let model = crate::db::entities::temp_bans::ActiveModel {
            guild_id: sea_orm::Set(guild_id.get() as i64),
            user_id: sea_orm::Set(user.id.get() as i64),
            expires_at: sea_orm::Set(expires_at.naive_utc()),
            reason: sea_orm::Set(reason),
            ..Default::default()
        };
        use sea_orm::ActiveModelTrait;
        model.insert(&ctx.data().db).await?;

        let mut args = FluentArgs::new();
        args.set("userId", user.id.get());
        args.set("duration", dur.to_string());
        args.set("reason", ban_reason);
        ctx.say(l10n.t("mod-ban-success-temp", Some(&args))).await?;
    } else {
        let mut args = FluentArgs::new();
        args.set("userId", user.id.get());
        args.set("reason", ban_reason);
        ctx.say(l10n.t("mod-ban-success-perm", Some(&args))).await?;
    }

    Ok(())
}

/// Kick a user from the server
#[poise::command(slash_command, guild_only, required_permissions = "KICK_MEMBERS")]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "User to kick"] user: serenity::User,
    #[description = "Reason for the kick"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let l10n = ctx.l10n_user();
    let kick_reason = reason.unwrap_or_else(|| "No reason provided".to_string());

    guild_id
        .kick(ctx.http(), user.id, Some(&kick_reason))
        .await?;

    let mut args = FluentArgs::new();
    args.set("userId", user.id.get());
    args.set("reason", kick_reason);
    ctx.say(l10n.t("mod-kick-success", Some(&args))).await?;

    Ok(())
}

/// Timeout a user
#[poise::command(slash_command, guild_only, required_permissions = "MODERATE_MEMBERS")]
pub async fn timeout(
    ctx: Context<'_>,
    #[description = "User to timeout"] user: serenity::User,
    #[description = "Duration of the timeout (e.g. 1h, 10m)"] duration: String,
    #[description = "Reason for the timeout"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let l10n = ctx.l10n_user();

    let dur = match parse_duration(&duration) {
        Some(d) => d,
        None => {
            ctx.say(l10n.t("mod-error-invalid-duration", None)).await?;
            return Ok(());
        }
    };

    let timeout_reason = reason.unwrap_or_else(|| "No reason provided".to_string());
    let mut member = guild_id.member(ctx, user.id).await?;

    member
        .disable_communication_until(ctx.http(), (chrono::Utc::now() + dur).into())
        .await?;

    let mut args = FluentArgs::new();
    args.set("userId", user.id.get());
    args.set("duration", duration);
    args.set("reason", timeout_reason);
    ctx.say(l10n.t("mod-timeout-success", Some(&args))).await?;

    Ok(())
}

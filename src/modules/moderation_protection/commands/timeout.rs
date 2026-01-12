use crate::modules::moderation_protection::duration_parser::parse_duration;
use crate::services::localization::ContextL10nExt;
use crate::{Context, Error};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;

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

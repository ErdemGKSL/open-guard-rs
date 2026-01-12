use crate::services::localization::ContextL10nExt;
use crate::{Context, Error};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;

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

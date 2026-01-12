use crate::db::entities::module_configs::ModuleType;
use crate::modules::moderation_protection::duration_parser::parse_duration;
use crate::services::localization::ContextL10nExt;
use crate::services::logger::LogLevel;
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

    let timeout_reason = reason
        .clone()
        .unwrap_or_else(|| l10n.t("log-val-no-reason", None));
    let mut member = guild_id.member(ctx, user.id).await?;

    member
        .disable_communication_until(ctx.http(), (chrono::Utc::now() + dur).into())
        .await?;

    // Log action
    let l10n_guild = ctx.l10n_guild();
    let mut log_args = FluentArgs::new();
    log_args.set("modId", ctx.author().id.get());
    log_args.set("userId", user.id.get());

    ctx.data()
        .logger
        .log_context(
            &ctx,
            Some(ModuleType::ModerationProtection),
            LogLevel::Audit,
            &l10n_guild.t("log-mod-timeout-cmd-title", None),
            &l10n_guild.t("log-mod-timeout-cmd-desc", Some(&log_args)),
            vec![
                (
                    &l10n_guild.t("log-field-user", None),
                    format!("<@{}>", user.id),
                ),
                (&l10n_guild.t("log-field-duration", None), duration.clone()),
                (
                    &l10n_guild.t("log-field-reason", None),
                    timeout_reason.clone(),
                ),
            ],
        )
        .await?;

    let mut args = FluentArgs::new();
    args.set("userId", user.id.get());
    args.set("duration", duration);
    args.set("reason", timeout_reason);
    ctx.say(l10n.t("mod-timeout-success", Some(&args))).await?;

    Ok(())
}

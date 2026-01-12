use crate::db::entities::module_configs::ModuleType;
use crate::modules::moderation_protection::duration_parser::parse_duration;
use crate::services::localization::ContextL10nExt;
use crate::services::logger::LogLevel;
use crate::{Context, Error};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;

/// Jail a user, swapping their roles with the jail role
#[poise::command(slash_command, guild_only, required_permissions = "MODERATE_MEMBERS")]
pub async fn jail(
    ctx: Context<'_>,
    #[description = "User to jail"] user: serenity::User,
    #[description = "Duration of the jail (e.g. 1d, 1h, 10m30s)"] duration: Option<String>,
    #[description = "Reason for the jail"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let l10n = ctx.l10n_user();

    let duration_parsed = if let Some(d) = duration.as_ref() {
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

    let jail_reason = reason
        .clone()
        .unwrap_or_else(|| l10n.t("log-val-no-reason", None));

    ctx.data()
        .jail
        .jail_user(ctx.http(), guild_id, user.id, duration_parsed, &jail_reason)
        .await?;

    // The jail_user service already logs, but we can log additionally if needed.
    // However, the user asked to log EVERY event, so I'll ensure services do it.
    // Commands should also log their context.
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
            &l10n_guild.t("log-mod-jail-cmd-title", None),
            &l10n_guild.t("log-mod-jail-cmd-desc", Some(&log_args)),
            vec![
                (
                    &l10n_guild.t("log-field-user", None),
                    format!("<@{}>", user.id),
                ),
                (
                    &l10n_guild.t("log-field-duration", None),
                    duration
                        .as_deref()
                        .unwrap_or_else(|| l10n_guild.t("log-val-permanent", None).leak())
                        .to_string(),
                ),
                (&l10n_guild.t("log-field-reason", None), jail_reason.clone()),
            ],
        )
        .await?;

    let mut args = FluentArgs::new();
    args.set("userId", user.id.get());
    args.set("reason", jail_reason);

    if let Some(dur) = duration_parsed {
        args.set("duration", format!("{:?}", dur));
        ctx.say(l10n.t("mod-jail-success-temp", Some(&args)))
            .await?;
    } else {
        ctx.say(l10n.t("mod-jail-success-perm", Some(&args)))
            .await?;
    }

    Ok(())
}

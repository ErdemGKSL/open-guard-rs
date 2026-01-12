use crate::db::entities::module_configs::ModuleType;
use crate::services::localization::ContextL10nExt;
use crate::services::logger::LogLevel;
use crate::{Context, Error};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;

/// Unjail a user, restoring their original roles
#[poise::command(slash_command, guild_only, required_permissions = "MODERATE_MEMBERS")]
pub async fn unjail(
    ctx: Context<'_>,
    #[description = "User to unjail"] user: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let l10n = ctx.l10n_user();

    ctx.data()
        .jail
        .unjail_user(ctx.http(), guild_id, user.id)
        .await?;

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
            &l10n_guild.t("log-mod-unjail-cmd-title", None),
            &l10n_guild.t("log-mod-unjail-cmd-desc", Some(&log_args)),
            vec![(
                &l10n_guild.t("log-field-user", None),
                format!("<@{}>", user.id),
            )],
        )
        .await?;

    let mut args = FluentArgs::new();
    args.set("userId", user.id.get());
    ctx.say(l10n.t("mod-unjail-success", Some(&args))).await?;

    Ok(())
}


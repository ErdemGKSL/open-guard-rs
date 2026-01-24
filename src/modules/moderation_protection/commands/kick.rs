use crate::db::entities::module_configs::ModuleType;
use crate::services::localization::ContextL10nExt;
use crate::services::logger::LogLevel;
use crate::{Context, Error};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;

/// Kick a user from the server
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "KICK_MEMBERS",
    ephemeral
)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "User to kick"] user: serenity::User,
    #[description = "Reason for the kick"] reason: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let guild_id = ctx.guild_id().unwrap();
    let l10n = ctx.l10n_user();
    let kick_reason = reason
        .clone()
        .unwrap_or_else(|| l10n.t("log-val-no-reason", None));

    guild_id
        .kick(ctx.http(), user.id, Some(&kick_reason))
        .await?;

    // Log action
    let l10n_guild = ctx.l10n_guild();
    let mut log_args = FluentArgs::new();
    log_args.set("modId", ctx.author().id.get().to_string());
    log_args.set("userId", user.id.get().to_string());

    ctx.data()
        .logger
        .log_context(
            &ctx,
            Some(ModuleType::ModerationProtection),
            LogLevel::Audit,
            &l10n_guild.t("log-mod-kick-cmd-title", None),
            &l10n_guild.t("log-mod-kick-cmd-desc", Some(&log_args)),
            vec![
                (
                    &l10n_guild.t("log-field-user", None),
                    format!("<@{}>", user.id.get()),
                ),
                (&l10n_guild.t("log-field-reason", None), kick_reason.clone()),
            ],
        )
        .await?;

    let mut args = FluentArgs::new();
    args.set("userId", user.id.get().to_string());
    args.set("reason", kick_reason);
    ctx.send(
        poise::CreateReply::default()
            .components(vec![serenity::CreateComponent::Container(
                serenity::CreateContainer::new(vec![
                    serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(l10n.t("mod-kick-success", Some(&args))))
                ])
            )])
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2 | serenity::MessageFlags::EPHEMERAL),
    )
        .await?;

    Ok(())
}

use crate::db::entities::module_configs::ModuleType;
use crate::modules::moderation_protection::duration_parser::parse_duration;
use crate::services::localization::ContextL10nExt;
use crate::services::logger::LogLevel;
use crate::{Context, Error};
use fluent::FluentArgs;
use poise::serenity_prelude as serenity;

/// Ban a user from the server
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    ephemeral
)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "User to ban"] user: serenity::User,
    #[description = "Duration of the ban (e.g. 1d, 1h, 10m30s)"] duration: Option<String>,
    #[description = "Reason for the ban"] reason: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let guild_id = ctx.guild_id().unwrap();
    let l10n = ctx.l10n_user();

    let duration_parsed = if let Some(d) = duration {
        match parse_duration(&d) {
            Some(dur) => Some(dur),
            None => {
                ctx.send(
                    poise::CreateReply::default()
                        .components(vec![serenity::CreateComponent::Container(
                            serenity::CreateContainer::new(vec![
                                serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(l10n.t("mod-error-invalid-duration", None)))
                            ])
                        )])
                        .flags(serenity::MessageFlags::IS_COMPONENTS_V2 | serenity::MessageFlags::EPHEMERAL),
                )
                .await?;
                return Ok(());
            }
        }
    } else {
        None
    };

    let ban_reason = reason
        .clone()
        .unwrap_or_else(|| l10n.t("log-val-no-reason", None));

    // Perform the ban
    guild_id
        .ban(ctx.http(), user.id, 0, Some(&ban_reason))
        .await?;

    let expires_at_str = if let Some(dur) = duration_parsed {
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
        args.set("userId", user.id.get().to_string());
        args.set("duration", format!("{:?}", dur));
        args.set("reason", ban_reason.clone());
        ctx.send(
            poise::CreateReply::default()
                .components(vec![serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(vec![
                        serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(l10n.t("mod-ban-success-temp", Some(&args))))
                    ])
                )])
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2 | serenity::MessageFlags::EPHEMERAL),
        )
        .await?;
        format!("{:?}", dur)
    } else {
        let mut args = FluentArgs::new();
        args.set("userId", user.id.get().to_string());
        args.set("reason", ban_reason.clone());
        ctx.send(
            poise::CreateReply::default()
                .components(vec![serenity::CreateComponent::Container(
                    serenity::CreateContainer::new(vec![
                        serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(l10n.t("mod-ban-success-perm", Some(&args))))
                    ])
                )])
                .flags(serenity::MessageFlags::IS_COMPONENTS_V2 | serenity::MessageFlags::EPHEMERAL),
        )
        .await?;
        l10n.t("log-val-permanent", None)
    };

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
            &l10n_guild.t("log-mod-ban-cmd-title", None),
            &l10n_guild.t("log-mod-ban-cmd-desc", Some(&log_args)),
            vec![
                (
                    &l10n_guild.t("log-field-user", None),
                    format!("<@{}>", user.id.get()),
                ),
                (&l10n_guild.t("log-field-duration", None), expires_at_str),
                (&l10n_guild.t("log-field-reason", None), ban_reason),
            ],
        )
        .await?;

    Ok(())
}

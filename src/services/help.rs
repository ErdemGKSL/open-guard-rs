use crate::services::localization::ContextL10nExt;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Help command
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let l10n = ctx.l10n_user();

    ctx.send(
        poise::CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .components(vec![
                serenity::CreateComponent::Separator(serenity::CreateSeparator::new(true)),
                serenity::CreateComponent::TextDisplay(serenity::CreateTextDisplay::new(
                    l10n.t("help-title", None),
                )),
            ]),
    )
    .await?;

    Ok(())
}

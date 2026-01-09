use crate::services::localization::ContextL10nExt;
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// A help command that lists all modules and their commands using Components V2
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild"
)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let l10n = ctx.l10n_user();
    let _modules = &ctx.data().module_definitions;

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

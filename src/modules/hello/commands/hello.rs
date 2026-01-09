use crate::{Context, Error, services::localization::ContextL10nExt};
use poise::serenity_prelude as serenity;

/// Hello command and its subcommands
#[poise::command(slash_command, prefix_command, subcommands("person", "world"))]
pub async fn hello(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

async fn autocomplete_greeting(_ctx: Context<'_>, partial: &str) -> Vec<String> {
    let greetings = vec!["Hello", "Hi", "Hey", "Greetings", "Salutations"];
    greetings
        .into_iter()
        .filter(|v| v.to_lowercase().contains(&partial.to_lowercase()))
        .map(|v| v.to_string())
        .collect()
}

/// Greet a specific person with a custom greeting
#[poise::command(slash_command, prefix_command)]
pub async fn person(
    ctx: Context<'_>,
    #[description = "The greeting to use"]
    #[autocomplete = "autocomplete_greeting"]
    _greeting: String,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let mut args = fluent::FluentArgs::new();
    args.set("user", u.name.clone());

    use crate::services::localization::ContextL10nExt;
    let l10n = ctx.l10n_user();

    // Priority: User locale -> Guild locale -> Default (en-US)
    let response = l10n.t("hello-user", Some(&args));

    ctx.say(response).await?;
    Ok(())
}

/// Greet the whole world
#[poise::command(slash_command, prefix_command)]
pub async fn world(ctx: Context<'_>) -> Result<(), Error> {
    let response = ctx.l10n_user().t("hello-world", None);
    ctx.say(response).await?;
    Ok(())
}

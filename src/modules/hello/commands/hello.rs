use crate::{Context, Error, services::localization::ContextL10nExt};
use poise::serenity_prelude as serenity;

/// Hello command
#[poise::command(
    slash_command,
    subcommands("person", "world"),
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn hello(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

async fn autocomplete_greeting<'a>(
    _ctx: Context<'a>,
    partial: &'a str,
) -> serenity::CreateAutocompleteResponse<'a> {
    let choices: Vec<_> = ["Hello", "Hi", "Hey", "Greetings", "Salutations"]
        .into_iter()
        .filter(|v| v.to_lowercase().contains(&partial.to_lowercase()))
        .map(serenity::AutocompleteChoice::from)
        .collect();

    serenity::CreateAutocompleteResponse::new().set_choices(choices)
}

/// Greet a person
#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn person(
    ctx: Context<'_>,
    #[autocomplete = "autocomplete_greeting"] _greeting: String,
    user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());

    let mut args = fluent::FluentArgs::new();
    args.set("user", u.name.to_string());

    let l10n = ctx.l10n_user();

    // Priority: User locale -> Guild locale -> Default (en-US)
    let response = l10n.t("hello-user", Some(&args));

    ctx.say(response).await?;
    Ok(())
}

/// Greet the world
#[poise::command(
    slash_command,
    install_context = "Guild|User",
    interaction_context = "Guild|BotDm|PrivateChannel"
)]
pub async fn world(ctx: Context<'_>) -> Result<(), Error> {
    let response = ctx.l10n_user().t("hello-world", None);
    ctx.say(response).await?;
    Ok(())
}

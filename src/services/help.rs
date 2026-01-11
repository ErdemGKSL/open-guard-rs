use crate::services::localization::{ContextL10nExt, L10nProxy};
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Help command
#[poise::command(
    slash_command,
    install_context = "Guild",
    interaction_context = "Guild|BotDm|PrivateChannel",
    ephemeral
)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
    let l10n = ctx.l10n_user();
    let components = get_help_components(0, &l10n);
    
    ctx.send(
        poise::CreateReply::default()
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
            .components(components),
    )
    .await?;

    Ok(())
}

pub fn get_help_components(page: u8, l10n: &L10nProxy) -> Vec<serenity::CreateComponent<'static>> {
    let mut inner_components = vec![];

    // Title
    let title_key = format!("help-page-{}-title", page);
    let title = l10n.t(&title_key, None);

    inner_components.push(serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(
        format!("## {}", title),
    )));

    inner_components.push(serenity::CreateContainerComponent::Separator(serenity::CreateSeparator::new(true)));

    // Content
    let content_key = format!("help-page-{}-content", page);
    let content = l10n.t(&content_key, None);

    inner_components.push(serenity::CreateContainerComponent::TextDisplay(serenity::CreateTextDisplay::new(content)));

    inner_components.push(serenity::CreateContainerComponent::Separator(serenity::CreateSeparator::new(false)));

    // Pagination Buttons
    let mut button_row = vec![];
    
    if page > 0 {
        button_row.push(serenity::CreateButton::new(format!("help-set-page::{}", page - 1))
            .label(l10n.t("help-prev-btn", None))
            .style(serenity::ButtonStyle::Secondary));
    }

    if page < 3 {
        button_row.push(serenity::CreateButton::new(format!("help-set-page::{}", page + 1))
            .label(l10n.t("help-next-btn", None))
            .style(serenity::ButtonStyle::Secondary));
    }

    if !button_row.is_empty() {
        inner_components.push(serenity::CreateContainerComponent::ActionRow(serenity::CreateActionRow::Buttons(button_row.into())));
    }

    vec![serenity::CreateComponent::Container(serenity::CreateContainer::new(inner_components))]
}

pub async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &crate::Data,
) -> Result<(), Error> {
    if !interaction.data.custom_id.starts_with("help-set-page::") {
        return Ok(());
    }

    let l10n = L10nProxy {
        manager: data.l10n.clone(),
        locale: interaction.locale.to_string(),
    };

    let page: u8 = interaction.data.custom_id.trim_start_matches("help-set-page::").parse().unwrap_or(0);
    
    let components = get_help_components(page, &l10n);
    
    interaction.create_response(&ctx.http, serenity::CreateInteractionResponse::UpdateMessage(
        serenity::CreateInteractionResponseMessage::new()
            .components(components)
    )).await?;

    Ok(())
}


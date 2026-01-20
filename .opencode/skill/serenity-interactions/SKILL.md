---
name: serenity-interactions
description: Guide for handling Discord interactions (buttons, selects, modals) in the Open Guard bot
metadata:
  version: "1.0"
---

# Serenity Interactions

## What this skill does
Provides guidance on handling Discord interactions including buttons, select menus, modals, and other interactive components in the Open Guard bot.

## When to use
Use this skill when you need to:
- Create interactive buttons in messages
- Handle button clicks
- Create select menus (dropdowns)
- Handle select menu selections
- Create and handle modals
- Handle file uploads from modals
- Update or respond to interactions

## ⚠️ CRITICAL: Do Not Await Interactions

**NEVER await button/select/menu interactions in command handlers.** This is a hard rule in Open Guard.

### Why?
- Commands must return immediately
- Awaiting interactions blocks the command handler
- Interaction handlers are separate and asynchronous
- Data should be transported via `custom_id` parsing

### The Pattern

```rust
// ❌ WRONG: Awaiting interaction in command
pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .components(vec![
            serenity::CreateButton::new("btn_click")
                .label("Click Me!")
                .style(serenity::ButtonStyle::Primary),
        ])
    ).await?;

    // WRONG: This blocks the command!
    let interaction = /* wait for click... */;

    Ok(())
}

// ✅ CORRECT: Send and return immediately
pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    let setup_id = generate_unique_id();

    ctx.send(poise::CreateReply::default()
        .content("Click the button!")
        .components(vec![
            serenity::CreateButton::new(format!("btn_click_{}", setup_id))
                .label("Click Me!")
                .style(serenity::ButtonStyle::Primary),
        ])
    ).await?;

    // ✅ Command returns immediately
    Ok(())
}

// Handle interaction in separate event handler
async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    // Parse data from custom_id
    if let Some(setup_id) = custom_id.strip_prefix("btn_click_") {
        // Process interaction with the extracted setup_id
        println!("Processing button for setup: {}", setup_id);

        // Respond to interaction
        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Button clicked!")
                    .components(vec![])
            )
        ).await?;
    }

    Ok(())
}
```

## Interaction Types

1. **Component Interactions** - Buttons and select menus
2. **Modal Submissions** - Form submissions
3. **Application Commands** - Slash commands (handled by Poise)

## Setting Up Interaction Handlers

### Event Handler Integration

The main interaction handler is in `src/services/event_manager/mod.rs`:

```rust
use crate::services::event_manager::Handler;

// Handler is registered in main.rs
.event_handler(Arc::new(services::event_manager::Handler::new()))
```

### Handling Interactions in Events

```rust
// In your event handler
use poise::serenity_prelude as serenity;

async fn handle_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::Interaction,
    data: &Data,
) -> Result<(), Error> {
    match interaction {
        serenity::Interaction::Component(component) => {
            handle_component_interaction(ctx, component, data).await?;
        }
        serenity::Interaction::Modal(modal) => {
            handle_modal_submission(ctx, modal, data).await?;
        }
        _ => {}
    }

    Ok(())
}
```

## Buttons

### Creating Buttons

```rust
use poise::serenity_prelude as serenity;

pub async fn send_with_buttons(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("Click a button!")
        .components(vec![
            serenity::CreateActionRow::Buttons(vec![
                // Primary (blue)
                serenity::CreateButton::new("btn_primary")
                    .label("Primary")
                    .style(serenity::ButtonStyle::Primary),

                // Secondary (gray)
                serenity::CreateButton::new("btn_secondary")
                    .label("Secondary")
                    .style(serenity::ButtonStyle::Secondary),

                // Success (green)
                serenity::CreateButton::new("btn_success")
                    .label("Success")
                    .style(serenity::ButtonStyle::Success),

                // Danger (red)
                serenity::CreateButton::new("btn_danger")
                    .label("Danger")
                    .style(serenity::ButtonStyle::Danger),

                // Link (opens URL)
                serenity::CreateButton::new("btn_link")
                    .label("Open Google")
                    .style(serenity::ButtonStyle::Link)
                    .url("https://google.com"),
            ])
        ])
    ).await?;

    Ok(())
}
```

### Button with Emoji

```rust
serenity::CreateButton::new("btn_emoji")
    .label("Click Me!")
    .emoji(serenity::ReactionType::Unicode("🎉".to_string()))
    .style(serenity::ButtonStyle::Primary)
```

### Disabled Button

```rust
serenity::CreateButton::new("btn_disabled")
    .label("Disabled")
    .style(serenity::ButtonStyle::Primary)
    .disabled(true)
```

### Handling Button Clicks

```rust
async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;
    let guild_id = interaction.guild_id;

    // Handle button clicks by parsing custom_id
    if custom_id.starts_with("btn_") {
        match custom_id.as_str() {
            "btn_primary" => {
                handle_primary_button(ctx, interaction, data).await?;
            }
            "btn_danger" => {
                handle_danger_button(ctx, interaction, data).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn handle_primary_button(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    // Send a follow-up message
    interaction
        .create_response(
            ctx,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("You clicked the primary button!")
            )
        )
        .await?;

    Ok(())
}
```

### Transporting Data via Custom IDs

Use custom_ids to pass data between command and handler:

```rust
// In command - embed data in custom_id
pub async fn start_process(ctx: Context<'_>) -> Result<(), Error> {
    let setup_id = "setup_12345";
    let user_id = ctx.author().id.get();

    // Embed data in custom_id
    ctx.send(poise::CreateReply::default()
        .components(vec![
            serenity::CreateButton::new(format!("btn_next_{}_{}", setup_id, user_id))
                .label("Next Step")
                .style(serenity::ButtonStyle::Primary),
        ])
    ).await?;

    Ok(())
}

// In handler - extract data from custom_id
async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    if let Some(rest) = custom_id.strip_prefix("btn_next_") {
        let parts: Vec<&str> = rest.split('_').collect();
        let setup_id = parts[0];
        let user_id: u64 = parts[1].parse()?;

        // Use extracted data
        println!("Setup: {}, User: {}", setup_id, user_id);
    }

    Ok(())
}
```

### Response Types for Interactions

### ⚠️ Important: Acknowledge-Then-Edit Pattern for Menus

When handling select menus (dropdowns), the recommended pattern is:

1. **Acknowledge the interaction** (shows "thinking" state)
2. **Edit the message later** with the new content

This is especially useful when you need to:
- Show a loading state while processing
- Update the message based on selection
- Keep the message in place while changing its content

```rust
// ✅ RECOMMENDED: Acknowledge then edit
async fn handle_select_menu(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id == "menu_id" {
        // Get selected values
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };

        // Step 1: Acknowledge to show "thinking" state
        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::Acknowledge
        ).await?;

        // Step 2: Process data (might take time)
        let result = process_selection(values).await?;

        // Step 3: Edit the original message with result
        interaction.edit_response(
            ctx,
            serenity::EditInteractionResponse::new()
                .content(format!("Selected: {}", result))
                .components(vec![])  // Remove the select menu
        ).await?;
    }

    Ok(())
}
```

**Why use this pattern?**
- Acknowledging prevents the interaction from timing out
- Users see immediate feedback (loading state)
- Editing the message updates it in place (no new message created)
- Works seamlessly with message interactions

### Response Types Summary

```rust
// 1. Acknowledge (shows "thinking" state, can edit later)
interaction.create_response(
    ctx,
    serenity::CreateInteractionResponse::Acknowledge
).await?;

// 2. Update the original message (one-time update)
interaction.create_response(
    ctx,
    serenity::CreateInteractionResponse::UpdateMessage(
        serenity::CreateInteractionResponseMessage::new()
            .content("Updated message!")
            .components(vec![])  // Remove buttons
    )
).await?;

// 3. Create a new message
interaction.create_response(
    ctx,
    serenity::CreateInteractionResponse::Message(
        serenity::CreateInteractionResponseMessage::new()
            .content("New message!")
    )
).await?
```

### Editing Messages After Acknowledging

When you acknowledge an interaction without a reply, you can edit the original message:

```rust
async fn handle_select_with_edit(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id == "menu_select" {
        // Get selected values
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };

        // Acknowledge first (no content)
        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::Acknowledge
        ).await?;

        // Simulate processing
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Edit the original message
        let message = interaction.message.as_ref().unwrap();
        let mut builder = message.to_builder();
        builder
            .content(format!("You selected: {}", values.join(", ")))
            .components(vec![]);  // Remove select menu

        message.edit(ctx, builder).await?;
    }

    Ok(())
}
```

**Key points:**
- Use `create_response(Acknowledge)` to prevent timeout
- Access original message via `interaction.message`
- Use `message.edit(ctx, builder)` to update content
- This keeps the original message, just changes its content

**When to use acknowledge-then-edit:**
- Select menus where you need to process the selection
- When showing a loading state is useful
- When you want to update the existing message in place
- When you don't want to create a new message

## Select Menus

### String Select (Custom Options)

```rust
ctx.send(poise::CreateReply::default()
    .content("Select an option!")
    .components(vec![
        serenity::CreateActionRow::SelectMenu(
            serenity::CreateSelectMenu::new(
                "string_select_id",
                serenity::CreateSelectMenuKind::String {
                    options: vec![
                        serenity::CreateSelectMenuOption::new("Option 1", "opt1"),
                        serenity::CreateSelectMenuOption::new("Option 2", "opt2")
                            .description("This is option 2"),
                        serenity::CreateSelectMenuOption::new("Option 3", "opt3")
                            .emoji(serenity::ReactionType::Unicode("⭐".to_string())),
                    ]
                }
            )
            .placeholder("Choose an option")
            .min_values(1)  // Require at least 1 selection
            .max_values(1)  // Allow max 1 selection
        )
    ])
).await?;
```

### User Select

```rust
serenity::CreateActionRow::SelectMenu(
    serenity::CreateSelectMenu::new(
        "user_select_id",
        serenity::CreateSelectMenuKind::User {
            default_users: None,  // Or Some(vec![user_id])
        }
    )
    .placeholder("Select users")
    .min_values(1)
    .max_values(3)  // Allow up to 3 users
)
```

### Role Select

```rust
serenity::CreateActionRow::SelectMenu(
    serenity::CreateSelectMenu::new(
        "role_select_id",
        serenity::CreateSelectMenuKind::Role {
            default_roles: None,
        }
    )
    .placeholder("Select roles")
    .min_values(1)
    .max_values(1)
)
```

### Channel Select

```rust
use poise::serenity_prelude as serenity;
use std::borrow::Cow;

serenity::CreateActionRow::SelectMenu(
    serenity::CreateSelectMenu::new(
        "channel_select_id",
        serenity::CreateSelectMenuKind::Channel {
            channel_types: Some(Cow::Borrowed(&[
                serenity::ChannelType::Text,
                serenity::ChannelType::Voice,
                serenity::ChannelType::Category,
            ])),
            default_channels: None,
        }
    )
    .placeholder("Select channels")
)
```

### Mentionable Select (Users + Roles)

```rust
serenity::CreateActionRow::SelectMenu(
    serenity::CreateSelectMenu::new(
        "mentionable_select_id",
        serenity::CreateSelectMenuKind::Mentionable {
            default_users: None,
            default_roles: None,
        }
    )
    .placeholder("Select users or roles")
)
```

### Handling Select Menu Selection

```rust
async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id == "string_select_id" {
        // Get selected values
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };

        for value in values {
            println!("Selected: {}", value);
        }

        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content(format!("You selected: {}", values.join(", ")))
                    .components(vec![])  // Remove select menu
            )
        ).await?;
    }

    Ok(())
}
```

## Modals

### Creating a Modal

```rust
use poise::serenity_prelude as serenity;
use std::borrow::Cow;

async fn show_modal(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let modal = serenity::CreateModal::new()
        .custom_id("feedback_modal")
        .title("Feedback Form")
        .components(vec![
            // Text input with label (recommended)
            serenity::CreateLabel::input_text(
                "Your Name",
                serenity::CreateInputText::new(
                    serenity::InputTextStyle::Short,
                    "name_field"
                )
                .placeholder("Enter your name")
                .required(true)
            ),
            // Paragraph text input
            serenity::CreateLabel::input_text(
                "Feedback",
                serenity::CreateInputText::new(
                    serenity::InputTextStyle::Paragraph,
                    "feedback_field"
                )
                .placeholder("Enter your feedback here...")
                .min_length(10)
                .max_length(1000)
                .required(true)
            ),
        ]);

    interaction.create_response(
        ctx,
        serenity::CreateInteractionResponse::Modal(modal)
    ).await?;

    Ok(())
}
```

### Modal with Select Menu

```rust
let modal = serenity::CreateModal::new()
    .custom_id("rating_modal")
    .title("Rate Your Experience")
    .components(vec![
        serenity::CreateLabel::select_menu(
            "Rating",
            serenity::CreateSelectMenu::new(
                "rating_select",
                serenity::CreateSelectMenuKind::String {
                    options: Cow::Borrowed(&[
                        serenity::CreateSelectMenuOption::new("⭐ Poor", "1"),
                        serenity::CreateSelectMenuOption::new("⭐⭐ Fair", "2"),
                        serenity::CreateSelectMenuOption::new("⭐⭐⭐ Good", "3"),
                        serenity::CreateSelectMenuOption::new("⭐⭐⭐⭐ Very Good", "4"),
                        serenity::CreateSelectMenuOption::new("⭐⭐⭐⭐⭐ Excellent", "5"),
                    ])
                }
            )
            .placeholder("Select a rating")
            .required(true)
        ),
        serenity::CreateLabel::input_text(
            "Comments",
            serenity::CreateInputText::new(
                serenity::InputTextStyle::Paragraph,
                "comments_field"
            )
            .placeholder("Additional comments (optional)")
            .required(false)
        ),
    ]);
```

### Modal with File Upload

```rust
let modal = serenity::CreateModal::new()
    .custom_id("file_upload_modal")
    .title("Upload Files")
    .components(vec![
        serenity::CreateLabel::file_upload(
            "Screenshots",
            serenity::CreateFileUpload::new("screenshot_upload")
            .min_values(1)
            .max_values(5)
            .required(true)
        ),
        serenity::CreateLabel::input_text(
            "Description",
            serenity::CreateInputText::new(
                serenity::InputTextStyle::Short,
                "description_field"
            )
            .placeholder("Describe the files...")
            .required(true)
        ),
    ]);
```

### Handling Modal Submissions

```rust
async fn handle_modal_submission(
    ctx: &serenity::Context,
    interaction: &serenity::ModalSubmitInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id == "feedback_modal" {
        // Extract values from components
        let mut name = String::new();
        let mut feedback = String::new();

        for component in &interaction.data.components {
            if let serenity::ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "name_field" => name = input.value.clone(),
                    "feedback_field" => feedback = input.value.clone(),
                    _ => {}
                }
            }
        }

        // Process the data
        println!("Name: {}, Feedback: {}", name, feedback);

        // Send confirmation
        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Thank you for your feedback!")
                    .components(vec![])
            )
        ).await?;
    }

    Ok(())
}
```

## Interaction Context

### Getting User Information

```rust
// In any interaction handler
let user = &interaction.user;
let member = interaction.member.as_ref();
let guild_id = interaction.guild_id;
let channel_id = interaction.channel_id;
let locale = &interaction.locale;
```

### Getting Locale-Specific Strings

```rust
use crate::services::localization::L10nProxy;

let l10n = L10nProxy {
    manager: data.l10n.clone(),
    locale: interaction.locale.to_string(),
};

let localized_text = l10n.t("message-key", None);
```

## Updating Components

### Update Button State

```rust
// Disable a button after click
let disabled_buttons = vec![
    serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("btn_click_me")
            .label("Already Clicked!")
            .style(serenity::ButtonStyle::Success)
            .disabled(true),
    ])
];

interaction.create_response(
    ctx,
    serenity::CreateInteractionResponse::UpdateMessage(
        serenity::CreateInteractionResponseMessage::new()
            .content("Button clicked!")
            .components(disabled_buttons)
    )
).await?;
```

## Best Practices

1. **⚠️ NEVER await interactions in commands**: Commands must return immediately
2. **⚠️ ALWAYS use separate handlers**: Handle interactions in event handlers, not commands
3. **⚠️ ALWAYS use custom_id parsing**: Transport data via custom_ids, not awaiting
4. **⚠️ For select menus, use acknowledge-then-edit**: Acknowledge first, then edit message with results
5. **Use descriptive custom_ids**: Make them unique and meaningful
6. **Handle all interaction types**: Don't leave interactions unhandled
7. **Use proper response types**: Choose UpdateMessage vs. Message vs. Acknowledge appropriately
8. **Validate user input**: Check permissions and data before processing
9. **Use localization**: Make all interaction responses translatable
10. **Clean up components**: Remove buttons/selects when no longer needed
11. **Use timeouts**: Consider using collectors for time-limited interactions
12. **Log important actions**: Use the logger service for audit trails

## Example: Complete Setup Workflow

See `src/services/setup/mod.rs` for a complete example of:
- Multi-step setup process
- Select menus for configuration
- State management across interactions
- Dynamic component updates
- Data transport via custom_id parsing

## Resources

- See `discord-modals` skill for modal-specific guidance

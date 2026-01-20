---
name: discord-modals
description: Guide for creating and working with Discord modals and labels using Serenity in the Open Guard bot
metadata:
  version: "1.0"
---

# Discord Modals in Open Guard

## What this skill does
Provides guidance on creating Discord modals with labels, text inputs, select menus, and file uploads using the Serenity library in the Open Guard bot.

## When to use
Use this skill when you need to:
- Create interactive popup forms in Discord
- Collect structured user input through modals
- Implement modal components with labels (text inputs, selects, file uploads)
- Handle modal submissions in the bot

## Modal Concepts

### What is a Modal?
A modal is an interactive popup form that can contain various input components. Modals are triggered in response to interactions and allow collecting multiple pieces of information from users in a structured way.

### What is a Label?
A Label is a top-level layout component that wraps modal input components with descriptive text, providing context for what the user should enter.

## Creating Modals

### Basic Modal Structure
```rust
use poise::serenity_prelude as serenity;

// Create a modal response
let modal = serenity::CreateModal::new()
    .custom_id("my_modal")
    .title("Modal Title")
    .components(vec![
        // Label components go here
    ]);
```

### Using Labels (Recommended Approach)
Labels wrap input components and provide better organization:

```rust
use poise::serenity_prelude as serenity;

let label = serenity::CreateLabel::input_text(
    "Field Label",
    serenity::CreateInputText::new(
        serenity::InputTextStyle::Short,
        "input_field_custom_id"
    )
    .placeholder("Enter text here...")
    .required(true)
);
```

## Supported Label Components

### 1. Text Input
```rust
serenity::CreateLabel::input_text(
    "Email Address",
    serenity::CreateInputText::new(
        serenity::InputTextStyle::Short,
        "user_email"
    )
    .placeholder("user@example.com")
    .min_length(5)
    .max_length(100)
    .required(true)
)
```

Text input styles:
- `InputTextStyle::Short` - Single line
- `InputTextStyle::Paragraph` - Multi-line

### 2. String Select
```rust
serenity::CreateLabel::select_menu(
    "Choose an option",
    serenity::CreateSelectMenu::new(
        "select_custom_id",
        serenity::CreateSelectMenuKind::String {
            options: Cow::Borrowed(&[
                serenity::CreateSelectMenuOption::new("Option 1", "opt1"),
                serenity::CreateSelectMenuOption::new("Option 2", "opt2"),
            ])
        }
    )
    .placeholder("Select an option")
    .required(true)
)
```

### 3. User Select
```rust
serenity::CreateLabel::select_menu(
    "Select Users",
    serenity::CreateSelectMenu::new(
        "user_select_id",
        serenity::CreateSelectMenuKind::User {
            default_users: None
        }
    )
    .min_values(1)
    .max_values(3)
    .required(true)
)
```

### 4. Role Select
```rust
serenity::CreateLabel::select_menu(
    "Select Roles",
    serenity::CreateSelectMenu::new(
        "role_select_id",
        serenity::CreateSelectMenuKind::Role {
            default_roles: None
        }
    )
    .required(true)
)
```

### 5. Channel Select
```rust
serenity::CreateLabel::select_menu(
    "Select Channels",
    serenity::CreateSelectMenu::new(
        "channel_select_id",
        serenity::CreateSelectMenuKind::Channel {
            channel_types: Some(Cow::Borrowed(&[
                serenity::ChannelType::Text,
                serenity::ChannelType::Voice
            ])),
            default_channels: None
        }
    )
)
```

### 6. File Upload
```rust
serenity::CreateLabel::file_upload(
    "Upload Files",
    serenity::CreateFileUpload::new("file_upload_id")
    .min_values(1)
    .max_values(5)
    .required(true)
)
```

### 7. Mentionable Select (Users + Roles)
```rust
serenity::CreateLabel::select_menu(
    "Select Users or Roles",
    serenity::CreateSelectMenu::new(
        "mentionable_select_id",
        serenity::CreateSelectMenuKind::Mentionable {
            default_users: None,
            default_roles: None
        }
    )
)
```

## Complete Modal Example

```rust
use poise::serenity_prelude as serenity;
use std::borrow::Cow;

async fn show_feedback_modal(ctx: &serenity::Context) -> Result<(), Box<dyn std::error::Error>> {
    let modal = serenity::CreateModal::new()
        .custom_id("feedback_modal")
        .title("Feedback Form")
        .components(vec![
            serenity::CreateLabel::input_text(
                "Your Feedback",
                serenity::CreateInputText::new(
                    serenity::InputTextStyle::Paragraph,
                    "feedback_text"
                )
                .placeholder("Write your feedback here...")
                .required(true)
            ),
            serenity::CreateLabel::select_menu(
                "Rating",
                serenity::CreateSelectMenu::new(
                    "rating",
                    serenity::CreateSelectMenuKind::String {
                        options: Cow::Borrowed(&[
                            serenity::CreateSelectMenuOption::new("Excellent", "5"),
                            serenity::CreateSelectMenuOption::new("Good", "4"),
                            serenity::CreateSelectMenuOption::new("Average", "3"),
                        ])
                    }
                )
                .placeholder("Select a rating")
                .required(true)
            )
        ]);

    // In a command handler, show the modal
    interaction.create_response(
        ctx,
        serenity::CreateInteractionResponse::Modal(modal)
    ).await?;

    Ok(())
}
```

## Handling Modal Submissions

Modal submissions arrive as interactions with `InteractionType::MODAL_SUBMIT` (type 5).

```rust
async fn handle_modal_submit(
    ctx: &serenity::Context,
    interaction: &serenity::ModalSubmitInteraction,
    data: &Data
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id == "feedback_modal" {
        // Extract values from labels
        for component in &interaction.data.components {
            if let serenity::ActionRowComponent::InputText(input) = component {
                let custom_id = &input.custom_id;
                let value = &input.value;

                match custom_id.as_str() {
                    "feedback_text" => {
                        println!("Feedback: {}", value);
                    }
                    _ => {}
                }
            }
        }

        // Send confirmation
        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Thank you for your feedback!")
            )
        ).await?;
    }

    Ok(())
}
```

## Best Practices

1. **Keep labels concise**: Maximum 45 characters for better display
2. **Use descriptions wisely**: Provide helpful context without overwhelming users
3. **Set appropriate required flags**: Only require fields that are truly necessary
4. **Use placeholder text**: Guide users on expected input format
5. **Validate min/max values**: Set appropriate limits for text length and selection counts
6. **Clear custom_ids**: Use descriptive identifiers to easily process submissions
7. **Modals can only be shown in response to interactions**: They cannot be triggered proactively

## Limitations

- Modals can only be shown in response to interactions (button clicks, slash commands)
- Each modal can contain multiple Label components
- You cannot currently disable components in modals (disabled fields will cause errors)
- The `required` field defaults to `true` for most components

## Resources

See `docs/MODALS.md` for comprehensive documentation and the Serenity implementation examples.

---
name: components-v2-modals
description: Guide for creating Discord modals with Labels using Components v2 in Open Guard
metadata:
  version: "2.0"
---

# Discord Modals (Components v2)

## What this skill does
Quick reference for creating Discord modals using Components v2 with Labels in the Open Guard bot.

## When to use
- Create popup forms in Discord
- Collect structured user input (text, selects, files)
- Show modals in response to button clicks or commands

## Core Concepts

**Modal**: Popup form triggered by interactions (button click, command)
**Label**: Wrapper component with label + optional description for inputs

## Creating Modals

### Basic Structure
```rust
use poise::serenity_prelude as serenity;

let modal = serenity::CreateModal::new(custom_id, title)
    .components(vec![
        serenity::CreateModalComponent::Label(label),
        // More labels...
    ]);

// Show modal
interaction.create_response(
    ctx,
    serenity::CreateInteractionResponse::Modal(modal)
).await?;
```

### Label Types

#### 1. Text Input Label
```rust
let label = serenity::CreateLabel::input_text(
    "Email Address",  // Label text (max 45 chars)
    serenity::CreateInputText::new(
        serenity::InputTextStyle::Short,  // or Paragraph
        "email_field"  // custom_id
    )
    .placeholder("user@example.com")
    .min_length(5)
    .max_length(100)
    .required(true)
)
.description("We'll never share your email");  // Optional (max 100 chars)
```

#### 2. Select Menu Label
```rust
let select = serenity::CreateSelectMenu::new(
    "level_select",
    serenity::CreateSelectMenuKind::String {
        options: vec![
            serenity::CreateSelectMenuOption::new("Low", "1"),
            serenity::CreateSelectMenuOption::new("High", "2"),
        ].into()
    }
).required(true);

let label = serenity::CreateLabel::select_menu("Priority Level", select)
    .description("Choose priority");
```

#### 3. User Select Label
```rust
let select = serenity::CreateSelectMenu::new(
    "user_select",
    serenity::CreateSelectMenuKind::User {
        default_users: Some(vec![user_id].into())  // Optional default
    }
)
.min_values(1)
.max_values(3);

let label = serenity::CreateLabel::select_menu("Select Users", select);
```

#### 4. File Upload Label
```rust
let upload = serenity::CreateFileUpload::new("screenshot_upload")
    .min_values(1)
    .max_values(5)
    .required(true);

let label = serenity::CreateLabel::file_upload("Upload Screenshot", upload)
    .description("PNG or JPG only");
```

### Complete Example
```rust
pub fn build_feedback_modal(id: &str) -> serenity::CreateModal {
    serenity::CreateModal::new(
        format!("feedback_{}", id),
        "Feedback Form"
    )
    .components(vec![
        // Text input
        serenity::CreateModalComponent::Label(
            serenity::CreateLabel::input_text(
                "Your Feedback",
                serenity::CreateInputText::new(
                    serenity::InputTextStyle::Paragraph,
                    "feedback_text"
                )
                .placeholder("Write your feedback...")
                .required(true)
            )
            .description("Tell us what you think")
        ),
        // String select
        serenity::CreateModalComponent::Label(
            serenity::CreateLabel::select_menu(
                "Rating",
                serenity::CreateSelectMenu::new(
                    "rating_select",
                    serenity::CreateSelectMenuKind::String {
                        options: vec![
                            serenity::CreateSelectMenuOption::new("Excellent", "5"),
                            serenity::CreateSelectMenuOption::new("Good", "4"),
                            serenity::CreateSelectMenuOption::new("Average", "3"),
                        ].into()
                    }
                )
                .required(true)
            )
        ),
    ])
}
```

## Handling Modal Submissions

### Extract Data from Submission
```rust
async fn handle_modal_submit(
    ctx: &serenity::Context,
    interaction: &serenity::ModalSubmitInteraction,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    if custom_id.starts_with("feedback_") {
        // Extract string select value
        let rating = extract_string_select_value(
            &interaction.data.components,
            "rating_select"
        );

        // Extract text input value
        let feedback = extract_text_input_value(
            &interaction.data.components,
            "feedback_text"
        );

        // Extract selected users from resolved data
        let resolved = &interaction.data.resolved;
        let selected_users: Vec<_> = resolved.users.iter().collect();

        // Acknowledge modal
        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::Acknowledge
        ).await?;
    }
    Ok(())
}

// Helper: Extract string select value
fn extract_string_select_value(
    components: &[serenity::Component],
    target_custom_id: &str,
) -> Option<String> {
    for component in components {
        if let serenity::Component::Label(label) = component {
            if let serenity::LabelComponent::SelectMenu(menu) = &label.component {
                if &*menu.custom_id == target_custom_id {
                    return menu.values.first().map(|s| s.to_string());
                }
            }
        }
    }
    None
}

// Helper: Extract text input value
fn extract_text_input_value(
    components: &[serenity::Component],
    target_custom_id: &str,
) -> Option<String> {
    for component in components {
        if let serenity::Component::Label(label) = component {
            if let serenity::LabelComponent::InputText(input) = &label.component {
                if &*input.custom_id == target_custom_id {
                    return Some(input.value.clone());
                }
            }
        }
    }
    None
}
```

### Response Pattern
```rust
// Acknowledge modal submission
interaction.create_response(
    ctx,
    serenity::CreateInteractionResponse::Acknowledge
).await?;

// Then edit the original message
let edit = serenity::EditInteractionResponse::new()
    .content("Thank you for your feedback!")
    .components(vec![]);
interaction.edit_response(ctx, edit).await?;
```

## Best Practices

1. **Concise labels**: Max 45 characters
2. **Helpful descriptions**: Max 100 characters, use sparingly
3. **Required flags**: Only for truly necessary fields
4. **Unique custom_ids**: Use descriptive, unique identifiers
5. **Modals require interaction**: Cannot show proactively
6. **No disabled components**: Disabled fields cause errors in modals

## Codebase Examples
- `src/services/config/whitelist.rs:323-402` - Modal with user/level selects
- `src/services/config/whitelist.rs:714-755` - Extract modal data
- `src/services/event_manager/mod.rs:181-236` - Handle modal submissions

## See Also
- `components-v2-buttons` - Creating buttons
- `components-v2-selects` - Select menu details
- `serenity-interactions` - Interaction handling patterns

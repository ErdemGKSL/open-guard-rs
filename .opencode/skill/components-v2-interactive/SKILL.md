---
name: components-v2-interactive
description: Guide for creating interactive Discord components (buttons, selects, action rows) in Open Guard
metadata:
  version: "2.0"
---

# Interactive Components (Components v2)

## What this skill does
Complete reference for creating buttons, select menus, and action rows in the Open Guard bot.

## When to use
- Add buttons, select menus to messages
- Handle user interactions (clicks, selections)
- Build navigation, forms, and interactive UIs

## ⚠️ CRITICAL: Never Await in Commands
**Commands send components and return immediately.** Handle interactions in separate event handlers.
Transport data via `custom_id`, not by awaiting.

---

# Buttons

## Creating Buttons

### Basic Button
```rust
use poise::serenity_prelude as serenity;

serenity::CreateButton::new("button_id")
    .label("Click Me")
    .style(serenity::ButtonStyle::Primary)
```

### Button Styles
```rust
.style(serenity::ButtonStyle::Primary)    // Blurple
.style(serenity::ButtonStyle::Secondary)  // Gray
.style(serenity::ButtonStyle::Success)    // Green
.style(serenity::ButtonStyle::Danger)     // Red
.style(serenity::ButtonStyle::Link)       // Blue, opens URL
    .url("https://example.com")
```

### Button Options
```rust
// With emoji
.emoji('💾')

// Disabled
.disabled(true)

// Dynamic custom_id (embed data)
.custom_id(format!("btn_{}_{}", action_id, user_id))
```

## Common Button Patterns

### Confirm/Cancel
```rust
vec![
    serenity::CreateButton::new("confirm").label("Confirm").emoji('✅')
        .style(serenity::ButtonStyle::Success),
    serenity::CreateButton::new("cancel").label("Cancel").emoji('❌')
        .style(serenity::ButtonStyle::Danger),
]
```

### Toggle
```rust
serenity::CreateButton::new(format!("toggle_{}", id))
    .label(if enabled { "Enabled" } else { "Disabled" })
    .style(if enabled { serenity::ButtonStyle::Success } else { serenity::ButtonStyle::Danger })
```

### Pagination
```rust
let mut buttons = vec![];
if page > 0 {
    buttons.push(serenity::CreateButton::new(format!("prev_{}", page - 1))
        .label("Previous").style(serenity::ButtonStyle::Secondary));
}
if page < max_page {
    buttons.push(serenity::CreateButton::new(format!("next_{}", page + 1))
        .label("Next").style(serenity::ButtonStyle::Secondary));
}
```

## Handling Button Clicks

```rust
async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;
    
    // Parse custom_id
    if let Some(action_id) = custom_id.strip_prefix("btn_confirm_") {
        let id: i32 = action_id.parse()?;
        
        // Update message
        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Confirmed!")
                    .components(vec![])  // Remove buttons
            )
        ).await?;
    }
    
    Ok(())
}
```

---

# Select Menus

## Select Menu Types

### 1. String Select (Custom Options)
```rust
serenity::CreateSelectMenu::new(
    "select_id",
    serenity::CreateSelectMenuKind::String {
        options: vec![
            serenity::CreateSelectMenuOption::new("Label 1", "value1"),
            serenity::CreateSelectMenuOption::new("Label 2", "value2")
                .description("Optional description")
                .emoji('⭐')
                .default_selection(true),
        ].into()
    }
)
.placeholder("Choose an option")
.min_values(1)
.max_values(1)
```

### 2. User Select
```rust
serenity::CreateSelectMenu::new(
    "user_select",
    serenity::CreateSelectMenuKind::User {
        default_users: None  // Or Some(vec![user_id].into())
    }
)
.min_values(1)
.max_values(25)
```

### 3. Role Select
```rust
serenity::CreateSelectMenu::new(
    "role_select",
    serenity::CreateSelectMenuKind::Role {
        default_roles: Some(vec![role_id].into())
    }
)
.min_values(0)
.max_values(5)
```

### 4. Channel Select
```rust
serenity::CreateSelectMenu::new(
    "channel_select",
    serenity::CreateSelectMenuKind::Channel {
        channel_types: Some(vec![
            serenity::ChannelType::Text,
            serenity::ChannelType::Voice,
        ].into()),
        default_channels: None
    }
)
```

### 5. Mentionable Select (Users + Roles)
```rust
serenity::CreateSelectMenu::new(
    "mentionable_select",
    serenity::CreateSelectMenuKind::Mentionable {
        default_users: None,
        default_roles: None
    }
)
```

## Select Menu Options

### String Options
```rust
serenity::CreateSelectMenuOption::new("Label", "value")
    .description("Optional description")
    .emoji('🔥')
    .default_selection(true)
```

### Min/Max Values
```rust
.min_values(0)   // Optional selection
.max_values(5)   // Up to 5 selections

.min_values(1)   // Required
.max_values(1)   // Single selection
```

### Required (Modals Only)
```rust
.required(true)  // In modals
```

## Handling Select Menus

### Recommended Pattern: Acknowledge → Edit
```rust
async fn handle_select_menu(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;
    
    if custom_id == "my_select" {
        // Step 1: Acknowledge (shows "thinking")
        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::Acknowledge
        ).await?;
        
        // Step 2: Extract values
        let values = match &interaction.data.kind {
            serenity::ComponentInteractionDataKind::StringSelect { values } => values,
            _ => return Ok(()),
        };
        
        // Step 3: Process selection
        let result = process_selection(values).await?;
        
        // Step 4: Edit message
        let edit = serenity::EditInteractionResponse::new()
            .content(format!("Selected: {}", result))
            .components(vec![]);
        interaction.edit_response(ctx, edit).await?;
    }
    
    Ok(())
}
```

### Extract Different Select Types
```rust
// Channel select
if let serenity::ComponentInteractionDataKind::ChannelSelect { values } = &interaction.data.kind {
    for channel_id in values { /* ... */ }
}

// User select
if let serenity::ComponentInteractionDataKind::UserSelect { values } = &interaction.data.kind {
    for user_id in values { /* ... */ }
}

// Role select
if let serenity::ComponentInteractionDataKind::RoleSelect { values } = &interaction.data.kind {
    for role_id in values { /* ... */ }
}
```

---

# Action Rows

## What are Action Rows?
Containers that organize interactive components into rows.

**Limitations:**
- Max 5 action rows per message
- Max 5 buttons per row
- Only 1 select menu per row
- Cannot mix buttons and selects in same row

## Creating Action Rows

### Button Row
```rust
use serenity::CreateActionRow;

// Single button
CreateActionRow::buttons(vec![
    serenity::CreateButton::new("btn_1").label("Click"),
])

// Multiple buttons (max 5)
CreateActionRow::buttons(vec![
    serenity::CreateButton::new("btn_1").label("1").style(serenity::ButtonStyle::Primary),
    serenity::CreateButton::new("btn_2").label("2").style(serenity::ButtonStyle::Primary),
    serenity::CreateButton::new("btn_3").label("3").style(serenity::ButtonStyle::Primary),
])
```

### Select Menu Row
```rust
// Only 1 select per row
CreateActionRow::SelectMenu(
    serenity::CreateSelectMenu::new(
        "select_id",
        serenity::CreateSelectMenuKind::String {
            options: vec![
                serenity::CreateSelectMenuOption::new("Option 1", "opt1"),
            ].into()
        }
    )
)
```

## Using in Messages

### In Command Response
```rust
#[poise::command(slash_command)]
pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    let components = vec![
        serenity::CreateComponent::ActionRow(
            CreateActionRow::buttons(vec![
                serenity::CreateButton::new("btn_1").label("Button 1"),
            ])
        ),
        serenity::CreateComponent::ActionRow(
            CreateActionRow::SelectMenu(select_menu)
        ),
    ];
    
    ctx.send(poise::CreateReply::default()
        .content("Choose:")
        .components(components)
    ).await?;
    
    Ok(())
}
```

### In Interaction Response
```rust
interaction.create_response(
    ctx,
    serenity::CreateInteractionResponse::UpdateMessage(
        serenity::CreateInteractionResponseMessage::new()
            .content("Updated")
            .components(vec![
                serenity::CreateComponent::ActionRow(
                    CreateActionRow::buttons(vec![
                        serenity::CreateButton::new("new_btn").label("New")
                    ])
                )
            ])
    )
).await?;
```

## Layout Patterns

### Multiple Button Rows
```rust
vec![
    CreateComponent::ActionRow(
        CreateActionRow::buttons(vec![
            serenity::CreateButton::new("1_1").label("1"),
            serenity::CreateButton::new("1_2").label("2"),
        ])
    ),
    CreateComponent::ActionRow(
        CreateActionRow::buttons(vec![
            serenity::CreateButton::new("2_1").label("3"),
            serenity::CreateButton::new("2_2").label("4"),
        ])
    ),
]
```

### Mixed Components
```rust
vec![
    // Row 1: Buttons
    CreateComponent::ActionRow(CreateActionRow::buttons(vec![...])),
    // Row 2: Select menu
    CreateComponent::ActionRow(CreateActionRow::SelectMenu(select)),
    // Row 3: More buttons
    CreateComponent::ActionRow(CreateActionRow::buttons(vec![...])),
]
```

### Conditional Rows
```rust
let mut components = vec![];

// Always show main buttons
components.push(CreateComponent::ActionRow(
    CreateActionRow::buttons(vec![
        serenity::CreateButton::new("confirm").label("Confirm"),
        serenity::CreateButton::new("cancel").label("Cancel"),
    ])
));

// Conditionally add select
if show_options {
    components.push(CreateComponent::ActionRow(
        CreateActionRow::SelectMenu(options_menu)
    ));
}

// Conditionally add pagination
if total_pages > 1 {
    let mut page_buttons = vec![];
    if current_page > 0 {
        page_buttons.push(serenity::CreateButton::new("prev").label("◀"));
    }
    if current_page < total_pages - 1 {
        page_buttons.push(serenity::CreateButton::new("next").label("▶"));
    }
    components.push(CreateComponent::ActionRow(CreateActionRow::buttons(page_buttons)));
}
```

## Removing Components
```rust
// Clear all components
.components(vec![])

// Update specific rows
let new_components = vec![
    CreateComponent::ActionRow(
        CreateActionRow::buttons(vec![
            serenity::CreateButton::new("btn").label("Done").disabled(true)
        ])
    )
];
```

---

# Best Practices

1. **⚠️ Never await in commands** - Return immediately after sending
2. **Use acknowledge-then-edit** - Best for select menus
3. **Unique custom_ids** - Embed data for parsing
4. **Max 5 buttons per row** - Discord limit
5. **1 select per row** - Cannot combine with buttons
6. **Max 5 action rows** - Per message
7. **Disable when done** - Prevent re-clicking
8. **Clear when finished** - Remove components after completion
9. **Descriptive labels** - Make purpose clear
10. **Use emojis wisely** - Only when they clarify meaning

---

# Codebase Examples

**Buttons:**
- `src/services/config/whitelist.rs:307-314` - Add user button
- `src/services/setup/steps/logging.rs:34-36` - Button with emoji
- `src/services/config/builders.rs:53-59` - Toggle button
- `src/services/help.rs:56-70` - Pagination buttons

**Select Menus:**
- `src/services/setup/steps/logging.rs:9-18` - Channel select
- `src/services/config/modules/logging.rs:51-61` - Channel select with default
- `src/services/setup/steps/systems.rs:37-49` - String select
- `src/services/config/modules/role_protection.rs:14-31` - Multi-select with defaults

**Handlers:**
- `src/services/config/modules/logging.rs:179-188` - Handle channel select
- `src/services/config/modules/role_protection.rs:48-56` - Handle string select
- `src/services/event_manager/mod.rs` - Main interaction handler

---

# See Also
- `components-v2-modals` - Modals with selects and inputs
- `components-v2-layout` - Container, Section, Separator
- `components-v2-content` - Text Display, Media, Files
- `serenity-interactions` - Complete interaction handling guide

---
name: poise-commands
description: Guide for creating slash commands using the Poise framework in the Open Guard bot
metadata:
  version: "1.0"
---

# Poise Commands

## What this skill does
Provides comprehensive guidance on creating Discord slash commands using the Poise framework in the Open Guard bot.

## When to use
Use this skill when you need to:
- Create new slash commands
- Add command parameters and options
- Handle command responses
- Work with command contexts and data
- Implement subcommands and command groups

## ⚠️ CRITICAL: Do Not Await Interactions in Commands

**NEVER await button/select/menu interactions in command handlers.** This is a hard rule in Open Guard.

### Why?
- Commands must return immediately
- Awaiting interactions blocks the command handler
- Interaction handlers are separate and asynchronous
- Data should be transported via `custom_id` parsing

### The Pattern

```rust
// ❌ WRONG: Awaiting interaction in command
#[poise::command(slash_command)]
pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .components(vec![
            serenity::CreateButton::new("btn_click")
                .label("Click Me!")
                .style(serenity::ButtonStyle::Primary),
        ])
    ).await?;

    // WRONG: This blocks the command!
    let msg = ctx.channel_id().await_message(&ctx).await?;

    Ok(())
}

// ✅ CORRECT: Send and return immediately
#[poise::command(slash_command)]
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

// Handle interaction in separate event handler (serenity-interactions skill)
```

## Basic Command Structure

### Simple Command

```rust
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// A simple greeting command
#[poise::command(
    slash_command,
    guild_only,
    ephemeral  // Only the user who ran the command sees the response
)]
pub async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("Hello, World!")
    ).await?;

    Ok(())
}
```

### Command with Parameters

```rust
/// Say hello to someone
#[poise::command(
    slash_command,
    guild_only,
    ephemeral
)]
pub async fn greet(
    ctx: Context<'_>,
    #[description = "The user to greet"] user: serenity::User,
    #[description = "Optional greeting message"] message: Option<String>,
) -> Result<(), Error> {
    let greeting = message.unwrap_or_else(|| "Hello".to_string());
    ctx.send(poise::CreateReply::default()
        .content(format!("{} {}!", greeting, user.name))
    ).await?;

    Ok(())
}
```

## Command Options

### Required Permissions

```rust
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "BAN_MEMBERS"  // Only users with BAN_MEMBERS can use
)]
pub async fn ban_command(ctx: Context<'_>) -> Result<(), Error> {
    // Command logic
    Ok(())
}
```

Available permissions: `ADMINISTRATOR`, `BAN_MEMBERS`, `KICK_MEMBERS`, `MANAGE_MESSAGES`, `MANAGE_GUILD`, etc.

### Cooldowns

```rust
#[poise::command(
    slash_command,
    guild_only,
    cooldown = 10,  // 10 seconds cooldown
)]
pub async fn cool_command(ctx: Context<'_>) -> Result<(), Error> {
    // Command logic
    Ok(())
}
```

### Category

```rust
#[poise::command(
    slash_command,
    guild_only,
    category = "Moderation"  // Command category
)]
pub async fn mod_command(ctx: Context<'_>) -> Result<(), Error> {
    // Command logic
    Ok(())
}
```

### Hide in Help

```rust
#[poise::command(
    slash_command,
    guild_only,
    hide_in_help  // Hide from built-in help command
)]
pub async fn secret_command(ctx: Context<'_>) -> Result<(), Error> {
    // Command logic
    Ok(())
}
```

## Command Parameters

### Basic Types

```rust
pub async fn parameters(
    ctx: Context<'_>,
    #[description = "A string value"] text: String,
    #[description = "An integer value"] number: i64,
    #[description = "A boolean value"] flag: bool,
    #[description = "A Discord user"] user: serenity::User,
    #[description = "A Discord member"] member: serenity::Member,
    #[description = "A Discord role"] role: serenity::Role,
    #[description = "A Discord channel"] channel: serenity::GuildChannel,
) -> Result<(), Error> {
    // Command logic
    Ok(())
}
```

### Optional Parameters

```rust
pub async fn optional_params(
    ctx: Context<'_>,
    #[description = "Optional text"] text: Option<String>,
    #[description = "Optional user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let text = text.unwrap_or_else(|| "default".to_string());
    // Command logic
    Ok(())
}
```

### Rest Parameters (Multiple Values)

```rust
pub async fn rest_params(
    ctx: Context<'_>,
    #[description = "Multiple values"]
    #[rest]  // Captures all remaining arguments
    values: String,
) -> Result<(), Error> {
    // Command logic
    Ok(())
}
```

### Choice Enumerations

```rust
#[derive(poise::ChoiceParameter)]
enum Action {
    #[name = "Ban"]
    Ban,
    #[name = "Kick"]
    Kick,
    #[name = "Mute"]
    Mute,
}

pub async fn enum_param(
    ctx: Context<'_>,
    #[description = "Select an action"]
    action: Action,
) -> Result<(), Error> {
    match action {
        Action::Ban => { /* ban logic */ }
        Action::Kick => { /* kick logic */ }
        Action::Mute => { /* mute logic */ }
    }
    Ok(())
}
```

## Subcommands

### Command Group with Subcommands

```rust
#[poise::command(
    slash_command,
    subcommands("add", "remove", "list"),
    category = "MyModule"
)]
pub async fn manage(
    ctx: Context<'_>
) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("Use a subcommand: /manage add, /manage remove, /manage list")
    ).await?;
    Ok(())
}

/// Add something
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Item to add"] item: String,
) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content(format!("Added: {}", item))
    ).await?;
    Ok(())
}

/// Remove something
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Item to remove"] item: String,
) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content(format!("Removed: {}", item))
    ).await?;
    Ok(())
}

/// List all items
#[poise::command(slash_command)]
pub async fn list(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("List of items...")
    ).await?;
    Ok(())
}
```

## Working with Context

### Accessing Data

```rust
pub async fn access_data(ctx: Context<'_>) -> Result<(), Error> {
    // Access database
    let db = &ctx.data().db;

    // Access localization manager
    let l10n = &ctx.data().l10n;

    // Access logger service
    let logger = &ctx.data().logger;

    // Access punishment service
    let punishment = &ctx.data().punishment;

    // Access other services
    let whitelist = &ctx.data().whitelist;
    let cache = &ctx.data().cache;
    let jail = &ctx.data().jail;
    let temp_ban = &ctx.data().temp_ban;

    Ok(())
}
```

### Accessing Guild and User Info

```rust
pub async fn guild_info(ctx: Context<'_>) -> Result<(), Error> {
    // Get guild ID
    let guild_id = ctx.guild_id().unwrap();

    // Get author
    let author = ctx.author();

    // Get member
    let member = ctx.author_member().await.unwrap().unwrap();

    // Get locale
    let locale = ctx.interaction().locale;

    Ok(())
}
```

### Localization Helpers

```rust
use crate::services::localization::ContextL10nExt;

pub async fn localized(ctx: Context<'_>) -> Result<(), Error> {
    // Get localized strings for the user
    let l10n_user = ctx.l10n_user();

    // Get localized strings for the guild
    let l10n_guild = ctx.l10n_guild();

    // Use with FluentArgs
    use fluent::FluentArgs;
    let mut args = FluentArgs::new();
    args.set("user", "username");
    args.set("count", 42);

    let message = l10n_user.t("my-message-key", Some(&args));

    ctx.send(poise::CreateReply::default()
        .content(message)
    ).await?;

    Ok(())
}
```

## Response Types

### Ephemeral Response (Only User Sees)

```rust
#[poise::command(slash_command, ephemeral)]
pub async fn ephemeral_response(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("Only you can see this!")
    ).await?;
    Ok(())
}
```

### Public Response

```rust
#[poise::command(slash_command)]
pub async fn public_response(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("Everyone can see this!")
    ).await?;
    Ok(())
}
```

### Edit Response

```rust
#[poise::command(slash_command)]
pub async fn edit_response(ctx: Context<'_>) -> Result<(), Error> {
    let reply = ctx.send(poise::CreateReply::default()
        .content("Initial message")
    ).await?;

    // Wait and edit
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    reply.edit(ctx, poise::CreateReply::default()
        .content("Updated message!")
    ).await?;

    Ok(())
}
```

### Defer Long Operations

```rust
#[poise::command(slash_command)]
pub async fn long_operation(ctx: Context<'_>) -> Result<(), Error> {
    // Defer immediately to show "thinking" state
    ctx.defer_ephemeral().await?;

    // Perform long operation
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Send final response
    ctx.send(poise::CreateReply::default()
        .content("Operation completed!")
    ).await?;

    Ok(())
}
```

## Components in Responses

### Buttons (Do Not Await!)

```rust
pub async fn buttons(ctx: Context<'_>) -> Result<(), Error> {
    let setup_id = generate_unique_id();

    ctx.send(poise::CreateReply::default()
        .content("Click a button!")
        .components(vec![
            serenity::CreateActionRow::Buttons(vec![
                // Embed data in custom_id for later handling
                serenity::CreateButton::new(format!("button_1_{}", setup_id))
                    .label("Button 1")
                    .style(serenity::ButtonStyle::Primary),
                serenity::CreateButton::new(format!("button_2_{}", setup_id))
                    .label("Button 2")
                    .style(serenity::ButtonStyle::Danger),
            ])
        ])
    ).await?;

    // ✅ Command returns immediately - DO NOT await button clicks here
    Ok(())
}

// Button clicks are handled in separate event handler (see serenity-interactions skill)
```

### Select Menus

```rust
pub async fn selects(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("Select an option!")
        .components(vec![
            serenity::CreateActionRow::SelectMenu(
                serenity::CreateSelectMenu::new(
                    "select_menu_id",
                    serenity::CreateSelectMenuKind::String {
                        options: vec![
                            serenity::CreateSelectMenuOption::new("Option 1", "opt1"),
                            serenity::CreateSelectMenuOption::new("Option 2", "opt2"),
                            serenity::CreateSelectMenuOption::new("Option 3", "opt3"),
                        ]
                    }
                )
                .placeholder("Choose an option")
            )
        ])
    ).await?;

    Ok(())
}
```

## Error Handling

### Returning Errors

```rust
use anyhow::anyhow;

pub async fn error_handling(ctx: Context<'_>) -> Result<(), Error> {
    let value = Some(42);

    match value {
        Some(v) => {
            ctx.send(poise::CreateReply::default()
                .content(format!("Value: {}", v))
            ).await?;
        }
        None => {
            return Err(anyhow!("Value is missing!"));
        }
    }

    Ok(())
}
```

### User-Friendly Error Messages

```rust
pub async fn user_errors(ctx: Context<'_>) -> Result<(), Error> {
    if let Some(guild_id) = ctx.guild_id() {
        // Do something
        Ok(())
    } else {
        ctx.send(poise::CreateReply::default()
            .content("This command can only be used in a server!")
        ).await?;
        Ok(())  // Return Ok instead of error for better UX
    }
}
```

## Logging Actions

### Using Logger Service

```rust
use crate::services::logger::LogLevel;
use crate::db::entities::module_configs::ModuleType;
use fluent::FluentArgs;

pub async fn log_action(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let l10n_guild = ctx.l10n_guild();

    // Log the action
    let mut log_args = FluentArgs::new();
    log_args.set("modId", ctx.author().id.get().to_string());
    log_args.set("action", "ban");

    ctx.data().logger.log_context(
        &ctx,
        Some(ModuleType::ModerationProtection),
        LogLevel::Audit,
        &l10n_guild.t("log-command-title", None),
        &l10n_guild.t("log-command-desc", Some(&log_args)),
        vec![
            (&l10n_guild.t("log-field-moderator", None),
             format!("<@{}>", ctx.author().id)),
            (&l10n_guild.t("log-field-action", None), "Ban".to_string()),
        ],
    ).await?;

    Ok(())
}
```

## Best Practices

1. **⚠️ NEVER await interactions in commands**: Commands must return immediately
2. **⚠️ ALWAYS use separate handlers**: Handle interactions in event handlers, not commands
3. **⚠️ ALWAYS use custom_id parsing**: Transport data via custom_ids, not awaiting
4. **Use `defer_ephemeral()`** for long-running commands
5. **Check permissions** before performing actions
6. **Use localization** for all user-facing text
7. **Log important actions** using the logger service
8. **Handle errors gracefully** with user-friendly messages
9. **Use `ephemeral`** for sensitive or private information
10. **Validate inputs** before processing
11. **Use descriptive parameter names** for better Discord UI

## Examples from Codebase

- `src/modules/moderation_protection/commands/ban.rs` - Command with optional parameters
- `src/services/help.rs` - Help command implementation
- `src/services/config/mod.rs` - Configuration commands with subcommands
- `src/services/status.rs` - Simple status command
- `src/services/setup/mod.rs` - Multi-step command with components

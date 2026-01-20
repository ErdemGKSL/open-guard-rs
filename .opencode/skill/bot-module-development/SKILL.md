---
name: bot-module-development
description: Guide for creating new protection modules in the Open Guard Discord bot
metadata:
  version: "1.0"
---

# Bot Module Development

## What this skill does
Provides guidance on creating new protection modules, including commands, event handlers, and database entities in the Open Guard bot.

## When to use
Use this skill when you need to:
- Create a new protection or utility module
- Add new slash commands to existing modules
- Implement event handlers for Discord events
- Add database entities and migrations
- Configure module settings

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
            serenity::CreateButton::new("btn_confirm")
                .label("Confirm")
                .style(serenity::ButtonStyle::Success),
        ])
    ).await?;

    // WRONG: This blocks the command!
    let interaction = ctx.channel_id().await_component(&ctx).await?;

    Ok(())
}

// ✅ CORRECT: Send and return immediately
#[poise::command(slash_command)]
pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    let setup_id = generate_unique_id();

    ctx.send(poise::CreateReply::default()
        .content("Confirm the action")
        .components(vec![
            serenity::CreateButton::new(format!("btn_confirm_{}", setup_id))
                .label("Confirm")
                .style(serenity::ButtonStyle::Success),
        ])
    ).await?;

    // ✅ Command returns immediately
    Ok(())
}

// Handle interaction in event handler (src/services/event_manager/mod.rs)
async fn handle_component_interaction(
    ctx: &serenity::Context,
    interaction: &serenity::ComponentInteraction,
    data: &Data,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;

    if let Some(setup_id) = custom_id.strip_prefix("btn_confirm_") {
        // Process with extracted setup_id
        println!("Confirming action for setup: {}", setup_id);

        interaction.create_response(
            ctx,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("Action confirmed!")
                    .components(vec![])
            )
        ).await?;
    }

    Ok(())
}
```

## Module Structure

A module consists of three main parts:
1. **Commands** - Slash commands for user interaction
2. **Event Handlers** - Functions that respond to Discord events
3. **Module Definition** - Metadata and registration

### Basic Module Layout

```
src/modules/my_module/
├── mod.rs          # Module definition and registration
├── commands/       # Slash commands
│   └── mod.rs
└── events/         # Event handlers
    └── mod.rs
```

## Creating a New Module

### Step 1: Create Module Directory Structure

```bash
mkdir -p src/modules/my_module/commands src/modules/my_module/events
```

### Step 2: Create Module Definition (mod.rs)

```rust
// src/modules/my_module/mod.rs
pub mod commands;
pub mod events;

use crate::modules::{Module, ModuleDefinition};

pub fn module() -> Module {
    Module {
        definition: ModuleDefinition {
            id: "my_module",
            name_key: "module-my-module-name",
            desc_key: "module-my-module-desc",
        },
        commands: vec![
            commands::my_command(),
        ],
        event_handlers: vec![events::handler],
    }
}
```

### Step 3: Register the Module

Add your module to `src/modules/mod.rs`:

```rust
// src/modules/mod.rs
pub mod my_module;  // Add this

pub fn get_modules() -> Vec<Module> {
    vec![
        // existing modules...
        my_module::module(),  // Add this
    ]
}
```

### Step 4: Add Module Type to Database

Update `src/db/entities/module_configs.rs` to include your module:

```rust
// In the ModuleType enum
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "module_type")]
pub enum ModuleType {
    // existing types...
    #[sea_orm(string_value = "MyModule")]
    MyModule,
}

// Update the iter() method to include your module
pub fn module_type_from_string(s: &str) -> Option<ModuleType> {
    match s {
        // existing matches...
        "MyModule" => Some(ModuleType::MyModule),
        _ => None,
    }
}
```

## Creating Commands

### Basic Command Structure

```rust
// src/modules/my_module/commands/mod.rs
use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Command description for help text
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    ephemeral  // Makes response ephemeral (only user sees it)
)]
pub async fn my_command(
    ctx: Context<'_>,
    #[description = "User to target"] user: serenity::User,
    #[description = "Optional reason"] reason: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    // Command logic here
    let guild_id = ctx.guild_id().unwrap();

    // Get localization for user
    let l10n = ctx.l10n_user();

    // Perform action
    // ...

    // ⚠️ If adding buttons/selects, DO NOT await their interactions
    // See serenity-interactions skill for handling interaction handlers
    ctx.send(
        poise::CreateReply::default()
            .content("Command executed successfully!")
    )
    .await?;

    Ok(())
}
```

### ⚠️ Components and Interaction Handling

When adding buttons, selects, or modals to your commands:

```rust
pub async fn interactive_command(ctx: Context<'_>) -> Result<(), Error> {
    let setup_id = generate_unique_id();

    // Send components with data in custom_ids
    ctx.send(poise::CreateReply::default()
        .components(vec![
            serenity::CreateButton::new(format!("btn_confirm_{}", setup_id))
                .label("Confirm")
                .style(serenity::ButtonStyle::Success),
            serenity::CreateButton::new(format!("btn_cancel_{}", setup_id))
                .label("Cancel")
                .style(serenity::ButtonStyle::Danger),
        ])
    ).await?;

    // ✅ Command returns immediately - interactions handled separately
    Ok(())
}

// Interactions handled in src/services/event_manager/mod.rs
// Add handler there to parse custom_ids like "btn_confirm_{}", "btn_cancel_{}"
```

### Using Database Entities

```rust
use crate::db::entities::module_configs::ModuleType;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryFilter, Set};

pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();
    let db = &ctx.data().db;

    // Check if module is enabled for this guild
    let config = crate::db::entities::module_configs::Entity::find()
        .filter(crate::db::entities::module_configs::Column::GuildId.eq(guild_id.get() as i64))
        .filter(crate::db::entities::module_configs::Column::ModuleType.eq(ModuleType::MyModule))
        .one(db)
        .await?;

    if !config.map(|c| c.enabled).unwrap_or(false) {
        ctx.send(poise::CreateReply::default()
            .content("Module is not enabled for this server!")
        ).await?;
        return Ok(());
    }

    // Your command logic...

    Ok(())
}
```

### Subcommands

```rust
#[poise::command(
    slash_command,
    subcommands("add", "remove", "list")
)]
pub async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    ctx.send(poise::CreateReply::default()
        .content("Use a subcommand: /my_command add, /my_command remove, /my_command list")
    ).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn add(ctx: Context<'_>) -> Result<(), Error> {
    // Add logic
    Ok(())
}

#[poise::command(slash_command)]
pub async fn remove(ctx: Context<'_>) -> Result<(), Error> {
    // Remove logic
    Ok(())
}

#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    // List logic
    Ok(())
}
```

## Creating Event Handlers

### Basic Event Handler

```rust
// src/modules/my_module/events/mod.rs
use crate::modules::EventHandler;
use crate::{Data, Error};
use poise::serenity_prelude as serenity;

pub async fn handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            // Handle message events
            handle_message(ctx, new_message, data).await?;
        }
        serenity::FullEvent::GuildMemberAddition { new_member } => {
            // Handle member join events
            handle_member_add(ctx, new_member, data).await?;
        }
        serenity::FullEvent::AuditLogEntryCreate { entry, .. } => {
            // Handle audit log events
            handle_audit_log(ctx, entry, data).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &Data,
) -> Result<(), Error> {
    // Check if module is enabled
    if let Some(guild_id) = message.guild_id {
        let enabled = is_module_enabled(guild_id.get(), &data.db).await?;
        if !enabled {
            return Ok(());
        }

        // Your message handling logic here
    }

    Ok(())
}

async fn is_module_enabled(guild_id: u64, db: &sea_orm::DatabaseConnection) -> Result<bool, Error> {
    use crate::db::entities::module_configs;
    use sea_orm::{EntityTrait, QueryFilter};

    let config = module_configs::Entity::find()
        .filter(module_configs::Column::GuildId.eq(guild_id as i64))
        .filter(module_configs::Column::ModuleType.eq(module_configs::ModuleType::MyModule))
        .one(db)
        .await?;

    Ok(config.map(|c| c.enabled).unwrap_or(false))
}
```

### Available Event Types

Common Discord events you can handle:

- `Message { new_message }` - New messages
- `MessageUpdate { old_if_available, new, event }` - Message edits
- `MessageDelete { channel_id, deleted_message_id, guild_id }` - Message deletions
- `GuildMemberAddition { new_member }` - Members joining
- `GuildMemberRemoval { guild_id, user, member_data_if_available }` - Members leaving
- `GuildMemberUpdate { old_if_available, new }` - Member updates
- `AuditLogEntryCreate { entry, guild_id }` - Audit log entries
- `VoiceStateUpdate { old, new }` - Voice state changes
- `ChannelCreate { channel }` - Channel creation
- `ChannelDelete { channel }` - Channel deletion
- `RoleCreate { role }` - Role creation
- `RoleDelete { role_id, guild_id }` - Role deletion

## Localization

Add localization keys to your locale files (in `locales/`):

```ftl
module-my-module-name = My Module
module-my-module-desc = Description of what this module does

my-command-name = My Command
my-command-desc = Description of the command

my-command-success = Command executed successfully!
my-command-error = Failed to execute command: { $error }
```

Use localization in your commands:

```rust
use fluent::FluentArgs;

let l10n = ctx.l10n_user();
let mut args = FluentArgs::new();
args.set("error", error_message);
ctx.send(poise::CreateReply::default()
    .content(l10n.t("my-command-error", Some(&args)))
).await?;
```

## Logging

Use the built-in logger service:

```rust
use crate::services::logger::LogLevel;
use crate::db::entities::module_configs::ModuleType;

ctx.data().logger.log_context(
    &ctx,
    Some(ModuleType::MyModule),
    LogLevel::Audit,
    &l10n_guild.t("log-my-command-title", None),
    &l10n_guild.t("log-my-command-desc", Some(&log_args)),
    vec![
        (&l10n_guild.t("log-field-user", None), format!("<@{}>", user.id)),
        (&l10n_guild.t("log-field-reason", None), reason),
    ],
).await?;
```

## Best Practices

1. **⚠️ NEVER await interactions in commands**: Commands must return immediately
2. **⚠️ ALWAYS use separate handlers**: Handle interactions in event handlers, not commands
3. **⚠️ ALWAYS use custom_id parsing**: Transport data via custom_ids, not awaiting
4. **Check module enablement**: Always check if the module is enabled before performing actions
5. **Use localization**: Make all user-facing text translatable
6. **Handle errors gracefully**: Provide meaningful error messages to users
7. **Log important actions**: Use the logger service for audit trails
8. **Use `defer_ephemeral()`**: For long-running commands to provide immediate feedback
9. **Validate permissions**: Ensure users have required permissions before actions
10. **Use transactions**: When making multiple database changes, wrap in a transaction

## Testing Your Module

1. Build the project: `cargo build --release`
2. Run the bot: `cargo run --release`
3. Register commands (first time): `cargo run --release -- --publish GUILD_ID`
4. Test commands in Discord
5. Check logs for errors
6. Test interaction handlers (buttons, selects, modals)

## Examples

See existing modules for reference:
- `src/modules/moderation_protection/` - Moderation commands
- `src/modules/logging/` - Logging events
- `src/modules/role_protection/` - Role protection events
- `src/services/setup/mod.rs` - Complete example of multi-step interactions with custom_id parsing

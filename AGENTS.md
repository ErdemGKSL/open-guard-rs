# Open Guard Bot - Quick Guide

## Overview
Open Guard is a high-performance Discord security bot written in Rust using Serenity and Poise frameworks.

## Architecture

### Core Components
- **Modules**: Protection and utility features (`src/modules/`)
- **Services**: Shared logic and state (`src/services/`)
- **Database**: PostgreSQL with Sea-ORM (`src/db/`)

### Module System
Each module contains:
1. **Commands**: Slash commands for user interaction
2. **Event Handlers**: Respond to Discord events
3. **Registration**: Defined in `src/modules/mod.rs`

### Interaction Handling Pattern
**⚠️ Critical Rule**: Commands never await interactions.

- Commands send components and return immediately
- Interactions (buttons, selects, modals) handled in separate event handlers
- Data transported via `custom_id` parsing
- See: `src/services/event_manager/mod.rs`

### Key Services
- **Logger**: Audit logging to configured channels
- **Localization**: Multi-language support via Fluent
- **Config**: Per-guild module settings (enabled/disabled)
- **Punishment**: Ban/kick/timeout/jail operations
- **Whitelist**: Users/roles exempt from protections

### Database
- PostgreSQL with automatic migrations
- Entities: Guild configs, module settings, violations, whitelists, jails, temp bans
- Migrations: Applied on startup (`src/db/migrations/`)

## Components v2 API
**⚠️ CRITICAL**: This project uses Discord Components v2 exclusively. Components v1 is NOT supported.

### When to Use Components v2
Components v2 MUST be used for ALL user-facing messages that contain interactive elements or rich formatting:
- Slash command responses with buttons, selects, or formatted text
- Interaction responses (button clicks, select menus)
- Direct messages sent by the bot
- Any message that needs to display formatted content

### Components v2 Requirements
**⚠️ CRITICAL**: This project uses Discord Components v2 exclusively. Components v1 is NOT supported.

### Key Requirements:
1. **Always set the IS_COMPONENTS_V2 flag** on all messages/replies with components
2. **Never use `.content()`** - all content must be inside components as `TextDisplay`
3. **Wrap all components in a `Container`** - no standalone ActionRows
4. **Use `CreateContainerComponent::ActionRow`** inside containers

#### Component Structure:
```rust
// Command: Send components v2 reply
let mut inner_components = vec![];

// Add text content as TextDisplay
inner_components.push(serenity::CreateContainerComponent::TextDisplay(
    serenity::CreateTextDisplay::new("## Title\nDescription text here"),
));

// Add separator
inner_components.push(serenity::CreateContainerComponent::Separator(
    serenity::CreateSeparator::new(true), // true = spacing
));

// Add interactive components (buttons, selects, etc.)
inner_components.push(serenity::CreateContainerComponent::ActionRow(
    serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("btn_id")
            .label("Click me")
            .style(serenity::ButtonStyle::Primary)
    ].into()),
));

// Wrap in Container and send with v2 flag
ctx.send(poise::CreateReply::default()
    .components(vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components)
    )])
    .flags(serenity::MessageFlags::IS_COMPONENTS_V2)
    .ephemeral(true)
).await?;
```

#### Interaction Responses:
```rust
// Update message in interaction response
interaction.create_response(
    &ctx.http,
    serenity::CreateInteractionResponse::UpdateMessage(
        serenity::CreateInteractionResponseMessage::new()
            .components(components)  // NO .content()!
            .flags(serenity::MessageFlags::IS_COMPONENTS_V2),
    ),
).await?;

// Edit interaction response
interaction.edit_response(
    &ctx.http,
    serenity::EditInteractionResponse::new()
        .components(components)  // NO .content()!
        .flags(serenity::MessageFlags::IS_COMPONENTS_V2),
).await?;
```

#### Component Functions:
All UI builder functions should return `Vec<serenity::CreateComponent<'static>>`:
```rust
pub fn build_my_ui(l10n: &L10nProxy) -> Vec<serenity::CreateComponent<'static>> {
    let mut inner_components = vec![];
    
    // Add content and components
    inner_components.push(serenity::CreateContainerComponent::TextDisplay(
        serenity::CreateTextDisplay::new("Text here"),
    ));
    
    // Return wrapped in Container
    vec![serenity::CreateComponent::Container(
        serenity::CreateContainer::new(inner_components)
    )]
}
```

#### Common Mistakes to Avoid:
- ❌ Using `.content()` method
- ❌ Returning `(String, Vec<CreateComponent>)` tuples
- ❌ Forgetting `.flags(MessageFlags::IS_COMPONENTS_V2)`
- ❌ Using `CreateComponent::ActionRow` directly (only inside Container)
- ❌ Standalone ActionRows without Container wrapper

#### Examples:
- Good: `src/services/help.rs` - Proper components v2 usage
- Good: `src/services/logger.rs` - Container with accent color
- Good: `src/services/setup/steps/systems.rs` - Setup flow with v2

## Development Flow

### Adding a Feature
1. Create module with commands/events
2. Add database entities if needed
3. Register module in `src/modules/mod.rs`
4. Add ModuleType to `src/db/entities/module_configs.rs`
5. Add localization keys to `locales/`
6. **Important**: All UI functions must return `Vec<CreateComponent>` wrapped in Container, NOT `(String, Vec<CreateComponent>)` tuples

## Running

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/open-guard-rs

# Register commands
cargo run --release -- --publish GUILD_ID
```

## Configuration
- `.env`: Discord token, database URL, logging level
- Database: Per-guild module enablement and settings
- Locales: Fluent files in `locales/` directory

## Key Files
- `src/main.rs`: Entry point, service initialization
- `src/services/event_manager/mod.rs`: Central event dispatcher
- `src/services/setup/mod.rs`: Setup workflow example with interactions
- `docs/MODALS.md`: Modal and Label component documentation

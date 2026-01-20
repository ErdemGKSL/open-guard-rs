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

## Development Flow

### Adding a Feature
1. Create module with commands/events
2. Add database entities if needed
3. Register module in `src/modules/mod.rs`
4. Add ModuleType to `src/db/entities/module_configs.rs`
5. Add localization keys to `locales/`

### Interaction Development
```rust
// Command: Send components, return immediately
ctx.send(poise::CreateReply::default()
    .components(vec![
        CreateButton::new(format!("btn_action_{}", id))
    ])
).await?;

// Event Handler: Parse custom_id
if let Some(id) = custom_id.strip_prefix("btn_action_") {
    // Handle interaction
}
```

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

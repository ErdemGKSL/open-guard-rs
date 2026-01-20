---
name: database-migrations
description: Guide for creating Sea-ORM database migrations in the Open Guard bot
metadata:
  version: "1.0"
---

# Database Migrations

## What this skill does
Provides guidance on creating database migrations using Sea-ORM in the Open Guard bot, including creating tables, adding columns, and updating schemas.

## When to use
Use this skill when you need to:
- Create new database tables
- Add or modify columns in existing tables
- Create database indexes
- Update table structures
- Rollback migrations

## Migration Structure

Migrations are located in `src/db/migrations/` and follow a naming convention: `m{number}_{description}.rs`

Example: `m000012_create_temp_bans.rs`

## Creating a New Migration

### Step 1: Create Migration File

Create a new file in `src/db/migrations/` with a sequential number:

```bash
# List existing migrations to find the next number
ls src/db/migrations/

# Create new migration file (e.g., m000012 for the 12th migration)
touch src/db/migrations/m000012_your_migration_name.rs
```

### Step 2: Basic Migration Template

```rust
// src/db/migrations/m000012_your_migration_name.rs
use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m000012_your_migration_name"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create table, add columns, etc. in this function
        manager
            .create_table(
                Table::create()
                    .table(YourTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(YourTable::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(ColumnDef::new(YourTable::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(YourTable::UserId).big_integer().not_null())
                    .col(ColumnDef::new(YourTable::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Undo changes in this function
        manager
            .drop_table(Table::drop().table(YourTable::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum YourTable {
    Table,
    Id,
    GuildId,
    UserId,
    CreatedAt,
}
```

### Step 3: Register the Migration

Add your migration to `src/db/migrations/mod.rs`:

```rust
// src/db/migrations/mod.rs
mod m000001_create_guild_table;
mod m000002_add_log_channel;
// ... other migrations ...
mod m000011_create_member_roles_tracking;
mod m000012_your_migration_name;  // Add your migration

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m000001_create_guild_table::Migration),
            Box::new(m000002_add_log_channel::Migration),
            // ... other migrations ...
            Box::new(m000011_create_member_roles_tracking::Migration),
            Box::new(m000012_your_migration_name::Migration),  // Add your migration
        ]
    }
}
```

## Migration Examples

### Creating a New Table

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TempBans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TempBans::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(ColumnDef::new(TempBans::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(TempBans::UserId).big_integer().not_null())
                    .col(ColumnDef::new(TempBans::ExpiresAt).timestamp().not_null())
                    .col(ColumnDef::new(TempBans::Reason).string().null())
                    .col(ColumnDef::new(TempBans::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Create index for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_temp_bans_guild_user")
                    .table(TempBans::Table)
                    .col(TempBans::GuildId)
                    .col(TempBans::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TempBans::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TempBans {
    Table,
    Id,
    GuildId,
    UserId,
    ExpiresAt,
    Reason,
    CreatedAt,
}
```

### Adding Columns to Existing Table

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ModuleConfigs::Table)
                    .add_column(ColumnDef::new(ModuleConfigs::Enabled).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ModuleConfigs::Table)
                    .drop_column(ModuleConfigs::Enabled)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ModuleConfigs {
    Table,
    Enabled,
}
```

### Creating Enum Type

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum type
        manager
            .create_type(
                Type::create()
                    .as_enum(WhitelistLevel::Enum)
                    .value(WhitelistLevel::Invulnerable)
                    .value(WhitelistLevel::Protected)
                    .value(WhitelistLevel::ProtectedWithReason)
                    .to_owned(),
            )
            .await?;

        // Create table using the enum
        manager
            .create_table(
                Table::create()
                    .table(WhitelistUser::Table)
                    .col(
                        ColumnDef::new(WhitelistUser::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key()
                    )
                    .col(ColumnDef::new(WhitelistUser::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(WhitelistUser::UserId).big_integer().not_null())
                    .col(ColumnDef::new(WhitelistUser::Level)
                        .enumeration(WhitelistLevel::Enum, vec![WhitelistLevel::Invulnerable])
                        .not_null()
                        .default(WhitelistLevel::Protected))
                    .col(ColumnDef::new(WhitelistUser::ModuleType).string().null())
                    .col(ColumnDef::new(WhitelistUser::Reason).string().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WhitelistUser::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().enum_(WhitelistLevel::Enum).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum WhitelistLevel {
    Enum,
    Invulnerable,
    Protected,
    ProtectedWithReason,
}

#[derive(DeriveIden)]
enum WhitelistUser {
    Table,
    Id,
    GuildId,
    UserId,
    Level,
    ModuleType,
    Reason,
}
```

### Creating Foreign Key Constraints

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WhitelistRole::Table)
                    .col(ColumnDef::new(WhitelistRole::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key()
                    )
                    .col(ColumnDef::new(WhitelistRole::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(WhitelistRole::RoleId).big_integer().not_null())
                    .col(ColumnDef::new(WhitelistRole::Level)
                        .enumeration(WhitelistLevel::Enum, vec![WhitelistLevel::Invulnerable])
                        .not_null()
                        .default(WhitelistLevel::Protected))
                    .col(ColumnDef::new(WhitelistRole::ModuleType).string().null())
                    .col(ColumnDef::new(WhitelistRole::Reason).string().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_whitelist_role_guild")
                            .from(WhitelistRole::Table, WhitelistRole::GuildId)
                            .to(ModuleConfig::Table, ModuleConfig::GuildId)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WhitelistRole::Table).to_owned())
            .await
    }
}
```

### Creating Indexes

```rust
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Single column index
        manager
            .create_index(
                Index::create()
                    .name("idx_whitelist_user_guild")
                    .table(WhitelistUser::Table)
                    .col(WhitelistUser::GuildId)
                    .to_owned(),
            )
            .await?;

        // Composite index
        manager
            .create_index(
                Index::create()
                    .name("idx_whitelist_user_guild_module")
                    .table(WhitelistUser::Table)
                    .col(WhitelistUser::GuildId)
                    .col(WhitelistUser::ModuleType)
                    .to_owned(),
            )
            .await?;

        // Unique index
        manager
            .create_index(
                Index::create()
                    .name("idx_whitelist_user_unique")
                    .table(WhitelistUser::Table)
                    .col(WhitelistUser::GuildId)
                    .col(WhitelistUser::UserId)
                    .col(WhitelistUser::ModuleType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes in reverse order
        manager
            .drop_index(Index::drop().name("idx_whitelist_user_unique").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_whitelist_user_guild_module").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_whitelist_user_guild").to_owned())
            .await?;

        Ok(())
    }
}
```

## Creating Database Entities

After creating a migration, create the corresponding entity using `sea-orm-cli` or manually:

### Using sea-orm-cli (Recommended)

```bash
# Generate entity from existing database
sea-orm-cli generate entity -o src/db/entities \
  -u postgres://user:pass@localhost/open_guard \
  --with-serde
```

### Manual Entity Creation

```rust
// src/db/entities/temp_bans.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "temp_bans")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub guild_id: i64,
    #[sea_orm(indexed)]
    pub user_id: i64,
    pub expires_at: NaiveDateTime,
    pub reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::violations::Entity")]
    Violations,
}

impl ActiveModelBehavior for ActiveModel {}
```

Add to `src/db/entities/mod.rs`:

```rust
pub mod temp_bans;

pub use temp_bans::{Entity as TempBans, Model as TempBan, ActiveModel as TempBanActiveModel};
```

## Running Migrations

### Apply Migrations

```bash
# Run migrations when starting the bot (automatic)
cargo run --release

# Or refresh migrations (rollback then re-run)
cargo run --release -- --refresh-migrations 1
```

### Rollback Specific Migration

```bash
# Rollback the last migration
cargo run --release -- --refresh-migrations 1

# Rollback multiple migrations
cargo run --release -- --refresh-migrations 3
```

## Best Practices

1. **Always provide both up() and down() methods**: Ensure migrations can be reversed
2. **Use descriptive migration names**: Make it clear what the migration does
3. **Test migrations**: Run migrations on a test database first
4. **Use indexes for frequently queried columns**: Improve query performance
5. **Use foreign keys**: Ensure data integrity
6. **Consider CASCADE deletes**: Decide if related data should be deleted
7. **Keep migrations atomic**: Each migration should do one logical thing
8. **Version control migrations**: Never modify committed migrations, create new ones

## Common Column Types

- `integer()` - 32-bit integer
- `big_integer()` - 64-bit integer
- `string()` - Text (variable length)
- `string_len(n)` - Text with max length n
- `text()` - Large text
- `boolean()` - Boolean
- `decimal()` - Decimal number
- `date()` - Date
- `time()` - Time
- `timestamp()` - Timestamp
- `timestamp_with_time_zone()` - Timestamp with timezone
- `json()` / `json_binary()` - JSON data

## Common Column Modifiers

- `.not_null()` - Cannot be NULL
- `.default(value)` - Default value
- `.auto_increment()` - Auto-incrementing
- `.primary_key()` - Primary key
- `.unique()` - Unique constraint

## Examples from Codebase

See existing migrations for reference:
- `m000001_create_guild_table.rs` - Basic table creation
- `m000003_create_module_config.rs` - Enum types and JSON columns
- `m000006_create_whitelist_tables.rs` - Multiple related tables with foreign keys
- `m000009_create_temp_bans.rs` - Table with indexes

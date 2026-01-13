use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create whitelist_roles table
        manager
            .create_table(
                Table::create()
                    .table(WhitelistRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WhitelistRoles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WhitelistRoles::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WhitelistRoles::RoleId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WhitelistRoles::Level).string().not_null())
                    .col(ColumnDef::new(WhitelistRoles::ModuleType).string().null())
                    .to_owned(),
            )
            .await?;

        // Create HASH index on whitelist_roles (guild_id)
        manager
            .create_index(
                Index::create()
                    .name("idx-whitelist-roles-guild-hash")
                    .table(WhitelistRoles::Table)
                    .col(WhitelistRoles::GuildId)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await?;

        // Create UNIQUE index on whitelist_roles (guild_id, role_id)
        manager
            .create_index(
                Index::create()
                    .name("idx-whitelist-roles-guild-role-unique")
                    .table(WhitelistRoles::Table)
                    .col(WhitelistRoles::GuildId)
                    .col(WhitelistRoles::RoleId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create whitelist_users table
        manager
            .create_table(
                Table::create()
                    .table(WhitelistUsers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WhitelistUsers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(WhitelistUsers::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(WhitelistUsers::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(WhitelistUsers::Level).string().not_null())
                    .col(ColumnDef::new(WhitelistUsers::ModuleType).string().null())
                    .to_owned(),
            )
            .await?;

        // Create UNIQUE index on whitelist_users (guild_id, user_id)
        manager
            .create_index(
                Index::create()
                    .name("idx-whitelist-users-guild-user-unique")
                    .table(WhitelistUsers::Table)
                    .col(WhitelistUsers::GuildId)
                    .col(WhitelistUsers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WhitelistUsers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(WhitelistRoles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum WhitelistRoles {
    Table,
    Id,
    GuildId,
    RoleId,
    Level,
    ModuleType,
}

#[derive(DeriveIden)]
enum WhitelistUsers {
    Table,
    Id,
    GuildId,
    UserId,
    Level,
    ModuleType,
}

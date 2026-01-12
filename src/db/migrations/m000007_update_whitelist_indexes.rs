use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop old indexes if they exist
        manager
            .drop_index(
                Index::drop()
                    .name("idx-whitelist-roles-guild")
                    .table(WhitelistRoles::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        
        manager
            .drop_index(
                Index::drop()
                    .name("idx-whitelist-users-guild")
                    .table(WhitelistUsers::Table)
                    .if_exists()
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
                    .if_not_exists()
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
                    .if_not_exists()
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
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-whitelist-users-guild-user-unique")
                    .table(WhitelistUsers::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-whitelist-roles-guild-role-unique")
                    .table(WhitelistRoles::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx-whitelist-roles-guild-hash")
                    .table(WhitelistRoles::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum WhitelistRoles {
    Table,
    GuildId,
    RoleId,
}

#[derive(DeriveIden)]
enum WhitelistUsers {
    Table,
    GuildId,
    UserId,
}


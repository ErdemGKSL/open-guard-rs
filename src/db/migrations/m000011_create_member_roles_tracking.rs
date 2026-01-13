use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create logging_guilds table to track when a guild last used logging
        manager
            .create_table(
                Table::create()
                    .table(LoggingGuilds::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LoggingGuilds::GuildId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(LoggingGuilds::LastAccessedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create member_old_roles table to store member's roles for logging
        manager
            .create_table(
                Table::create()
                    .table(MemberOldRoles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MemberOldRoles::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MemberOldRoles::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(MemberOldRoles::RoleIds).json().not_null())
                    .col(
                        ColumnDef::new(MemberOldRoles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(MemberOldRoles::GuildId)
                            .col(MemberOldRoles::UserId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-member-old-roles-guild")
                            .from(MemberOldRoles::Table, MemberOldRoles::GuildId)
                            .to(LoggingGuilds::Table, LoggingGuilds::GuildId)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create HASH index on member_old_roles (guild_id)
        manager
            .create_index(
                Index::create()
                    .name("idx-member-old-roles-guild-hash")
                    .table(MemberOldRoles::Table)
                    .col(MemberOldRoles::GuildId)
                    .index_type(IndexType::Hash)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MemberOldRoles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(LoggingGuilds::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum LoggingGuilds {
    Table,
    GuildId,
    LastAccessedAt,
}

#[derive(DeriveIden)]
enum MemberOldRoles {
    Table,
    GuildId,
    UserId,
    RoleIds,
    UpdatedAt,
}

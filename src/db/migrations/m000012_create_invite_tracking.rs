use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create invite_snapshots table
        manager
            .create_table(
                Table::create()
                    .table(InviteSnapshots::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InviteSnapshots::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InviteSnapshots::Code)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(InviteSnapshots::InviterId).big_integer())
                    .col(ColumnDef::new(InviteSnapshots::ChannelId).big_integer())
                    .col(
                        ColumnDef::new(InviteSnapshots::Uses)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(ColumnDef::new(InviteSnapshots::MaxUses).integer())
                    .col(ColumnDef::new(InviteSnapshots::MaxAge).integer())
                    .col(
                        ColumnDef::new(InviteSnapshots::Temporary)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(InviteSnapshots::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InviteSnapshots::ExpiresAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(InviteSnapshots::InviteType)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InviteSnapshots::LastSyncedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(InviteSnapshots::GuildId)
                            .col(InviteSnapshots::Code),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for invite_snapshots
        manager
            .create_index(
                Index::create()
                    .name("idx-invite-snapshots-guild-inviter")
                    .table(InviteSnapshots::Table)
                    .col(InviteSnapshots::GuildId)
                    .col(InviteSnapshots::InviterId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-invite-snapshots-expires")
                    .table(InviteSnapshots::Table)
                    .col(InviteSnapshots::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // Create invite_events table
        manager
            .create_table(
                Table::create()
                    .table(InviteEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InviteEvents::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InviteEvents::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InviteEvents::EventType)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(ColumnDef::new(InviteEvents::InviteCode).string_len(32))
                    .col(ColumnDef::new(InviteEvents::InviterId).big_integer())
                    .col(ColumnDef::new(InviteEvents::TargetUserId).big_integer())
                    .col(
                        ColumnDef::new(InviteEvents::JoinType)
                            .string_len(32),
                    )
                    .col(ColumnDef::new(InviteEvents::Metadata).json())
                    .col(
                        ColumnDef::new(InviteEvents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .extra("DEFAULT NOW()".to_string()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for invite_events
        manager
            .create_index(
                Index::create()
                    .name("idx-invite-events-guild-time")
                    .table(InviteEvents::Table)
                    .col(InviteEvents::GuildId)
                    .col((InviteEvents::CreatedAt, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-invite-events-inviter")
                    .table(InviteEvents::Table)
                    .col(InviteEvents::GuildId)
                    .col(InviteEvents::InviterId)
                    .col(InviteEvents::EventType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-invite-events-target")
                    .table(InviteEvents::Table)
                    .col(InviteEvents::GuildId)
                    .col(InviteEvents::TargetUserId)
                    .col(InviteEvents::EventType)
                    .to_owned(),
            )
            .await?;

        // Create invite_stats table
        manager
            .create_table(
                Table::create()
                    .table(InviteStats::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InviteStats::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InviteStats::UserId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InviteStats::TotalInvites)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InviteStats::CurrentMembers)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InviteStats::LeftMembers)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InviteStats::FakeMembers)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(InviteStats::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(InviteStats::GuildId)
                            .col(InviteStats::UserId),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for invite_stats
        manager
            .create_index(
                Index::create()
                    .name("idx-invite-stats-guild-total")
                    .table(InviteStats::Table)
                    .col(InviteStats::GuildId)
                    .col((InviteStats::TotalInvites, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-invite-stats-guild-current")
                    .table(InviteStats::Table)
                    .col(InviteStats::GuildId)
                    .col((InviteStats::CurrentMembers, IndexOrder::Desc))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(InviteStats::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(InviteEvents::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(InviteSnapshots::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum InviteSnapshots {
    Table,
    GuildId,
    Code,
    InviterId,
    ChannelId,
    Uses,
    MaxUses,
    MaxAge,
    Temporary,
    CreatedAt,
    ExpiresAt,
    InviteType,
    LastSyncedAt,
}

#[derive(DeriveIden)]
enum InviteEvents {
    Table,
    Id,
    GuildId,
    EventType,
    InviteCode,
    InviterId,
    TargetUserId,
    JoinType,
    Metadata,
    CreatedAt,
}

#[derive(DeriveIden)]
enum InviteStats {
    Table,
    GuildId,
    UserId,
    TotalInvites,
    CurrentMembers,
    LeftMembers,
    FakeMembers,
    UpdatedAt,
}

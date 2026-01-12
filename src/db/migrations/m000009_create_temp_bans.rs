use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

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
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TempBans::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(TempBans::UserId).big_integer().not_null())
                    .col(ColumnDef::new(TempBans::ExpiresAt).date_time().not_null())
                    .col(ColumnDef::new(TempBans::Reason).string())
                    .to_owned(),
            )
            .await?;

        // Index for faster cleanup
        manager
            .create_index(
                Index::create()
                    .name("idx-temp-bans-expires-at")
                    .table(TempBans::Table)
                    .col(TempBans::ExpiresAt)
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
}


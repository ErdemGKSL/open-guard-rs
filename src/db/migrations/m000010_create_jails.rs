use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Jails::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Jails::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Jails::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(Jails::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Jails::OldRoles).json().not_null())
                    .col(ColumnDef::new(Jails::ExpiresAt).date_time())
                    .col(ColumnDef::new(Jails::Reason).string())
                    .to_owned(),
            )
            .await?;

        // Index for faster cleanup
        manager
            .create_index(
                Index::create()
                    .name("idx-jails-expires-at")
                    .table(Jails::Table)
                    .col(Jails::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Jails::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Jails {
    Table,
    Id,
    GuildId,
    UserId,
    OldRoles,
    ExpiresAt,
    Reason,
}


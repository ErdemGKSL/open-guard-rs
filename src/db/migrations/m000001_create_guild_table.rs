use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GuildConfigs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GuildConfigs::GuildId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GuildConfigs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GuildConfigs {
    Table,
    GuildId,
}


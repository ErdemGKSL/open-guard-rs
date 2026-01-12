use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(GuildConfigs::Table)
                    .add_column(
                        ColumnDef::new(GuildConfigs::LogChannelId)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(GuildConfigs::Table)
                    .drop_column(GuildConfigs::LogChannelId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum GuildConfigs {
    Table,
    LogChannelId,
}


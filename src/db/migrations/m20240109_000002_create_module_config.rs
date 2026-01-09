use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ModuleConfigs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ModuleConfigs::GuildId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModuleConfigs::ModuleType)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ModuleConfigs::LogChannelId)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ModuleConfigs::Config)
                            .json_binary()
                            .not_null()
                            .default(Expr::value("{}")),
                    )
                    .primary_key(
                        Index::create()
                            .col(ModuleConfigs::GuildId)
                            .col(ModuleConfigs::ModuleType),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ModuleConfigs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ModuleConfigs {
    Table,
    GuildId,
    ModuleType,
    LogChannelId,
    Config,
}

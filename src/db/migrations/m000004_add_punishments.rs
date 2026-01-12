use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ModuleConfigs::Table)
                    .add_column(
                        ColumnDef::new(ModuleConfigs::Punishment)
                            .string_len(32)
                            .not_null()
                            .default("none"),
                    )
                    .add_column(
                        ColumnDef::new(ModuleConfigs::PunishmentAt)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(ModuleConfigs::PunishmentAtInterval)
                            .integer()
                            .not_null()
                            .default(5),
                    )
                    .add_column(
                        ColumnDef::new(ModuleConfigs::Revert)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GuildConfigs::Table)
                    .add_column(
                        ColumnDef::new(GuildConfigs::JailRoleId)
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
                    .table(ModuleConfigs::Table)
                    .drop_column(ModuleConfigs::Punishment)
                    .drop_column(ModuleConfigs::PunishmentAt)
                    .drop_column(ModuleConfigs::PunishmentAtInterval)
                    .drop_column(ModuleConfigs::Revert)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(GuildConfigs::Table)
                    .drop_column(GuildConfigs::JailRoleId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ModuleConfigs {
    Table,
    Punishment,
    PunishmentAt,
    PunishmentAtInterval,
    Revert,
}

#[derive(DeriveIden)]
enum GuildConfigs {
    Table,
    JailRoleId,
}


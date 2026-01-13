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
                        ColumnDef::new(Alias::new("enabled"))
                            .boolean()
                            .not_null()
                            .default(false),
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
                    .drop_column(Alias::new("enabled"))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ModuleConfigs {
    Table,
}

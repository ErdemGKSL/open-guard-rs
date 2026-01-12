use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db_backend = manager.get_database_backend();

        // Create the parent partitioned table
        // Note: In Postgres, all columns used in partitioning must be part of the primary key.
        manager
            .get_connection()
            .execute(Statement::from_string(
                db_backend,
                r#"CREATE TABLE "violations" (
                    "id" SERIAL,
                    "guild_id" BIGINT NOT NULL,
                    "user_id" BIGINT NOT NULL,
                    "module_type" VARCHAR(32) NOT NULL,
                    "count" INTEGER NOT NULL DEFAULT 1,
                    "last_violation_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    PRIMARY KEY ("id", "last_violation_at")
                ) PARTITION BY RANGE ("last_violation_at");"#
                    .to_owned(),
            ))
            .await?;

        // Create a default partition to handle data by default
        manager
            .get_connection()
            .execute(Statement::from_string(
                db_backend,
                r#"CREATE TABLE "violations_default" PARTITION OF "violations" DEFAULT;"#
                    .to_owned(),
            ))
            .await?;

        // Composite index for fast lookups by guild/user/module
        manager
            .create_index(
                Index::create()
                    .name("idx-violations-lookup")
                    .table(Violations::Table)
                    .col(Violations::GuildId)
                    .col(Violations::UserId)
                    .col(Violations::ModuleType)
                    .to_owned(),
            )
            .await?;

        // Hash index for user_id
        manager
            .create_index(
                Index::create()
                    .name("idx-violations-hash-user")
                    .table(Violations::Table)
                    .col(Violations::UserId)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await?;

        // Hash index for guild_id
        manager
            .create_index(
                Index::create()
                    .name("idx-violations-hash-guild")
                    .table(Violations::Table)
                    .col(Violations::GuildId)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await?;

        // Index on id alone for fast lookups (e.g. for auto-complete)
        manager
            .create_index(
                Index::create()
                    .name("idx-violations-id")
                    .table(Violations::Table)
                    .col(Violations::Id)
                    .to_owned(),
            )
            .await?;

        // BRIN index for last_violation_at (efficient for time-series lookups on large tables)
        manager
            .create_index(
                Index::create()
                    .name("idx-violations-brin-last-at")
                    .table(Violations::Table)
                    .col(Violations::LastViolationAt)
                    .index_type(IndexType::Custom(SeaRc::new(Brin)))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Violations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Violations {
    Table,
    Id,
    GuildId,
    UserId,
    ModuleType,
    Count,
    LastViolationAt,
}

#[derive(DeriveIden)]
struct Brin;


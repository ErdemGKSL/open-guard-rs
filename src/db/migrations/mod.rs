use sea_orm_migration::prelude::*;

pub mod m000001_create_guild_table;
pub mod m20240109_000001_add_log_channel;
pub mod m20240109_000002_create_module_config;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m000001_create_guild_table::Migration),
            Box::new(m20240109_000001_add_log_channel::Migration),
            Box::new(m20240109_000002_create_module_config::Migration),
        ]
    }
}

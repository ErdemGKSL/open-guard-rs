#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused)]

pub mod m000001_create_guild_table;
pub mod m000002_add_log_channel;
pub mod m000003_create_module_config;
pub mod m000004_add_punishments;
pub mod m000005_create_violations;
pub mod m000006_create_whitelist_tables;
pub mod m000007_update_whitelist_indexes;

use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m000001_create_guild_table::Migration),
            Box::new(m000002_add_log_channel::Migration),
            Box::new(m000003_create_module_config::Migration),
            Box::new(m000004_add_punishments::Migration),
            Box::new(m000005_create_violations::Migration),
            Box::new(m000006_create_whitelist_tables::Migration),
            Box::new(m000007_update_whitelist_indexes::Migration),
        ]
    }
}

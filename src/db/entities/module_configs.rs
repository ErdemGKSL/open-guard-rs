use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    poise::ChoiceParameter,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum ModuleType {
    #[sea_orm(string_value = "config")]
    Config,
    #[sea_orm(string_value = "channel_protection")]
    ChannelProtection,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    poise::ChoiceParameter,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum PunishmentType {
    #[sea_orm(string_value = "none")]
    None,
    #[sea_orm(string_value = "unperm")]
    Unperm,
    #[sea_orm(string_value = "ban")]
    Ban,
    #[sea_orm(string_value = "kick")]
    Kick,
    #[sea_orm(string_value = "jail")]
    Jail,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize, Eq)]
#[sea_orm(table_name = "module_configs")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub module_type: ModuleType,
    pub log_channel_id: Option<i64>,
    pub punishment: PunishmentType,
    pub punishment_at: i32,
    pub punishment_at_interval: i32,
    pub revert: bool,
    pub config: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelProtectionModuleConfig {
    pub lock_new_channels: bool,
    pub audit_log_channel_id: Option<u64>,
}

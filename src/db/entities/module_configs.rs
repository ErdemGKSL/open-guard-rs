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
    #[sea_orm(string_value = "channel_protection")]
    ChannelProtection,
    #[sea_orm(string_value = "channel_permission_protection")]
    ChannelPermissionProtection,
    #[sea_orm(string_value = "role_protection")]
    RoleProtection,
    #[sea_orm(string_value = "role_permission_protection")]
    RolePermissionProtection,
    #[sea_orm(string_value = "member_permission_protection")]
    MemberPermissionProtection,
    #[sea_orm(string_value = "bot_adding_protection")]
    BotAddingProtection,
    #[sea_orm(string_value = "moderation_protection")]
    ModerationProtection,
    #[sea_orm(string_value = "logging")]
    Logging,
}

impl std::fmt::Display for ModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleType::ChannelProtection => write!(f, "channel_protection"),
            ModuleType::ChannelPermissionProtection => write!(f, "channel_permission_protection"),
            ModuleType::RoleProtection => write!(f, "role_protection"),
            ModuleType::RolePermissionProtection => write!(f, "role_permission_protection"),
            ModuleType::MemberPermissionProtection => write!(f, "member_permission_protection"),
            ModuleType::BotAddingProtection => write!(f, "bot_adding_protection"),
            ModuleType::ModerationProtection => write!(f, "moderation_protection"),
            ModuleType::Logging => write!(f, "logging"),
        }
    }
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
    pub enabled: bool,
    pub revert: bool,
    pub config: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ChannelProtectionModuleConfig {
    #[serde(default)]
    pub ignore_private_channels: bool,
    #[serde(default)]
    pub punish_when: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ChannelPermissionProtectionModuleConfig {
    #[serde(default)]
    pub ignore_private_channels: bool,
    #[serde(default)]
    pub punish_when: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RoleProtectionModuleConfig {
    #[serde(default)]
    pub punish_when: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RolePermissionProtectionModuleConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MemberPermissionProtectionModuleConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BotAddingProtectionModuleConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ModerationProtectionModuleConfig {
    #[serde(default)]
    pub punish_when: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LoggingModuleConfig {
    #[serde(default)]
    pub log_messages: bool,
    #[serde(default)]
    pub log_voice: bool,
    #[serde(default)]
    pub log_membership: bool,
    #[serde(default)]
    pub message_log_channel_id: Option<i64>,
    #[serde(default)]
    pub voice_log_channel_id: Option<i64>,
    #[serde(default)]
    pub membership_log_channel_id: Option<i64>,
}


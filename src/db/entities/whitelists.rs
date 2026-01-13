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
pub enum WhitelistLevel {
    #[sea_orm(string_value = "head")]
    Head,
    #[sea_orm(string_value = "admin")]
    Admin,
    #[sea_orm(string_value = "invulnerable")]
    Invulnerable,
}

impl std::fmt::Display for WhitelistLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WhitelistLevel::Head => write!(f, "head"),
            WhitelistLevel::Admin => write!(f, "admin"),
            WhitelistLevel::Invulnerable => write!(f, "invulnerable"),
        }
    }
}

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "invite_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub guild_id: i64,
    pub event_type: String,
    pub invite_code: Option<String>,
    pub inviter_id: Option<i64>,
    pub target_user_id: Option<i64>,
    pub join_type: Option<String>,
    pub metadata: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

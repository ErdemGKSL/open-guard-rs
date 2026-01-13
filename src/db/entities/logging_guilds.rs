use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "logging_guilds")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    pub last_accessed_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::member_old_roles::Entity")]
    MemberOldRoles,
}

impl Related<super::member_old_roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MemberOldRoles.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

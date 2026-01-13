use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "member_old_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub guild_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,
    pub role_ids: Json,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::logging_guilds::Entity",
        from = "Column::GuildId",
        to = "super::logging_guilds::Column::GuildId"
    )]
    LoggingGuild,
}

impl Related<super::logging_guilds::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LoggingGuild.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "connect_townhalls")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i64,
    pub townhall_code: String,
    pub title: String,
    pub meeting_date: String,
    pub meeting_time: String,
    pub mode: String,
    pub host: String,
    pub agenda: Option<String>,
    pub attendees_count: i32,
    pub status: String,
    pub minutes: Option<String>,
    pub actions: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "connect_comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub comment_id: String,
    pub post_id: String,
    pub author_id: String,
    pub author_name: String,
    pub body: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "connect_posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub post_id: String,
    pub author_id: String,
    pub author_name: String,
    pub author_dept: Option<String>,
    pub post_type: String,
    pub to_employee_id: Option<String>,
    pub to_employee_name: Option<String>,
    pub body: String,
    pub group_id: Option<String>,
    pub pinned: bool,
    pub likes_count: i32,
    pub comments_count: i32,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

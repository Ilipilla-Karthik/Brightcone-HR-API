use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "hr_policies")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i64,
    pub policy_id: String,
    pub name: String,
    pub category: String,
    pub version: String,
    pub effective_date: Option<String>,
    pub next_review_date: Option<String>,
    pub owner: String,
    pub file_name: Option<String>,
    pub ack_required: bool,
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

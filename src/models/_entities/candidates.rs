use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "candidates")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i64,
    pub cand_id: Uuid,
    pub candidate_code: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: String,
    pub job_id: Option<Uuid>,
    pub job_title: Option<String>,
    pub match_score: Option<i32>,
    pub stage: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "review_cycles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub stage: i32, // 0: Goal setting, 1: Self-review, 2: Manager review, 3: Calibration, 4: Results shared
    pub self_review_due: String,
    pub status: String, // "Active", "Archived", "Completed"
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::cycle_reviews::Entity")]
    CycleReviews,
}

impl Related<super::cycle_reviews::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CycleReviews.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

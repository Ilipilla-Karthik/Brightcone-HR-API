use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "cycle_reviews")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub cycle_id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub manager_name: String,
    pub department: String,
    pub self_review_status: String,
    pub self_rating: Option<String>,
    pub goals_count: i32,
    pub rating: Option<String>,
    pub recommended_increment_percentage: Option<f64>,
    pub rated_by: Option<String>,
    pub rated_at: Option<DateTimeWithTimeZone>,
    pub last_reminded_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::review_cycles::Entity",
        from = "Column::CycleId",
        to = "super::review_cycles::Column::Id"
    )]
    ReviewCycle,
}

impl Related<super::review_cycles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ReviewCycle.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

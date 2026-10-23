use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "course_assignments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub course_id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub progress: i32,
    pub status: String, // "Not started", "In progress", "Completed"
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::courses::Entity",
        from = "Column::CourseId",
        to = "super::courses::Column::Id"
    )]
    Courses,
}

impl Related<super::courses::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Courses.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

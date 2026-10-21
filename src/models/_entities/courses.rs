use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "courses")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub title: String,
    pub course_type: String, // "Mandatory" or "Elective"
    pub duration: String,    // e.g. "1 h", "45 min"
    pub audience: String,    // "Everyone", "Managers", or department name
    pub due_date: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::course_assignments::Entity")]
    CourseAssignments,
}

impl Related<super::course_assignments::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CourseAssignments.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

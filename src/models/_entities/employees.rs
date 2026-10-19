use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "employees")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i64,
    pub emp_id: Uuid,
    #[sea_orm(unique)]
    pub employee_code: String,
    pub full_name: String,
    pub job_title: String,
    pub department: String,
    pub grade: String,
    pub location: String,
    pub manager_name: Option<String>,
    pub status: String,
    pub work_email: Option<String>,
    pub phone: Option<String>,
    pub shift: Option<String>,
    pub joined: Option<String>,
    pub ctc: Option<f64>,
    pub address: Option<String>,
    pub dob: Option<String>,
    pub blood_group: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_relation: Option<String>,
    pub emergency_contact_phone: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

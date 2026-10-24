use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "sign_envelopes")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub envelope_id: String,
    pub code: String,
    pub document_name: String,
    pub template_id: Option<String>,
    pub sender_id: Option<String>,
    pub sender_name: String,
    pub recipient_employee_id: Option<String>,
    pub recipient_name: String,
    pub recipient_email: String,
    pub is_external: bool,
    pub status: String,
    pub sent_at: String,
    pub due_at: String,
    pub completed_at: Option<String>,
    pub signed_name: Option<String>,
    pub signed_ip: Option<String>,
    pub decline_reason: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

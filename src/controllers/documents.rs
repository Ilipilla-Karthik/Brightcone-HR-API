use crate::models::_entities::employee_documents::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateDocParams {
    pub employee_code: Option<String>,
    pub title: String,
    pub document_type: String,
    pub file_size: Option<String>,
}

#[debug_handler]
pub async fn my_documents(State(ctx): State<AppContext>) -> Result<Response> {
    let mut list: Vec<Model> = Entity::find()
        .order_by_desc(employee_documents::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    // Auto-seed in PostgreSQL if empty
    if list.is_empty() {
        let defaults = vec![
            (
                "Offer & Employment Agreement",
                "Legal Agreement",
                "1.2 MB",
                "2022-08-15",
            ),
            (
                "Non-Disclosure Agreement (NDA)",
                "Compliance",
                "480 KB",
                "2022-08-15",
            ),
            (
                "Form 16 Tax Certificate (FY 2025-26)",
                "Tax Document",
                "640 KB",
                "2026-06-10",
            ),
            (
                "Monthly Payslip — September 2026",
                "Payroll Slip",
                "210 KB",
                "2026-10-01",
            ),
            (
                "Monthly Payslip — August 2026",
                "Payroll Slip",
                "210 KB",
                "2026-09-01",
            ),
        ];

        for (title, dtype, size, date) in defaults {
            let active = ActiveModel {
                doc_id: Set(Uuid::new_v4()),
                employee_code: Set("BC-1038".to_string()),
                title: Set(title.to_string()),
                document_type: Set(dtype.to_string()),
                file_size: Set(size.to_string()),
                issued_date: Set(date.to_string()),
                download_url: Set(Some(format!(
                    "/api/v1/documents/sample/{}.pdf",
                    title.replace(' ', "_")
                ))),
                ..Default::default()
            };
            let _ = active.insert(&ctx.db).await;
        }

        list = Entity::find()
            .order_by_desc(employee_documents::Column::CreatedAt)
            .all(&ctx.db)
            .await?;
    }

    format::json(list)
}

#[debug_handler]
pub async fn create_document(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateDocParams>,
) -> Result<Response> {
    let active = ActiveModel {
        doc_id: Set(Uuid::new_v4()),
        employee_code: Set(params.employee_code.unwrap_or_else(|| "BC-1038".to_string())),
        title: Set(params.title.clone()),
        document_type: Set(params.document_type),
        file_size: Set(params.file_size.unwrap_or_else(|| "350 KB".to_string())),
        issued_date: Set("2026-10-26".to_string()),
        download_url: Set(Some(format!(
            "/api/v1/documents/sample/{}.pdf",
            params.title.replace(' ', "_")
        ))),
        ..Default::default()
    };
    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/documents")
        .add("/my", get(my_documents))
        .add("", get(my_documents))
        .add("", post(create_document))
}

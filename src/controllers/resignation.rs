use crate::models::_entities::resignation_requests::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct ResignationParams {
    pub employee_code: Option<String>,
    pub employee_name: Option<String>,
    pub reason: String,
    pub proposed_last_day: String,
    pub notice_period_days: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResignationStatusParams {
    pub status: String,
}

#[debug_handler]
pub async fn my_resignation(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(resignation_requests::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn initiate(
    State(ctx): State<AppContext>,
    Json(params): Json<ResignationParams>,
) -> Result<Response> {
    let code = params.employee_code.unwrap_or_default();
    let trimmed = code.trim().to_string();
    if trimmed.is_empty() {
        return bad_request("employee_code is required to initiate a resignation.");
    }
    let name = match params.employee_name {
        Some(n) if !n.trim().is_empty() => n,
        _ => {
            // Resolve the name from the employee directory instead of guessing.
            match crate::models::_entities::employees::Entity::find()
                .filter(
                    crate::models::_entities::employees::Column::EmployeeCode.eq(&trimmed),
                )
                .one(&ctx.db)
                .await?
            {
                Some(e) => e.full_name,
                None => return bad_request("Unknown employee_code."),
            }
        }
    };
    let notice = params.notice_period_days.unwrap_or(60);

    let active = ActiveModel {
        request_id: Set(Uuid::new_v4()),
        employee_code: Set(trimmed),
        employee_name: Set(name),
        reason: Set(params.reason),
        proposed_last_day: Set(params.proposed_last_day),
        notice_period_days: Set(notice),
        status: Set("Under Review by Manager".to_string()),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update_status(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<ResignationStatusParams>,
) -> Result<Response> {
    let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    active.status = Set(params.status);
    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/resignation")
        .add("/my", get(my_resignation))
        .add("/initiate", post(initiate))
        .add("/", get(my_resignation))
        .add("/{id}/status", patch(update_status))
}

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
    let code = params
        .employee_code
        .unwrap_or_else(|| "BC-1038".to_string());
    let name = params
        .employee_name
        .unwrap_or_else(|| "Rahul Varma".to_string());
    let notice = params.notice_period_days.unwrap_or(60);

    let active = ActiveModel {
        request_id: Set(Uuid::new_v4()),
        employee_code: Set(code),
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

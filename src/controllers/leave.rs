use crate::models::_entities::leave_requests::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct LeaveApplyParams {
    pub employee_code: Option<String>,
    pub employee_name: Option<String>,
    pub leave_type: String,
    pub from_date: String,
    pub to_date: String,
    pub days: Option<String>,
    pub reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeaveStatusParams {
    pub status: String,
}

#[debug_handler]
pub async fn my_leaves(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(leave_requests::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn apply(
    State(ctx): State<AppContext>,
    Json(params): Json<LeaveApplyParams>,
) -> Result<Response> {
    let code = params
        .employee_code
        .unwrap_or_else(|| "BC-1038".to_string());
    let name = params
        .employee_name
        .unwrap_or_else(|| "Rahul Varma".to_string());
    let days = params.days.unwrap_or_else(|| "Requested".to_string());

    let active = ActiveModel {
        request_id: Set(Uuid::new_v4()),
        employee_code: Set(code),
        employee_name: Set(name),
        leave_type: Set(params.leave_type),
        from_date: Set(params.from_date),
        to_date: Set(params.to_date),
        days: Set(days),
        reason: Set(params.reason),
        status: Set("Pending HR Approval".to_string()),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update_status(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<LeaveStatusParams>,
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
        .prefix("/api/v1/leave")
        .add("/my", get(my_leaves))
        .add("/apply", post(apply))
        .add("/", get(my_leaves))
        .add("/{id}/status", patch(update_status))
}

use crate::models::_entities::helpdesk_tickets::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TicketParams {
    pub employee_code: Option<String>,
    pub employee_name: Option<String>,
    pub category: String,
    pub subject: String,
    pub description: String,
    pub priority: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TicketStatusParams {
    pub status: String,
}

#[debug_handler]
pub async fn my_tickets(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(helpdesk_tickets::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create_ticket(
    State(ctx): State<AppContext>,
    Json(params): Json<TicketParams>,
) -> Result<Response> {
    let code = params
        .employee_code
        .unwrap_or_else(|| "BC-1038".to_string());
    let name = params
        .employee_name
        .unwrap_or_else(|| "Rahul Varma".to_string());
    let priority = params.priority.unwrap_or_else(|| "Medium".to_string());
    let count = Entity::find().all(&ctx.db).await?.len();
    let ticket_code = format!("HD-{}", 100 + count + 1);

    let active = ActiveModel {
        ticket_code: Set(ticket_code),
        employee_code: Set(code),
        employee_name: Set(name),
        category: Set(params.category),
        subject: Set(params.subject),
        description: Set(params.description),
        priority: Set(priority),
        status: Set("Open".to_string()),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update_status(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<TicketStatusParams>,
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
        .prefix("/api/v1/helpdesk")
        .add("/my", get(my_tickets))
        .add("", get(my_tickets))
        .add("", post(create_ticket))
        .add("/tickets", post(create_ticket))
        .add("/{id}/status", patch(update_status))
}

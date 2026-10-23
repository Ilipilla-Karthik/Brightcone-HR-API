use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::_entities::roster_assignments::{self, ActiveModel, Entity, Model};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RosterPayload {
    pub employee_code: String,
    pub employee_name: String,
    pub shift_code: String,
    pub start_date: String,
    pub end_date: String,
    pub status: Option<String>,
}

pub async fn seed_defaults(db: &DatabaseConnection) -> Result<()> {
    let defaults = [
        ("BC-1038", "Rahul Varma", "GEN", "2026-10-19", "2026-10-25", "Published"),
        ("BC-1042", "Sneha Iyer", "GEN", "2026-10-19", "2026-10-25", "Published"),
        ("BC-1045", "Arjun Mehta", "MORN", "2026-10-19", "2026-10-25", "Published"),
        ("BC-1048", "Kavita Nair", "EVE", "2026-10-19", "2026-10-25", "Published"),
    ];
    for (code, name, shift, start, end, status) in defaults {
        let exists = Entity::find().filter(roster_assignments::Column::EmployeeCode.eq(code)).one(db).await?;
        if exists.is_none() {
            let m = ActiveModel {
                roster_id: Set(Uuid::new_v4()),
                employee_code: Set(code.to_string()),
                employee_name: Set(name.to_string()),
                shift_code: Set(shift.to_string()),
                start_date: Set(start.to_string()),
                end_date: Set(end.to_string()),
                status: Set(status.to_string()),
                ..Default::default()
            };
            let _ = m.insert(db).await;
        }
    }
    Ok(())
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    if Entity::find().count(&ctx.db).await? == 0 {
        let _ = seed_defaults(&ctx.db).await;
    }
    let records: Vec<Model> = Entity::find().order_by_desc(roster_assignments::Column::StartDate).all(&ctx.db).await?;
    format::json(records)
}

#[debug_handler]
pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    match Entity::find_by_id(id).one(&ctx.db).await? {
        Some(m) => format::json(m),
        None => not_found(),
    }
}

#[debug_handler]
pub async fn create(State(ctx): State<AppContext>, Json(payload): Json<RosterPayload>) -> Result<Response> {
    let active = ActiveModel {
        roster_id: Set(Uuid::new_v4()),
        employee_code: Set(payload.employee_code),
        employee_name: Set(payload.employee_name),
        shift_code: Set(payload.shift_code),
        start_date: Set(payload.start_date),
        end_date: Set(payload.end_date),
        status: Set(payload.status.unwrap_or_else(|| "Scheduled".to_string())),
        ..Default::default()
    };
    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn publish(State(ctx): State<AppContext>) -> Result<Response> {
    let all: Vec<Model> = Entity::find().all(&ctx.db).await?;
    for m in all {
        let mut a: ActiveModel = m.into();
        a.status = Set("Published".to_string());
        let _ = a.update(&ctx.db).await;
    }
    format::json(serde_json::json!({ "success": true, "message": "Weekly roster published successfully." }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/roster")
        .add("", get(list))
        .add("", post(create))
        .add("/publish", post(publish))
        .add("/{id}", get(get_one))
}

pub fn legacy_routes() -> Routes {
    Routes::new()
        .prefix("/roster")
        .add("", get(list))
        .add("", post(create))
        .add("/publish", post(publish))
        .add("/{id}", get(get_one))
}

use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::_entities::shifts::{self, ActiveModel, Entity, Model};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShiftPayload {
    pub name: String,
    pub code: String,
    #[serde(alias = "start_time")]
    pub start_time: String,
    #[serde(alias = "end_time")]
    pub end_time: String,
    #[serde(alias = "break_duration_minutes")]
    pub break_duration_minutes: Option<i32>,
    #[serde(alias = "grace_period_minutes")]
    pub grace_period_minutes: Option<i32>,
    #[serde(alias = "is_overnight")]
    pub is_overnight: Option<bool>,
    #[serde(alias = "is_active")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateShiftPayload {
    pub name: Option<String>,
    pub code: Option<String>,
    #[serde(alias = "start_time")]
    pub start_time: Option<String>,
    #[serde(alias = "end_time")]
    pub end_time: Option<String>,
    #[serde(alias = "break_duration_minutes")]
    pub break_duration_minutes: Option<i32>,
    #[serde(alias = "grace_period_minutes")]
    pub grace_period_minutes: Option<i32>,
    #[serde(alias = "is_overnight")]
    pub is_overnight: Option<bool>,
    #[serde(alias = "is_active")]
    pub is_active: Option<bool>,
}

pub async fn seed_defaults(db: &DatabaseConnection) -> Result<()> {
    let defaults = [
        ("General Shift", "GEN", "09:00", "18:00", 60, 15, false, true),
        ("Morning Shift", "MORN", "06:00", "15:00", 45, 10, false, true),
        ("Evening Shift", "EVE", "14:00", "23:00", 45, 10, false, true),
        ("Night Shift", "NIGHT", "22:00", "07:00", 60, 15, true, true),
    ];
    for (name, code, start, end, brk, grace, overnight, active) in defaults {
        let exists = Entity::find().filter(shifts::Column::Code.eq(code)).one(db).await?;
        if exists.is_none() {
            let m = ActiveModel {
                shift_id: Set(Uuid::new_v4()),
                name: Set(name.to_string()),
                code: Set(code.to_string()),
                start_time: Set(start.to_string()),
                end_time: Set(end.to_string()),
                break_duration_minutes: Set(brk),
                grace_period_minutes: Set(grace),
                is_overnight: Set(overnight),
                is_active: Set(active),
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
    let records: Vec<Model> = Entity::find().order_by_asc(shifts::Column::Code).all(&ctx.db).await?;
    format::json(records)
}

#[debug_handler]
pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let record = if let Ok(num_id) = id.parse::<i64>() {
        Entity::find_by_id(num_id).one(&ctx.db).await?
    } else {
        Entity::find().filter(shifts::Column::Code.eq(&id)).one(&ctx.db).await?
    };
    match record {
        Some(m) => format::json(m),
        None => not_found(),
    }
}

#[debug_handler]
pub async fn create(State(ctx): State<AppContext>, Json(payload): Json<ShiftPayload>) -> Result<Response> {
    let active = ActiveModel {
        shift_id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        code: Set(payload.code),
        start_time: Set(payload.start_time),
        end_time: Set(payload.end_time),
        break_duration_minutes: Set(payload.break_duration_minutes.unwrap_or(60)),
        grace_period_minutes: Set(payload.grace_period_minutes.unwrap_or(15)),
        is_overnight: Set(payload.is_overnight.unwrap_or(false)),
        is_active: Set(payload.is_active.unwrap_or(true)),
        ..Default::default()
    };
    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateShiftPayload>,
) -> Result<Response> {
    let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return not_found();
    };
    let mut active: ActiveModel = existing.into();
    if let Some(n) = payload.name { active.name = Set(n); }
    if let Some(c) = payload.code { active.code = Set(c); }
    if let Some(s) = payload.start_time { active.start_time = Set(s); }
    if let Some(e) = payload.end_time { active.end_time = Set(e); }
    if let Some(b) = payload.break_duration_minutes { active.break_duration_minutes = Set(b); }
    if let Some(g) = payload.grace_period_minutes { active.grace_period_minutes = Set(g); }
    if let Some(o) = payload.is_overnight { active.is_overnight = Set(o); }
    if let Some(a) = payload.is_active { active.is_active = Set(a); }
    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    if let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? {
        let active: ActiveModel = existing.into();
        active.delete(&ctx.db).await?;
    }
    format::empty_json()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/shifts")
        .add("", get(list))
        .add("", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

pub fn legacy_routes() -> Routes {
    Routes::new()
        .prefix("/shifts")
        .add("", get(list))
        .add("", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::_entities::holidays::{self, ActiveModel, Entity, Model};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct HolidayPayload {
    pub name: String,
    #[serde(alias = "holidayDate", alias = "date")]
    pub holiday_date: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, alias = "applicableLocation")]
    pub applicable_location: Option<String>,
    #[serde(default, alias = "isRecurring")]
    pub is_recurring: Option<bool>,
}

pub async fn seed_defaults(db: &DatabaseConnection) -> Result<()> {
    let defaults = [
        ("New Year's Day", "2026-01-01", "Global holiday", "All Locations", true),
        ("Republic Day", "2026-01-26", "National holiday", "India", true),
        ("Labor Day", "2026-05-01", "International Workers' Day", "All Locations", true),
        ("Independence Day", "2026-08-15", "National holiday", "India", true),
        ("Gandhi Jayanti", "2026-10-02", "National holiday", "India", true),
        ("Diwali", "2026-11-08", "Festival of Lights", "India", false),
        ("Christmas Day", "2026-12-25", "Public holiday", "All Locations", true),
    ];
    for (name, date, desc, loc, recur) in defaults {
        let exists = Entity::find().filter(holidays::Column::HolidayDate.eq(date)).one(db).await?;
        if exists.is_none() {
            let m = ActiveModel {
                holiday_id: Set(Uuid::new_v4()),
                name: Set(name.to_string()),
                holiday_date: Set(date.to_string()),
                description: Set(Some(desc.to_string())),
                applicable_location: Set(Some(loc.to_string())),
                is_recurring: Set(recur),
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
    let records: Vec<Model> = Entity::find().order_by_asc(holidays::Column::HolidayDate).all(&ctx.db).await?;
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
pub async fn create(State(ctx): State<AppContext>, Json(payload): Json<HolidayPayload>) -> Result<Response> {
    let active = ActiveModel {
        holiday_id: Set(Uuid::new_v4()),
        name: Set(payload.name),
        holiday_date: Set(payload.holiday_date),
        description: Set(payload.description),
        applicable_location: Set(payload.applicable_location),
        is_recurring: Set(payload.is_recurring.unwrap_or(false)),
        ..Default::default()
    };
    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<HolidayPayload>,
) -> Result<Response> {
    let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return not_found();
    };
    let mut active: ActiveModel = existing.into();
    active.name = Set(payload.name);
    active.holiday_date = Set(payload.holiday_date);
    if let Some(d) = payload.description { active.description = Set(Some(d)); }
    if let Some(l) = payload.applicable_location { active.applicable_location = Set(Some(l)); }
    if let Some(r) = payload.is_recurring { active.is_recurring = Set(r); }
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
        .prefix("/api/v1/holidays")
        .add("", get(list))
        .add("", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

pub fn legacy_routes() -> Routes {
    Routes::new()
        .prefix("/holidays")
        .add("", get(list))
        .add("", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", put(update))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

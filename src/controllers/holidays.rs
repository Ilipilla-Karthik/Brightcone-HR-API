use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use loco_rs::prelude::*;

use crate::modules::time::holidays;
use crate::modules::time::models::{CreateHolidayPayload, UpdateHolidayPayload};

pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match holidays::list_holidays(pool).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match holidays::get_holiday(pool, id).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn create(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateHolidayPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match holidays::create_holiday(pool, payload).await {
        Ok(data) => Ok((StatusCode::CREATED, axum::Json(data)).into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateHolidayPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match holidays::update_holiday(pool, id, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match holidays::delete_holiday(pool, id).await {
        Ok(code) => Ok(code.into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/holidays", get(list).post(create))
        .add("/holidays/{id}", get(get_one).put(update).delete(remove))
        .add("/api/holidays", get(list).post(create))
        .add(
            "/api/holidays/{id}",
            get(get_one).put(update).delete(remove),
        )
}

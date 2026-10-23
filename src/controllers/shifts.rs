use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use loco_rs::prelude::*;

use crate::modules::time::models::{CreateShiftPayload, UpdateShiftPayload};
use crate::modules::time::shifts;

pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match shifts::list_shifts(pool).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match shifts::get_shift(pool, id).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn create(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateShiftPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match shifts::create_shift(pool, payload).await {
        Ok(data) => Ok((StatusCode::CREATED, axum::Json(data)).into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateShiftPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match shifts::update_shift(pool, id, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match shifts::delete_shift(pool, id).await {
        Ok(code) => Ok(code.into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/shifts", get(list).post(create))
        .add("/shifts/{id}", get(get_one).put(update).delete(remove))
        .add("/api/shifts", get(list).post(create))
        .add("/api/shifts/{id}", get(get_one).put(update).delete(remove))
}

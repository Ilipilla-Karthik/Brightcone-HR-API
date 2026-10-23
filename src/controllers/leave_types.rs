use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use loco_rs::prelude::*;

use crate::modules::time::leave_types;
use crate::modules::time::models::{CreateLeaveTypePayload, UpdateLeaveTypePayload};

pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_types::list_leave_types(pool).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_types::get_leave_type(pool, id).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn create(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateLeaveTypePayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_types::create_leave_type(pool, payload).await {
        Ok(data) => Ok((StatusCode::CREATED, axum::Json(data)).into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateLeaveTypePayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_types::update_leave_type(pool, id, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_types::delete_leave_type(pool, id).await {
        Ok(code) => Ok(code.into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/leave-types", get(list).post(create))
        .add("/leave-types/{id}", get(get_one).put(update).delete(remove))
        .add("/api/leave-types", get(list).post(create))
        .add(
            "/api/leave-types/{id}",
            get(get_one).put(update).delete(remove),
        )
}

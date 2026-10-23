use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use loco_rs::prelude::*;

use crate::modules::time::leave_quotas;
use crate::modules::time::models::{CreateLeaveQuotaPayload, UpdateLeaveQuotaPayload};

pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_quotas::list_leave_quotas(pool).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_quotas::get_leave_quota(pool, id).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn create(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateLeaveQuotaPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_quotas::create_leave_quota(pool, payload).await {
        Ok(data) => Ok((StatusCode::CREATED, axum::Json(data)).into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateLeaveQuotaPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_quotas::update_leave_quota(pool, id, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_quotas::delete_leave_quota(pool, id).await {
        Ok(code) => Ok(code.into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/leave-quotas", get(list).post(create))
        .add(
            "/leave-quotas/{id}",
            get(get_one).put(update).delete(remove),
        )
        .add("/api/leave-quotas", get(list).post(create))
        .add(
            "/api/leave-quotas/{id}",
            get(get_one).put(update).delete(remove),
        )
}

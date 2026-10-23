use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use loco_rs::prelude::*;

use crate::modules::time::leave_calc::{self, CountLeaveDaysRequest};

pub async fn preview_post(
    State(ctx): State<AppContext>,
    Json(payload): Json<CountLeaveDaysRequest>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_calc::count_leave_days_service(pool, &payload).await {
        Ok(result) => format::json(result),
        Err(e) => {
            let (code, body) = e.to_status_and_response();
            Ok((code, body).into_response())
        }
    }
}

pub async fn preview_get(
    State(ctx): State<AppContext>,
    Query(query): Query<CountLeaveDaysRequest>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match leave_calc::count_leave_days_service(pool, &query).await {
        Ok(result) => format::json(result),
        Err(e) => {
            let (code, body) = e.to_status_and_response();
            Ok((code, body).into_response())
        }
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/leave/count-days", get(preview_get).post(preview_post))
        .add("/leave/preview", get(preview_get).post(preview_post))
        .add("/api/leave/count-days", get(preview_get).post(preview_post))
        .add("/api/leave/preview", get(preview_get).post(preview_post))
}

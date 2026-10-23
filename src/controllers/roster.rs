use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use loco_rs::prelude::*;

use crate::modules::time::models::{AssignShiftPayload, PublishRosterPayload, WeeklyRosterQuery};
use crate::modules::time::roster;

pub async fn list_employees(State(ctx): State<AppContext>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match roster::list_employees_db(pool).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn get_weekly_roster(
    State(ctx): State<AppContext>,
    Query(params): Query<WeeklyRosterQuery>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match roster::get_weekly_roster_db(pool, params).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn assign_shift(
    State(ctx): State<AppContext>,
    Json(payload): Json<AssignShiftPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match roster::assign_shift_db(pool, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn publish_roster(
    State(ctx): State<AppContext>,
    Json(payload): Json<PublishRosterPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match roster::publish_roster_db(pool, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/roster", get(get_weekly_roster))
        .add("/roster/assign", post(assign_shift))
        .add("/roster/publish", post(publish_roster))
        .add("/employees", get(list_employees))
        .add("/api/roster", get(get_weekly_roster))
        .add("/api/roster/assign", post(assign_shift))
        .add("/api/roster/publish", post(publish_roster))
        .add("/api/employees", get(list_employees))
}

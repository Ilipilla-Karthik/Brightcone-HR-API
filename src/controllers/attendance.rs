use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use loco_rs::prelude::*;

use crate::modules::time::{attendance, corrections};
use crate::modules::time::models::{
    ApproveCorrectionPayload, AttendanceQuery, ClockInPayload, ClockOutPayload,
    CorrectionQuery, CreateAttendancePayload, CreateCorrectionRequestPayload,
    RejectCorrectionPayload, TodayAttendanceQuery, UpdateAttendancePayload,
};

pub async fn list(
    State(ctx): State<AppContext>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::list_attendance_db(pool, params).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn create(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateAttendancePayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::create_attendance_db(pool, payload).await {
        Ok(data) => Ok((StatusCode::CREATED, axum::Json(data)).into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateAttendancePayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::update_attendance_db(pool, id, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn toggle_archive(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::toggle_archive_attendance_db(pool, id).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::delete_attendance_db(pool, id).await {
        Ok(code) => Ok(code.into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn export_csv(
    State(ctx): State<AppContext>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::export_attendance_csv_db(pool, params).await {
        Ok(resp) => Ok(resp),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn clock_in(
    State(ctx): State<AppContext>,
    Json(payload): Json<ClockInPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::clock_in_db(pool, payload).await {
        Ok(data) => Ok((StatusCode::OK, axum::Json(data)).into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn clock_out(
    State(ctx): State<AppContext>,
    Json(payload): Json<ClockOutPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::clock_out_db(pool, payload).await {
        Ok(data) => Ok((StatusCode::OK, axum::Json(data)).into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn today_status(
    State(ctx): State<AppContext>,
    Query(params): Query<TodayAttendanceQuery>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match attendance::get_today_attendance_db(pool, params).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn create_correction(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateCorrectionRequestPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match corrections::create_correction_request_db(pool, payload).await {
        Ok(data) => Ok((StatusCode::CREATED, axum::Json(data)).into_response()),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn list_corrections(
    State(ctx): State<AppContext>,
    Query(params): Query<CorrectionQuery>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match corrections::list_correction_requests_db(pool, params).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn get_correction(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match corrections::get_correction_request_db(pool, id).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn approve_correction(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<ApproveCorrectionPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match corrections::approve_correction_request_db(pool, id, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub async fn reject_correction(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(payload): Json<RejectCorrectionPayload>,
) -> Result<Response> {
    let pool = ctx.db.get_postgres_connection_pool();
    match corrections::reject_correction_request_db(pool, id, payload).await {
        Ok(data) => format::json(data),
        Err((code, body)) => Ok((code, body).into_response()),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/attendance/clock-in", post(clock_in))
        .add("/attendance/clock-out", post(clock_out))
        .add("/attendance/today", get(today_status))
        .add("/attendance/export", get(export_csv))
        .add("/attendance/corrections", get(list_corrections).post(create_correction))
        .add("/attendance/corrections/{id}", get(get_correction))
        .add("/attendance/corrections/{id}/approve", post(approve_correction))
        .add("/attendance/corrections/{id}/reject", post(reject_correction))
        .add("/attendance", get(list).post(create))
        .add("/attendance/{id}", put(update).delete(remove))
        .add("/attendance/{id}/archive", post(toggle_archive))
        .add("/api/attendance/clock-in", post(clock_in))
        .add("/api/attendance/clock-out", post(clock_out))
        .add("/api/attendance/today", get(today_status))
        .add("/api/attendance/export", get(export_csv))
        .add("/api/attendance/corrections", get(list_corrections).post(create_correction))
        .add("/api/attendance/corrections/{id}", get(get_correction))
        .add("/api/attendance/corrections/{id}/approve", post(approve_correction))
        .add("/api/attendance/corrections/{id}/reject", post(reject_correction))
        .add("/api/attendance", get(list).post(create))
        .add("/api/attendance/{id}", put(update).delete(remove))
        .add("/api/attendance/{id}/archive", post(toggle_archive))
}


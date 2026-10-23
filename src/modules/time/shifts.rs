use axum::{http::StatusCode, response::Json};
use sqlx::PgPool;

use super::models::{CreateShiftPayload, ErrorResponse, Shift, UpdateShiftPayload};

pub async fn list_shifts(pool: &PgPool) -> Result<Vec<Shift>, (StatusCode, Json<ErrorResponse>)> {
    let shifts: Vec<Shift> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, \
                start_time::TEXT as start_time, end_time::TEXT as end_time, \
                break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM shifts ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?;

    Ok(shifts)
}

pub async fn get_shift(pool: &PgPool, id: i64) -> Result<Shift, (StatusCode, Json<ErrorResponse>)> {
    let shift: Option<Shift> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, \
                start_time::TEXT as start_time, end_time::TEXT as end_time, \
                break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM shifts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
    })?;

    match shift {
        Some(s) => Ok(s),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Shift with id {} not found", id),
            }),
        )),
    }
}

pub async fn create_shift(
    pool: &PgPool,
    payload: CreateShiftPayload,
) -> Result<Shift, (StatusCode, Json<ErrorResponse>)> {
    let name = payload.name.trim();
    let code = payload.code.trim().to_uppercase();
    if name.is_empty() || code.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Shift name and code are required".into(),
            }),
        ));
    }

    // Check code uniqueness
    let existing_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM shifts WHERE code = $1")
        .bind(&code)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    if existing_count.0 > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!("Shift with code '{}' already exists", code),
            }),
        ));
    }

    let break_duration = payload.break_duration_minutes.unwrap_or(60);
    let grace_period = payload.grace_period_minutes.unwrap_or(15);
    if break_duration < 0 || grace_period < 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Break duration and grace period must be non-negative".into(),
            }),
        ));
    }

    let is_overnight = payload.is_overnight.unwrap_or(false);
    let start = payload.start_time.trim();
    let end = payload.end_time.trim();

    if (!is_overnight && start >= end) || (is_overnight && start <= end) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: if !is_overnight {
                    "For regular daytime shifts, Start Time must be earlier than End Time. For shifts crossing midnight, enable 'Overnight Shift'.".into()
                } else {
                    "For overnight shifts crossing midnight, Start Time must be later than End Time (e.g. 21:00 to 06:00).".into()
                },
            }),
        ));
    }

    let is_active = payload.is_active.unwrap_or(true);

    let new_shift: Shift = sqlx::query_as(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active) \
         VALUES ($1, $2, $3::TIME, $4::TIME, $5, $6, $7, $8) \
         RETURNING id, pid::TEXT as pid, name, code, \
                   start_time::TEXT as start_time, end_time::TEXT as end_time, \
                   break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at"
    )
    .bind(name)
    .bind(&code)
    .bind(start)
    .bind(end)
    .bind(break_duration)
    .bind(grace_period)
    .bind(is_overnight)
    .bind(is_active)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(new_shift)
}

pub async fn update_shift(
    pool: &PgPool,
    id: i64,
    payload: UpdateShiftPayload,
) -> Result<Shift, (StatusCode, Json<ErrorResponse>)> {
    let existing: Option<Shift> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, \
                start_time::TEXT as start_time, end_time::TEXT as end_time, \
                break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM shifts WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let existing = match existing {
        Some(s) => s,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Shift with id {} not found", id),
                }),
            ));
        }
    };

    let target_name = match payload.name {
        Some(ref n) => {
            let clean = n.trim();
            if clean.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Shift name cannot be empty".into(),
                    }),
                ));
            }
            clean.to_string()
        }
        None => existing.name,
    };

    let target_code = match payload.code {
        Some(ref c) => {
            let clean = c.trim().to_uppercase();
            if clean.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Shift code cannot be empty".into(),
                    }),
                ));
            }
            let duplicate_count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM shifts WHERE id != $1 AND code = $2")
                    .bind(id)
                    .bind(&clean)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: e.to_string(),
                            }),
                        )
                    })?;

            if duplicate_count.0 > 0 {
                return Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: format!("Shift with code '{}' already exists", clean),
                    }),
                ));
            }
            clean
        }
        None => existing.code,
    };

    let target_start = payload.start_time.unwrap_or(existing.start_time);
    let target_end = payload.end_time.unwrap_or(existing.end_time);
    let target_is_overnight = payload.is_overnight.unwrap_or(existing.is_overnight);

    let start = target_start.trim();
    let end = target_end.trim();

    if (!target_is_overnight && start >= end) || (target_is_overnight && start <= end) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: if !target_is_overnight {
                    "For regular daytime shifts, Start Time must be earlier than End Time. For shifts crossing midnight, enable 'Overnight Shift'.".into()
                } else {
                    "For overnight shifts crossing midnight, Start Time must be later than End Time (e.g. 21:00 to 06:00).".into()
                },
            }),
        ));
    }

    let target_break = payload
        .break_duration_minutes
        .unwrap_or(existing.break_duration_minutes);
    let target_grace = payload
        .grace_period_minutes
        .unwrap_or(existing.grace_period_minutes);
    if target_break < 0 || target_grace < 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Break duration and grace period must be non-negative".into(),
            }),
        ));
    }

    let target_is_active = payload.is_active.unwrap_or(existing.is_active);

    let updated_shift: Shift = sqlx::query_as(
        "UPDATE shifts \
         SET name = $1, \
             code = $2, \
             start_time = $3::TIME, \
             end_time = $4::TIME, \
             break_duration_minutes = $5, \
             grace_period_minutes = $6, \
             is_overnight = $7, \
             is_active = $8, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = $9 \
         RETURNING id, pid::TEXT as pid, name, code, \
                   start_time::TEXT as start_time, end_time::TEXT as end_time, \
                   break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at",
    )
    .bind(target_name)
    .bind(target_code)
    .bind(start)
    .bind(end)
    .bind(target_break)
    .bind(target_grace)
    .bind(target_is_overnight)
    .bind(target_is_active)
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(updated_shift)
}

pub async fn delete_shift(
    pool: &PgPool,
    id: i64,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM shifts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Shift with id {} not found", id),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

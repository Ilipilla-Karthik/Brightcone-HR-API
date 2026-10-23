use axum::{http::StatusCode, response::Json};
use sqlx::PgPool;

use super::models::{CreateLeaveTypePayload, ErrorResponse, LeaveType, UpdateLeaveTypePayload};

pub async fn list_leave_types(
    pool: &PgPool,
) -> Result<Vec<LeaveType>, (StatusCode, Json<ErrorResponse>)> {
    let leave_types: Vec<LeaveType> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, description, is_paid, is_active, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM leave_types ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    Ok(leave_types)
}

pub async fn get_leave_type(
    pool: &PgPool,
    id: i64,
) -> Result<LeaveType, (StatusCode, Json<ErrorResponse>)> {
    let lt: Option<LeaveType> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, description, is_paid, is_active, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM leave_types WHERE id = $1",
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

    match lt {
        Some(item) => Ok(item),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Leave type with id {} not found", id),
            }),
        )),
    }
}

pub async fn create_leave_type(
    pool: &PgPool,
    payload: CreateLeaveTypePayload,
) -> Result<LeaveType, (StatusCode, Json<ErrorResponse>)> {
    let name = payload.name.trim();
    let code = payload.code.trim().to_uppercase();
    if name.is_empty() || code.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Leave type name and code are required".into(),
            }),
        ));
    }

    let conflict_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_types WHERE code = $1")
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

    if conflict_count.0 > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!("Leave type with code '{}' already exists", code),
            }),
        ));
    }

    let is_paid = payload.is_paid.unwrap_or(true);
    let is_active = payload.is_active.unwrap_or(true);

    let new_lt: LeaveType = sqlx::query_as(
        "INSERT INTO leave_types (name, code, description, is_paid, is_active) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, pid::TEXT as pid, name, code, description, is_paid, is_active, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at",
    )
    .bind(name)
    .bind(&code)
    .bind(payload.description)
    .bind(is_paid)
    .bind(is_active)
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

    Ok(new_lt)
}

pub async fn update_leave_type(
    pool: &PgPool,
    id: i64,
    payload: UpdateLeaveTypePayload,
) -> Result<LeaveType, (StatusCode, Json<ErrorResponse>)> {
    let existing: Option<LeaveType> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, description, is_paid, is_active, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM leave_types WHERE id = $1",
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
        Some(item) => item,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Leave type with id {} not found", id),
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
                        error: "Leave type name cannot be empty".into(),
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
                        error: "Leave type code cannot be empty".into(),
                    }),
                ));
            }
            let conflict_count: (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM leave_types WHERE id != $1 AND code = $2")
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

            if conflict_count.0 > 0 {
                return Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: format!("Leave type with code '{}' already exists", clean),
                    }),
                ));
            }
            clean
        }
        None => existing.code,
    };

    let target_desc = payload.description.or(existing.description);
    let target_is_paid = payload.is_paid.unwrap_or(existing.is_paid);
    let target_is_active = payload.is_active.unwrap_or(existing.is_active);

    let updated_lt: LeaveType = sqlx::query_as(
        "UPDATE leave_types \
         SET name = $1, \
             code = $2, \
             description = $3, \
             is_paid = $4, \
             is_active = $5, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = $6 \
         RETURNING id, pid::TEXT as pid, name, code, description, is_paid, is_active, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at",
    )
    .bind(target_name)
    .bind(target_code)
    .bind(target_desc)
    .bind(target_is_paid)
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

    Ok(updated_lt)
}

pub async fn delete_leave_type(
    pool: &PgPool,
    id: i64,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // Foreign key reference check: prevent deleting leave types assigned to active quotas
    let quota_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM leave_quotas WHERE leave_type_id = $1")
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

    if quota_count.0 > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Cannot delete leave type that is assigned to active quotas".into(),
            }),
        ));
    }

    let result = sqlx::query("DELETE FROM leave_types WHERE id = $1")
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
                error: format!("Leave type with id {} not found", id),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

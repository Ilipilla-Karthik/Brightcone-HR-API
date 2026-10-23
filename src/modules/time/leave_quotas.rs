use axum::{http::StatusCode, response::Json};
use sqlx::PgPool;

use super::models::{CreateLeaveQuotaPayload, ErrorResponse, LeaveQuota, UpdateLeaveQuotaPayload};

pub async fn list_leave_quotas(
    pool: &PgPool,
) -> Result<Vec<LeaveQuota>, (StatusCode, Json<ErrorResponse>)> {
    let quotas: Vec<LeaveQuota> = sqlx::query_as(
        "SELECT lq.id, lq.pid::TEXT as pid, lq.leave_type_id, \
                lt.name as leave_type_name, lt.code as leave_type_code, \
                lq.year, lq.entitled_days::FLOAT8 as entitled_days, lq.applicable_group, \
                to_char(lq.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(lq.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM leave_quotas lq \
         LEFT JOIN leave_types lt ON lq.leave_type_id = lt.id \
         ORDER BY lq.year DESC, lq.id ASC",
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

    Ok(quotas)
}

pub async fn get_leave_quota(
    pool: &PgPool,
    id: i64,
) -> Result<LeaveQuota, (StatusCode, Json<ErrorResponse>)> {
    let quota: Option<LeaveQuota> = sqlx::query_as(
        "SELECT lq.id, lq.pid::TEXT as pid, lq.leave_type_id, \
                lt.name as leave_type_name, lt.code as leave_type_code, \
                lq.year, lq.entitled_days::FLOAT8 as entitled_days, lq.applicable_group, \
                to_char(lq.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(lq.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM leave_quotas lq \
         LEFT JOIN leave_types lt ON lq.leave_type_id = lt.id \
         WHERE lq.id = $1",
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

    match quota {
        Some(q) => Ok(q),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Leave quota with id {} not found", id),
            }),
        )),
    }
}

pub async fn create_leave_quota(
    pool: &PgPool,
    payload: CreateLeaveQuotaPayload,
) -> Result<LeaveQuota, (StatusCode, Json<ErrorResponse>)> {
    if payload.entitled_days <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Entitled days must be greater than 0".into(),
            }),
        ));
    }
    if payload.year < 2000 || payload.year > 2100 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Year must be between 2000 and 2100".into(),
            }),
        ));
    }

    let group = payload
        .applicable_group
        .unwrap_or_else(|| "ALL".to_string())
        .trim()
        .to_uppercase();

    // Verify leave_type exists
    let lt_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_types WHERE id = $1")
        .bind(payload.leave_type_id)
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

    if lt_count.0 == 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!(
                    "Leave type with id {} does not exist",
                    payload.leave_type_id
                ),
            }),
        ));
    }

    // Verify unique constraint: (leave_type_id, year, applicable_group)
    let conflict_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM leave_quotas WHERE leave_type_id = $1 AND year = $2 AND applicable_group = $3"
    )
    .bind(payload.leave_type_id)
    .bind(payload.year)
    .bind(&group)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    if conflict_count.0 > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Quota already exists for this leave type, year, and group".into(),
            }),
        ));
    }

    let new_quota: LeaveQuota = sqlx::query_as(
        "WITH inserted AS ( \
             INSERT INTO leave_quotas (leave_type_id, year, entitled_days, applicable_group) \
             VALUES ($1, $2, $3::NUMERIC, $4) \
             RETURNING * \
         ) \
         SELECT i.id, i.pid::TEXT as pid, i.leave_type_id, \
                 lt.name as leave_type_name, lt.code as leave_type_code, \
                 i.year, i.entitled_days::FLOAT8 as entitled_days, i.applicable_group, \
                 to_char(i.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                 to_char(i.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
          FROM inserted i \
          LEFT JOIN leave_types lt ON i.leave_type_id = lt.id",
    )
    .bind(payload.leave_type_id)
    .bind(payload.year)
    .bind(payload.entitled_days)
    .bind(group)
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

    Ok(new_quota)
}

pub async fn update_leave_quota(
    pool: &PgPool,
    id: i64,
    payload: UpdateLeaveQuotaPayload,
) -> Result<LeaveQuota, (StatusCode, Json<ErrorResponse>)> {
    let existing: Option<LeaveQuota> = sqlx::query_as(
        "SELECT lq.id, lq.pid::TEXT as pid, lq.leave_type_id, \
                lt.name as leave_type_name, lt.code as leave_type_code, \
                lq.year, lq.entitled_days::FLOAT8 as entitled_days, lq.applicable_group, \
                to_char(lq.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(lq.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM leave_quotas lq \
         LEFT JOIN leave_types lt ON lq.leave_type_id = lt.id \
         WHERE lq.id = $1",
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
        Some(q) => q,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Leave quota with id {} not found", id),
                }),
            ));
        }
    };

    let target_days = match payload.entitled_days {
        Some(d) => {
            if d <= 0.0 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Entitled days must be greater than 0".into(),
                    }),
                ));
            }
            d
        }
        None => existing.entitled_days,
    };

    let target_year = match payload.year {
        Some(y) => {
            if !(2000..=2100).contains(&y) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Year must be between 2000 and 2100".into(),
                    }),
                ));
            }
            y
        }
        None => existing.year,
    };

    let target_group = match payload.applicable_group {
        Some(g) => g.trim().to_uppercase(),
        None => existing.applicable_group,
    };

    let target_lt_id = match payload.leave_type_id {
        Some(lt_id) => {
            let lt_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_types WHERE id = $1")
                .bind(lt_id)
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

            if lt_count.0 == 0 {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: format!("Leave type with id {} does not exist", lt_id),
                    }),
                ));
            }
            lt_id
        }
        None => existing.leave_type_id,
    };

    let conflict_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM leave_quotas WHERE id != $1 AND leave_type_id = $2 AND year = $3 AND applicable_group = $4"
    )
    .bind(id)
    .bind(target_lt_id)
    .bind(target_year)
    .bind(&target_group)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    if conflict_count.0 > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Quota already exists for this leave type, year, and group".into(),
            }),
        ));
    }

    let updated_quota: LeaveQuota = sqlx::query_as(
        "WITH updated AS ( \
             UPDATE leave_quotas \
             SET leave_type_id = $1, \
                 year = $2, \
                 entitled_days = $3::NUMERIC, \
                 applicable_group = $4, \
                 updated_at = CURRENT_TIMESTAMP \
             WHERE id = $5 \
             RETURNING * \
         ) \
         SELECT u.id, u.pid::TEXT as pid, u.leave_type_id, \
                 lt.name as leave_type_name, lt.code as leave_type_code, \
                 u.year, u.entitled_days::FLOAT8 as entitled_days, u.applicable_group, \
                 to_char(u.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                 to_char(u.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
          FROM updated u \
          LEFT JOIN leave_types lt ON u.leave_type_id = lt.id",
    )
    .bind(target_lt_id)
    .bind(target_year)
    .bind(target_days)
    .bind(target_group)
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

    Ok(updated_quota)
}

pub async fn delete_leave_quota(
    pool: &PgPool,
    id: i64,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM leave_quotas WHERE id = $1")
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
                error: format!("Leave quota with id {} not found", id),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

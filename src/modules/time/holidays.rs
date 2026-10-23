use axum::{http::StatusCode, response::Json};
use sqlx::PgPool;

use super::models::{CreateHolidayPayload, ErrorResponse, Holiday, UpdateHolidayPayload};

pub async fn list_holidays(
    pool: &PgPool,
) -> Result<Vec<Holiday>, (StatusCode, Json<ErrorResponse>)> {
    let holidays: Vec<Holiday> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, holiday_date::TEXT as holiday_date, \
                description, applicable_location, is_recurring, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM holidays ORDER BY holiday_date ASC",
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

    Ok(holidays)
}

pub async fn get_holiday(
    pool: &PgPool,
    id: i64,
) -> Result<Holiday, (StatusCode, Json<ErrorResponse>)> {
    let holiday: Option<Holiday> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, holiday_date::TEXT as holiday_date, \
                description, applicable_location, is_recurring, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM holidays WHERE id = $1",
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

    match holiday {
        Some(h) => Ok(h),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Holiday with id {} not found", id),
            }),
        )),
    }
}

pub async fn create_holiday(
    pool: &PgPool,
    payload: CreateHolidayPayload,
) -> Result<Holiday, (StatusCode, Json<ErrorResponse>)> {
    let name = payload.name.trim();
    let date = payload.holiday_date.trim();
    if name.is_empty() || date.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Holiday name and holiday_date are required".into(),
            }),
        ));
    }

    let location = payload
        .applicable_location
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty());

    let conflict_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM holidays WHERE holiday_date = $1::DATE AND applicable_location IS NOT DISTINCT FROM $2"
    )
    .bind(date)
    .bind(&location)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    if conflict_count.0 > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!(
                    "A holiday on {} for location '{}' already exists (unique constraint).",
                    date,
                    location.as_deref().unwrap_or("All Locations")
                ),
            }),
        ));
    }

    let is_recurring = payload.is_recurring.unwrap_or(false);

    let new_holiday: Holiday = sqlx::query_as(
        "INSERT INTO holidays (name, holiday_date, description, applicable_location, is_recurring) \
         VALUES ($1, $2::DATE, $3, $4, $5) \
         RETURNING id, pid::TEXT as pid, name, holiday_date::TEXT as holiday_date, \
                   description, applicable_location, is_recurring, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at"
    )
    .bind(name)
    .bind(date)
    .bind(payload.description)
    .bind(&location)
    .bind(is_recurring)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(new_holiday)
}

pub async fn update_holiday(
    pool: &PgPool,
    id: i64,
    payload: UpdateHolidayPayload,
) -> Result<Holiday, (StatusCode, Json<ErrorResponse>)> {
    let existing: Option<Holiday> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, holiday_date::TEXT as holiday_date, \
                description, applicable_location, is_recurring, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM holidays WHERE id = $1",
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
        Some(h) => h,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Holiday with id {} not found", id),
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
                        error: "Holiday name cannot be empty".into(),
                    }),
                ));
            }
            clean.to_string()
        }
        None => existing.name,
    };

    let target_date = match payload.holiday_date {
        Some(ref d) => {
            let clean = d.trim();
            if clean.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "Holiday date cannot be empty".into(),
                    }),
                ));
            }
            clean.to_string()
        }
        None => existing.holiday_date,
    };

    let target_loc = match payload.applicable_location {
        Some(ref l) => {
            let clean = l.trim().to_string();
            if clean.is_empty() {
                None
            } else {
                Some(clean)
            }
        }
        None => existing.applicable_location,
    };

    let conflict_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM holidays WHERE id != $1 AND holiday_date = $2::DATE AND applicable_location IS NOT DISTINCT FROM $3"
    )
    .bind(id)
    .bind(&target_date)
    .bind(&target_loc)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    if conflict_count.0 > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!(
                    "A holiday on {} for location '{}' already exists (unique constraint).",
                    target_date,
                    target_loc.as_deref().unwrap_or("All Locations")
                ),
            }),
        ));
    }

    let target_desc = payload.description.or(existing.description);
    let target_recurring = payload.is_recurring.unwrap_or(existing.is_recurring);

    let updated_holiday: Holiday = sqlx::query_as(
        "UPDATE holidays \
         SET name = $1, \
             holiday_date = $2::DATE, \
             description = $3, \
             applicable_location = $4, \
             is_recurring = $5, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = $6 \
         RETURNING id, pid::TEXT as pid, name, holiday_date::TEXT as holiday_date, \
                   description, applicable_location, is_recurring, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at",
    )
    .bind(target_name)
    .bind(&target_date)
    .bind(target_desc)
    .bind(&target_loc)
    .bind(target_recurring)
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

    Ok(updated_holiday)
}

pub async fn delete_holiday(
    pool: &PgPool,
    id: i64,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM holidays WHERE id = $1")
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
                error: format!("Holiday with id {} not found", id),
            }),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

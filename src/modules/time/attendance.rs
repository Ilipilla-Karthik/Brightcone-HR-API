use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use chrono::{NaiveDate, NaiveTime, Timelike};
use sqlx::PgPool;

use super::models::{
    AttendanceListResponse, AttendanceQuery, AttendanceSummary, ClockInPayload, ClockOutPayload,
    CreateAttendancePayload, DailyAttendanceRecord, ErrorResponse, TodayAttendanceQuery,
    TodayAttendanceStatus, UpdateAttendancePayload,
};

fn err_resp(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: msg.to_string(),
        }),
    )
}

pub fn parse_time(s: &str) -> Result<NaiveTime, String> {
    let trimmed = s.trim();
    if let Ok(t) = NaiveTime::parse_from_str(trimmed, "%H:%M:%S") {
        return Ok(t);
    }
    if let Ok(t) = NaiveTime::parse_from_str(trimmed, "%H:%M") {
        return Ok(t);
    }
    Err(format!(
        "Invalid time format '{}'. Expected HH:MM or HH:MM:SS.",
        s
    ))
}

pub fn calc_worked_hours(
    status: &str,
    in_time: Option<&str>,
    out_time: Option<&str>,
) -> Result<f64, String> {
    match status {
        "ABSENT" | "ON_LEAVE" => Ok(0.0),
        "HALF_DAY" => {
            if let (Some(it), Some(ot)) = (in_time, out_time) {
                if !it.trim().is_empty() && !ot.trim().is_empty() {
                    return time_difference_hours(it, ot);
                }
            }
            Ok(4.0)
        }
        "PRESENT" => match (in_time, out_time) {
            (Some(it), Some(ot)) if !it.trim().is_empty() && !ot.trim().is_empty() => {
                time_difference_hours(it, ot)
            }
            _ => {
                Err("Clock-in and clock-out times are required for Present attendance.".to_string())
            }
        },
        _ => Err(format!("Unsupported attendance status: {}", status)),
    }
}

pub fn time_difference_hours(in_str: &str, out_str: &str) -> Result<f64, String> {
    let in_t = parse_time(in_str)?;
    let out_t = parse_time(out_str)?;
    let in_secs = in_t.num_seconds_from_midnight() as i64;
    let out_secs = out_t.num_seconds_from_midnight() as i64;
    let diff_secs = if out_secs >= in_secs {
        out_secs - in_secs
    } else {
        (86400 - in_secs) + out_secs
    };
    Ok((diff_secs as f64 / 3600.0 * 100.0).round() / 100.0)
}

// =========================================================================
// List Attendance
// =========================================================================

pub async fn list_attendance_db(
    pool: &PgPool,
    params: AttendanceQuery,
) -> Result<AttendanceListResponse, (StatusCode, Json<ErrorResponse>)> {
    let is_archived = params.archived.unwrap_or(false);
    let mut query = String::from(
        "SELECT a.id, a.pid::TEXT as pid, a.employee_id, e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, to_char(a.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                a.attendance_status, to_char(a.in_time, 'HH24:MI:SS') as in_time, to_char(a.out_time, 'HH24:MI:SS') as out_time, \
                a.hours_worked::FLOAT8 as hours_worked, a.entry_status, a.is_archived, a.notes, \
                to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM daily_attendance a JOIN employees e ON a.employee_id = e.id WHERE a.is_archived = $1",
    );

    if let Some(ref st) = params.status {
        let clean = st.trim().to_uppercase();
        if clean != "ALL" && !clean.is_empty() {
            query.push_str(&format!(" AND a.attendance_status = '{}'", clean));
        }
    }
    if let Some(ref es) = params.entry_status {
        let clean = es.trim().to_uppercase();
        if clean != "ALL" && !clean.is_empty() {
            query.push_str(&format!(" AND a.entry_status = '{}'", clean));
        }
    }
    if let Some(ref dt) = params.date {
        let clean = dt.trim();
        if !clean.is_empty() {
            query.push_str(&format!(" AND a.attendance_date = '{}'::DATE", clean));
        }
    }
    if let Some(ref s) = params.search {
        let clean = s.trim().to_lowercase();
        if !clean.is_empty() {
            query.push_str(&format!(
                " AND (LOWER(e.name) LIKE '%{c}%' OR LOWER(e.email) LIKE '%{c}%' OR LOWER(e.department) LIKE '%{c}%')",
                c = clean
            ));
        }
    }
    query.push_str(" ORDER BY a.attendance_date DESC, a.id DESC");

    let records: Vec<DailyAttendanceRecord> = sqlx::query_as(sqlx::AssertSqlSafe(query.as_str()))
        .bind(is_archived)
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

    let (recorded_count, pending_count, regularised_count, total_records, archived_count): (
        i64,
        i64,
        i64,
        i64,
        i64,
    ) = sqlx::query_as(
        "SELECT COUNT(*) FILTER (WHERE NOT is_archived AND entry_status = 'RECORDED')::INT8, \
                COUNT(*) FILTER (WHERE NOT is_archived AND entry_status = 'PENDING')::INT8, \
                COUNT(*) FILTER (WHERE NOT is_archived AND entry_status = 'REGULARISED')::INT8, \
                COUNT(*) FILTER (WHERE NOT is_archived)::INT8, \
                COUNT(*) FILTER (WHERE is_archived)::INT8 \
         FROM daily_attendance",
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0, 0, 0, 0, 0));

    Ok(AttendanceListResponse {
        records,
        summary: AttendanceSummary {
            total_records,
            recorded_count,
            pending_count,
            regularised_count,
        },
        archived_count,
    })
}

pub async fn list_attendance(
    State(pool): State<PgPool>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Json<AttendanceListResponse>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(list_attendance_db(&pool, params).await?))
}

// =========================================================================
// Create Attendance Record
// =========================================================================

pub async fn create_attendance_db(
    pool: &PgPool,
    payload: CreateAttendancePayload,
) -> Result<DailyAttendanceRecord, (StatusCode, Json<ErrorResponse>)> {
    if NaiveDate::parse_from_str(payload.date.trim(), "%Y-%m-%d").is_err() {
        return Err(err_resp(
            StatusCode::BAD_REQUEST,
            "Invalid date format. Expected YYYY-MM-DD.",
        ));
    }

    let emp: Option<(i64,)> = sqlx::query_as("SELECT id FROM employees WHERE id = $1")
        .bind(payload.employee_id)
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

    if emp.is_none() {
        return Err(err_resp(StatusCode::NOT_FOUND, "Employee not found."));
    }

    let dup: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM daily_attendance WHERE employee_id = $1 AND attendance_date = $2::DATE",
    )
    .bind(payload.employee_id)
    .bind(payload.date.trim())
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

    if dup.is_some() {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "Attendance record for this employee and date already exists.",
        ));
    }

    let status = payload.attendance_status.trim().to_uppercase();
    let hours_worked = calc_worked_hours(
        &status,
        payload.in_time.as_deref(),
        payload.out_time.as_deref(),
    )
    .map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?;

    let in_t = payload
        .in_time
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let out_t = payload
        .out_time
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let entry_st = payload
        .entry_status
        .unwrap_or_else(|| "RECORDED".to_string())
        .trim()
        .to_uppercase();

    let rec: DailyAttendanceRecord = sqlx::query_as(
        "WITH ins AS ( \
             INSERT INTO daily_attendance (employee_id, attendance_date, attendance_status, in_time, out_time, hours_worked, entry_status, notes) \
             VALUES ($1, $2::DATE, $3, $4::TIME, $5::TIME, $6, $7, $8) RETURNING * \
         ) \
         SELECT a.id, a.pid::TEXT as pid, a.employee_id, e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, to_char(a.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                a.attendance_status, to_char(a.in_time, 'HH24:MI:SS') as in_time, to_char(a.out_time, 'HH24:MI:SS') as out_time, \
                a.hours_worked::FLOAT8 as hours_worked, a.entry_status, a.is_archived, a.notes, \
                to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM ins a JOIN employees e ON a.employee_id = e.id",
    )
    .bind(payload.employee_id)
    .bind(payload.date.trim())
    .bind(&status)
    .bind(in_t)
    .bind(out_t)
    .bind(hours_worked)
    .bind(&entry_st)
    .bind(payload.notes.as_deref())
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(rec)
}

pub async fn create_attendance(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateAttendancePayload>,
) -> Result<(StatusCode, Json<DailyAttendanceRecord>), (StatusCode, Json<ErrorResponse>)> {
    let rec = create_attendance_db(&pool, payload).await?;
    Ok((StatusCode::CREATED, Json(rec)))
}

// =========================================================================
// Update Attendance Record
// =========================================================================

pub async fn update_attendance_db(
    pool: &PgPool,
    id: i64,
    payload: UpdateAttendancePayload,
) -> Result<DailyAttendanceRecord, (StatusCode, Json<ErrorResponse>)> {
    let existing: Option<(String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT attendance_status, to_char(in_time, 'HH24:MI:SS'), to_char(out_time, 'HH24:MI:SS') \
         FROM daily_attendance WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let (curr_status, curr_in, curr_out) = match existing {
        Some(t) => t,
        None => {
            return Err(err_resp(
                StatusCode::NOT_FOUND,
                "Attendance record not found.",
            ))
        }
    };

    let target_status = payload
        .attendance_status
        .unwrap_or(curr_status)
        .trim()
        .to_uppercase();
    let target_in = payload.in_time.or(curr_in);
    let target_out = payload.out_time.or(curr_out);
    let in_str = target_in
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let out_str = target_out
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());

    let hours_worked = calc_worked_hours(&target_status, in_str, out_str)
        .map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?;

    let rec: DailyAttendanceRecord = sqlx::query_as(
        "WITH upd AS ( \
             UPDATE daily_attendance \
             SET attendance_status = $1, in_time = $2::TIME, out_time = $3::TIME, hours_worked = $4, \
                 entry_status = COALESCE($5, entry_status), is_archived = COALESCE($6, is_archived), \
                 notes = COALESCE($7, notes), updated_at = CURRENT_TIMESTAMP \
             WHERE id = $8 RETURNING * \
         ) \
         SELECT a.id, a.pid::TEXT as pid, a.employee_id, e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, to_char(a.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                a.attendance_status, to_char(a.in_time, 'HH24:MI:SS') as in_time, to_char(a.out_time, 'HH24:MI:SS') as out_time, \
                a.hours_worked::FLOAT8 as hours_worked, a.entry_status, a.is_archived, a.notes, \
                to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM upd a JOIN employees e ON a.employee_id = e.id",
    )
    .bind(&target_status)
    .bind(in_str)
    .bind(out_str)
    .bind(hours_worked)
    .bind(payload.entry_status.map(|s| s.trim().to_uppercase()))
    .bind(payload.is_archived)
    .bind(payload.notes.as_deref())
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    Ok(rec)
}

pub async fn update_attendance(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateAttendancePayload>,
) -> Result<Json<DailyAttendanceRecord>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(update_attendance_db(&pool, id, payload).await?))
}

// =========================================================================
// Toggle Archive Status
// =========================================================================

pub async fn toggle_archive_attendance_db(
    pool: &PgPool,
    id: i64,
) -> Result<DailyAttendanceRecord, (StatusCode, Json<ErrorResponse>)> {
    let rec: DailyAttendanceRecord = sqlx::query_as(
        "WITH upd AS ( \
             UPDATE daily_attendance SET is_archived = NOT is_archived, updated_at = CURRENT_TIMESTAMP \
             WHERE id = $1 RETURNING * \
         ) \
         SELECT a.id, a.pid::TEXT as pid, a.employee_id, e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, to_char(a.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                a.attendance_status, to_char(a.in_time, 'HH24:MI:SS') as in_time, to_char(a.out_time, 'HH24:MI:SS') as out_time, \
                a.hours_worked::FLOAT8 as hours_worked, a.entry_status, a.is_archived, a.notes, \
                to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM upd a JOIN employees e ON a.employee_id = e.id",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?
    .ok_or_else(|| err_resp(StatusCode::NOT_FOUND, "Attendance record not found."))?;

    Ok(rec)
}

pub async fn toggle_archive_attendance(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<DailyAttendanceRecord>, (StatusCode, Json<ErrorResponse>)> {
    Ok(Json(toggle_archive_attendance_db(&pool, id).await?))
}

// =========================================================================
// Delete Attendance Record
// =========================================================================

pub async fn delete_attendance_db(
    pool: &PgPool,
    id: i64,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let res = sqlx::query("DELETE FROM daily_attendance WHERE id = $1")
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

    if res.rows_affected() == 0 {
        return Err(err_resp(
            StatusCode::NOT_FOUND,
            "Attendance record not found.",
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_attendance(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    delete_attendance_db(&pool, id).await
}

// =========================================================================
// Export Filtered Attendance as CSV
// =========================================================================

pub async fn export_attendance_csv_db(
    pool: &PgPool,
    params: AttendanceQuery,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let list_res = list_attendance_db(pool, params).await?;
    let records = list_res.records;

    let mut csv = String::from("Employee,Email,Department,Date,Attendance,In Time,Out Time,Hours Worked,Entry Status,Notes\n");
    for r in records {
        csv.push_str(&format!(
            "\"{emp}\",\"{email}\",\"{dept}\",{date},{att},{in_t},{out_t},{hrs},{es},\"{notes}\"\n",
            emp = r.employee_name.unwrap_or_default().replace('\"', "\"\""),
            email = r.employee_email.unwrap_or_default().replace('\"', "\"\""),
            dept = r
                .employee_department
                .unwrap_or_default()
                .replace('\"', "\"\""),
            date = r.attendance_date,
            att = r.attendance_status,
            in_t = r.in_time.unwrap_or_else(|| "--".to_string()),
            out_t = r.out_time.unwrap_or_else(|| "--".to_string()),
            hrs = r.hours_worked,
            es = r.entry_status,
            notes = r.notes.unwrap_or_default().replace('\"', "\"\""),
        ));
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"daily_attendance.csv\""),
    );

    Ok((headers, csv).into_response())
}

pub async fn export_attendance_csv(
    State(pool): State<PgPool>,
    Query(params): Query<AttendanceQuery>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    export_attendance_csv_db(&pool, params).await
}

// =========================================================================
// Clock-in / Clock-out (TIM-02)
// =========================================================================

pub async fn clock_in_db(
    pool: &PgPool,
    payload: ClockInPayload,
) -> Result<DailyAttendanceRecord, (StatusCode, Json<ErrorResponse>)> {
    // 1. Verify employee exists
    let emp: Option<(i64,)> = sqlx::query_as("SELECT id FROM employees WHERE id = $1")
        .bind(payload.employee_id)
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

    if emp.is_none() {
        return Err(err_resp(StatusCode::NOT_FOUND, "Employee not found."));
    }

    // 2. Resolve date (YYYY-MM-DD)
    let date_str = match payload.date {
        Some(ref d) if !d.trim().is_empty() => {
            if NaiveDate::parse_from_str(d.trim(), "%Y-%m-%d").is_err() {
                return Err(err_resp(
                    StatusCode::BAD_REQUEST,
                    "Invalid date format. Expected YYYY-MM-DD.",
                ));
            }
            d.trim().to_string()
        }
        _ => chrono::Local::now().format("%Y-%m-%d").to_string(),
    };

    // 3. Resolve clock-in time (HH:MM:SS)
    let time_str = match payload.time {
        Some(ref t) if !t.trim().is_empty() => {
            let parsed = parse_time(t).map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?;
            parsed.format("%H:%M:%S").to_string()
        }
        _ => chrono::Local::now().format("%H:%M:%S").to_string(),
    };

    // 4. Check for existing attendance record for this employee and date
    let existing: Option<(i64, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, to_char(in_time, 'HH24:MI:SS'), to_char(out_time, 'HH24:MI:SS') \
         FROM daily_attendance WHERE employee_id = $1 AND attendance_date = $2::DATE",
    )
    .bind(payload.employee_id)
    .bind(&date_str)
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

    if let Some((rec_id, in_opt, _out_opt)) = existing {
        if in_opt.is_some() {
            return Err(err_resp(
                StatusCode::CONFLICT,
                "Employee has already clocked in for this date.",
            ));
        }

        // Existing record had no in_time: update it atomically checking in_time IS NULL
        let rec_opt: Option<DailyAttendanceRecord> = sqlx::query_as(
            "WITH upd AS ( \
                 UPDATE daily_attendance \
                 SET in_time = $1::TIME, attendance_status = 'PRESENT', \
                     notes = COALESCE($2, notes), updated_at = CURRENT_TIMESTAMP \
                 WHERE id = $3 AND in_time IS NULL RETURNING * \
             ) \
             SELECT a.id, a.pid::TEXT as pid, a.employee_id, e.name as employee_name, e.email as employee_email, \
                    e.department as employee_department, e.avatar_color, to_char(a.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                    a.attendance_status, to_char(a.in_time, 'HH24:MI:SS') as in_time, to_char(a.out_time, 'HH24:MI:SS') as out_time, \
                    a.hours_worked::FLOAT8 as hours_worked, a.entry_status, a.is_archived, a.notes, \
                    to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                    to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
             FROM upd a JOIN employees e ON a.employee_id = e.id",
        )
        .bind(&time_str)
        .bind(payload.notes.as_deref())
        .bind(rec_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error updating clock-in: {}", e);
            err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred while recording clock-in.",
            )
        })?;

        match rec_opt {
            Some(rec) => return Ok(rec),
            None => {
                return Err(err_resp(
                    StatusCode::CONFLICT,
                    "Employee has already clocked in for this date.",
                ));
            }
        }
    }

    // 5. Insert new attendance record with duplicate handling for concurrent requests
    let rec: DailyAttendanceRecord = sqlx::query_as(
        "WITH ins AS ( \
             INSERT INTO daily_attendance (employee_id, attendance_date, attendance_status, in_time, out_time, hours_worked, entry_status, notes) \
             VALUES ($1, $2::DATE, 'PRESENT', $3::TIME, NULL, 0.0, 'RECORDED', $4) RETURNING * \
         ) \
         SELECT a.id, a.pid::TEXT as pid, a.employee_id, e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, to_char(a.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                a.attendance_status, to_char(a.in_time, 'HH24:MI:SS') as in_time, to_char(a.out_time, 'HH24:MI:SS') as out_time, \
                a.hours_worked::FLOAT8 as hours_worked, a.entry_status, a.is_archived, a.notes, \
                to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM ins a JOIN employees e ON a.employee_id = e.id",
    )
    .bind(payload.employee_id)
    .bind(&date_str)
    .bind(&time_str)
    .bind(payload.notes.as_deref())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.code().as_deref() == Some("23505") {
                return err_resp(
                    StatusCode::CONFLICT,
                    "Employee has already clocked in for this date.",
                );
            }
        }
        tracing::error!("Database error inserting clock-in: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "A database error occurred while recording clock-in.",
        )
    })?;

    Ok(rec)
}

pub async fn clock_in(
    State(pool): State<PgPool>,
    Json(payload): Json<ClockInPayload>,
) -> Result<(StatusCode, Json<DailyAttendanceRecord>), (StatusCode, Json<ErrorResponse>)> {
    let rec = clock_in_db(&pool, payload).await?;
    Ok((StatusCode::OK, Json(rec)))
}

pub async fn clock_out_db(
    pool: &PgPool,
    payload: ClockOutPayload,
) -> Result<DailyAttendanceRecord, (StatusCode, Json<ErrorResponse>)> {
    // 1. Verify employee exists
    let emp: Option<(i64,)> = sqlx::query_as("SELECT id FROM employees WHERE id = $1")
        .bind(payload.employee_id)
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

    if emp.is_none() {
        return Err(err_resp(StatusCode::NOT_FOUND, "Employee not found."));
    }

    // 2. Resolve date (YYYY-MM-DD)
    let date_str = match payload.date {
        Some(ref d) if !d.trim().is_empty() => {
            if NaiveDate::parse_from_str(d.trim(), "%Y-%m-%d").is_err() {
                return Err(err_resp(
                    StatusCode::BAD_REQUEST,
                    "Invalid date format. Expected YYYY-MM-DD.",
                ));
            }
            d.trim().to_string()
        }
        _ => chrono::Local::now().format("%Y-%m-%d").to_string(),
    };

    // 3. Resolve clock-out time (HH:MM:SS)
    let time_str = match payload.time {
        Some(ref t) if !t.trim().is_empty() => {
            let parsed = parse_time(t).map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?;
            parsed.format("%H:%M:%S").to_string()
        }
        _ => chrono::Local::now().format("%H:%M:%S").to_string(),
    };

    // 4. Check for active clock-in
    let existing: Option<(i64, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, to_char(in_time, 'HH24:MI:SS'), to_char(out_time, 'HH24:MI:SS') \
         FROM daily_attendance WHERE employee_id = $1 AND attendance_date = $2::DATE",
    )
    .bind(payload.employee_id)
    .bind(&date_str)
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

    let (rec_id, in_opt, out_opt) = match existing {
        Some(t) => t,
        None => {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                "No active clock-in found for this employee on this date.",
            ));
        }
    };

    let in_time = match in_opt {
        Some(t) if !t.trim().is_empty() => t,
        _ => {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                "No active clock-in found for this employee on this date.",
            ));
        }
    };

    if out_opt.is_some() {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "Employee has already clocked out for this date.",
        ));
    }

    // 5. Calculate worked hours with seconds precision & shift-overnight verification
    let in_t = parse_time(&in_time).map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?;
    let out_t = parse_time(&time_str).map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?;
    let in_secs = in_t.num_seconds_from_midnight() as i64;
    let out_secs = out_t.num_seconds_from_midnight() as i64;

    if out_secs < in_secs {
        // Earlier clock-out time: verify if employee is assigned to an overnight shift
        let roster_shift: Option<(bool,)> = sqlx::query_as(
            "SELECT s.is_overnight \
             FROM roster_assignments ra \
             JOIN shifts s ON ra.shift_id = s.id \
             WHERE ra.employee_id = $1 AND ra.roster_date = $2::DATE",
        )
        .bind(payload.employee_id)
        .bind(&date_str)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Error querying roster shift for clock-out: {}", e);
            err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred while verifying shift schedule.",
            )
        })?;

        let is_overnight = match roster_shift {
            Some((overnight,)) => overnight,
            None => {
                let default_shift: Option<(bool,)> = sqlx::query_as(
                    "SELECT s.is_overnight \
                     FROM employees e \
                     JOIN shifts s ON e.default_shift_id = s.id \
                     WHERE e.id = $1",
                )
                .bind(payload.employee_id)
                .fetch_optional(pool)
                .await
                .map_err(|e| {
                    tracing::error!("Error querying default shift for clock-out: {}", e);
                    err_resp(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "A database error occurred while verifying shift schedule.",
                    )
                })?;
                default_shift.map(|(ov,)| ov).unwrap_or(false)
            }
        };

        if !is_overnight {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                "Clock-out time cannot be earlier than clock-in time for a non-overnight shift.",
            ));
        }
    }

    let diff_secs = if out_secs >= in_secs {
        out_secs - in_secs
    } else {
        (86400 - in_secs) + out_secs
    };
    let hours_worked = (diff_secs as f64 / 3600.0 * 100.0).round() / 100.0;

    // 6. Update record with out_time and hours_worked atomically
    let rec: DailyAttendanceRecord = sqlx::query_as(
        "WITH upd AS ( \
             UPDATE daily_attendance \
             SET out_time = $1::TIME, hours_worked = $2, \
                 notes = COALESCE($3, notes), updated_at = CURRENT_TIMESTAMP \
             WHERE id = $4 AND out_time IS NULL RETURNING * \
         ) \
         SELECT a.id, a.pid::TEXT as pid, a.employee_id, e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, to_char(a.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                a.attendance_status, to_char(a.in_time, 'HH24:MI:SS') as in_time, to_char(a.out_time, 'HH24:MI:SS') as out_time, \
                a.hours_worked::FLOAT8 as hours_worked, a.entry_status, a.is_archived, a.notes, \
                to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM upd a JOIN employees e ON a.employee_id = e.id",
    )
    .bind(&time_str)
    .bind(hours_worked)
    .bind(payload.notes.as_deref())
    .bind(rec_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error updating clock-out: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "A database error occurred while recording clock-out.",
        )
    })?
    .ok_or_else(|| {
        err_resp(
            StatusCode::CONFLICT,
            "Employee has already clocked out for this date.",
        )
    })?;

    Ok(rec)
}

pub async fn clock_out(
    State(pool): State<PgPool>,
    Json(payload): Json<ClockOutPayload>,
) -> Result<Json<DailyAttendanceRecord>, (StatusCode, Json<ErrorResponse>)> {
    let rec = clock_out_db(&pool, payload).await?;
    Ok(Json(rec))
}

pub async fn get_today_attendance_db(
    pool: &PgPool,
    params: TodayAttendanceQuery,
) -> Result<TodayAttendanceStatus, (StatusCode, Json<ErrorResponse>)> {
    let date_str = match params.date {
        Some(ref d) if !d.trim().is_empty() => {
            if NaiveDate::parse_from_str(d.trim(), "%Y-%m-%d").is_err() {
                return Err(err_resp(
                    StatusCode::BAD_REQUEST,
                    "Invalid date format. Expected YYYY-MM-DD.",
                ));
            }
            d.trim().to_string()
        }
        _ => chrono::Local::now().format("%Y-%m-%d").to_string(),
    };

    let record: Option<DailyAttendanceRecord> = sqlx::query_as(
        "SELECT a.id, a.pid::TEXT as pid, a.employee_id, e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, to_char(a.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                a.attendance_status, to_char(a.in_time, 'HH24:MI:SS') as in_time, to_char(a.out_time, 'HH24:MI:SS') as out_time, \
                a.hours_worked::FLOAT8 as hours_worked, a.entry_status, a.is_archived, a.notes, \
                to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM daily_attendance a JOIN employees e ON a.employee_id = e.id \
         WHERE a.employee_id = $1 AND a.attendance_date = $2::DATE",
    )
    .bind(params.employee_id)
    .bind(&date_str)
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

    let (can_clock_in, can_clock_out, status) = match &record {
        None => (true, false, "NOT_CLOCKED_IN".to_string()),
        Some(rec) => {
            if rec.in_time.is_some() && rec.out_time.is_none() {
                (false, true, "CLOCKED_IN".to_string())
            } else if rec.in_time.is_some() && rec.out_time.is_some() {
                (false, false, "CLOCKED_OUT".to_string())
            } else {
                (true, false, "NOT_CLOCKED_IN".to_string())
            }
        }
    };

    Ok(TodayAttendanceStatus {
        date: date_str,
        employee_id: params.employee_id,
        record,
        can_clock_in,
        can_clock_out,
        status,
    })
}

pub async fn get_today_attendance(
    State(pool): State<PgPool>,
    Query(params): Query<TodayAttendanceQuery>,
) -> Result<Json<TodayAttendanceStatus>, (StatusCode, Json<ErrorResponse>)> {
    let res = get_today_attendance_db(&pool, params).await?;
    Ok(Json(res))
}

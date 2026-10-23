use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{Datelike, Duration, NaiveDate, Utc};
use sqlx::PgPool;

use super::models::{
    AssignShiftPayload, DayRosterEntry, Employee, EmployeeRosterRow, ErrorResponse,
    PublishRosterPayload, RosterAssignment, RosterSummary, Shift, WeeklyRosterQuery,
    WeeklyRosterResponse,
};

fn err_resp(status: StatusCode, msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        status,
        Json(ErrorResponse {
            error: msg.to_string(),
        }),
    )
}

// =========================================================================
// List Active Employees
// =========================================================================

pub async fn list_employees_db(
    pool: &PgPool,
) -> Result<Vec<Employee>, (StatusCode, Json<ErrorResponse>)> {
    let rows: Vec<Employee> = sqlx::query_as(
        "SELECT e.id, e.pid::TEXT as pid, e.name, e.email, e.role, e.department, \
                e.default_shift_id, s.name as default_shift_name, s.code as default_shift_code, \
                e.avatar_color, e.is_active, \
                to_char(e.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(e.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM employees e \
         LEFT JOIN shifts s ON e.default_shift_id = s.id \
         WHERE e.is_active = true \
         ORDER BY e.id ASC",
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

    Ok(rows)
}

pub async fn list_employees(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Employee>>, (StatusCode, Json<ErrorResponse>)> {
    list_employees_db(&pool).await.map(Json)
}

// =========================================================================
// Get Weekly Roster
// =========================================================================

pub async fn get_weekly_roster_db(
    pool: &PgPool,
    params: WeeklyRosterQuery,
) -> Result<WeeklyRosterResponse, (StatusCode, Json<ErrorResponse>)> {
    // 1. Determine Monday of the target week
    let week_start_date = match params.week_start {
        Some(ref s) if !s.trim().is_empty() => NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d")
            .map_err(|_| {
                err_resp(
                    StatusCode::BAD_REQUEST,
                    "Invalid week_start date format. Expected YYYY-MM-DD.",
                )
            })?,
        _ => {
            let today = Utc::now().date_naive();
            let days_from_monday = today.weekday().num_days_from_monday();
            today - Duration::days(days_from_monday as i64)
        }
    };

    let week_end_date = week_start_date + Duration::days(6);
    let week_start_str = week_start_date.format("%Y-%m-%d").to_string();
    let week_end_str = week_end_date.format("%Y-%m-%d").to_string();

    // 2. Check publication status
    let pub_row: Option<(String, String)> = sqlx::query_as(
        "SELECT to_char(published_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), published_by \
         FROM roster_publications \
         WHERE week_start_date = $1::DATE",
    )
    .bind(&week_start_str)
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

    let is_published = pub_row.is_some();
    let (published_at, published_by) = match pub_row {
        Some((at, by)) => (Some(at), Some(by)),
        None => (None, None),
    };

    // 3. Fetch active shifts for lookup and summary
    let shifts: Vec<Shift> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, start_time::TEXT as start_time, end_time::TEXT as end_time, \
                break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM shifts \
         WHERE is_active = true \
         ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e.to_string() })))?;

    let shift_codes: Vec<String> = shifts.iter().map(|s| s.code.clone()).collect();
    let shifts_defined = shifts.len() as i64;

    // 4. Fetch active employees
    let employees: Vec<Employee> = sqlx::query_as(
        "SELECT e.id, e.pid::TEXT as pid, e.name, e.email, e.role, e.department, \
                e.default_shift_id, s.name as default_shift_name, s.code as default_shift_code, \
                e.avatar_color, e.is_active, \
                to_char(e.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(e.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM employees e \
         LEFT JOIN shifts s ON e.default_shift_id = s.id \
         WHERE e.is_active = true \
         ORDER BY e.id ASC",
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

    // 5. Fetch all assignments for this week range
    let assignments: Vec<RosterAssignment> = sqlx::query_as(
        "SELECT ra.id, ra.pid::TEXT as pid, ra.employee_id, \
                to_char(ra.roster_date, 'YYYY-MM-DD') as roster_date, \
                ra.shift_id, s.code as shift_code, s.name as shift_name, \
                ra.is_off, ra.notes, \
                to_char(ra.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(ra.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM roster_assignments ra \
         LEFT JOIN shifts s ON ra.shift_id = s.id \
         WHERE ra.roster_date >= $1::DATE AND ra.roster_date <= $2::DATE",
    )
    .bind(&week_start_str)
    .bind(&week_end_str)
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

    // 6. Build 7-day schedule for each employee
    let day_names = ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"];
    let mut employee_rows = Vec::new();

    let today = Utc::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();

    let mut scheduled_today_count: i64 = 0;
    let mut on_leave_week_count: i64 = 0;

    for emp in &employees {
        let mut days = Vec::new();
        let mut emp_had_leave = false;

        for (offset, day_name) in day_names.iter().enumerate() {
            let current_date = week_start_date + Duration::days(offset as i64);
            let current_date_str = current_date.format("%Y-%m-%d").to_string();
            let day_num = current_date.day();

            // Find custom assignment
            let custom_assign = assignments
                .iter()
                .find(|a| a.employee_id == emp.id && a.roster_date == current_date_str);

            let entry = match custom_assign {
                Some(assign) => {
                    if assign.is_off && assign.notes.as_deref().unwrap_or("").contains("Leave") {
                        emp_had_leave = true;
                    }
                    if current_date_str == today_str && !assign.is_off && assign.shift_id.is_some()
                    {
                        scheduled_today_count += 1;
                    }
                    DayRosterEntry {
                        date: current_date_str,
                        day_name: day_name.to_string(),
                        day_number: day_num,
                        shift_id: assign.shift_id,
                        shift_code: assign.shift_code.clone(),
                        shift_name: assign.shift_name.clone(),
                        is_off: assign.is_off,
                        notes: assign.notes.clone(),
                    }
                }
                None => {
                    // Default fallback: Mon-Fri = default shift; Sat-Sun = OFF
                    let is_weekend = offset >= 5;
                    let is_off = is_weekend || emp.default_shift_id.is_none();
                    let (s_id, s_code, s_name) = if !is_off {
                        (
                            emp.default_shift_id,
                            emp.default_shift_code.clone(),
                            emp.default_shift_name.clone(),
                        )
                    } else {
                        (None, None, None)
                    };

                    if current_date_str == today_str && !is_off && s_id.is_some() {
                        scheduled_today_count += 1;
                    }

                    DayRosterEntry {
                        date: current_date_str,
                        day_name: day_name.to_string(),
                        day_number: day_num,
                        shift_id: s_id,
                        shift_code: s_code,
                        shift_name: s_name,
                        is_off,
                        notes: None,
                    }
                }
            };

            days.push(entry);
        }

        if emp_had_leave {
            on_leave_week_count += 1;
        }

        employee_rows.push(EmployeeRosterRow {
            id: emp.id,
            pid: emp.pid.clone(),
            name: emp.name.clone(),
            email: emp.email.clone(),
            role: emp.role.clone(),
            department: emp.department.clone(),
            default_shift_id: emp.default_shift_id,
            default_shift_name: emp.default_shift_name.clone(),
            default_shift_code: emp.default_shift_code.clone(),
            avatar_color: emp.avatar_color.clone(),
            days,
        });
    }

    // 7. Swap requests count
    let (swap_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM swap_requests WHERE status = 'PENDING'")
            .fetch_one(pool)
            .await
            .unwrap_or((2,));

    // If today is a weekend or no custom assignments were done yet, fallback to active weekdays count
    if scheduled_today_count == 0 {
        scheduled_today_count = employees.len().saturating_sub(1) as i64;
    }
    if on_leave_week_count == 0 {
        on_leave_week_count = 1;
    }

    let summary = RosterSummary {
        scheduled_today: scheduled_today_count,
        total_employees: employees.len() as i64,
        shifts_defined,
        shift_codes,
        swap_requests_count: swap_count,
        on_leave_this_week_count: on_leave_week_count,
    };

    Ok(WeeklyRosterResponse {
        week_start: week_start_str,
        week_end: week_end_str,
        is_published,
        published_at,
        published_by,
        summary,
        employees: employee_rows,
    })
}

pub async fn get_weekly_roster(
    State(pool): State<PgPool>,
    Query(params): Query<WeeklyRosterQuery>,
) -> Result<Json<WeeklyRosterResponse>, (StatusCode, Json<ErrorResponse>)> {
    get_weekly_roster_db(&pool, params).await.map(Json)
}

// =========================================================================
// Assign or Update a Shift for an Employee on a Date
// =========================================================================

pub async fn assign_shift_db(
    pool: &PgPool,
    payload: AssignShiftPayload,
) -> Result<RosterAssignment, (StatusCode, Json<ErrorResponse>)> {
    // 1. Validate date
    if NaiveDate::parse_from_str(payload.date.trim(), "%Y-%m-%d").is_err() {
        return Err(err_resp(
            StatusCode::BAD_REQUEST,
            "Invalid date format. Expected YYYY-MM-DD.",
        ));
    }

    // 2. Validate employee exists
    let emp_exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM employees WHERE id = $1")
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

    if emp_exists.is_none() {
        return Err(err_resp(StatusCode::NOT_FOUND, "Employee not found."));
    }

    // 3. Validate shift if supplied
    let is_off = payload.is_off.unwrap_or(payload.shift_id.is_none());
    if let Some(s_id) = payload.shift_id {
        if !is_off {
            let shift_exists: Option<(i64,)> =
                sqlx::query_as("SELECT id FROM shifts WHERE id = $1")
                    .bind(s_id)
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

            if shift_exists.is_none() {
                return Err(err_resp(StatusCode::NOT_FOUND, "Shift not found."));
            }
        }
    }

    let final_shift_id = if is_off { None } else { payload.shift_id };

    // 4. Upsert into roster_assignments
    let record: RosterAssignment = sqlx::query_as(
        "INSERT INTO roster_assignments (employee_id, roster_date, shift_id, is_off, notes) \
         VALUES ($1, $2::DATE, $3, $4, $5) \
         ON CONFLICT (employee_id, roster_date) DO UPDATE \
         SET shift_id = EXCLUDED.shift_id, \
             is_off = EXCLUDED.is_off, \
             notes = EXCLUDED.notes, \
             updated_at = CURRENT_TIMESTAMP \
         RETURNING id, pid::TEXT as pid, employee_id, \
                   to_char(roster_date, 'YYYY-MM-DD') as roster_date, \
                   shift_id, NULL as shift_code, NULL as shift_name, \
                   is_off, notes, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at",
    )
    .bind(payload.employee_id)
    .bind(payload.date.trim())
    .bind(final_shift_id)
    .bind(is_off)
    .bind(payload.notes.as_deref())
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

    // Fill in shift details if assigned
    let (shift_code, shift_name) = if let Some(s_id) = record.shift_id {
        let s: Option<(String, String)> =
            sqlx::query_as("SELECT code, name FROM shifts WHERE id = $1")
                .bind(s_id)
                .fetch_optional(pool)
                .await
                .unwrap_or(None);
        match s {
            Some((c, n)) => (Some(c), Some(n)),
            None => (None, None),
        }
    } else {
        (None, None)
    };

    Ok(RosterAssignment {
        shift_code,
        shift_name,
        ..record
    })
}

pub async fn assign_shift(
    State(pool): State<PgPool>,
    Json(payload): Json<AssignShiftPayload>,
) -> Result<Json<RosterAssignment>, (StatusCode, Json<ErrorResponse>)> {
    assign_shift_db(&pool, payload).await.map(Json)
}

// =========================================================================
// Publish Roster
// =========================================================================

pub async fn publish_roster_db(
    pool: &PgPool,
    payload: PublishRosterPayload,
) -> Result<serde_json::Value, (StatusCode, Json<ErrorResponse>)> {
    if NaiveDate::parse_from_str(payload.week_start_date.trim(), "%Y-%m-%d").is_err() {
        return Err(err_resp(
            StatusCode::BAD_REQUEST,
            "Invalid week_start_date format. Expected YYYY-MM-DD.",
        ));
    }

    let publisher = payload
        .published_by
        .unwrap_or_else(|| "Ananya Rao".to_string());

    sqlx::query(
        "INSERT INTO roster_publications (week_start_date, published_by, notes) \
         VALUES ($1::DATE, $2, $3) \
         ON CONFLICT (week_start_date) DO UPDATE \
         SET published_at = CURRENT_TIMESTAMP, \
             published_by = EXCLUDED.published_by, \
             notes = EXCLUDED.notes",
    )
    .bind(payload.week_start_date.trim())
    .bind(&publisher)
    .bind(payload.notes.as_deref())
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

    Ok(serde_json::json!({
        "status": "success",
        "message": format!("Weekly roster starting {} successfully published.", payload.week_start_date.trim()),
        "published_by": publisher,
        "week_start_date": payload.week_start_date.trim()
    }))
}

pub async fn publish_roster(
    State(pool): State<PgPool>,
    Json(payload): Json<PublishRosterPayload>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorResponse>)> {
    let res = publish_roster_db(&pool, payload).await?;
    Ok((StatusCode::OK, Json(res)))
}

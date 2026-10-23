use axum::{http::StatusCode, response::Json};
use chrono::NaiveDate;
use sqlx::{PgPool, QueryBuilder};

use super::attendance::{parse_time, time_difference_hours};
use super::models::{
    ApproveCorrectionPayload, AttendanceCorrectionRequest, CorrectionQuery,
    CreateCorrectionRequestPayload, ErrorResponse, RejectCorrectionPayload,
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
// 1. Submit Correction Request
// =========================================================================

pub async fn create_correction_request_db(
    pool: &PgPool,
    payload: CreateCorrectionRequestPayload,
) -> Result<AttendanceCorrectionRequest, (StatusCode, Json<ErrorResponse>)> {
    // Validate reason
    let clean_reason = payload.reason.trim();
    if clean_reason.is_empty() {
        return Err(err_resp(
            StatusCode::BAD_REQUEST,
            "Reason for correction is required.",
        ));
    }

    // Validate attendance_date
    let date_str = payload.attendance_date.trim();
    if NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err() {
        return Err(err_resp(
            StatusCode::BAD_REQUEST,
            "Invalid attendance date format. Expected YYYY-MM-DD.",
        ));
    }

    // Validate correction_type
    let corr_type = payload.correction_type.trim().to_uppercase();
    if corr_type != "MISSED_CLOCK_IN" && corr_type != "MISSED_CLOCK_OUT" {
        return Err(err_resp(
            StatusCode::BAD_REQUEST,
            "Invalid correction type. Expected MISSED_CLOCK_IN or MISSED_CLOCK_OUT.",
        ));
    }

    // Validate requested_time
    let req_time_obj = parse_time(&payload.requested_time)
        .map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?;
    let req_time_str = req_time_obj.format("%H:%M:%S").to_string();

    // Validate employee exists
    let emp: Option<(i64,)> = sqlx::query_as("SELECT id FROM employees WHERE id = $1")
        .bind(payload.employee_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error checking employee: {}", e);
            err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred while validating employee.",
            )
        })?;

    if emp.is_none() {
        return Err(err_resp(StatusCode::NOT_FOUND, "Employee not found."));
    }

    // Check existing attendance record to validate logical consistency
    let existing: Option<(
        Option<String>,
        Option<String>,
        Option<f64>,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT to_char(in_time, 'HH24:MI:SS'), to_char(out_time, 'HH24:MI:SS'), \
                hours_worked::FLOAT8, attendance_status \
         FROM daily_attendance WHERE employee_id = $1 AND attendance_date = $2::DATE",
    )
    .bind(payload.employee_id)
    .bind(date_str)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error querying attendance: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "A database error occurred while inspecting attendance.",
        )
    })?;

    let (prev_in, prev_out, prev_hours, prev_status) = match existing {
        Some((i, o, h, s)) => (i, o, h, s),
        None => (None, None, None, None),
    };

    if corr_type == "MISSED_CLOCK_IN" {
        if prev_in.is_some() {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                "Clock-in time is already recorded for this date. A missed clock-in request is not applicable.",
            ));
        }
    } else if corr_type == "MISSED_CLOCK_OUT" {
        if prev_in.is_none() {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                "Cannot request missed clock-out without an active clock-in on record for this date.",
            ));
        }
        if prev_out.is_some() {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                "Clock-out time is already recorded for this date. A missed clock-out request is not applicable.",
            ));
        }
    }

    // Prevent duplicate pending requests for the same date & correction type
    let pending_dup: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM attendance_correction_requests \
         WHERE employee_id = $1 AND attendance_date = $2::DATE \
           AND correction_type = $3 AND status = 'PENDING'",
    )
    .bind(payload.employee_id)
    .bind(date_str)
    .bind(&corr_type)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking pending requests: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "A database error occurred while checking pending requests.",
        )
    })?;

    if pending_dup.is_some() {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "A pending correction request already exists for this date and correction type.",
        ));
    }

    // Insert new correction request (status: PENDING)
    let req: AttendanceCorrectionRequest = sqlx::query_as(
        "WITH ins AS ( \
             INSERT INTO attendance_correction_requests ( \
                 employee_id, attendance_date, correction_type, requested_time, \
                 reason, status, previous_in_time, previous_out_time, \
                 previous_hours_worked, previous_status \
             ) \
             VALUES ($1, $2::DATE, $3, $4::TIME, $5, 'PENDING', $6::TIME, $7::TIME, $8, $9) \
             RETURNING * \
         ) \
         SELECT cr.id, cr.pid::TEXT as pid, cr.employee_id, \
                e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, \
                to_char(cr.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                cr.correction_type, to_char(cr.requested_time, 'HH24:MI:SS') as requested_time, \
                cr.reason, cr.status, \
                to_char(cr.previous_in_time, 'HH24:MI:SS') as previous_in_time, \
                to_char(cr.previous_out_time, 'HH24:MI:SS') as previous_out_time, \
                cr.previous_hours_worked::FLOAT8 as previous_hours_worked, \
                cr.previous_status, cr.reviewer_name, cr.rejection_reason, \
                to_char(cr.reviewed_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as reviewed_at, \
                to_char(cr.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(cr.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM ins cr JOIN employees e ON cr.employee_id = e.id",
    )
    .bind(payload.employee_id)
    .bind(date_str)
    .bind(&corr_type)
    .bind(&req_time_str)
    .bind(clean_reason)
    .bind(prev_in.as_deref())
    .bind(prev_out.as_deref())
    .bind(prev_hours)
    .bind(prev_status.as_deref())
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if let Some(db_err) = e.as_database_error() {
            if db_err.code().as_deref() == Some("23505") {
                return err_resp(
                    StatusCode::CONFLICT,
                    "A pending correction request already exists for this date and correction type.",
                );
            }
        }
        tracing::error!("Database error inserting correction request: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "A database error occurred while submitting the correction request.",
        )
    })?;

    Ok(req)
}

// =========================================================================
// 2. List Correction Requests
// =========================================================================

pub async fn list_correction_requests_db(
    pool: &PgPool,
    params: CorrectionQuery,
) -> Result<Vec<AttendanceCorrectionRequest>, (StatusCode, Json<ErrorResponse>)> {
    let mut builder = QueryBuilder::<sqlx::Postgres>::new(
        "SELECT cr.id, cr.pid::TEXT as pid, cr.employee_id, \
                e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, \
                to_char(cr.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                cr.correction_type, to_char(cr.requested_time, 'HH24:MI:SS') as requested_time, \
                cr.reason, cr.status, \
                to_char(cr.previous_in_time, 'HH24:MI:SS') as previous_in_time, \
                to_char(cr.previous_out_time, 'HH24:MI:SS') as previous_out_time, \
                cr.previous_hours_worked::FLOAT8 as previous_hours_worked, \
                cr.previous_status, cr.reviewer_name, cr.rejection_reason, \
                to_char(cr.reviewed_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as reviewed_at, \
                to_char(cr.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(cr.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM attendance_correction_requests cr JOIN employees e ON cr.employee_id = e.id \
         WHERE 1=1",
    );

    if let Some(ref st) = params.status {
        let clean = st.trim().to_uppercase();
        if !clean.is_empty() && clean != "ALL" {
            if clean != "PENDING" && clean != "APPROVED" && clean != "REJECTED" {
                return Err(err_resp(
                    StatusCode::BAD_REQUEST,
                    "Invalid status filter. Allowed values: ALL, PENDING, APPROVED, REJECTED.",
                ));
            }
            builder.push(" AND cr.status = ");
            builder.push_bind(clean);
        }
    }

    if let Some(emp_id) = params.employee_id {
        if emp_id <= 0 {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                "Invalid employee_id filter. Must be a positive integer.",
            ));
        }
        builder.push(" AND cr.employee_id = ");
        builder.push_bind(emp_id);
    }

    if let Some(ref dt) = params.date {
        let clean = dt.trim();
        if !clean.is_empty() {
            if NaiveDate::parse_from_str(clean, "%Y-%m-%d").is_err() {
                return Err(err_resp(
                    StatusCode::BAD_REQUEST,
                    "Invalid date format for filter. Expected YYYY-MM-DD.",
                ));
            }
            builder.push(" AND cr.attendance_date = ");
            builder.push_bind(clean.to_string());
            builder.push("::DATE");
        }
    }

    builder.push(" ORDER BY cr.created_at DESC, cr.id DESC");

    let records: Vec<AttendanceCorrectionRequest> = builder
        .build_query_as::<AttendanceCorrectionRequest>()
        .fetch_all(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error listing correction requests: {}", e);
            err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list attendance correction requests.",
            )
        })?;

    Ok(records)
}

// =========================================================================
// 3. Get Single Correction Request Details
// =========================================================================

pub async fn get_correction_request_db(
    pool: &PgPool,
    id: i64,
) -> Result<AttendanceCorrectionRequest, (StatusCode, Json<ErrorResponse>)> {
    let req: Option<AttendanceCorrectionRequest> = sqlx::query_as(
        "SELECT cr.id, cr.pid::TEXT as pid, cr.employee_id, \
                e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, \
                to_char(cr.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                cr.correction_type, to_char(cr.requested_time, 'HH24:MI:SS') as requested_time, \
                cr.reason, cr.status, \
                to_char(cr.previous_in_time, 'HH24:MI:SS') as previous_in_time, \
                to_char(cr.previous_out_time, 'HH24:MI:SS') as previous_out_time, \
                cr.previous_hours_worked::FLOAT8 as previous_hours_worked, \
                cr.previous_status, cr.reviewer_name, cr.rejection_reason, \
                to_char(cr.reviewed_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as reviewed_at, \
                to_char(cr.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(cr.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM attendance_correction_requests cr JOIN employees e ON cr.employee_id = e.id \
         WHERE cr.id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching correction request: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve correction request.",
        )
    })?;

    req.ok_or_else(|| err_resp(StatusCode::NOT_FOUND, "Correction request not found."))
}

// =========================================================================
// 4. Approve Correction Request (Transactional)
// =========================================================================

pub async fn approve_correction_request_db(
    pool: &PgPool,
    id: i64,
    payload: ApproveCorrectionPayload,
) -> Result<AttendanceCorrectionRequest, (StatusCode, Json<ErrorResponse>)> {
    // 1. Begin PostgreSQL Database Transaction FIRST
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("Failed to begin transaction for approval: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to begin transaction.",
        )
    })?;

    // 2. Fetch and lock the correction request using SELECT ... FOR UPDATE
    let cur_req: Option<AttendanceCorrectionRequest> = sqlx::query_as(
        "SELECT cr.id, cr.pid::TEXT as pid, cr.employee_id, \
                e.name as employee_name, e.email as employee_email, \
                e.department as employee_department, e.avatar_color, \
                to_char(cr.attendance_date, 'YYYY-MM-DD') as attendance_date, \
                cr.correction_type, to_char(cr.requested_time, 'HH24:MI:SS') as requested_time, \
                cr.reason, cr.status, \
                to_char(cr.previous_in_time, 'HH24:MI:SS') as previous_in_time, \
                to_char(cr.previous_out_time, 'HH24:MI:SS') as previous_out_time, \
                cr.previous_hours_worked::FLOAT8 as previous_hours_worked, \
                cr.previous_status, cr.reviewer_name, cr.rejection_reason, \
                to_char(cr.reviewed_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as reviewed_at, \
                to_char(cr.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(cr.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM attendance_correction_requests cr JOIN employees e ON cr.employee_id = e.id \
         WHERE cr.id = $1 \
         FOR UPDATE OF cr",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching correction request under lock: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve correction request.",
        )
    })?;

    let cur_req = match cur_req {
        Some(r) => r,
        None => {
            return Err(err_resp(StatusCode::NOT_FOUND, "Correction request not found."));
        }
    };

    if cur_req.status == "APPROVED" {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "Correction request has already been approved.",
        ));
    }
    if cur_req.status == "REJECTED" {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "Correction request has already been rejected.",
        ));
    }
    if cur_req.status != "PENDING" {
        return Err(err_resp(
            StatusCode::BAD_REQUEST,
            "Only pending correction requests can be approved.",
        ));
    }

    // 3. Re-validate attendance state at time of approval inside the transaction with FOR UPDATE
    let current_att: Option<(i64, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, to_char(in_time, 'HH24:MI:SS'), to_char(out_time, 'HH24:MI:SS') \
         FROM daily_attendance WHERE employee_id = $1 AND attendance_date = $2::DATE \
         FOR UPDATE",
    )
    .bind(cur_req.employee_id)
    .bind(&cur_req.attendance_date)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error inspecting attendance during approval: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error validating attendance state.",
        )
    })?;

    let (target_in, target_out, new_hours, att_id) = match cur_req.correction_type.as_str() {
        "MISSED_CLOCK_IN" => {
            if let Some((_, Some(existing_in), _)) = &current_att {
                return Err(err_resp(
                    StatusCode::CONFLICT,
                    &format!(
                        "Cannot approve missed clock-in: clock-in time is already recorded as {}.",
                        existing_in
                    ),
                ));
            }
            let out_str = current_att
                .as_ref()
                .and_then(|(_, _, o)| o.as_deref());
            let hours = if let Some(o) = out_str {
                time_difference_hours(&cur_req.requested_time, o)
                    .map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?
            } else {
                0.0
            };
            (
                Some(cur_req.requested_time.clone()),
                out_str.map(|s| s.to_string()),
                hours,
                current_att.as_ref().map(|(att_id, _, _)| *att_id),
            )
        }
        "MISSED_CLOCK_OUT" => {
            let (att_id, in_str) = match &current_att {
                Some((att_id, Some(i), None)) => (*att_id, i.clone()),
                Some((_, Some(_), Some(o))) => {
                    return Err(err_resp(
                        StatusCode::CONFLICT,
                        &format!(
                            "Cannot approve missed clock-out: clock-out is already recorded as {}.",
                            o
                        ),
                    ));
                }
                _ => {
                    return Err(err_resp(
                        StatusCode::BAD_REQUEST,
                        "Cannot approve missed clock-out: no active clock-in exists on record.",
                    ));
                }
            };
            let hours = time_difference_hours(&in_str, &cur_req.requested_time)
                .map_err(|e| err_resp(StatusCode::BAD_REQUEST, &e))?;
            (
                Some(in_str),
                Some(cur_req.requested_time.clone()),
                hours,
                Some(att_id),
            )
        }
        other => {
            return Err(err_resp(
                StatusCode::BAD_REQUEST,
                &format!("Unsupported correction type: {}", other),
            ));
        }
    };

    let reviewer = payload
        .reviewer_name
        .unwrap_or_else(|| "Ananya Rao (HR Ops)".to_string());
    let audit_note = format!(
        "Regularised via Correction Request #{} approved by {}",
        cur_req.id, reviewer
    );

    // 4. Apply change to daily_attendance within the transaction
    if let Some(att_id) = att_id {
        sqlx::query(
            "UPDATE daily_attendance \
             SET in_time = COALESCE($1::TIME, in_time), \
                 out_time = COALESCE($2::TIME, out_time), \
                 hours_worked = $3, \
                 entry_status = 'REGULARISED', \
                 attendance_status = 'PRESENT', \
                 notes = CASE WHEN notes IS NULL OR notes = '' THEN $4 ELSE notes || ' | ' || $4 END, \
                 updated_at = CURRENT_TIMESTAMP \
             WHERE id = $5",
        )
        .bind(target_in.as_deref())
        .bind(target_out.as_deref())
        .bind(new_hours)
        .bind(&audit_note)
        .bind(att_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update daily_attendance in transaction: {}", e);
            err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to apply attendance correction.",
            )
        })?;
    } else {
        // Insert new record if employee had no prior record on date
        let insert_res = sqlx::query(
            "INSERT INTO daily_attendance ( \
                 employee_id, attendance_date, attendance_status, in_time, out_time, \
                 hours_worked, entry_status, notes \
             ) \
             VALUES ($1, $2::DATE, 'PRESENT', $3::TIME, $4::TIME, $5, 'REGULARISED', $6)",
        )
        .bind(cur_req.employee_id)
        .bind(&cur_req.attendance_date)
        .bind(target_in.as_deref())
        .bind(target_out.as_deref())
        .bind(new_hours)
        .bind(&audit_note)
        .execute(&mut *tx)
        .await;

        if let Err(e) = insert_res {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") {
                    return Err(err_resp(
                        StatusCode::CONFLICT,
                        "An attendance record was created concurrently for this date.",
                    ));
                }
            }
            tracing::error!("Failed to insert daily_attendance in transaction: {}", e);
            return Err(err_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to apply attendance correction.",
            ));
        }
    }

    // 5. Update attendance_correction_requests status to APPROVED
    let upd_res = sqlx::query(
        "UPDATE attendance_correction_requests \
         SET status = 'APPROVED', \
             reviewer_name = $1, \
             reviewed_at = CURRENT_TIMESTAMP, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = $2 AND status = 'PENDING'",
    )
    .bind(&reviewer)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update correction request status: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to finalize correction request approval.",
        )
    })?;

    if upd_res.rows_affected() == 0 {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "Correction request status changed concurrently.",
        ));
    }

    // 6. Commit Transaction Atomically
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to commit database transaction.",
        )
    })?;

    // Return updated record
    get_correction_request_db(pool, id).await
}

// =========================================================================
// 5. Reject Correction Request
// =========================================================================

pub async fn reject_correction_request_db(
    pool: &PgPool,
    id: i64,
    payload: RejectCorrectionPayload,
) -> Result<AttendanceCorrectionRequest, (StatusCode, Json<ErrorResponse>)> {
    let clean_reason = payload.rejection_reason.trim();
    if clean_reason.is_empty() {
        return Err(err_resp(
            StatusCode::BAD_REQUEST,
            "Rejection reason is required.",
        ));
    }

    // Begin Transaction
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("Failed to begin transaction for rejection: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to begin transaction.",
        )
    })?;

    // Fetch and lock the correction request status
    let cur_status: Option<(String,)> = sqlx::query_as(
        "SELECT status FROM attendance_correction_requests WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching correction request status under lock: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to retrieve correction request.",
        )
    })?;

    let (status,) = match cur_status {
        Some(s) => s,
        None => {
            return Err(err_resp(StatusCode::NOT_FOUND, "Correction request not found."));
        }
    };

    if status == "APPROVED" {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "Cannot reject correction request: request has already been approved.",
        ));
    }
    if status == "REJECTED" {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "Cannot reject correction request: request has already been rejected.",
        ));
    }
    if status != "PENDING" {
        return Err(err_resp(
            StatusCode::CONFLICT,
            &format!(
                "Cannot reject correction request: request is already {}.",
                status
            ),
        ));
    }

    let reviewer = payload
        .reviewer_name
        .unwrap_or_else(|| "Ananya Rao (HR Ops)".to_string());

    let res = sqlx::query(
        "UPDATE attendance_correction_requests \
         SET status = 'REJECTED', \
             reviewer_name = $1, \
             rejection_reason = $2, \
             reviewed_at = CURRENT_TIMESTAMP, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE id = $3 AND status = 'PENDING'",
    )
    .bind(&reviewer)
    .bind(clean_reason)
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error rejecting correction request: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to reject correction request.",
        )
    })?;

    if res.rows_affected() == 0 {
        return Err(err_resp(
            StatusCode::CONFLICT,
            "Correction request status was modified concurrently.",
        ));
    }

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit rejection transaction: {}", e);
        err_resp(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to commit database transaction.",
        )
    })?;

    get_correction_request_db(pool, id).await
}

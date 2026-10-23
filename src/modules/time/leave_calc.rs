use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use std::fmt;

use super::models::ErrorResponse;

// =========================================================================
// Models and Types for Leave Day Calculation
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateDayDetail {
    pub date: String,
    pub day_name: String,
    pub is_weekend: bool,
    pub is_scheduled_off: bool,
    pub is_holiday: bool,
    pub holiday_name: Option<String>,
    pub is_chargeable: bool,
    pub chargeable_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveCalculationResult {
    pub start_date: String,
    pub end_date: String,
    pub total_calendar_days: i64,
    pub chargeable_days: f64,
    pub weekend_days: i64,
    pub holiday_days: i64,
    pub working_days: i64,
    pub policy_applied: String,
    pub leave_type_code: Option<String>,
    pub breakdown: Vec<DateDayDetail>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CountLeaveDaysRequest {
    pub start_date: String,
    pub end_date: String,
    pub employee_id: Option<i64>,
    pub leave_type_id: Option<i64>,
    pub leave_type_code: Option<String>,
    pub is_half_day: Option<bool>,
    pub count_calendar_days: Option<bool>,
    pub exclude_weekends: Option<bool>,
    pub exclude_holidays: Option<bool>,
    pub applicable_location: Option<String>,
}

#[derive(Debug)]
pub enum LeaveCalcError {
    InvalidDateRange { start: String, end: String },
    InvalidDateFormat(String),
    NotFound(String),
    DatabaseError(sqlx::Error),
}

impl fmt::Display for LeaveCalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeaveCalcError::InvalidDateRange { start, end } => {
                write!(
                    f,
                    "End date ({}) cannot be earlier than start date ({}).",
                    end, start
                )
            }
            LeaveCalcError::InvalidDateFormat(d) => {
                write!(
                    f,
                    "Invalid date format: {}. Expected format is YYYY-MM-DD.",
                    d
                )
            }
            LeaveCalcError::NotFound(msg) => write!(f, "{}", msg),
            LeaveCalcError::DatabaseError(err) => write!(f, "Database error: {}", err),
        }
    }
}

impl std::error::Error for LeaveCalcError {}

impl From<sqlx::Error> for LeaveCalcError {
    fn from(err: sqlx::Error) -> Self {
        LeaveCalcError::DatabaseError(err)
    }
}

impl LeaveCalcError {
    pub fn to_status_and_response(&self) -> (StatusCode, Json<ErrorResponse>) {
        let (status, msg) = match self {
            LeaveCalcError::InvalidDateRange { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            LeaveCalcError::InvalidDateFormat(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            LeaveCalcError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            LeaveCalcError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
        };
        (status, Json(ErrorResponse { error: msg }))
    }
}

// =========================================================================
// Pure Core Calculation Function (Isolated & Unit-Testable)
// =========================================================================

pub fn calculate_leave_days_pure(
    start_date: NaiveDate,
    end_date: NaiveDate,
    is_calendar_days: bool,
    exclude_weekends: bool,
    exclude_holidays: bool,
    is_half_day: bool,
    holidays_map: &HashMap<NaiveDate, String>,
    scheduled_offs: &HashSet<NaiveDate>,
    scheduled_working_days: &HashSet<NaiveDate>,
    leave_type_code: Option<String>,
) -> Result<LeaveCalculationResult, LeaveCalcError> {
    if end_date < start_date {
        return Err(LeaveCalcError::InvalidDateRange {
            start: start_date.format("%Y-%m-%d").to_string(),
            end: end_date.format("%Y-%m-%d").to_string(),
        });
    }

    let mut breakdown = Vec::new();
    let mut total_calendar_days: i64 = 0;
    let mut chargeable_days: f64 = 0.0;
    let mut weekend_days: i64 = 0;
    let mut holiday_days: i64 = 0;
    let mut working_days: i64 = 0;

    let day_names = ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"];
    let is_single_day = start_date == end_date;
    let mut curr = start_date;

    while curr <= end_date {
        total_calendar_days += 1;
        let weekday = curr.weekday();
        let is_weekend = weekday == Weekday::Sat || weekday == Weekday::Sun;
        let day_idx = weekday.num_days_from_monday() as usize;
        let day_name = day_names[day_idx];

        // Roster / schedule resolution
        let is_scheduled_off = if scheduled_offs.contains(&curr) {
            true
        } else if scheduled_working_days.contains(&curr) {
            false
        } else {
            is_weekend
        };

        if is_weekend {
            weekend_days += 1;
        }

        let holiday_name = holidays_map.get(&curr).cloned();
        let is_holiday = holiday_name.is_some();
        if is_holiday {
            holiday_days += 1;
        }

        if !is_scheduled_off && !is_holiday {
            working_days += 1;
        }

        // Chargeable status determination
        let is_chargeable: bool;
        let chargeable_val: f64;

        if is_calendar_days {
            is_chargeable = true;
            chargeable_val = if is_half_day && is_single_day {
                0.5
            } else {
                1.0
            };
        } else {
            let excluded_by_weekend = exclude_weekends && is_scheduled_off;
            let excluded_by_holiday = exclude_holidays && is_holiday;

            if excluded_by_weekend || excluded_by_holiday {
                is_chargeable = false;
                chargeable_val = 0.0;
            } else {
                is_chargeable = true;
                chargeable_val = if is_half_day && is_single_day {
                    0.5
                } else {
                    1.0
                };
            }
        }

        chargeable_days += chargeable_val;

        breakdown.push(DateDayDetail {
            date: curr.format("%Y-%m-%d").to_string(),
            day_name: day_name.to_string(),
            is_weekend,
            is_scheduled_off,
            is_holiday,
            holiday_name,
            is_chargeable,
            chargeable_value: chargeable_val,
        });

        curr += Duration::days(1);
    }

    let policy_applied = if is_calendar_days {
        "CALENDAR_DAYS".to_string()
    } else {
        "WORKING_DAYS".to_string()
    };

    Ok(LeaveCalculationResult {
        start_date: start_date.format("%Y-%m-%d").to_string(),
        end_date: end_date.format("%Y-%m-%d").to_string(),
        total_calendar_days,
        chargeable_days,
        weekend_days,
        holiday_days,
        working_days,
        policy_applied,
        leave_type_code,
        breakdown,
    })
}

// =========================================================================
// Database-Backed Service (Reuses existing models, holidays & roster)
// =========================================================================

pub async fn count_leave_days_service(
    pool: &PgPool,
    req: &CountLeaveDaysRequest,
) -> Result<LeaveCalculationResult, LeaveCalcError> {
    let start_date = NaiveDate::parse_from_str(req.start_date.trim(), "%Y-%m-%d")
        .map_err(|_| LeaveCalcError::InvalidDateFormat(req.start_date.clone()))?;
    let end_date = NaiveDate::parse_from_str(req.end_date.trim(), "%Y-%m-%d")
        .map_err(|_| LeaveCalcError::InvalidDateFormat(req.end_date.clone()))?;

    if end_date < start_date {
        return Err(LeaveCalcError::InvalidDateRange {
            start: req.start_date.clone(),
            end: req.end_date.clone(),
        });
    }

    // 1. Resolve Leave Type policy rules
    let mut resolved_code = req.leave_type_code.clone();
    let mut default_calendar_days = false;

    if let Some(lt_id) = req.leave_type_id {
        let row: Option<(String,)> = sqlx::query_as("SELECT code FROM leave_types WHERE id = $1")
            .bind(lt_id)
            .fetch_optional(pool)
            .await?;
        match row {
            Some((code,)) => {
                resolved_code = Some(code.clone());
                if code == "ML" {
                    default_calendar_days = true;
                }
            }
            None => {
                return Err(LeaveCalcError::NotFound(format!(
                    "Leave type with ID {} not found",
                    lt_id
                )));
            }
        }
    } else if let Some(ref code) = resolved_code {
        if code == "ML" {
            default_calendar_days = true;
        }
    }

    let is_calendar_days = req.count_calendar_days.unwrap_or(default_calendar_days);
    let exclude_weekends = req.exclude_weekends.unwrap_or(!is_calendar_days);
    let exclude_holidays = req.exclude_holidays.unwrap_or(!is_calendar_days);
    let is_half_day = req.is_half_day.unwrap_or(false);

    // 2. Fetch Employee Work Schedule and Roster Assignments if applicable
    let mut scheduled_offs = HashSet::new();
    let mut scheduled_working_days = HashSet::new();
    let employee_location: Option<String> = req.applicable_location.clone();

    if let Some(emp_id) = req.employee_id {
        let emp_row: Option<(Option<String>,)> =
            sqlx::query_as("SELECT department FROM employees WHERE id = $1")
                .bind(emp_id)
                .fetch_optional(pool)
                .await?;

        if emp_row.is_none() {
            return Err(LeaveCalcError::NotFound(format!(
                "Employee with ID {} not found",
                emp_id
            )));
        }

        let assignments: Vec<(NaiveDate, bool)> = sqlx::query_as(
            "SELECT roster_date, is_off FROM roster_assignments \
             WHERE employee_id = $1 AND roster_date >= $2 AND roster_date <= $3",
        )
        .bind(emp_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(pool)
        .await?;

        for (r_date, is_off) in assignments {
            if is_off {
                scheduled_offs.insert(r_date);
            } else {
                scheduled_working_days.insert(r_date);
            }
        }
    }

    // 3. Query Configured Holidays in range
    let holiday_rows: Vec<(NaiveDate, String, Option<String>)> = sqlx::query_as(
        "SELECT holiday_date, name, applicable_location FROM holidays \
         WHERE holiday_date >= $1 AND holiday_date <= $2",
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    let mut holidays_map = HashMap::new();
    for (h_date, h_name, h_loc) in holiday_rows {
        let applies = match (&h_loc, &employee_location) {
            (None, _) => true,
            (Some(hl), Some(el)) => hl.eq_ignore_ascii_case(el),
            (Some(_), None) => true,
        };
        if applies {
            holidays_map.insert(h_date, h_name);
        }
    }

    calculate_leave_days_pure(
        start_date,
        end_date,
        is_calendar_days,
        exclude_weekends,
        exclude_holidays,
        is_half_day,
        &holidays_map,
        &scheduled_offs,
        &scheduled_working_days,
        resolved_code,
    )
}

// =========================================================================
// Axum HTTP Handlers
// =========================================================================

pub async fn preview_leave_days_post(
    State(pool): State<PgPool>,
    Json(payload): Json<CountLeaveDaysRequest>,
) -> Result<Json<LeaveCalculationResult>, (StatusCode, Json<ErrorResponse>)> {
    count_leave_days_service(&pool, &payload)
        .await
        .map(Json)
        .map_err(|e| e.to_status_and_response())
}

pub async fn preview_leave_days_get(
    State(pool): State<PgPool>,
    Query(query): Query<CountLeaveDaysRequest>,
) -> Result<Json<LeaveCalculationResult>, (StatusCode, Json<ErrorResponse>)> {
    count_leave_days_service(&pool, &query)
        .await
        .map(Json)
        .map_err(|e| e.to_status_and_response())
}

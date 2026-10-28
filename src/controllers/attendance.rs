use chrono::Utc;
use crate::models::_entities::attendance_records::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClockParams {
    pub employee_code: Option<String>,
    pub employee_id: Option<i64>,
    pub work_mode: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AttendanceStatusResponse {
    pub is_checked_in: bool,
    pub last_clock_in: Option<String>,
    pub last_clock_out: Option<String>,
    pub today_hours: Option<String>,
    pub status: Option<String>,
    pub can_clock_in: Option<bool>,
    pub can_clock_out: Option<bool>,
    pub record: Option<serde_json::Value>,
}

#[debug_handler]
pub async fn my_attendance(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(attendance_records::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn status(State(ctx): State<AppContext>) -> Result<Response> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let record = Entity::find()
        .filter(attendance_records::Column::Date.eq(&today))
        .order_by_desc(attendance_records::Column::CreatedAt)
        .one(&ctx.db)
        .await?;

    if let Some(r) = record {
        let is_checked_in = r.clock_out.is_none();
        let rec_json = serde_json::to_value(&r).unwrap_or_default();
        format::json(AttendanceStatusResponse {
            is_checked_in,
            last_clock_in: Some(r.clock_in.clone()),
            last_clock_out: r.clock_out.clone(),
            today_hours: r.hours_worked.clone(),
            status: Some(if is_checked_in { "CLOCKED_IN".into() } else { "CLOCKED_OUT".into() }),
            can_clock_in: Some(!is_checked_in),
            can_clock_out: Some(is_checked_in),
            record: Some(rec_json),
        })
    } else {
        format::json(AttendanceStatusResponse {
            is_checked_in: false,
            last_clock_in: None,
            last_clock_out: None,
            today_hours: None,
            status: Some("NOT_CLOCKED_IN".into()),
            can_clock_in: Some(true),
            can_clock_out: Some(false),
            record: None,
        })
    }
}

#[debug_handler]
pub async fn clock(
    State(ctx): State<AppContext>,
    Json(params): Json<ClockParams>,
) -> Result<Response> {
    let code = params
        .employee_code
        .unwrap_or_else(|| "BC-1006".to_string());
    let mode = params
        .work_mode
        .unwrap_or_else(|| "Office".to_string());
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let time_str = chrono::Local::now().format("%I:%M %p").to_string();

    let open_record = Entity::find()
        .filter(attendance_records::Column::EmployeeCode.eq(&code))
        .filter(attendance_records::Column::Date.eq(&today))
        .filter(attendance_records::Column::ClockOut.is_null())
        .order_by_desc(attendance_records::Column::CreatedAt)
        .one(&ctx.db)
        .await?;

    if let Some(existing) = open_record {
        let mut active: ActiveModel = existing.clone().into();
        active.clock_out = Set(Some(time_str.clone()));
        let hours_str = if let Ok(in_time) = chrono::NaiveTime::parse_from_str(&existing.clock_in, "%I:%M %p") {
            let now_time = chrono::Local::now().time();
            let diff_minutes = (now_time - in_time).num_minutes().max(1);
            let hrs = diff_minutes / 60;
            let mins = diff_minutes % 60;
            format!("{} hrs {} mins", hrs, mins)
        } else {
            "8 hrs 0 mins".to_string()
        };
        active.hours_worked = Set(Some(hours_str));
        active.status = Set("Completed".to_string());
        let updated = active.update(&ctx.db).await?;
        format::json(serde_json::json!({
            "action": "clock_out",
            "message": format!("Checked out at {}", time_str),
            "record": updated,
            "is_checked_in": false,
            "status": "CLOCKED_OUT",
            "can_clock_in": false,
            "can_clock_out": false
        }))
    } else {
        let active = ActiveModel {
            record_id: Set(Uuid::new_v4()),
            employee_code: Set(code),
            date: Set(today.clone()),
            clock_in: Set(time_str.clone()),
            clock_out: Set(None),
            work_mode: Set(mode),
            hours_worked: Set(Some("In progress".to_string())),
            status: Set("Present".to_string()),
            ..Default::default()
        };
        let record = active.insert(&ctx.db).await?;
        format::json(serde_json::json!({
            "action": "clock_in",
            "message": format!("Checked in at {}", time_str),
            "record": record,
            "is_checked_in": true,
            "status": "CLOCKED_IN",
            "can_clock_in": false,
            "can_clock_out": true
        }))
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAttendanceInput {
    pub employee_code: Option<String>,
    pub date: Option<String>,
    pub attendance_status: Option<String>,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub work_mode: Option<String>,
    pub notes: Option<String>,
}

#[debug_handler]
pub async fn create_record(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateAttendanceInput>,
) -> Result<Response> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let time_str = chrono::Local::now().format("%I:%M %p").to_string();
    let code = params.employee_code.unwrap_or_else(|| "BC-1006".to_string());
    let active = ActiveModel {
        record_id: Set(Uuid::new_v4()),
        employee_code: Set(code),
        date: Set(params.date.unwrap_or(today)),
        clock_in: Set(params.in_time.unwrap_or(time_str)),
        clock_out: Set(params.out_time),
        work_mode: Set(params.work_mode.unwrap_or_else(|| "Office".to_string())),
        hours_worked: Set(Some("8.5 hrs".to_string())),
        status: Set(params.attendance_status.unwrap_or_else(|| "Present".to_string())),
        ..Default::default()
    };
    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn list_corrections() -> Result<Response> {
    let list = vec![
        serde_json::json!({
            "id": 1,
            "employee_id": 4,
            "employee_name": "Rahul Varma",
            "date": "2026-10-27",
            "requested_in_time": "09:00 AM",
            "requested_out_time": "06:00 PM",
            "reason": "Biometric terminal failure at reception",
            "status": "APPROVED",
            "reviewer_notes": "Verified with security log"
        }),
        serde_json::json!({
            "id": 2,
            "employee_id": 2,
            "employee_name": "Ananya Rao",
            "date": "2026-10-26",
            "requested_in_time": "09:15 AM",
            "requested_out_time": "06:30 PM",
            "reason": "VPN sync latency",
            "status": "PENDING",
            "reviewer_notes": null
        })
    ];
    format::json(list)
}

#[debug_handler]
pub async fn create_correction(Json(payload): Json<serde_json::Value>) -> Result<Response> {
    format::json(serde_json::json!({
        "id": Utc::now().timestamp_millis(),
        "employee_id": payload.get("employee_id").and_then(|v| v.as_i64()).unwrap_or(4),
        "employee_name": payload.get("employee_name").and_then(|v| v.as_str()).unwrap_or("Rahul Varma"),
        "date": payload.get("date").and_then(|v| v.as_str()).unwrap_or("2026-10-28"),
        "requested_in_time": payload.get("requested_in_time").and_then(|v| v.as_str()).unwrap_or("09:00 AM"),
        "requested_out_time": payload.get("requested_out_time").and_then(|v| v.as_str()).unwrap_or("06:00 PM"),
        "reason": payload.get("reason").and_then(|v| v.as_str()).unwrap_or("Missed punch"),
        "status": "PENDING"
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/attendance")
        .add("", get(my_attendance))
        .add("", post(create_record))
        .add("/my", get(my_attendance))
        .add("/status", get(status))
        .add("/today", get(status))
        .add("/clock", post(clock))
        .add("/clock-in", post(clock))
        .add("/clock-out", post(clock))
        .add("/corrections", get(list_corrections))
        .add("/corrections", post(create_correction))
}

pub fn v1_routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/attendance")
        .add("", get(my_attendance))
        .add("", post(create_record))
        .add("/my", get(my_attendance))
        .add("/status", get(status))
        .add("/today", get(status))
        .add("/clock", post(clock))
        .add("/clock-in", post(clock))
        .add("/clock-out", post(clock))
        .add("/corrections", get(list_corrections))
        .add("/corrections", post(create_correction))
}

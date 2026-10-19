use crate::models::_entities::attendance_records::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClockParams {
    pub employee_code: Option<String>,
    pub work_mode: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AttendanceStatusResponse {
    pub is_checked_in: bool,
    pub last_clock_in: Option<String>,
    pub last_clock_out: Option<String>,
    pub today_hours: Option<String>,
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
        format::json(AttendanceStatusResponse {
            is_checked_in,
            last_clock_in: Some(r.clock_in),
            last_clock_out: r.clock_out,
            today_hours: r.hours_worked,
        })
    } else {
        format::json(AttendanceStatusResponse {
            is_checked_in: false,
            last_clock_in: None,
            last_clock_out: None,
            today_hours: None,
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
        .unwrap_or_else(|| "BC-1038".to_string());
    let mode = params
        .work_mode
        .unwrap_or_else(|| "Hybrid (Office)".to_string());
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
        // Clock out
        let mut active: ActiveModel = existing.into();
        active.clock_out = Set(Some(time_str.clone()));
        active.hours_worked = Set(Some("8 hrs 30 mins".to_string()));
        active.status = Set("Completed".to_string());
        let updated = active.update(&ctx.db).await?;
        format::json(serde_json::json!({
            "action": "clock_out",
            "message": format!("Checked out at {}", time_str),
            "record": updated,
            "is_checked_in": false
        }))
    } else {
        // Clock in
        let active = ActiveModel {
            record_id: Set(Uuid::new_v4()),
            employee_code: Set(code),
            date: Set(today),
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
            "is_checked_in": true
        }))
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/attendance")
        .add("/my", get(my_attendance))
        .add("/status", get(status))
        .add("/clock", post(clock))
        .add("/", get(my_attendance))
}

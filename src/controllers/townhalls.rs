use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

use crate::models::_entities::connect_townhalls::{self, ActiveModel, Entity as ConnectTownhalls, Model};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTownhallPayload {
    pub title: String,
    pub date: String,
    pub time: String,
    pub mode: Option<String>,
    pub host: Option<String>,
    pub agenda: Option<String>,
    pub att: Option<i32>,
    pub status: Option<String>,
    pub minutes: Option<String>,
    pub actions: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTownhallPayload {
    pub title: Option<String>,
    pub date: Option<String>,
    pub time: Option<String>,
    pub mode: Option<String>,
    pub host: Option<String>,
    pub agenda: Option<String>,
    pub att: Option<i32>,
    pub status: Option<String>,
    pub minutes: Option<String>,
    pub actions: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TownhallDto {
    pub id: String,
    pub title: String,
    pub date: String,
    pub time: String,
    pub mode: String,
    pub host: String,
    pub agenda: String,
    pub att: i32,
    pub status: String,
    pub minutes: String,
    pub actions: String,
}

impl From<Model> for TownhallDto {
    fn from(m: Model) -> Self {
        Self {
            id: m.townhall_code,
            title: m.title,
            date: m.meeting_date,
            time: m.meeting_time,
            mode: m.mode,
            host: m.host,
            agenda: m.agenda.unwrap_or_default(),
            att: m.attendees_count,
            status: m.status,
            minutes: m.minutes.unwrap_or_default(),
            actions: m.actions.unwrap_or_default(),
        }
    }
}

pub async fn list_townhalls(State(ctx): State<AppContext>) -> Result<Response> {
    let mut list = ConnectTownhalls::find()
        .order_by_desc(connect_townhalls::Column::MeetingDate)
        .all(&ctx.db)
        .await?;

    if list.is_empty() {
        let now = Utc::now();
        let seeds = vec![
            ("TH-26-01", "Q3 All-Hands and AI Product Roadmap", "2026-10-30", "15:00", "Hybrid", "Naresh Vemparala", "Q3 performance review, customer ARR milestones, and preview of Sign & Connect apps.", 142, "Scheduled"),
            ("TH-26-02", "Engineering Architecture & Rust Migration", "2026-11-12", "16:00", "Virtual", "Karthik Reddy", "Deep-dive on Loco.rs microservices, database optimizations, and real-time event streaming.", 68, "Scheduled"),
            ("TH-26-03", "People Operations & Benefits Open Enrolment", "2026-10-18", "11:00", "In-person", "Ananya Rao", "2027 health cover coverage, parental leave updates, and wellness allowances.", 110, "Completed"),
        ];

        for (code, title, date, time, mode, host, agenda, att, status) in seeds {
            let active = ActiveModel {
                townhall_code: Set(code.to_string()),
                title: Set(title.to_string()),
                meeting_date: Set(date.to_string()),
                meeting_time: Set(time.to_string()),
                mode: Set(mode.to_string()),
                host: Set(host.to_string()),
                agenda: Set(Some(agenda.to_string())),
                attendees_count: Set(att),
                status: Set(status.to_string()),
                minutes: Set(Some("Detailed notes shared on Slack #general".to_string())),
                actions: Set(Some("Department leads to finalize action checklists".to_string())),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
                ..Default::default()
            };
            let _ = active.insert(&ctx.db).await;
        }

        list = ConnectTownhalls::find()
            .order_by_desc(connect_townhalls::Column::MeetingDate)
            .all(&ctx.db)
            .await?;
    }

    let dtos: Vec<TownhallDto> = list.into_iter().map(TownhallDto::from).collect();
    format::json(dtos)
}

pub async fn create_townhall(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateTownhallPayload>,
) -> Result<Response> {
    let now = Utc::now();
    let count = ConnectTownhalls::find().all(&ctx.db).await?.len();
    let code = format!("TH-26-{:02}", count + 1);

    let active = ActiveModel {
        townhall_code: Set(code),
        title: Set(payload.title),
        meeting_date: Set(payload.date),
        meeting_time: Set(payload.time),
        mode: Set(payload.mode.unwrap_or_else(|| "Hybrid".to_string())),
        host: Set(payload.host.unwrap_or_else(|| "Naresh Vemparala".to_string())),
        agenda: Set(payload.agenda),
        attendees_count: Set(payload.att.unwrap_or(100)),
        status: Set(payload.status.unwrap_or_else(|| "Scheduled".to_string())),
        minutes: Set(payload.minutes),
        actions: Set(payload.actions),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    let model = active.insert(&ctx.db).await?;
    let dto = TownhallDto::from(model);
    format::json(dto)
}

pub async fn update_townhall(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(payload): Json<UpdateTownhallPayload>,
) -> Result<Response> {
    let existing = if let Ok(num) = id.parse::<i64>() {
        ConnectTownhalls::find_by_id(num).one(&ctx.db).await?
    } else {
        ConnectTownhalls::find().filter(connect_townhalls::Column::TownhallCode.eq(&id)).one(&ctx.db).await?
    };

    let Some(model) = existing else {
        return not_found();
    };

    let mut active: ActiveModel = model.into();
    if let Some(t) = payload.title { active.title = Set(t); }
    if let Some(d) = payload.date { active.meeting_date = Set(d); }
    if let Some(tm) = payload.time { active.meeting_time = Set(tm); }
    if let Some(m) = payload.mode { active.mode = Set(m); }
    if let Some(h) = payload.host { active.host = Set(h); }
    if let Some(a) = payload.agenda { active.agenda = Set(Some(a)); }
    if let Some(att) = payload.att { active.attendees_count = Set(att); }
    if let Some(s) = payload.status { active.status = Set(s); }
    if let Some(min) = payload.minutes { active.minutes = Set(Some(min)); }
    if let Some(act) = payload.actions { active.actions = Set(Some(act)); }
    active.updated_at = Set(Utc::now().into());

    let updated = active.update(&ctx.db).await?;
    let dto = TownhallDto::from(updated);
    format::json(dto)
}

pub async fn delete_townhall(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let existing = if let Ok(num) = id.parse::<i64>() {
        ConnectTownhalls::find_by_id(num).one(&ctx.db).await?
    } else {
        ConnectTownhalls::find().filter(connect_townhalls::Column::TownhallCode.eq(&id)).one(&ctx.db).await?
    };

    if let Some(model) = existing {
        let active: ActiveModel = model.into();
        let _ = active.delete(&ctx.db).await;
    }

    format::empty_json()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/connect/townhalls")
        .add("", get(list_townhalls))
        .add("", post(create_townhall))
        .add("/{id}", patch(update_townhall))
        .add("/{id}", put(update_townhall))
        .add("/{id}", delete(delete_townhall))
}

pub fn legacy_routes() -> Routes {
    Routes::new()
        .prefix("/connect/townhalls")
        .add("", get(list_townhalls))
        .add("", post(create_townhall))
        .add("/{id}", patch(update_townhall))
        .add("/{id}", put(update_townhall))
        .add("/{id}", delete(delete_townhall))
}

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

use crate::models::_entities::company_events::{self, ActiveModel, Entity as CompanyEvents, Model};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEventPayload {
    pub name: String,
    pub etype: String,
    pub date: String,
    pub time: String,
    pub venue: String,
    pub organiser: String,
    pub budget: Option<i64>,
    pub att: Option<i32>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateEventPayload {
    pub name: Option<String>,
    pub etype: Option<String>,
    pub date: Option<String>,
    pub time: Option<String>,
    pub venue: Option<String>,
    pub organiser: Option<String>,
    pub budget: Option<i64>,
    pub att: Option<i32>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventResponseDto {
    pub id: String,
    pub name: String,
    pub etype: String,
    pub date: String,
    pub time: String,
    pub venue: String,
    pub organiser: String,
    pub budget: i64,
    pub att: i32,
    pub status: String,
    pub notes: String,
}

impl From<Model> for EventResponseDto {
    fn from(m: Model) -> Self {
        Self {
            id: m.event_code,
            name: m.name,
            etype: m.etype,
            date: m.event_date,
            time: m.event_time,
            venue: m.venue,
            organiser: m.organiser,
            budget: m.budget,
            att: m.attendees_count,
            status: m.status,
            notes: m.notes.unwrap_or_default(),
        }
    }
}

pub async fn list_events(State(ctx): State<AppContext>) -> Result<Response> {
    let mut list = CompanyEvents::find()
        .order_by_desc(company_events::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    if list.is_empty() {
        let defaults = vec![
            ("EV-101", "Brightcone Diwali Celebration", "Festival", "2026-10-31", "16:00", "Hyderabad HQ, 4th Floor Cafeteria", "Ananya Rao", 75000, 140, "Confirmed", "Traditional attire, rangoli competition, sweets and high tea."),
            ("EV-102", "Agentic AI Hackathon 2026", "Celebration", "2026-11-14", "09:00", "Bengaluru Innovation Lab and Zoom", "Karthik Reddy", 120000, 85, "Planned", "48-hour buildathon on autonomous agentic systems."),
            ("EV-103", "Brightcone Premier League (Cricket)", "Sports", "2026-11-22", "07:30", "Gachibowli Sports Complex", "Vikram Malhotra", 50000, 60, "Planned", "Inter-department box cricket tournament."),
            ("EV-104", "All-Hands Townhall Q3", "Townhall", "2026-10-28", "17:00", "Townhall Auditorium & Livestream", "Naresh Vemparala", 25000, 200, "Confirmed", "Quarterly financial review, roadmap unveil, and AMA."),
        ];

        for (code, name, etype, date, time, venue, org, bgt, att, st, notes) in defaults {
            let active = ActiveModel {
                event_code: Set(code.to_string()),
                name: Set(name.to_string()),
                etype: Set(etype.to_string()),
                event_date: Set(date.to_string()),
                event_time: Set(time.to_string()),
                venue: Set(venue.to_string()),
                organiser: Set(org.to_string()),
                budget: Set(bgt),
                attendees_count: Set(att),
                status: Set(st.to_string()),
                notes: Set(Some(notes.to_string())),
                created_at: Set(Utc::now().into()),
                updated_at: Set(Utc::now().into()),
                ..Default::default()
            };
            if let Ok(m) = active.insert(&ctx.db).await {
                list.push(m);
            }
        }
    }

    let dtos: Vec<EventResponseDto> = list.into_iter().map(EventResponseDto::from).collect();
    format::json(dtos)
}

pub async fn create_event(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateEventPayload>,
) -> Result<Response> {
    let count = CompanyEvents::find().all(&ctx.db).await?.len();
    let event_code = format!("EV-{}", 100 + count + 1);

    let active = ActiveModel {
        event_code: Set(event_code),
        name: Set(payload.name),
        etype: Set(payload.etype),
        event_date: Set(payload.date),
        event_time: Set(payload.time),
        venue: Set(payload.venue),
        organiser: Set(payload.organiser),
        budget: Set(payload.budget.unwrap_or(0)),
        attendees_count: Set(payload.att.unwrap_or(0)),
        status: Set(payload.status.unwrap_or_else(|| "Planned".to_string())),
        notes: Set(payload.notes),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    };

    let inserted = active.insert(&ctx.db).await?;
    let dto = EventResponseDto::from(inserted);
    format::json(dto)
}

pub async fn update_event(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(payload): Json<UpdateEventPayload>,
) -> Result<Response> {
    let existing = if let Ok(num) = id.parse::<i64>() {
        CompanyEvents::find_by_id(num).one(&ctx.db).await?
    } else {
        CompanyEvents::find().filter(company_events::Column::EventCode.eq(&id)).one(&ctx.db).await?
    };

    let Some(model) = existing else {
        return not_found();
    };

    let mut active: ActiveModel = model.into();
    if let Some(n) = payload.name { active.name = Set(n); }
    if let Some(e) = payload.etype { active.etype = Set(e); }
    if let Some(d) = payload.date { active.event_date = Set(d); }
    if let Some(t) = payload.time { active.event_time = Set(t); }
    if let Some(v) = payload.venue { active.venue = Set(v); }
    if let Some(o) = payload.organiser { active.organiser = Set(o); }
    if let Some(b) = payload.budget { active.budget = Set(b); }
    if let Some(a) = payload.att { active.attendees_count = Set(a); }
    if let Some(s) = payload.status { active.status = Set(s); }
    if let Some(notes) = payload.notes { active.notes = Set(Some(notes)); }
    active.updated_at = Set(Utc::now().into());

    let updated = active.update(&ctx.db).await?;
    let dto = EventResponseDto::from(updated);
    format::json(dto)
}

pub async fn delete_event(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let existing = if let Ok(num) = id.parse::<i64>() {
        CompanyEvents::find_by_id(num).one(&ctx.db).await?
    } else {
        CompanyEvents::find().filter(company_events::Column::EventCode.eq(&id)).one(&ctx.db).await?
    };

    if let Some(model) = existing {
        let active: ActiveModel = model.into();
        let _ = active.delete(&ctx.db).await;
    }

    format::empty_json()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/connect/events")
        .add("", get(list_events))
        .add("", post(create_event))
        .add("/{id}", patch(update_event))
        .add("/{id}", put(update_event))
        .add("/{id}", delete(delete_event))
}

pub fn legacy_routes() -> Routes {
    Routes::new()
        .prefix("/connect/events")
        .add("", get(list_events))
        .add("", post(create_event))
        .add("/{id}", patch(update_event))
        .add("/{id}", put(update_event))
        .add("/{id}", delete(delete_event))
}

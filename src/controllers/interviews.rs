use crate::models::_entities::interviews::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct InterviewParams {
    pub candidate_id: Option<Uuid>,
    pub candidate_name: String,
    pub job_title: String,
    pub round_name: String,
    pub scheduled_at: String,
    pub panel: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InterviewUpdateParams {
    pub round_name: Option<String>,
    pub scheduled_at: Option<String>,
    pub panel: Option<String>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(interviews::Column::ScheduledAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<InterviewParams>,
) -> Result<Response> {
    let active = ActiveModel {
        interview_id: Set(Uuid::new_v4()),
        candidate_id: Set(params.candidate_id),
        candidate_name: Set(params.candidate_name),
        job_title: Set(params.job_title),
        round_name: Set(params.round_name),
        scheduled_at: Set(params.scheduled_at),
        panel: Set(params.panel),
        status: Set("Scheduled".to_string()),
        notes: Set(params.notes),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<InterviewUpdateParams>,
) -> Result<Response> {
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad_request("Invalid interview UUID");
    };

    let Some(existing) = Entity::find()
        .filter(interviews::Column::InterviewId.eq(uuid))
        .one(&ctx.db)
        .await?
    else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    if let Some(r) = params.round_name {
        active.round_name = Set(r);
    }
    if let Some(s) = params.scheduled_at {
        active.scheduled_at = Set(s);
    }
    if let Some(p) = params.panel {
        active.panel = Set(p);
    }
    if let Some(st) = params.status {
        active.status = Set(st);
    }
    if let Some(n) = params.notes {
        active.notes = Set(Some(n));
    }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn cancel(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad_request("Invalid interview UUID");
    };

    let Some(existing) = Entity::find()
        .filter(interviews::Column::InterviewId.eq(uuid))
        .one(&ctx.db)
        .await?
    else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    active.status = Set("Cancelled".to_string());
    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad_request("Invalid interview UUID");
    };

    let Some(existing) = Entity::find()
        .filter(interviews::Column::InterviewId.eq(uuid))
        .one(&ctx.db)
        .await?
    else {
        return not_found();
    };

    existing.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/interviews")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
        .add("/{id}/cancel", post(cancel))
}

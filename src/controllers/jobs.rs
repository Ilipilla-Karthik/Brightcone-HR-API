use crate::models::_entities::jobs::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct JobParams {
    pub title: String,
    pub department: String,
    pub location: String,
    pub employment_type: String,
    pub referral_bonus: i32,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JobUpdateParams {
    pub title: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub employment_type: Option<String>,
    pub referral_bonus: Option<i32>,
    pub status: Option<String>,
    pub description: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(jobs::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<JobParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let job_code = format!("J{}", count + 1);

    let active = ActiveModel {
        job_id: Set(Uuid::new_v4()),
        job_code: Set(job_code),
        title: Set(params.title),
        department: Set(params.department),
        location: Set(params.location),
        employment_type: Set(params.employment_type),
        status: Set("Open".to_string()),
        referral_bonus: Set(params.referral_bonus),
        description: Set(params.description),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(jobs::Column::JobId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(jobs::Column::JobCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    match record {
        Some(j) => format::json(j),
        None => not_found(),
    }
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<JobUpdateParams>,
) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(jobs::Column::JobId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(jobs::Column::JobCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    if let Some(t) = params.title {
        active.title = Set(t);
    }
    if let Some(d) = params.department {
        active.department = Set(d);
    }
    if let Some(l) = params.location {
        active.location = Set(l);
    }
    if let Some(e) = params.employment_type {
        active.employment_type = Set(e);
    }
    if let Some(b) = params.referral_bonus {
        active.referral_bonus = Set(b);
    }
    if let Some(s) = params.status {
        active.status = Set(s);
    }
    if let Some(desc) = params.description {
        active.description = Set(Some(desc));
    }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn publish(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(jobs::Column::JobId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(jobs::Column::JobCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    active.status = Set("Open".to_string());
    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn close(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(jobs::Column::JobId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(jobs::Column::JobCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    active.status = Set("Closed".to_string());
    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(jobs::Column::JobId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(jobs::Column::JobCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    existing.delete(&ctx.db).await?;
    format::empty()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/jobs")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
        .add("/{id}/publish", post(publish))
        .add("/{id}/close", post(close))
}

use crate::models::_entities::candidates::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateParams {
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: String,
    pub job_id: Option<Uuid>,
    pub job_title: Option<String>,
    pub match_score: Option<i32>,
    pub stage: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CandidateUpdateParams {
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub source: Option<String>,
    pub job_id: Option<Uuid>,
    pub job_title: Option<String>,
    pub match_score: Option<i32>,
    pub stage: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(candidates::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CandidateParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let cand_code = format!("C{}", count + 1);

    let active = ActiveModel {
        cand_id: Set(Uuid::new_v4()),
        candidate_code: Set(cand_code),
        full_name: Set(params.full_name),
        email: Set(params.email),
        phone: Set(params.phone),
        source: Set(params.source),
        job_id: Set(params.job_id),
        job_title: Set(params.job_title),
        match_score: Set(params.match_score.or(Some(80))),
        stage: Set(params.stage.unwrap_or_else(|| "Applied".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(candidates::Column::CandId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(candidates::Column::CandidateCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    match record {
        Some(c) => format::json(c),
        None => not_found(),
    }
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<CandidateUpdateParams>,
) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(candidates::Column::CandId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(candidates::Column::CandidateCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    if let Some(n) = params.full_name {
        active.full_name = Set(n);
    }
    if let Some(e) = params.email {
        active.email = Set(Some(e));
    }
    if let Some(p) = params.phone {
        active.phone = Set(Some(p));
    }
    if let Some(s) = params.source {
        active.source = Set(s);
    }
    if let Some(j) = params.job_id {
        active.job_id = Set(Some(j));
    }
    if let Some(jt) = params.job_title {
        active.job_title = Set(Some(jt));
    }
    if let Some(sc) = params.match_score {
        active.match_score = Set(Some(sc));
    }
    if let Some(st) = params.stage {
        active.stage = Set(st);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(candidates::Column::CandId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(candidates::Column::CandidateCode.eq(&id))
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
        .prefix("/api/v1/candidates")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", get(get_one))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

use crate::models::_entities::{
    application_histories::{self, ActiveModel as HistoryActiveModel},
    applications::{self, ActiveModel, Entity, Model},
    candidates,
};
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct ApplicationParams {
    pub candidate_id: Uuid,
    pub job_id: Uuid,
    pub stage: Option<String>,
    pub score: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatusUpdateParams {
    pub stage: String,
    pub reason: Option<String>,
    pub changed_by: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(applications::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<ApplicationParams>,
) -> Result<Response> {
    let active = ActiveModel {
        app_id: Set(Uuid::new_v4()),
        candidate_id: Set(params.candidate_id),
        job_id: Set(params.job_id),
        stage: Set(params.stage.unwrap_or_else(|| "Applied".to_string())),
        score: Set(params.score),
        notes: Set(params.notes),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad_request("Invalid application UUID");
    };

    let record = Entity::find()
        .filter(applications::Column::AppId.eq(uuid))
        .one(&ctx.db)
        .await?;

    match record {
        Some(a) => format::json(a),
        None => not_found(),
    }
}

#[debug_handler]
pub async fn update_status(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<StatusUpdateParams>,
) -> Result<Response> {
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad_request("Invalid application UUID");
    };

    let Some(existing) = Entity::find()
        .filter(applications::Column::AppId.eq(uuid))
        .one(&ctx.db)
        .await?
    else {
        return not_found();
    };

    let old_stage = existing.stage.clone();
    let new_stage = params.stage.clone();

    // Use transaction for stage change and history logging
    let txn = ctx.db.begin().await?;

    let mut active: ActiveModel = existing.into();
    active.stage = Set(new_stage.clone());
    let updated = active.update(&txn).await?;

    // Also update candidate stage
    if let Some(cand) = candidates::Entity::find()
        .filter(candidates::Column::CandId.eq(updated.candidate_id))
        .one(&txn)
        .await?
    {
        let mut cand_active: candidates::ActiveModel = cand.into();
        cand_active.stage = Set(new_stage.clone());
        let _ = cand_active.update(&txn).await?;
    }

    // Insert history
    let history_entry = HistoryActiveModel {
        history_id: Set(Uuid::new_v4()),
        candidate_name: Set(format!("Candidate {}", updated.candidate_id)),
        job_title: Set("Application".to_string()),
        from_stage: Set(old_stage),
        to_stage: Set(new_stage),
        changed_by: Set(params.changed_by.or(Some("HR User".to_string()))),
        reason: Set(params.reason),
        ..Default::default()
    };
    let _ = history_entry.insert(&txn).await?;

    txn.commit().await?;

    format::json(updated)
}

#[debug_handler]
pub async fn get_history(State(ctx): State<AppContext>) -> Result<Response> {
    let histories = application_histories::Entity::find()
        .order_by_desc(application_histories::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(histories)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/applications")
        .add("/", get(list))
        .add("/", post(create))
        .add("/history", get(get_history))
        .add("/{id}", get(get_one))
        .add("/{id}/status", patch(update_status))
}

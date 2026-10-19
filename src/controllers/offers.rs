use crate::models::_entities::offers::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct OfferParams {
    pub candidate_name: String,
    pub job_title: String,
    pub offered_ctc: Option<f64>,
    pub joining_date: Option<String>,
    pub status: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(offers::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<OfferParams>,
) -> Result<Response> {
    let active = ActiveModel {
        offer_id: Set(Uuid::new_v4()),
        candidate_name: Set(params.candidate_name),
        job_title: Set(params.job_title),
        offered_ctc: Set(params.offered_ctc),
        joining_date: Set(params.joining_date),
        status: Set(params.status.unwrap_or_else(|| "Draft".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/offers")
        .add("/", get(list))
        .add("/", post(create))
}

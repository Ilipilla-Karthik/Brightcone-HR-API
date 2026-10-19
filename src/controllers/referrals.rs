use crate::models::_entities::referrals::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct ReferralParams {
    pub referred_by: Option<String>,
    pub referrer_name: Option<String>,
    pub candidate_name: String,
    pub candidate_email: Option<String>,
    pub candidate_phone: Option<String>,
    pub job_id: Option<String>,
    pub job_title: Option<String>,
    pub role: Option<String>,
    pub bonus_amount: Option<i32>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(referrals::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<ReferralParams>,
) -> Result<Response> {
    let referrer = params
        .referred_by
        .or(params.referrer_name)
        .unwrap_or_else(|| "Rahul Varma".to_string());

    let title = params
        .job_title
        .or(params.role)
        .unwrap_or_else(|| "Senior ML Engineer".to_string());

    let bonus = params.bonus_amount.unwrap_or(50000);

    let active = ActiveModel {
        referral_id: Set(Uuid::new_v4()),
        referred_by: Set(referrer),
        candidate_name: Set(params.candidate_name),
        job_title: Set(title),
        bonus_amount: Set(bonus),
        status: Set(params.status.unwrap_or_else(|| "Interview".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/referrals")
        .add("/", get(list))
        .add("/", post(create))
}

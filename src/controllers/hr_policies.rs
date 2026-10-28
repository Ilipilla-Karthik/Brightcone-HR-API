use crate::models::_entities::hr_policies::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePolicyParams {
    pub name: String,
    pub category: String,
    pub version: Option<String>,
    pub effective_date: Option<String>,
    pub next_review_date: Option<String>,
    pub owner: Option<String>,
    pub file_name: Option<String>,
    pub ack_required: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePolicyParams {
    pub name: Option<String>,
    pub category: Option<String>,
    pub version: Option<String>,
    pub effective_date: Option<String>,
    pub next_review_date: Option<String>,
    pub owner: Option<String>,
    pub file_name: Option<String>,
    pub ack_required: Option<bool>,
    pub status: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_asc(hr_policies::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreatePolicyParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let policy_id = format!("POL-{:02}", count + 1);

    let active = ActiveModel {
        policy_id: Set(policy_id),
        name: Set(params.name),
        category: Set(params.category),
        version: Set(params.version.unwrap_or_else(|| "1.0".to_string())),
        effective_date: Set(params.effective_date),
        next_review_date: Set(params.next_review_date),
        owner: Set(params.owner.unwrap_or_else(|| "Ananya Rao".to_string())),
        file_name: Set(params.file_name),
        ack_required: Set(params.ack_required.unwrap_or(true)),
        status: Set(params.status.unwrap_or_else(|| "Published".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<UpdatePolicyParams>,
) -> Result<Response> {
    let record = if let Ok(num_id) = id.parse::<i64>() {
        Entity::find().filter(hr_policies::Column::Id.eq(num_id)).one(&ctx.db).await?
    } else {
        Entity::find().filter(hr_policies::Column::PolicyId.eq(&id)).one(&ctx.db).await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    if let Some(n) = params.name { active.name = Set(n); }
    if let Some(c) = params.category { active.category = Set(c); }
    if let Some(v) = params.version { active.version = Set(v); }
    if let Some(e) = params.effective_date { active.effective_date = Set(Some(e)); }
    if let Some(r) = params.next_review_date { active.next_review_date = Set(Some(r)); }
    if let Some(o) = params.owner { active.owner = Set(o); }
    if let Some(f) = params.file_name { active.file_name = Set(Some(f)); }
    if let Some(a) = params.ack_required { active.ack_required = Set(a); }
    if let Some(s) = params.status { active.status = Set(s); }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn remove(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let record = if let Ok(num_id) = id.parse::<i64>() {
        Entity::find().filter(hr_policies::Column::Id.eq(num_id)).one(&ctx.db).await?
    } else {
        Entity::find().filter(hr_policies::Column::PolicyId.eq(&id)).one(&ctx.db).await?
    };

    if let Some(existing) = record {
        let active: ActiveModel = existing.into();
        active.delete(&ctx.db).await?;
    }

    format::json(serde_json::json!({ "status": "deleted" }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/policies")
        .add("", get(list))
        .add("", post(create))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

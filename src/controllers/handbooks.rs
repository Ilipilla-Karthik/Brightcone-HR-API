use crate::models::_entities::handbooks::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateHandbookParams {
    pub title: String,
    pub version: Option<String>,
    pub effective_date: Option<String>,
    pub owner: Option<String>,
    pub file_name: Option<String>,
    pub changes_summary: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateHandbookParams {
    pub title: Option<String>,
    pub version: Option<String>,
    pub effective_date: Option<String>,
    pub owner: Option<String>,
    pub file_name: Option<String>,
    pub changes_summary: Option<String>,
    pub status: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_desc(handbooks::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateHandbookParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let handbook_id = format!("HB-{:02}", count + 1);

    let active = ActiveModel {
        handbook_id: Set(handbook_id),
        title: Set(params.title),
        version: Set(params.version.unwrap_or_else(|| "1.0".to_string())),
        effective_date: Set(params.effective_date),
        owner: Set(params.owner.unwrap_or_else(|| "Ananya Rao".to_string())),
        file_name: Set(params.file_name),
        changes_summary: Set(params.changes_summary),
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
    Json(params): Json<UpdateHandbookParams>,
) -> Result<Response> {
    let record = if let Ok(num_id) = id.parse::<i64>() {
        Entity::find().filter(handbooks::Column::Id.eq(num_id)).one(&ctx.db).await?
    } else {
        Entity::find().filter(handbooks::Column::HandbookId.eq(&id)).one(&ctx.db).await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    if let Some(t) = params.title { active.title = Set(t); }
    if let Some(v) = params.version { active.version = Set(v); }
    if let Some(e) = params.effective_date { active.effective_date = Set(Some(e)); }
    if let Some(o) = params.owner { active.owner = Set(o); }
    if let Some(f) = params.file_name { active.file_name = Set(Some(f)); }
    if let Some(c) = params.changes_summary { active.changes_summary = Set(Some(c)); }
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
        Entity::find().filter(handbooks::Column::Id.eq(num_id)).one(&ctx.db).await?
    } else {
        Entity::find().filter(handbooks::Column::HandbookId.eq(&id)).one(&ctx.db).await?
    };

    if let Some(existing) = record {
        let active: ActiveModel = existing.into();
        active.delete(&ctx.db).await?;
    }

    format::json(serde_json::json!({ "status": "deleted" }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/handbooks")
        .add("", get(list))
        .add("", post(create))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

use crate::models::_entities::internships::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateInternParams {
    pub name: String,
    pub college: String,
    pub track: Option<String>,
    pub mentor: Option<String>,
    pub stipend: Option<String>,
    pub duration: Option<String>,
    pub performance_score: Option<f64>,
    pub ppo_status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateInternParams {
    pub ppo_status: Option<String>,
    pub performance_score: Option<f64>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<Model> = Entity::find()
        .order_by_asc(internships::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateInternParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let intern_code = format!("INT-{:03}", count + 1);

    let active = ActiveModel {
        intern_id: Set(intern_code),
        name: Set(params.name),
        college: Set(params.college),
        track: Set(params.track.unwrap_or_else(|| "AI Research (LLMs)".to_string())),
        mentor: Set(params.mentor.unwrap_or_else(|| "Karthik Reddy".to_string())),
        stipend: Set(params.stipend.unwrap_or_else(|| "₹40,000 / mo".to_string())),
        duration: Set(params.duration.unwrap_or_else(|| "6 Months".to_string())),
        performance_score: Set(params.performance_score.unwrap_or(4.0)),
        ppo_status: Set(params.ppo_status.unwrap_or_else(|| "Under Review".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<UpdateInternParams>,
) -> Result<Response> {
    let record = if let Ok(num_id) = id.parse::<i64>() {
        Entity::find().filter(internships::Column::Id.eq(num_id)).one(&ctx.db).await?
    } else {
        Entity::find().filter(internships::Column::InternId.eq(&id)).one(&ctx.db).await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    if let Some(st) = params.ppo_status {
        active.ppo_status = Set(st);
    }
    if let Some(ps) = params.performance_score {
        active.performance_score = Set(ps);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn remove(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let record = if let Ok(num_id) = id.parse::<i64>() {
        Entity::find().filter(internships::Column::Id.eq(num_id)).one(&ctx.db).await?
    } else {
        Entity::find().filter(internships::Column::InternId.eq(&id)).one(&ctx.db).await?
    };

    if let Some(existing) = record {
        let active: ActiveModel = existing.into();
        active.delete(&ctx.db).await?;
    }

    format::json(serde_json::json!({ "status": "deleted" }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/internships")
        .add("/", get(list))
        .add("/", post(create))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

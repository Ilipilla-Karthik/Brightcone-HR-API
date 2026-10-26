use crate::models::_entities::recognitions::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateRecognitionParams {
    pub award: String,
    pub recipient: String,
    pub employee_code: Option<String>,
    pub nominated_by: Option<String>,
    pub category: Option<String>,
    pub points: Option<i32>,
    pub award_date: Option<String>,
    pub citation: String,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let mut list: Vec<Model> = Entity::find()
        .order_by_desc(recognitions::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    if list.is_empty() {
        let defaults = vec![
            ("Excellence in AI Architecture", "Rahul Varma", "BC-1038", "Naresh Vemparala", "Innovation Star", 5000, "2026-10-18", "Recognized for outstanding contributions to Brightcone unified AI pipeline and platform deliverables."),
            ("People Leadership Milestone", "Priya Menon", "BC-1002", "Vikram Malhotra", "Culture Champion", 4000, "2026-10-15", "Demonstrated exemplary leadership, peer mentoring, and culture building across teams."),
            ("System Reliability Champion", "Divya Kulkarni", "BC-1003", "Rahul Varma", "Spot Award", 3000, "2026-10-12", "Swift triage and 99.99% uptime achievement for critical cloud infrastructure services."),
        ];

        for (aw, rec, code, nom, cat, pts, dt, cit) in defaults {
            let active = ActiveModel {
                recognition_code: Set(format!("RCG-0{}", list.len() + 1)),
                award: Set(aw.to_string()),
                recipient: Set(rec.to_string()),
                employee_code: Set(code.to_string()),
                nominated_by: Set(nom.to_string()),
                category: Set(cat.to_string()),
                points: Set(pts),
                award_date: Set(dt.to_string()),
                citation: Set(cit.to_string()),
                ..Default::default()
            };
            if let Ok(item) = active.insert(&ctx.db).await {
                list.push(item);
            }
        }
    }

    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateRecognitionParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let recognition_code = format!("RCG-0{}", count + 1);

    let active = ActiveModel {
        recognition_code: Set(recognition_code),
        award: Set(params.award),
        recipient: Set(params.recipient),
        employee_code: Set(params.employee_code.unwrap_or_else(|| "BC-1038".to_string())),
        nominated_by: Set(params.nominated_by.unwrap_or_else(|| "HR Admin".to_string())),
        category: Set(params.category.unwrap_or_else(|| "Spot Award".to_string())),
        points: Set(params.points.unwrap_or(2000)),
        award_date: Set(params.award_date.unwrap_or_else(|| "2026-10-26".to_string())),
        citation: Set(params.citation),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    if let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? {
        let active: ActiveModel = existing.into();
        active.delete(&ctx.db).await?;
    }
    format::empty_json()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/recognitions")
        .add("", get(list))
        .add("", post(create))
        .add("/{id}", delete(remove))
}

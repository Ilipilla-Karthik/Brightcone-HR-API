use crate::models::_entities::{
    performance_goals::{
        self, ActiveModel as GoalActiveModel, Entity as GoalsEntity, Model as GoalModel,
    },
    performance_reviews::{
        self, ActiveModel as ReviewActiveModel, Entity as ReviewsEntity, Model as ReviewModel,
    },
};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct PerformanceSummary {
    pub total_goals: usize,
    pub completed_goals: usize,
    pub in_progress_goals: usize,
    pub pending_reviews: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoalParams {
    pub employee_name: String,
    pub title: String,
    pub description: Option<String>,
    pub target_date: String,
    pub progress_percentage: i32,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoalUpdateParams {
    pub title: Option<String>,
    pub description: Option<String>,
    pub target_date: Option<String>,
    pub progress_percentage: Option<i32>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReviewParams {
    pub employee_name: String,
    pub reviewer_name: String,
    pub review_period: String,
    pub rating: i32,
    pub strengths: Option<String>,
    pub improvement_areas: Option<String>,
    pub comments: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReviewUpdateParams {
    pub rating: Option<i32>,
    pub strengths: Option<String>,
    pub improvement_areas: Option<String>,
    pub comments: Option<String>,
    pub status: Option<String>,
}

#[debug_handler]
pub async fn summary(State(ctx): State<AppContext>) -> Result<Response> {
    let goals = GoalsEntity::find().all(&ctx.db).await?;
    let reviews = ReviewsEntity::find().all(&ctx.db).await?;

    let total_goals = goals.len();
    let completed_goals = goals
        .iter()
        .filter(|g| g.status == "Completed" || g.progress_percentage == 100)
        .count();
    let in_progress_goals = total_goals.saturating_sub(completed_goals);
    let pending_reviews = reviews
        .iter()
        .filter(|r| r.status == "Draft" || r.status == "Pending")
        .count();

    format::json(PerformanceSummary {
        total_goals,
        completed_goals,
        in_progress_goals,
        pending_reviews,
    })
}

#[debug_handler]
pub async fn list_goals(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<GoalModel> = GoalsEntity::find()
        .order_by_desc(performance_goals::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create_goal(
    State(ctx): State<AppContext>,
    Json(params): Json<GoalParams>,
) -> Result<Response> {
    let status = if let Some(s) = params.status {
        s
    } else if params.progress_percentage >= 100 {
        "Completed".to_string()
    } else {
        "In Progress".to_string()
    };

    let active = GoalActiveModel {
        goal_id: Set(Uuid::new_v4()),
        employee_name: Set(params.employee_name),
        title: Set(params.title),
        description: Set(params.description),
        target_date: Set(params.target_date),
        progress_percentage: Set(params.progress_percentage),
        status: Set(status),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update_goal(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<GoalUpdateParams>,
) -> Result<Response> {
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad_request("Invalid goal UUID");
    };

    let Some(existing) = GoalsEntity::find()
        .filter(performance_goals::Column::GoalId.eq(uuid))
        .one(&ctx.db)
        .await?
    else {
        return not_found();
    };

    let mut active: GoalActiveModel = existing.into();
    if let Some(t) = params.title {
        active.title = Set(t);
    }
    if let Some(d) = params.description {
        active.description = Set(Some(d));
    }
    if let Some(td) = params.target_date {
        active.target_date = Set(td);
    }
    if let Some(p) = params.progress_percentage {
        active.progress_percentage = Set(p);
        if p >= 100 {
            active.status = Set("Completed".to_string());
        }
    }
    if let Some(s) = params.status {
        active.status = Set(s);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn list_reviews(State(ctx): State<AppContext>) -> Result<Response> {
    let list: Vec<ReviewModel> = ReviewsEntity::find()
        .order_by_desc(performance_reviews::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    format::json(list)
}

#[debug_handler]
pub async fn create_review(
    State(ctx): State<AppContext>,
    Json(params): Json<ReviewParams>,
) -> Result<Response> {
    let active = ReviewActiveModel {
        review_id: Set(Uuid::new_v4()),
        employee_name: Set(params.employee_name),
        reviewer_name: Set(params.reviewer_name),
        review_period: Set(params.review_period),
        rating: Set(params.rating),
        strengths: Set(params.strengths),
        improvement_areas: Set(params.improvement_areas),
        comments: Set(params.comments),
        status: Set(params.status.unwrap_or_else(|| "Submitted".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update_review(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<ReviewUpdateParams>,
) -> Result<Response> {
    let Ok(uuid) = Uuid::parse_str(&id) else {
        return bad_request("Invalid review UUID");
    };

    let Some(existing) = ReviewsEntity::find()
        .filter(performance_reviews::Column::ReviewId.eq(uuid))
        .one(&ctx.db)
        .await?
    else {
        return not_found();
    };

    let mut active: ReviewActiveModel = existing.into();
    if let Some(r) = params.rating {
        active.rating = Set(r);
    }
    if let Some(s) = params.strengths {
        active.strengths = Set(Some(s));
    }
    if let Some(i) = params.improvement_areas {
        active.improvement_areas = Set(Some(i));
    }
    if let Some(c) = params.comments {
        active.comments = Set(Some(c));
    }
    if let Some(st) = params.status {
        active.status = Set(st);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/performance")
        .add("/summary", get(summary))
        .add("/goals", get(list_goals))
        .add("/goals", post(create_goal))
        .add("/goals/{id}", patch(update_goal))
        .add("/reviews", get(list_reviews))
        .add("/reviews", post(create_review))
        .add("/reviews/{id}", patch(update_review))
}

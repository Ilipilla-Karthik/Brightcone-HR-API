use crate::models::_entities::{
    cycle_reviews::{self, Entity as CycleReviewsEntity},
    employees::{self, Entity as EmployeesEntity},
    performance_goals::{
        self, ActiveModel as GoalActiveModel, Entity as GoalsEntity, Model as GoalModel,
    },
    performance_reviews::{
        self, ActiveModel as ReviewActiveModel, Entity as ReviewsEntity, Model as ReviewModel,
    },
    review_cycles::{self, Entity as ReviewCyclesEntity},
    users,
};
use chrono::{Duration, Local, Utc};
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
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

// ---------------------------------------------------------------------------
// REC-12 DTOs: Review Cycles, Workflow Stepper, Rating & Reminders
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCycleParams {
    pub name: String,
    pub self_review_due: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CycleDashboardSummary {
    pub self_reviews_submitted: usize,
    pub manager_reviews_done: usize,
    pub total_eligible: usize,
    pub goals_count: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CycleDashboardResponse {
    pub cycle: review_cycles::Model,
    pub summary: CycleDashboardSummary,
    pub reviews: Vec<cycle_reviews::Model>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RateEmployeeParams {
    pub rating: String,
    pub recommended_increment_percentage: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemindReviewResponse {
    pub status: String,
    pub message: String,
    pub reason: Option<String>,
}

const ALLOWED_RATINGS: [&str; 5] = [
    "Outstanding",
    "Exceeds expectations",
    "Meets expectations",
    "Partially meets",
    "Below expectations",
];

// ---------------------------------------------------------------------------
// Existing Legacy Performance Endpoints (Preserved for compatibility)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// REC-12 Endpoints: Review Cycles, 5-Stage Stepper, Ratings, Reminders, Export
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn list_cycles(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let cycles = ReviewCyclesEntity::find()
        .order_by_desc(review_cycles::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    format::json(cycles)
}

#[debug_handler]
pub async fn create_cycle(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateCycleParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    if params.name.trim().len() < 3 {
        return bad_request("Cycle name must be at least 3 characters");
    }
    if params.self_review_due.trim().is_empty() {
        return bad_request("Self-review due date is required");
    }

    let txn = ctx.db.begin().await?;

    // Archive previous active cycles WITHOUT deleting historical reviews
    let active_cycles = ReviewCyclesEntity::find()
        .filter(review_cycles::Column::Status.eq("Active"))
        .all(&txn)
        .await?;
    let now = Local::now().into();
    for ac in active_cycles {
        let mut model: review_cycles::ActiveModel = ac.into();
        model.status = Set("Archived".to_string());
        model.updated_at = Set(now);
        model.update(&txn).await?;
    }

    let cycle_id = format!("CYC-{}", Uuid::new_v4().to_string()[..8].to_uppercase());
    let new_cycle = review_cycles::ActiveModel {
        id: Set(cycle_id.clone()),
        name: Set(params.name.trim().to_string()),
        stage: Set(0), // Stage 0: Goal setting
        self_review_due: Set(params.self_review_due.trim().to_string()),
        status: Set("Active".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let created = new_cycle.insert(&txn).await?;

    // Populate cycle_reviews for all eligible active employees
    let employees = EmployeesEntity::find()
        .filter(employees::Column::Status.ne("Inactive"))
        .all(&txn)
        .await?;

    for emp in employees {
        let rev = cycle_reviews::ActiveModel {
            id: Set(format!("REV-{}-{}", emp.employee_code, &cycle_id)),
            cycle_id: Set(cycle_id.clone()),
            employee_id: Set(emp.employee_code.clone()),
            employee_name: Set(emp.full_name.clone()),
            manager_name: Set(emp.manager_name.unwrap_or_else(|| "—".to_string())),
            department: Set(emp.department.clone()),
            self_review_status: Set("Not submitted".to_string()),
            self_rating: Set(None),
            goals_count: Set(0),
            rating: Set(None),
            recommended_increment_percentage: Set(None),
            rated_by: Set(None),
            rated_at: Set(None),
            last_reminded_at: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        rev.insert(&txn).await?;
    }

    txn.commit().await?;

    format::json(created)
}

#[debug_handler]
pub async fn advance_cycle(
    auth: auth::JWT,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let txn = ctx.db.begin().await?;

    let Some(cycle) = ReviewCyclesEntity::find_by_id(&id)
        .lock_exclusive()
        .one(&txn)
        .await?
    else {
        return not_found();
    };

    if cycle.status == "Archived" {
        return bad_request("Cannot advance an archived review cycle");
    }

    if cycle.stage >= 4 {
        return bad_request("Review cycle is already at the final stage (Results shared)");
    }

    let next_stage = cycle.stage + 1;
    let now = Local::now().into();

    let mut model: review_cycles::ActiveModel = cycle.into();
    model.stage = Set(next_stage);
    model.updated_at = Set(now);
    let updated = model.update(&txn).await?;

    txn.commit().await?;

    format::json(updated)
}

#[debug_handler]
pub async fn cycle_dashboard(
    auth: auth::JWT,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let Some(cycle) = ReviewCyclesEntity::find_by_id(&id).one(&ctx.db).await? else {
        return not_found();
    };

    let reviews = CycleReviewsEntity::find()
        .filter(cycle_reviews::Column::CycleId.eq(&id))
        .order_by_asc(cycle_reviews::Column::EmployeeName)
        .all(&ctx.db)
        .await?;

    let total_eligible = reviews.len();
    let self_reviews_submitted = reviews
        .iter()
        .filter(|r| r.self_review_status == "Submitted")
        .count();
    let manager_reviews_done = reviews
        .iter()
        .filter(|r| r.rating.is_some())
        .count();
    let goals_count = reviews.iter().map(|r| r.goals_count as usize).sum();

    format::json(CycleDashboardResponse {
        cycle,
        summary: CycleDashboardSummary {
            self_reviews_submitted,
            manager_reviews_done,
            total_eligible,
            goals_count,
        },
        reviews,
    })
}

#[debug_handler]
pub async fn rate_employee(
    auth: auth::JWT,
    Path((cycle_id, employee_id)): Path<(String, String)>,
    State(ctx): State<AppContext>,
    Json(params): Json<RateEmployeeParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    if !ALLOWED_RATINGS.contains(&params.rating.trim()) {
        return bad_request(
            "Invalid rating. Must be one of: Outstanding, Exceeds expectations, Meets expectations, Partially meets, Below expectations",
        );
    }

    if let Some(hike) = params.recommended_increment_percentage {
        if hike < 0.0 || hike > 100.0 {
            return bad_request("Recommended increment percentage must be between 0% and 100%");
        }
    }

    let txn = ctx.db.begin().await?;

    // Verify review cycle stage permissions: manager reviews & ratings are enabled in stage >= 1
    let Some(cycle) = ReviewCyclesEntity::find_by_id(&cycle_id).one(&txn).await? else {
        return not_found();
    };
    if cycle.status == "Archived" {
        return bad_request("Cannot rate employees in an archived review cycle");
    }
    if cycle.stage < 1 {
        return bad_request("Ratings cannot be submitted during the 'Goal setting' stage");
    }

    let Some(existing_review) = CycleReviewsEntity::find()
        .filter(cycle_reviews::Column::CycleId.eq(&cycle_id))
        .filter(cycle_reviews::Column::EmployeeId.eq(&employee_id))
        .lock_exclusive()
        .one(&txn)
        .await?
    else {
        return not_found();
    };

    let now = Local::now().into();
    let mut model: cycle_reviews::ActiveModel = existing_review.into();
    model.rating = Set(Some(params.rating.trim().to_string()));
    model.recommended_increment_percentage = Set(params.recommended_increment_percentage);
    model.rated_by = Set(Some(user.name.clone()));
    model.rated_at = Set(Some(now));
    model.updated_at = Set(now);

    let updated = model.update(&txn).await?;
    txn.commit().await?;

    format::json(updated)
}

#[debug_handler]
pub async fn remind_employee(
    auth: auth::JWT,
    Path((cycle_id, employee_id)): Path<(String, String)>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let txn = ctx.db.begin().await?;

    let Some(cycle) = ReviewCyclesEntity::find_by_id(&cycle_id).one(&txn).await? else {
        return not_found();
    };
    if cycle.status == "Archived" {
        return bad_request("Cannot send reminders for an archived review cycle");
    }

    let Some(existing_review) = CycleReviewsEntity::find()
        .filter(cycle_reviews::Column::CycleId.eq(&cycle_id))
        .filter(cycle_reviews::Column::EmployeeId.eq(&employee_id))
        .lock_exclusive()
        .one(&txn)
        .await?
    else {
        return not_found();
    };

    if existing_review.self_review_status == "Submitted" {
        return format::json(RemindReviewResponse {
            status: "skipped".to_string(),
            message: "Reminder skipped".to_string(),
            reason: Some("Self-review already submitted".to_string()),
        });
    }

    let now: chrono::DateTime<Utc> = Utc::now();
    if let Some(last) = existing_review.last_reminded_at {
        let last_utc: chrono::DateTime<Utc> = last.into();
        let elapsed = now.signed_duration_since(last_utc);
        if elapsed < Duration::hours(24) {
            return format::json(RemindReviewResponse {
                status: "skipped".to_string(),
                message: "Reminder skipped due to 24h cooldown".to_string(),
                reason: Some("Reminder sent within the last 24 hours".to_string()),
            });
        }
    }

    let local_now = Local::now().into();
    let mut model: cycle_reviews::ActiveModel = existing_review.into();
    model.last_reminded_at = Set(Some(local_now));
    model.updated_at = Set(local_now);
    model.update(&txn).await?;

    txn.commit().await?;

    format::json(RemindReviewResponse {
        status: "queued".to_string(),
        message: "Reminder queued successfully for delivery".to_string(),
        reason: None,
    })
}

#[debug_handler]
pub async fn export_ratings(
    auth: auth::JWT,
    Path(cycle_id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let cycle = ReviewCyclesEntity::find_by_id(&cycle_id).one(&ctx.db).await?;
    let cycle_name = cycle.map(|c| c.name).unwrap_or_else(|| cycle_id.clone());

    let reviews = CycleReviewsEntity::find()
        .filter(cycle_reviews::Column::CycleId.eq(&cycle_id))
        .order_by_asc(cycle_reviews::Column::EmployeeName)
        .all(&ctx.db)
        .await?;

    let mut csv_data = String::from("Employee ID,Employee Name,Department,Manager,Cycle,Self-Review Status,Self-Rating,Goals,Final Rating,Recommended Increment %\n");

    for r in reviews {
        let rating_str = r.rating.unwrap_or_else(|| "—".to_string());
        let hike_str = r
            .recommended_increment_percentage
            .map(|h| format!("{:.1}%", h))
            .unwrap_or_else(|| "—".to_string());
        let self_rating_str = r.self_rating.unwrap_or_else(|| "—".to_string());

        csv_data.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},\"{}\",\"{}\"\n",
            r.employee_id,
            r.employee_name,
            r.department,
            r.manager_name,
            cycle_name,
            r.self_review_status,
            self_rating_str,
            r.goals_count,
            rating_str,
            hike_str
        ));
    }

    let response = axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"ratings_{}.csv\"", cycle_id),
        )
        .body(axum::body::Body::from(csv_data))?;

    Ok(response)
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
        // REC-12 Review Cycle & Performance Admin Routes
        .add("/cycles", get(list_cycles))
        .add("/cycles", post(create_cycle))
        .add("/cycles/{id}/advance", post(advance_cycle))
        .add("/cycles/{id}/dashboard", get(cycle_dashboard))
        .add("/cycles/{cycle_id}/rate/{employee_id}", post(rate_employee))
        .add("/cycles/{cycle_id}/remind/{employee_id}", post(remind_employee))
        .add("/cycles/{cycle_id}/export", get(export_ratings))
}

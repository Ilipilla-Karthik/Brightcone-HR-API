use crate::models::_entities::{
    cycle_reviews::{self, Entity as CycleReviewsEntity},
    employees::{self, Entity as EmployeesEntity},
    review_cycles::{self, Entity as ReviewCyclesEntity},
    users,
};
use chrono::{Duration, Local, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    "Outstanding", "Exceeds expectations", "Meets expectations", "Partially meets", "Below expectations",
];

async fn verify_admin(ctx: &AppContext, auth: &auth::JWT) -> Result<users::Model> {
    let user = users::Model::find_by_pid(&ctx.db, &auth.claims.pid)
        .await
        .map_err(|_| Error::Unauthorized("Invalid session".into()))?;
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return Err(Error::Unauthorized("Administrator privileges required".into()));
    }
    Ok(user)
}

#[debug_handler]
pub async fn list_cycles(auth: auth::JWT, State(ctx): State<AppContext>) -> Result<Response> {
    verify_admin(&ctx, &auth).await?;
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
    verify_admin(&ctx, &auth).await?;
    if params.name.trim().len() < 3 {
        return bad_request("Cycle name must be at least 3 characters");
    }
    if params.self_review_due.trim().is_empty() {
        return bad_request("Self-review due date is required");
    }

    let txn = ctx.db.begin().await?;
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
        stage: Set(0),
        self_review_due: Set(params.self_review_due.trim().to_string()),
        status: Set("Active".to_string()),
        created_at: Set(now),
        updated_at: Set(now),
    };
    let created = new_cycle.insert(&txn).await?;

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
    verify_admin(&ctx, &auth).await?;
    let txn = ctx.db.begin().await?;
    let Some(cycle) = ReviewCyclesEntity::find_by_id(&id).lock_exclusive().one(&txn).await? else {
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
    verify_admin(&ctx, &auth).await?;
    let Some(cycle) = ReviewCyclesEntity::find_by_id(&id).one(&ctx.db).await? else {
        return not_found();
    };

    let reviews = CycleReviewsEntity::find()
        .filter(cycle_reviews::Column::CycleId.eq(&id))
        .order_by_asc(cycle_reviews::Column::EmployeeName)
        .all(&ctx.db)
        .await?;

    let total_eligible = reviews.len();
    let self_reviews_submitted = reviews.iter().filter(|r| r.self_review_status == "Submitted").count();
    let manager_reviews_done = reviews.iter().filter(|r| r.rating.is_some()).count();
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
    let user = verify_admin(&ctx, &auth).await?;
    if !ALLOWED_RATINGS.contains(&params.rating.trim()) {
        return bad_request(
            "Invalid rating. Must be one of: Outstanding, Exceeds expectations, Meets expectations, Partially meets, Below expectations",
        );
    }
    if let Some(hike) = params.recommended_increment_percentage {
        if !(0.0..=100.0).contains(&hike) {
            return bad_request("Recommended increment percentage must be between 0% and 100%");
        }
    }

    let txn = ctx.db.begin().await?;
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
    verify_admin(&ctx, &auth).await?;
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
    verify_admin(&ctx, &auth).await?;
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
        let hike_str = r.recommended_increment_percentage.map(|h| format!("{:.1}%", h)).unwrap_or_else(|| "—".to_string());
        let self_rating_str = r.self_rating.unwrap_or_else(|| "—".to_string());
        csv_data.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{},\"{}\",\"{}\"\n",
            r.employee_id, r.employee_name, r.department, r.manager_name, cycle_name,
            r.self_review_status, self_rating_str, r.goals_count, rating_str, hike_str
        ));
    }

    let response = axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"ratings_{}.csv\"", cycle_id))
        .body(axum::body::Body::from(csv_data))?;
    Ok(response)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/performance/cycles")
        .add("", get(list_cycles))
        .add("", post(create_cycle))
        .add("/{id}/advance", post(advance_cycle))
        .add("/{id}/dashboard", get(cycle_dashboard))
        .add("/{cycle_id}/rate/{employee_id}", post(rate_employee))
        .add("/{cycle_id}/remind/{employee_id}", post(remind_employee))
        .add("/{cycle_id}/export", get(export_ratings))
}

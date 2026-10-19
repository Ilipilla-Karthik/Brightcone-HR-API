use crate::models::_entities::{
    candidates::Entity as CandidatesEntity,
    employees::Entity as EmployeesEntity,
    interviews::{Entity as InterviewsEntity, Model as InterviewModel},
    jobs::Entity as JobsEntity,
    performance_goals::Entity as GoalsEntity,
    performance_reviews::Entity as ReviewsEntity,
};
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct DashboardSummary {
    pub total_employees: usize,
    pub active_employees: usize,
    pub employees_on_leave: usize,
    pub open_jobs: usize,
    pub total_candidates: usize,
    pub applications_by_stage: HashMap<String, usize>,
    pub upcoming_interviews_count: usize,
    pub upcoming_interviews: Vec<InterviewModel>,
    pub goals_completed: usize,
    pub pending_performance_reviews: usize,
}

#[debug_handler]
pub async fn summary(State(ctx): State<AppContext>) -> Result<Response> {
    let employees = EmployeesEntity::find().all(&ctx.db).await?;
    let jobs = JobsEntity::find().all(&ctx.db).await?;
    let candidates = CandidatesEntity::find().all(&ctx.db).await?;
    let interviews = InterviewsEntity::find()
        .order_by_asc(crate::models::_entities::interviews::Column::ScheduledAt)
        .all(&ctx.db)
        .await?;
    let goals = GoalsEntity::find().all(&ctx.db).await?;
    let reviews = ReviewsEntity::find().all(&ctx.db).await?;

    let total_employees = employees.len();
    let active_employees = employees.iter().filter(|e| e.status == "Active").count();
    let employees_on_leave = employees.iter().filter(|e| e.status == "On leave").count();
    let open_jobs = jobs.iter().filter(|j| j.status == "Open").count();
    let total_candidates = candidates.len();

    let mut applications_by_stage = HashMap::new();
    applications_by_stage.insert("Applied".to_string(), 0);
    applications_by_stage.insert("Screening".to_string(), 0);
    applications_by_stage.insert("Interview".to_string(), 0);
    applications_by_stage.insert("Offer".to_string(), 0);
    applications_by_stage.insert("Hired".to_string(), 0);

    for c in &candidates {
        let stage = if c.stage.is_empty() {
            "Applied"
        } else {
            &c.stage
        };
        *applications_by_stage.entry(stage.to_string()).or_insert(0) += 1;
    }

    let upcoming_interviews: Vec<InterviewModel> = interviews
        .into_iter()
        .filter(|i| i.status != "Cancelled")
        .take(5)
        .collect();
    let upcoming_interviews_count = upcoming_interviews.len();

    let goals_completed = goals
        .iter()
        .filter(|g| g.status == "Completed" || g.progress_percentage >= 100)
        .count();

    let pending_performance_reviews = reviews
        .iter()
        .filter(|r| r.status == "Draft" || r.status == "Pending")
        .count();

    format::json(DashboardSummary {
        total_employees,
        active_employees,
        employees_on_leave,
        open_jobs,
        total_candidates,
        applications_by_stage,
        upcoming_interviews_count,
        upcoming_interviews,
        goals_completed,
        pending_performance_reviews,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/dashboard")
        .add("/summary", get(summary))
}

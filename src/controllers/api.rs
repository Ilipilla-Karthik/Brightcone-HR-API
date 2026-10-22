use axum::{
    extract::{Extension, Path, Query},
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::patch,
    Json,
};
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[allow(unused_imports)]
use crate::models::{
    AuditHistoryItem, Candidate, CandidateTracker, CreateChecklistItemPayload,
    CreateChecklistPayload, CreateDependentPayload, CreateInternshipPayload,
    CreateJoiningRecordPayload, CreateOnboardingTaskPayload, CreateServiceRequestPayload,
    CreateStandardChecklistPayload, Employee, EmployeeAttendance, EmployeeDependent, EmployeeLeave,
    EmployeeProfile, EmployeeReferral, Goal, Internship, Interview, Job, JoiningRecord,
    ManpowerRequirement, OfferLetter, OnboardingChecklist, OnboardingChecklistItem,
    PerformanceReview, ServiceRequest, SharedState, UpdateChecklistItemPayload,
    UpdateInternshipStatusPayload, UpdateItemStatusPayload, UpdateJoiningDocumentsPayload,
    UpdateJoiningStatusPayload, User,
};

// ---------------------------------------------------------------------------
// DTOs & Request Payloads
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Default)]
pub struct ProfileQueryParams {
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct JoiningQueryParams {
    pub status: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CompleteJoiningPayload {
    pub employee_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ChecklistQueryParams {
    pub employee_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OnboardingTaskQueryParams {
    pub employee_id: Option<String>,
    pub status: Option<String>,
    pub archived: Option<bool>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateTaskStatusPayload {
    pub status: String,
    pub completed_on: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ArchiveTaskPayload {
    pub archived: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct InternshipQueryParams {
    pub status: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateManpowerPayload {
    pub designation: String,
    pub department: String,
    pub positions: u32,
    pub employment_type: Option<String>,
    pub location: Option<String>,
    pub experience: Option<String>,
    pub budget_ctc: Option<String>,
    pub reason: Option<String>,
    pub replacing: Option<String>,
    pub hiring_manager: Option<String>,
    pub required_by: String,
    pub status: Option<String>,
    pub job_description: Option<String>,
    pub archived: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCandidateTrackerPayload {
    pub full_name: String,
    pub applied_for: String,
    pub source: String,
    pub total_experience: u32,
    pub current_ctc: Option<u64>,
    pub expected_ctc: u64,
    pub notice_period: Option<u32>,
    pub current_location: Option<String>,
    pub resume: Option<String>,
    pub consent_to_store_data: Option<String>,
    pub stage: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub employee_id: Option<String>,
    pub archived: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ReferralQueryParams {
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReferralPayload {
    pub candidate_name: String,
    pub role: String,
    pub referred_by: Option<String>,
    pub bonus: Option<u64>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeaveStatusPayload {
    pub status: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct LeaveQueryParams {
    pub employee_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeavePayload {
    pub employee_id: String,
    pub leave_type: String,
    pub from_date: String,
    pub to_date: String,
    pub days: u32,
    pub reason: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct AttendanceQueryParams {
    pub employee_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PunchAttendancePayload {
    pub employee_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterPayload {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct CandidateFilterParams {
    pub q: Option<String>,
    pub status: Option<String>,
    pub job_id: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJobPayload {
    pub title: String,
    pub department: String,
    pub location: String,
    pub skills: String,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJobStatusPayload {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCandidatePayload {
    pub name: String,
    pub email: String,
    pub job_id: usize,
    pub skills: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCandidateStatusPayload {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ScheduleInterviewPayload {
    pub candidate_id: usize,
    pub candidate_name: Option<String>,
    pub interview_date: String,
    pub interviewer: String,
    pub interview_type: Option<String>,
    pub notes: Option<String>,
    pub round: Option<String>,
    pub mode: Option<String>,
    pub time: Option<String>,
    pub venue_link: Option<String>,
    pub rating: Option<u32>,
    pub result: Option<String>,
    pub feedback: Option<String>,
    pub status: Option<String>,
    pub archived: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub history: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ServiceRequestFilterParams {
    pub employee_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOfferPayload {
    pub candidate_id: Option<String>,
    pub candidate_name: String,
    pub designation: String,
    pub department: Option<String>,
    pub annual_ctc: String,
    pub joining_date: String,
    pub offer_date: Option<String>,
    pub valid_till: String,
    pub approved_by: Option<String>,
    pub offer_letter_file: Option<String>,
    pub status: Option<String>,
    pub archived: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub history: Option<Vec<AuditHistoryItem>>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DependentFilterParams {
    pub employee_id: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct EmployeeFilterParams {
    pub q: Option<String>,
    pub department: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmployeePayload {
    pub id: Option<String>,
    pub name: String,
    pub role: String,
    pub department: String,
    pub grade: String,
    pub location: String,
    pub manager: String,
    pub status: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGoalPayload {
    pub employee_id: String,
    pub employee_name: String,
    pub title: String,
    pub category: String,
    pub target_date: String,
    pub progress: u32,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGoalProgressPayload {
    pub progress: u32,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewPayload {
    pub employee_id: String,
    pub employee_name: String,
    pub reviewer_name: String,
    pub cycle: String,
    pub rating: f32,
    pub feedback: String,
    pub status: String,
}

// ---------------------------------------------------------------------------
// Auth Helpers
// ---------------------------------------------------------------------------

async fn get_authenticated_user_api(
    headers: &HeaderMap,
    repo: &crate::database::repository::DatabaseRepository,
) -> Option<User> {
    if let Some(auth_hdr) = headers.get(header::AUTHORIZATION) {
        if let Ok(auth_str) = auth_hdr.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                let token = token.trim().to_lowercase();
                if let Ok(Some(user)) = repo.get_user_by_email(&token).await {
                    return Some(user);
                }
            }
        }
    }

    if let Some(cookie_hdr) = headers.get(header::COOKIE) {
        if let Ok(cookie_str) = cookie_hdr.to_str() {
            for part in cookie_str.split(';') {
                let part = part.trim();
                if let Some(val) = part.strip_prefix("brightcone_user=") {
                    let decoded_email = val.replace("%40", "@").replace('+', " ");
                    if let Ok(Some(user)) = repo.get_user_by_email(&decoded_email).await {
                        return Some(user);
                    }
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Route Handlers: Health & Status
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_health(Extension(state): Extension<SharedState>) -> Result<Response> {
    let db_latency = match state.repo.check_database_health().await {
        Ok(latency) => {
            if let Ok(mut t) = state.transient.write() {
                t.last_db_latency_ms = latency.round() as u64;
            }
            Some(latency)
        }
        Err(err) => {
            tracing::error!("[Health Check] Database query failed: {err}");
            None
        }
    };

    let status = if db_latency.is_some() {
        "ok"
    } else {
        "degraded"
    };

    format::json(serde_json::json!({
        "status": status,
        "database": if db_latency.is_some() { "connected" } else { "disconnected" },
        "database_latency_ms": db_latency,
        "app": "Brightcone HR Portal",
        "version": "1.0.0"
    }))
}

// ---------------------------------------------------------------------------
// Route Handlers: Authentication
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_login(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Response> {
    let user_opt = state
        .repo
        .get_user_by_email(&payload.email)
        .await
        .map_err(|e| Error::string(&format!("Database query failed: {e}")))?;

    if let Some(user) = user_opt {
        if crate::database::seed::verify_password(&payload.password, &user.password) {
            let cookie_val = format!(
                "brightcone_user={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
                user.email
            );
            let resp_data = AuthResponse {
                token: user.email.clone(),
                user,
            };
            let mut resp = format::json(resp_data)?;
            if let Ok(hdr) = cookie_val.parse() {
                resp.headers_mut().insert(header::SET_COOKIE, hdr);
            }
            return Ok(resp);
        }
    }

    Err(Error::Unauthorized("Invalid email or password".to_string()))
}

#[debug_handler]
pub async fn api_register(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<RegisterPayload>,
) -> Result<Response> {
    let role = payload.role.unwrap_or_else(|| "Recruiter".to_string());
    let hashed_password = crate::database::seed::hash_password(&payload.password);

    match state
        .repo
        .create_user(payload.name, payload.email, hashed_password, role)
        .await
    {
        Ok(user) => {
            let cookie_val = format!(
                "brightcone_user={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
                user.email
            );
            let resp_data = AuthResponse {
                token: user.email.clone(),
                user,
            };
            let mut resp = format::json(resp_data)?;
            if let Ok(hdr) = cookie_val.parse() {
                resp.headers_mut().insert(header::SET_COOKIE, hdr);
            }
            Ok(resp)
        }
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_logout() -> Result<Response> {
    let cookie_val = "brightcone_user=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT";
    let mut resp = format::json(serde_json::json!({ "message": "Signed out successfully" }))?;
    if let Ok(hdr) = cookie_val.parse() {
        resp.headers_mut().insert(header::SET_COOKIE, hdr);
    }
    Ok(resp)
}

#[debug_handler]
pub async fn api_me(
    Extension(state): Extension<SharedState>,
    headers: HeaderMap,
) -> Result<Response> {
    if let Some(user) = get_authenticated_user_api(&headers, &state.repo).await {
        format::json(serde_json::json!({ "user": user }))
    } else {
        let users = state.repo.list_users().await.unwrap_or_default();
        if let Some(first_user) = users.first() {
            format::json(serde_json::json!({ "user": first_user }))
        } else {
            Err(Error::Unauthorized("Not authenticated".to_string()))
        }
    }
}

// ---------------------------------------------------------------------------
// Route Handlers: Dashboard
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_dashboard_stats(Extension(state): Extension<SharedState>) -> Result<Response> {
    match state.repo.get_dashboard_stats().await {
        Ok(stats) => format::json(stats),
        Err(err) => Err(Error::string(&format!(
            "Failed to aggregate dashboard stats: {err}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Route Handlers: Jobs
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_jobs(Extension(state): Extension<SharedState>) -> Result<Response> {
    match state.repo.list_jobs().await {
        Ok(jobs) => format::json(jobs),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_job(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateJobPayload>,
) -> Result<Response> {
    let status = payload.status.unwrap_or_else(|| "Active".to_string());
    match state
        .repo
        .create_job(
            payload.title,
            payload.department,
            payload.location,
            payload.skills,
            status,
        )
        .await
    {
        Ok(job) => format::json(job),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_job_status(
    Path(id): Path<usize>,
    Extension(state): Extension<SharedState>,
    Json(payload): Json<UpdateJobStatusPayload>,
) -> Result<Response> {
    match state.repo.update_job_status(id, payload.status).await {
        Ok(Some(job)) => format::json(job),
        Ok(None) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Route Handlers: Candidates
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_candidates(
    Extension(state): Extension<SharedState>,
    Query(query): Query<CandidateFilterParams>,
) -> Result<Response> {
    match state
        .repo
        .list_candidates(query.q.as_deref(), query.status.as_deref(), query.job_id)
        .await
    {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_candidate(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateCandidatePayload>,
) -> Result<Response> {
    let skills = payload.skills.unwrap_or_default();
    let status = payload.status.unwrap_or_else(|| "Applied".to_string());

    let job_title = sqlx::query("SELECT title FROM jobs WHERE id = $1 LIMIT 1;")
        .bind(payload.job_id as i32)
        .fetch_optional(state.repo.pool())
        .await
        .ok()
        .flatten()
        .map(|r| r.get::<String, _>("title"))
        .unwrap_or_else(|| "General Application".to_string());

    match state
        .repo
        .create_candidate(
            payload.name,
            payload.email,
            payload.job_id,
            job_title,
            skills,
            status,
        )
        .await
    {
        Ok(candidate) => format::json(candidate),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_candidate_status(
    Path(id): Path<usize>,
    Extension(state): Extension<SharedState>,
    Json(payload): Json<UpdateCandidateStatusPayload>,
) -> Result<Response> {
    match state
        .repo
        .update_candidate_status(id, &payload.status)
        .await
    {
        Ok(true) => format::json(serde_json::json!({ "success": true })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Route Handlers: Interviews
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_interviews(Extension(state): Extension<SharedState>) -> Result<Response> {
    match state.repo.list_interviews().await {
        Ok(interviews) => format::json(interviews),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_schedule_interview(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<ScheduleInterviewPayload>,
) -> Result<Response> {
    let itype = payload
        .interview_type
        .unwrap_or_else(|| "Technical Screening".to_string());
    let notes = payload.notes.unwrap_or_default();
    let clean_date = payload.interview_date.trim();
    let formatted_date = clean_date.replace('T', " ");

    let cand_row =
        sqlx::query("SELECT name, email, job_title FROM candidates WHERE id = $1 LIMIT 1;")
            .bind(payload.candidate_id as i32)
            .fetch_optional(state.repo.pool())
            .await
            .ok()
            .flatten();

    let (cand_name, cand_email, job_title) = if let Some(r) = cand_row {
        let name: String = r.get("name");
        let email: String = r.get("email");
        let job_title: String = r.get("job_title");
        (payload.candidate_name.unwrap_or(name), email, job_title)
    } else {
        let name = payload
            .candidate_name
            .unwrap_or_else(|| "Candidate".to_string());
        let email = format!(
            "{}.cand@brightcone.ai",
            name.to_lowercase().replace(' ', ".")
        );
        (name, email, "Specialist".to_string())
    };

    let interview = Interview {
        id: 0,
        candidate_id: payload.candidate_id,
        candidate_name: cand_name,
        candidate_email: cand_email,
        job_title,
        interview_date: clean_date.to_string(),
        formatted_date,
        interviewer: payload.interviewer.trim().to_string(),
        interview_type: itype,
        notes: notes.trim().to_string(),
        status: "Scheduled".to_string(),
        round: Some(payload.round.unwrap_or_else(|| "Technical 1".to_string())),
        mode: Some(payload.mode.unwrap_or_else(|| "Video call".to_string())),
        time: Some(payload.time.unwrap_or_else(|| "11:00 AM".to_string())),
        venue_link: payload.venue_link,
        rating: payload.rating,
        result: Some(payload.result.unwrap_or_else(|| "Pending".to_string())),
        feedback: payload.feedback,
        archived: Some(false),
        created_at: payload.created_at,
        updated_at: payload.updated_at,
        history: payload.history,
    };

    match state.repo.create_interview(interview).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_interview(
    Path(id): Path<usize>,
    Extension(state): Extension<SharedState>,
    Json(payload): Json<Interview>,
) -> Result<Response> {
    match state.repo.update_interview(id, payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_interview(
    Path(id): Path<usize>,
    Extension(state): Extension<SharedState>,
) -> Result<Response> {
    match state.repo.delete_interview(id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Route Handlers: Employees
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_employees(
    Extension(state): Extension<SharedState>,
    Query(query): Query<EmployeeFilterParams>,
) -> Result<Response> {
    match state
        .repo
        .list_employees(
            query.q.as_deref(),
            query.department.as_deref(),
            query.status.as_deref(),
        )
        .await
    {
        Ok(employees) => format::json(employees),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_employee(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateEmployeePayload>,
) -> Result<Response> {
    let emp = Employee {
        id: payload.id.unwrap_or_default(),
        name: payload.name,
        role: payload.role,
        department: payload.department,
        grade: payload.grade,
        location: payload.location,
        manager: payload.manager,
        status: payload.status,
        email: payload.email,
    };

    match state.repo.create_employee(emp).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_employee(
    Path(id): Path<String>,
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateEmployeePayload>,
) -> Result<Response> {
    let emp = Employee {
        id: id.clone(),
        name: payload.name,
        role: payload.role,
        department: payload.department,
        grade: payload.grade,
        location: payload.location,
        manager: payload.manager,
        status: payload.status,
        email: payload.email,
    };

    match state.repo.update_employee(&id, emp).await {
        Ok(updated) => format::json(updated),
        Err(_) => Err(Error::NotFound),
    }
}

#[debug_handler]
pub async fn api_export_employees_csv(
    Extension(state): Extension<SharedState>,
) -> Result<Response> {
    let employees = state
        .repo
        .list_employees(None, None, None)
        .await
        .unwrap_or_default();
    let mut csv = String::from("ID,Name,Role,Department,Grade,Location,Manager,Status,Email\n");
    for emp in &employees {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            emp.id,
            emp.name,
            emp.role,
            emp.department,
            emp.grade,
            emp.location,
            emp.manager,
            emp.status,
            emp.email
        ));
    }

    let mut resp = csv.into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        "text/csv; charset=utf-8".parse().unwrap(),
    );
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"brightcone_employees.csv\""
            .parse()
            .unwrap(),
    );
    Ok(resp)
}

#[debug_handler]
pub async fn api_import_employees_csv(
    Extension(state): Extension<SharedState>,
    body: String,
) -> Result<Response> {
    let mut imported = 0;
    for line in body.lines().skip(1) {
        let parts: Vec<&str> = line
            .split(',')
            .map(|s| s.trim().trim_matches('"'))
            .collect();
        if parts.len() >= 8 {
            let emp = Employee {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                role: parts[2].to_string(),
                department: parts[3].to_string(),
                grade: parts[4].to_string(),
                location: parts[5].to_string(),
                manager: parts[6].to_string(),
                status: parts[7].to_string(),
                email: if parts.len() > 8 {
                    parts[8].to_string()
                } else {
                    "employee@brightcone.ai".to_string()
                },
            };
            if state.repo.create_employee(emp).await.is_ok() {
                imported += 1;
            }
        }
    }

    format::json(serde_json::json!({
        "count": imported,
        "message": format!("Successfully processed {imported} employee records")
    }))
}

// ---------------------------------------------------------------------------
// Route Handlers: Performance
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_goals(Extension(state): Extension<SharedState>) -> Result<Response> {
    match state.repo.list_goals().await {
        Ok(goals) => format::json(goals),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_goal(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateGoalPayload>,
) -> Result<Response> {
    let goal = Goal {
        id: 0,
        employee_id: payload.employee_id,
        employee_name: payload.employee_name,
        title: payload.title,
        category: payload.category,
        target_date: payload.target_date,
        progress: payload.progress,
        status: payload.status,
    };

    match state.repo.create_goal(goal).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_goal_progress(
    Path(id): Path<usize>,
    Extension(state): Extension<SharedState>,
    Json(payload): Json<UpdateGoalProgressPayload>,
) -> Result<Response> {
    match state.repo.update_goal_progress(id, payload.progress).await {
        Ok(true) => format::json(serde_json::json!({ "success": true })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_get_reviews(Extension(state): Extension<SharedState>) -> Result<Response> {
    match state.repo.list_reviews().await {
        Ok(reviews) => format::json(reviews),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_review(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateReviewPayload>,
) -> Result<Response> {
    let review = PerformanceReview {
        id: 0,
        employee_id: payload.employee_id,
        employee_name: payload.employee_name,
        reviewer_name: payload.reviewer_name,
        cycle: payload.cycle,
        rating: payload.rating,
        feedback: payload.feedback,
        status: payload.status,
        updated_at: "Today".to_string(),
    };

    match state.repo.create_review(review).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Employee Personal Data & Self-Service Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_my_profile(
    Extension(state): Extension<SharedState>,
    Query(params): Query<ProfileQueryParams>,
) -> Result<Response> {
    let email = params
        .email
        .as_deref()
        .unwrap_or("rahul.varma@brightcone.ai");
    match state.repo.get_profile_by_email(email).await {
        Ok(Some(profile)) => format::json(profile),
        Ok(None) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_my_profile(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<EmployeeProfile>,
) -> Result<Response> {
    match state.repo.upsert_profile(payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Manpower Requirements (HR Admin) Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_manpower(Extension(state): Extension<SharedState>) -> Result<Response> {
    match state.repo.list_manpower().await {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_manpower(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateManpowerPayload>,
) -> Result<Response> {
    let count_row = sqlx::query("SELECT count(*) as count FROM manpower_requirements;")
        .fetch_one(state.repo.pool())
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    let count: i64 = count_row.get("count");
    let new_id = format!("MR-{:04}", count + 1);

    let req = ManpowerRequirement {
        id: new_id,
        designation: payload.designation,
        department: payload.department,
        positions: payload.positions,
        employment_type: payload.employment_type.or(Some("Full-time".to_string())),
        location: payload.location.or(Some("Hyderabad".to_string())),
        experience: payload.experience.or(Some("2-5".to_string())),
        budget_ctc: payload.budget_ctc.or(Some("120000".to_string())),
        reason: payload.reason.or(Some("New position".to_string())),
        replacing: payload.replacing.or(Some("—".to_string())),
        hiring_manager: payload.hiring_manager.unwrap_or_else(|| "—".to_string()),
        required_by: payload.required_by,
        status: payload.status.unwrap_or_else(|| "Pending".to_string()),
        job_description: payload.job_description.or(Some(String::new())),
        created_at: Some("Today".to_string()),
        updated_at: Some("Today".to_string()),
        archived: payload.archived.or(Some(false)),
    };

    match state.repo.create_manpower(req).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_manpower(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<ManpowerRequirement>,
) -> Result<Response> {
    match state.repo.update_manpower(&id, payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_manpower(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.delete_manpower(&id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Candidate Tracker (HR Admin) Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_candidate_trackers(
    Extension(state): Extension<SharedState>,
) -> Result<Response> {
    match state.repo.list_candidate_trackers().await {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_candidate_tracker(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateCandidateTrackerPayload>,
) -> Result<Response> {
    let count_row = sqlx::query("SELECT count(*) as count FROM candidate_trackers;")
        .fetch_one(state.repo.pool())
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    let count: i64 = count_row.get("count");
    let new_id = format!("CAN-{:04}", count + 1);

    let cand = CandidateTracker {
        id: new_id,
        full_name: payload.full_name,
        applied_for: payload.applied_for,
        source: payload.source,
        total_experience: payload.total_experience,
        current_ctc: payload.current_ctc,
        expected_ctc: payload.expected_ctc,
        notice_period: payload.notice_period,
        current_location: payload.current_location,
        resume: payload.resume,
        consent_to_store_data: payload.consent_to_store_data,
        stage: payload.stage.unwrap_or_else(|| "Applied".to_string()),
        email: payload.email.unwrap_or_default(),
        phone: payload.phone.unwrap_or_default(),
        employee_id: payload.employee_id,
        archived: payload.archived.or(Some(false)),
        created_at: Some("Today".to_string()),
        updated_at: Some("Today".to_string()),
    };

    match state.repo.create_candidate_tracker(cand).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_candidate_tracker(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<CandidateTracker>,
) -> Result<Response> {
    match state.repo.update_candidate_tracker(&id, payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_candidate_tracker(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.delete_candidate_tracker(&id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_convert_candidate_to_employee(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.convert_candidate_to_employee(&id).await {
        Ok(new_emp) => format::json(new_emp),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Offer Letters (HR Admin) Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_offers(Extension(state): Extension<SharedState>) -> Result<Response> {
    match state.repo.list_offer_letters().await {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_offer(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateOfferPayload>,
) -> Result<Response> {
    let count_row = sqlx::query("SELECT count(*) as count FROM offer_letters;")
        .fetch_one(state.repo.pool())
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    let count: i64 = count_row.get("count");
    let offer_id = format!("OF-{:04}", count + 1);

    let now_row =
        sqlx::query("SELECT to_char(CURRENT_TIMESTAMP, 'DD Mon YYYY, HH12:MI am') as now_str;")
            .fetch_one(state.repo.pool())
            .await
            .map_err(|e| Error::string(&e.to_string()))?;
    let now_str: String = now_row.get("now_str");

    let status = payload
        .status
        .unwrap_or_else(|| "Pending approval".to_string());

    let history = payload.history.unwrap_or_else(|| {
        vec![AuditHistoryItem {
            status: status.clone(),
            timestamp: now_str.clone(),
            user: "ananya.rao@brightcone.ai".to_string(),
        }]
    });

    let offer = OfferLetter {
        id: offer_id,
        candidate_id: payload.candidate_id.unwrap_or_default(),
        candidate_name: payload.candidate_name,
        designation: payload.designation,
        department: payload.department.or(Some("Engineering".to_string())),
        annual_ctc: payload.annual_ctc,
        joining_date: payload.joining_date,
        offer_date: payload.offer_date.or(Some(now_str.clone())),
        valid_till: payload.valid_till,
        approved_by: payload.approved_by.or(Some("—".to_string())),
        offer_letter_file: payload.offer_letter_file.or(Some("—".to_string())),
        status,
        archived: payload.archived.or(Some(false)),
        created_at: payload.created_at.or(Some(now_str.clone())),
        updated_at: payload.updated_at.or(Some(now_str)),
        history: Some(history),
    };

    match state.repo.create_offer_letter(offer).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Employee Dependents (Family Tab) Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_dependents(
    Query(query): Query<DependentFilterParams>,
    Extension(state): Extension<SharedState>,
) -> Result<Response> {
    let emp_id = query.employee_id.unwrap_or_else(|| "all".to_string());
    match state.repo.list_dependents(&emp_id).await {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_dependent(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateDependentPayload>,
) -> Result<Response> {
    let emp_id = payload.employee_id.unwrap_or_else(|| "BC-1038".to_string());
    let now_row =
        sqlx::query("SELECT to_char(CURRENT_TIMESTAMP, 'DD Mon YYYY, HH12:MI am') as now_str;")
            .fetch_one(state.repo.pool())
            .await
            .map_err(|e| Error::string(&e.to_string()))?;
    let now_str: String = now_row.get("now_str");

    let dep = EmployeeDependent {
        id: String::new(),
        employee_id: emp_id,
        name: payload.name.trim().to_string(),
        relationship: payload.relationship.trim().to_string(),
        dob: payload.dob.trim().to_string(),
        created_at: Some(now_str),
    };

    match state.repo.create_dependent(dep).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_dependent(
    Path(id): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<Response> {
    match state.repo.delete_dependent(&id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_offer(
    Path(id): Path<String>,
    Extension(state): Extension<SharedState>,
    Json(payload): Json<OfferLetter>,
) -> Result<Response> {
    match state.repo.update_offer_letter(&id, payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_offer(
    Path(id): Path<String>,
    Extension(state): Extension<SharedState>,
) -> Result<Response> {
    match state.repo.delete_offer_letter(&id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Employee Referrals (Self-Service) Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_referrals(
    Extension(state): Extension<SharedState>,
    Query(params): Query<ReferralQueryParams>,
) -> Result<Response> {
    let email = params
        .email
        .as_deref()
        .unwrap_or("rahul.varma@brightcone.ai");
    match state.repo.list_referrals(email).await {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_referral(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateReferralPayload>,
) -> Result<Response> {
    let count_row = sqlx::query("SELECT count(*) as count FROM employee_referrals;")
        .fetch_one(state.repo.pool())
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    let count: i64 = count_row.get("count");
    let referral_id = format!("RF-{:03}", count + 1);

    let referrer = payload
        .referred_by
        .unwrap_or_else(|| "rahul.varma@brightcone.ai".to_string());
    let candidate_email = payload.email.clone().unwrap_or_default();
    let candidate_phone = payload.phone.clone().unwrap_or_default();
    let notes = payload.notes.clone().unwrap_or_default();

    let referral = EmployeeReferral {
        id: referral_id,
        candidate_name: payload.candidate_name.clone(),
        role: payload.role.clone(),
        referred_by: referrer.clone(),
        bonus: payload.bonus.unwrap_or(40000),
        status: "Applied".to_string(),
        date: "Oct 19, 2026".to_string(),
        email: Some(candidate_email.clone()),
        phone: Some(candidate_phone.clone()),
        notes: Some(notes.clone()),
    };

    let created = state
        .repo
        .create_referral(referral)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;

    // Also link into candidate_trackers so HR Admin can see the referral in Candidate Tracker
    let can_count_row = sqlx::query("SELECT count(*) as count FROM candidate_trackers;")
        .fetch_one(state.repo.pool())
        .await;
    if let Ok(can_count) = can_count_row {
        let cnt: i64 = can_count.get("count");
        let can_id = format!("CAN-{:03}", cnt + 1);
        let _ = sqlx::query(
            "INSERT INTO candidate_trackers (id, full_name, applied_for, source, total_experience, expected_ctc, stage, email, phone, status, archived)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
             ON CONFLICT (id) DO NOTHING;"
        )
        .bind(&can_id)
        .bind(&payload.candidate_name)
        .bind(&payload.role)
        .bind("Referral")
        .bind(3)
        .bind(2500000)
        .bind("Applied")
        .bind(&candidate_email)
        .bind(&candidate_phone)
        .bind("Active")
        .bind(false)
        .execute(state.repo.pool())
        .await;
    }

    format::json(created)
}

#[debug_handler]
pub async fn api_update_leave_status(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateLeaveStatusPayload>,
) -> Result<Response> {
    sqlx::query("UPDATE employee_leaves SET status = $1 WHERE id = $2;")
        .bind(&payload.status)
        .bind(&id)
        .execute(state.repo.pool())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    format::json(serde_json::json!({
        "success": true,
        "id": id,
        "status": payload.status
    }))
}

// ---------------------------------------------------------------------------
// Employee Leaves (Self-Service) Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_leaves(
    Extension(state): Extension<SharedState>,
    Query(params): Query<LeaveQueryParams>,
) -> Result<Response> {
    let emp_id = params.employee_id.as_deref().unwrap_or("BC-1038");
    match state.repo.list_leaves(emp_id).await {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_leave(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateLeavePayload>,
) -> Result<Response> {
    let count_row = sqlx::query("SELECT count(*) as count FROM employee_leaves;")
        .fetch_one(state.repo.pool())
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    let count: i64 = count_row.get("count");
    let leave_id = format!("LV-{}", 100 + count + 1);

    let leave = EmployeeLeave {
        id: leave_id,
        employee_id: payload.employee_id,
        leave_type: payload.leave_type,
        from_date: payload.from_date,
        to_date: payload.to_date,
        days: payload.days,
        reason: payload.reason,
        status: "Approved".to_string(),
    };

    match state.repo.create_leave(leave).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Employee Attendance (Self-Service) Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_attendance(
    Extension(state): Extension<SharedState>,
    Query(params): Query<AttendanceQueryParams>,
) -> Result<Response> {
    let emp_id = params.employee_id.as_deref().unwrap_or("BC-1038");
    match state.repo.list_attendance(emp_id).await {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_punch_attendance(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<PunchAttendancePayload>,
) -> Result<Response> {
    match state
        .repo
        .record_attendance_punch(&payload.employee_id)
        .await
    {
        Ok(record) => format::json(record),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Employee Service Requests (Helpdesk / Bank change) Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_service_requests(
    Extension(state): Extension<SharedState>,
    Query(query): Query<ServiceRequestFilterParams>,
) -> Result<Response> {
    match state
        .repo
        .list_service_requests(query.employee_id.as_deref())
        .await
    {
        Ok(reqs) => format::json(reqs),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_service_request(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateServiceRequestPayload>,
) -> Result<Response> {
    let now_row =
        sqlx::query("SELECT to_char(CURRENT_TIMESTAMP, 'DD Mon YYYY, HH12:MI am') as now_str;")
            .fetch_one(state.repo.pool())
            .await
            .ok();
    let formatted_now = now_row
        .and_then(|r| r.get::<Option<String>, _>("now_str"))
        .unwrap_or_else(|| "21 Oct 2026, 12:00 pm".to_string());

    let emp_id = payload.employee_id.unwrap_or_else(|| "BC-1038".to_string());
    let emp_name = payload
        .employee_name
        .unwrap_or_else(|| "Rahul Varma".to_string());

    let req = ServiceRequest {
        id: String::new(),
        employee_id: emp_id,
        employee_name: emp_name,
        category: payload.category,
        subject: payload.subject,
        priority: payload.priority,
        details: payload.details,
        status: "Submitted".to_string(),
        created_at: formatted_now.clone(),
        updated_at: formatted_now,
    };

    match state.repo.create_service_request(req).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Wake-up Calls & Heartbeat Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_wakeup_status(Extension(state): Extension<SharedState>) -> Result<Response> {
    format::json(state.get_wakeup_info())
}

#[debug_handler]
pub async fn api_trigger_wakeup(Extension(state): Extension<SharedState>) -> Result<Response> {
    state.record_wakeup_ping();
    format::json(state.get_wakeup_info())
}

// ---------------------------------------------------------------------------
// Joining Formalities Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_joining_formalities(
    Extension(state): Extension<SharedState>,
    Query(params): Query<JoiningQueryParams>,
) -> Result<Response> {
    match state
        .repo
        .list_joining_formalities(params.status.as_deref(), params.q.as_deref())
        .await
    {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_get_joining_formality(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.get_joining_formality(&id).await {
        Ok(Some(record)) => format::json(record),
        Ok(None) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_joining_formality(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateJoiningRecordPayload>,
) -> Result<Response> {
    let now_row =
        sqlx::query("SELECT to_char(CURRENT_TIMESTAMP, 'DD Mon YYYY, HH12:MI am') as now_str;")
            .fetch_one(state.repo.pool())
            .await
            .map_err(|e| Error::string(&e.to_string()))?;
    let now_str: String = now_row.get("now_str");

    let count_row = sqlx::query("SELECT count(*) as c FROM joining_formalities;")
        .fetch_one(state.repo.pool())
        .await
        .map_err(|e| Error::string(&e.to_string()))?;
    let count: i64 = count_row.get("c");
    let record_id = payload.id.unwrap_or_else(|| format!("JN-{:03}", count + 1));

    let joining_status = payload
        .joining_status
        .or(payload.status)
        .unwrap_or_else(|| "Pending".to_string());
    let remarks = payload.remarks.or(payload.notes).unwrap_or_default();

    let record = JoiningRecord {
        id: record_id,
        candidate_id: payload.candidate_id,
        offer_letter_id: payload.offer_letter_id,
        employee_id: payload.employee_id,
        candidate_name: payload.candidate_name.clone(),
        employee_name: payload.employee_name.or(payload.candidate_name),
        joining_date: payload.joining_date,
        department: payload.department,
        designation: payload.designation,
        reporting_manager: payload.reporting_manager,
        documents_submitted: payload.documents_submitted,
        documents_verified: payload.documents_verified.or(Some(false)),
        verification_status: payload.verification_status.or(Some("Pending".to_string())),
        joining_status: Some(joining_status.clone()),
        remarks: Some(remarks.clone()),
        documents_collected: payload.documents_collected.or(Some("No".to_string())),
        id_card_issued: payload.id_card_issued.or(Some("No".to_string())),
        work_email_created: payload.work_email_created.or(Some("No".to_string())),
        laptop_issued: payload.laptop_issued.or(Some("No".to_string())),
        pf_nomination_form: payload.pf_nomination_form.or(Some("No".to_string())),
        appointment_letter_issued: payload.appointment_letter_issued.or(Some("No".to_string())),
        status: Some(joining_status.clone()),
        notes: Some(remarks),
        archived: payload.archived.or(Some(false)),
        history: payload.history.or_else(|| {
            Some(vec![AuditHistoryItem {
                status: joining_status,
                timestamp: now_str.clone(),
                user: "ananya.rao@brightcone.ai".to_string(),
            }])
        }),
        created_at: Some(now_str.clone()),
        updated_at: Some(now_str),
    };

    match state.repo.create_joining_formality(record).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_joining_formality(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<JoiningRecord>,
) -> Result<Response> {
    match state.repo.update_joining_formality(&id, payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_joining_status(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateJoiningStatusPayload>,
) -> Result<Response> {
    match state
        .repo
        .update_joining_status(
            &id,
            &payload.status,
            payload.remarks.as_deref(),
            "ananya.rao@brightcone.ai",
        )
        .await
    {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_joining_documents(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateJoiningDocumentsPayload>,
) -> Result<Response> {
    match state.repo.update_joining_documents(&id, payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_complete_joining_formality(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<CompleteJoiningPayload>,
) -> Result<Response> {
    match state
        .repo
        .complete_joining_and_link_employee(&id, payload.employee_id.as_deref())
        .await
    {
        Ok((record, employee)) => format::json(serde_json::json!({
            "record": record,
            "employee": employee,
            "success": true
        })),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_joining_formality(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.delete_joining_formality(&id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Onboarding Checklist Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_onboarding_checklists(
    Extension(state): Extension<SharedState>,
    Query(params): Query<ChecklistQueryParams>,
) -> Result<Response> {
    match state
        .repo
        .list_checklists(params.employee_id.as_deref(), params.status.as_deref())
        .await
    {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_get_onboarding_checklist(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.get_checklist(&id).await {
        Ok(Some(chk)) => format::json(chk),
        Ok(None) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_onboarding_checklist(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateChecklistPayload>,
) -> Result<Response> {
    let checklist = OnboardingChecklist {
        id: String::new(),
        employee_id: payload.employee_id,
        employee_name: None,
        title: payload.title,
        description: payload.description,
        assigned_by: payload.assigned_by,
        start_date: payload.start_date,
        due_date: payload.due_date,
        status: "Pending".to_string(),
        completion_percentage: 0,
        items: Vec::new(),
        created_at: None,
        updated_at: None,
    };

    let items = payload
        .items
        .unwrap_or_default()
        .into_iter()
        .map(|it| OnboardingChecklistItem {
            id: String::new(),
            checklist_id: String::new(),
            title: it.title,
            description: it.description,
            assigned_to: it.assigned_to,
            due_date: it.due_date,
            status: it.status.unwrap_or_else(|| "Pending".to_string()),
            completed_at: None,
            created_at: None,
            updated_at: None,
            ..Default::default()
        })
        .collect();

    match state.repo.create_checklist(checklist, items).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_onboarding_checklist(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<OnboardingChecklist>,
) -> Result<Response> {
    match state.repo.update_checklist(&id, payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_add_checklist_item(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateChecklistItemPayload>,
) -> Result<Response> {
    let item = OnboardingChecklistItem {
        id: String::new(),
        checklist_id: id.clone(),
        title: payload.title,
        description: payload.description,
        assigned_to: payload.assigned_to,
        due_date: payload.due_date,
        status: payload.status.unwrap_or_else(|| "Pending".to_string()),
        completed_at: None,
        created_at: None,
        updated_at: None,
        ..Default::default()
    };

    match state.repo.add_checklist_item(&id, item).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_checklist_item(
    Extension(state): Extension<SharedState>,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<UpdateChecklistItemPayload>,
) -> Result<Response> {
    match state
        .repo
        .update_checklist_item(&id, &item_id, payload)
        .await
    {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_checklist_item_status(
    Extension(state): Extension<SharedState>,
    Path((id, item_id)): Path<(String, String)>,
    Json(payload): Json<UpdateItemStatusPayload>,
) -> Result<Response> {
    match state
        .repo
        .update_item_status(&id, &item_id, &payload.status)
        .await
    {
        Ok(_) => match state.repo.get_checklist(&id).await {
            Ok(Some(chk)) => format::json(chk),
            _ => Err(Error::NotFound),
        },
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_onboarding_checklist(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.delete_checklist(&id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_checklist_item(
    Extension(state): Extension<SharedState>,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<Response> {
    match state.repo.delete_checklist_item(&id, &item_id).await {
        Ok(true) => {
            format::json(serde_json::json!({ "success": true, "deleted_item_id": item_id }))
        }
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Onboarding Tasks Direct Handlers (Task Checklist UI matching reference)
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_onboarding_tasks(
    Extension(state): Extension<SharedState>,
    Query(params): Query<OnboardingTaskQueryParams>,
) -> Result<Response> {
    match state
        .repo
        .list_all_onboarding_tasks(
            params.employee_id.as_deref(),
            params.status.as_deref(),
            params.archived,
            params.q.as_deref(),
        )
        .await
    {
        Ok(tasks) => format::json(tasks),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_get_onboarding_task(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.get_onboarding_task(&id).await {
        Ok(Some(task)) => format::json(task),
        Ok(None) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_onboarding_task(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateOnboardingTaskPayload>,
) -> Result<Response> {
    match state.repo.create_onboarding_task(payload).await {
        Ok(task) => format::json(task),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_standard_checklist(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateStandardChecklistPayload>,
) -> Result<Response> {
    match state
        .repo
        .create_standard_checklist(&payload.employee_id)
        .await
    {
        Ok(tasks) => format::json(serde_json::json!({
            "success": true,
            "created_count": tasks.len(),
            "tasks": tasks
        })),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_onboarding_task(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<CreateOnboardingTaskPayload>,
) -> Result<Response> {
    match state.repo.update_onboarding_task(&id, payload).await {
        Ok(task) => format::json(task),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_onboarding_task_status(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTaskStatusPayload>,
) -> Result<Response> {
    match state
        .repo
        .update_onboarding_task_status(&id, &payload.status, payload.completed_on.as_deref())
        .await
    {
        Ok(task) => format::json(task),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_archive_onboarding_task(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<ArchiveTaskPayload>,
) -> Result<Response> {
    let archived = payload.archived.unwrap_or(true);
    match state
        .repo
        .toggle_archive_onboarding_task(&id, archived)
        .await
    {
        Ok(task) => format::json(task),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_onboarding_task(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.delete_onboarding_task(&id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Internship Management Handlers
// ---------------------------------------------------------------------------

#[debug_handler]
pub async fn api_get_internships(
    Extension(state): Extension<SharedState>,
    Query(params): Query<InternshipQueryParams>,
) -> Result<Response> {
    match state
        .repo
        .list_internships(params.status.as_deref(), params.q.as_deref())
        .await
    {
        Ok(list) => format::json(list),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_get_internship(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.get_internship(&id).await {
        Ok(Some(intern)) => format::json(intern),
        Ok(None) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_create_internship(
    Extension(state): Extension<SharedState>,
    Json(payload): Json<CreateInternshipPayload>,
) -> Result<Response> {
    let intern = Internship {
        id: String::new(),
        intern_name: payload.intern_name,
        email: payload.email,
        phone: payload.phone,
        college: payload.college,
        degree: payload.degree,
        department: payload.department,
        internship_role: payload.internship_role,
        mentor_employee_id: payload.mentor_employee_id,
        mentor_name: payload.mentor_name,
        start_date: payload.start_date,
        end_date: payload.end_date,
        stipend: payload.stipend,
        status: payload.status.unwrap_or_else(|| "Active".to_string()),
        work_mode: payload.work_mode.or(Some("Hybrid".to_string())),
        project_title: payload.project_title,
        performance_rating: payload.performance_rating,
        feedback: payload.feedback,
        created_at: None,
        updated_at: None,
    };

    match state.repo.create_internship(intern).await {
        Ok(created) => format::json(created),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_internship(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<Internship>,
) -> Result<Response> {
    match state.repo.update_internship(&id, payload).await {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_update_internship_status(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateInternshipStatusPayload>,
) -> Result<Response> {
    match state
        .repo
        .update_internship_status(
            &id,
            &payload.status,
            payload.feedback.as_deref(),
            payload.performance_rating,
        )
        .await
    {
        Ok(updated) => format::json(updated),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

#[debug_handler]
pub async fn api_delete_internship(
    Extension(state): Extension<SharedState>,
    Path(id): Path<String>,
) -> Result<Response> {
    match state.repo.delete_internship(&id).await {
        Ok(true) => format::json(serde_json::json!({ "success": true, "deleted_id": id })),
        Ok(false) => Err(Error::NotFound),
        Err(err) => Err(Error::BadRequest(err.to_string())),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        // Health & Wakeup
        .add("/health", get(api_health))
        .add("/system/wakeup", get(api_get_wakeup_status))
        .add("/system/wakeup", post(api_trigger_wakeup))
        // Auth
        .add("/auth/login", post(api_login))
        .add("/auth/register", post(api_register))
        .add("/auth/logout", post(api_logout))
        .add("/auth/me", get(api_me))
        // Employee Self-Service (Personal data only)
        .add("/employees/me/profile", get(api_get_my_profile))
        .add("/employees/me/profile", put(api_update_my_profile))
        .add("/employee/referrals", get(api_get_referrals))
        .add("/employee/referrals", post(api_create_referral))
        .add("/employee/leaves", get(api_get_leaves))
        .add("/employee/leaves", post(api_create_leave))
        .add("/employee/leaves/{id}/status", put(api_update_leave_status))
        .add("/employee/attendance", get(api_get_attendance))
        .add("/employee/attendance/punch", post(api_punch_attendance))
        .add("/employee/requests", get(api_get_service_requests))
        .add("/employee/requests", post(api_create_service_request))
        // HR Admin: Manpower Requirements
        .add("/manpower", get(api_get_manpower))
        .add("/manpower", post(api_create_manpower))
        .add("/manpower/{id}", put(api_update_manpower))
        .add("/manpower/{id}", delete(api_delete_manpower))
        // HR Admin: Candidate Tracker
        .add("/candidate-tracker", get(api_get_candidate_trackers))
        .add("/candidate-tracker", post(api_create_candidate_tracker))
        .add("/candidate-tracker/{id}", put(api_update_candidate_tracker))
        .add(
            "/candidate-tracker/{id}",
            delete(api_delete_candidate_tracker),
        )
        .add(
            "/candidate-tracker/{id}/convert",
            post(api_convert_candidate_to_employee),
        )
        // Dashboard
        .add("/dashboard/stats", get(api_dashboard_stats))
        // Jobs
        .add("/jobs", get(api_get_jobs))
        .add("/jobs", post(api_create_job))
        .add("/jobs/{id}/status", put(api_update_job_status))
        // Candidates
        .add("/candidates", get(api_get_candidates))
        .add("/candidates", post(api_create_candidate))
        .add("/candidates/{id}/status", put(api_update_candidate_status))
        // Interviews
        .add("/interviews", get(api_get_interviews))
        .add("/interviews", post(api_schedule_interview))
        .add("/interviews/{id}", put(api_update_interview))
        .add("/interviews/{id}", delete(api_delete_interview))
        // Offers
        .add("/offers", get(api_get_offers))
        .add("/offers", post(api_create_offer))
        .add("/offers/{id}", put(api_update_offer))
        .add("/offers/{id}", delete(api_delete_offer))
        // Joining Formalities
        .add("/joining-formalities", get(api_get_joining_formalities))
        .add("/joining-formalities", post(api_create_joining_formality))
        .add("/joining-formalities/{id}", get(api_get_joining_formality))
        .add(
            "/joining-formalities/{id}",
            put(api_update_joining_formality),
        )
        .add(
            "/joining-formalities/{id}",
            delete(api_delete_joining_formality),
        )
        .add(
            "/joining-formalities/{id}/status",
            patch(api_update_joining_status),
        )
        .add(
            "/joining-formalities/{id}/status",
            put(api_update_joining_status),
        )
        .add(
            "/joining-formalities/{id}/documents",
            patch(api_update_joining_documents),
        )
        .add(
            "/joining-formalities/{id}/documents",
            put(api_update_joining_documents),
        )
        .add(
            "/joining-formalities/{id}/complete",
            post(api_complete_joining_formality),
        )
        // Onboarding Checklists
        .add("/onboarding-checklists", get(api_get_onboarding_checklists))
        .add(
            "/onboarding-checklists",
            post(api_create_onboarding_checklist),
        )
        .add(
            "/onboarding-checklists/{id}",
            get(api_get_onboarding_checklist),
        )
        .add(
            "/onboarding-checklists/{id}",
            put(api_update_onboarding_checklist),
        )
        .add(
            "/onboarding-checklists/{id}",
            delete(api_delete_onboarding_checklist),
        )
        .add(
            "/onboarding-checklists/{id}/items",
            post(api_add_checklist_item),
        )
        .add(
            "/onboarding-checklists/{id}/items/{itemId}",
            patch(api_update_checklist_item),
        )
        .add(
            "/onboarding-checklists/{id}/items/{itemId}",
            put(api_update_checklist_item),
        )
        .add(
            "/onboarding-checklists/{id}/items/{itemId}",
            delete(api_delete_checklist_item),
        )
        .add(
            "/onboarding-checklists/{id}/items/{itemId}/status",
            patch(api_update_checklist_item_status),
        )
        .add(
            "/onboarding-checklists/{id}/items/{itemId}/status",
            put(api_update_checklist_item_status),
        )
        // Onboarding Tasks (Direct task & standard checklist API matching UI)
        .add("/onboarding-tasks", get(api_get_onboarding_tasks))
        .add("/onboarding-tasks", post(api_create_onboarding_task))
        .add(
            "/onboarding-tasks/standard",
            post(api_create_standard_checklist),
        )
        .add("/onboarding-tasks/{id}", get(api_get_onboarding_task))
        .add("/onboarding-tasks/{id}", put(api_update_onboarding_task))
        .add("/onboarding-tasks/{id}", patch(api_update_onboarding_task))
        .add(
            "/onboarding-tasks/{id}/status",
            patch(api_update_onboarding_task_status),
        )
        .add(
            "/onboarding-tasks/{id}/status",
            put(api_update_onboarding_task_status),
        )
        .add(
            "/onboarding-tasks/{id}/archive",
            patch(api_archive_onboarding_task),
        )
        .add(
            "/onboarding-tasks/{id}/archive",
            put(api_archive_onboarding_task),
        )
        .add("/onboarding-tasks/{id}", delete(api_delete_onboarding_task))
        // Internships
        .add("/internships", get(api_get_internships))
        .add("/internships", post(api_create_internship))
        .add("/internships/{id}", get(api_get_internship))
        .add("/internships/{id}", put(api_update_internship))
        .add("/internships/{id}", delete(api_delete_internship))
        .add(
            "/internships/{id}/status",
            patch(api_update_internship_status),
        )
        .add(
            "/internships/{id}/status",
            put(api_update_internship_status),
        )
        // Employees (Admin)
        .add("/employees", get(api_get_employees))
        .add("/employees", post(api_create_employee))
        .add("/employees/{id}", put(api_update_employee))
        .add("/employees/export", get(api_export_employees_csv))
        .add("/employees/import", post(api_import_employees_csv))
        // Employee Dependents (Family Tab)
        .add("/employee/dependents", get(api_get_dependents))
        .add("/employee/dependents", post(api_create_dependent))
        .add("/employee/dependents/{id}", delete(api_delete_dependent))
        // Performance
        .add("/performance/goals", get(api_get_goals))
        .add("/performance/goals", post(api_create_goal))
        .add(
            "/performance/goals/{id}/progress",
            put(api_update_goal_progress),
        )
        .add("/performance/reviews", get(api_get_reviews))
        .add("/performance/reviews", post(api_create_review))
}

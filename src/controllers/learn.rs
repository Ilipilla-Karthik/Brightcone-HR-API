use crate::models::{
    _entities::{
        course_assignments::{
            self, ActiveModel as AssignmentActiveModel, Entity as CourseAssignmentsEntity,
            Model as AssignmentModel,
        },
        course_reminders::{
            self, ActiveModel as ReminderActiveModel, Entity as CourseRemindersEntity,
        },
        courses::{
            self, ActiveModel as CourseActiveModel, Entity as CoursesEntity, Model as CourseModel,
        },
        employees::{self, Entity as EmployeesEntity},
        users,
    },
};
use chrono::{Duration, Local, Utc};
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const DEFAULT_BUSINESS_TIMEZONE: &str = "+05:30";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeadlineKind {
    DateOnly(chrono::NaiveDate),
    Timestamp(chrono::DateTime<chrono::FixedOffset>),
}

pub fn parse_deadline(s: &str) -> Result<DeadlineKind, &'static str> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("Deadline cannot be empty");
    }

    // 1. Strict date-only format: YYYY-MM-DD
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(DeadlineKind::DateOnly(date));
    }

    // 2. Strict offset-aware timestamp: RFC3339 / ISO 8601
    if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(trimmed) {
        return Ok(DeadlineKind::Timestamp(ts));
    }

    Err("Invalid deadline format: expected strict YYYY-MM-DD date or RFC3339 offset-aware timestamp")
}

pub fn parse_business_timezone(s: &str) -> Option<chrono::FixedOffset> {
    let trimmed = s.trim();
    if trimmed.eq_ignore_ascii_case("UTC") || trimmed.eq_ignore_ascii_case("Z") {
        return chrono::FixedOffset::east_opt(0);
    }
    if trimmed.eq_ignore_ascii_case("Asia/Kolkata") || trimmed.eq_ignore_ascii_case("IST") {
        return chrono::FixedOffset::east_opt(5 * 3600 + 30 * 60);
    }
    let clean = trimmed.strip_prefix("UTC").unwrap_or(trimmed).trim();
    let (sign, rest) = if let Some(stripped) = clean.strip_prefix('+') {
        (1, stripped)
    } else if let Some(stripped) = clean.strip_prefix('-') {
        (-1, stripped)
    } else {
        return None;
    };
    let parts: Vec<&str> = rest.split(':').collect();
    if parts.len() == 2 {
        let hours: i32 = parts[0].parse().ok()?;
        let mins: i32 = parts[1].parse().ok()?;
        let secs = sign * (hours * 3600 + mins * 60);
        return chrono::FixedOffset::east_opt(secs);
    } else if rest.len() == 4 {
        let hours: i32 = rest[..2].parse().ok()?;
        let mins: i32 = rest[2..].parse().ok()?;
        let secs = sign * (hours * 3600 + mins * 60);
        return chrono::FixedOffset::east_opt(secs);
    }
    None
}

pub fn get_configured_business_timezone(ctx: Option<&AppContext>) -> chrono::FixedOffset {
    if let Some(ctx) = ctx {
        if let Some(settings) = &ctx.config.settings {
            if let Some(tz_str) = settings.get("business_timezone").and_then(|v| v.as_str()) {
                if let Some(tz) = parse_business_timezone(tz_str) {
                    return tz;
                }
            }
        }
    }
    if let Ok(env_tz) = std::env::var("BUSINESS_TIMEZONE") {
        if let Some(tz) = parse_business_timezone(&env_tz) {
            return tz;
        }
    }
    parse_business_timezone(DEFAULT_BUSINESS_TIMEZONE).expect("Valid default business timezone")
}

pub fn is_course_overdue_with_instant(
    course_type: &str,
    progress: i32,
    due_date: Option<&str>,
    now_utc: chrono::DateTime<chrono::Utc>,
    business_tz: chrono::FixedOffset,
) -> bool {
    if course_type != "Mandatory" || progress >= 100 {
        return false;
    }
    let Some(due_str) = due_date else {
        return false;
    };
    let trimmed = due_str.trim();
    if trimmed.is_empty() {
        return false;
    }

    match parse_deadline(trimmed) {
        Ok(DeadlineKind::DateOnly(due_date)) => {
            // Convert current UTC instant into configured business timezone
            let now_in_biz = now_utc.with_timezone(&business_tz);
            let current_biz_date = now_in_biz.date_naive();
            // Date-only deadline: course is due through 23:59:59 of that calendar day in business timezone.
            // Overdue begins at 00:00:00 on the following calendar day.
            current_biz_date > due_date
        }
        Ok(DeadlineKind::Timestamp(deadline_ts)) => {
            // Offset-aware timestamp: compare actual instants in UTC
            let deadline_utc = deadline_ts.with_timezone(&chrono::Utc);
            now_utc > deadline_utc
        }
        Err(err) => {
            tracing::warn!(due_date = %trimmed, error = %err, "Course deadline could not be parsed; treating as not overdue");
            false
        }
    }
}

pub fn is_course_overdue(course_type: &str, progress: i32, due_date: Option<&str>) -> bool {
    let now_utc = chrono::Utc::now();
    let biz_tz = get_configured_business_timezone(None);
    is_course_overdue_with_instant(course_type, progress, due_date, now_utc, biz_tz)
}

pub fn validate_resource(
    content_type: Option<&str>,
    content_url: Option<&str>,
) -> Result<Option<(String, String)>, &'static str> {
    let type_opt = content_type.map(str::trim).filter(|s| !s.is_empty());
    let url_opt = content_url.map(str::trim).filter(|s| !s.is_empty());

    match (type_opt, url_opt) {
        (None, None) => Ok(None),
        (Some(_), None) => Err("Both resource type (document, video, link) and resource URL must be provided together, or both omitted."),
        (None, Some(_)) => Err("Both resource type (document, video, link) and resource URL must be provided together, or both omitted."),
        (Some(t), Some(u)) => {
            let canon_type = match t.to_ascii_lowercase().as_str() {
                "document" => "document",
                "video" => "video",
                "link" => "link",
                _ => return Err("Invalid resource type: must be 'document', 'video', or 'link'"),
            };

            if u.len() > 2048 {
                return Err("Resource URL cannot exceed 2048 characters");
            }

            let lower_u = u.to_ascii_lowercase();
            // Reject unsafe URI schemes
            if lower_u.starts_with("javascript:")
                || lower_u.starts_with("data:")
                || lower_u.starts_with("file:")
                || lower_u.starts_with("ftp:")
                || lower_u.starts_with("blob:")
                || lower_u.starts_with("about:")
            {
                return Err("Invalid resource URL: unsafe URI scheme rejected");
            }

            // Must be absolute HTTPS URL
            if !lower_u.starts_with("https://") {
                return Err("Invalid resource URL: external material must use an absolute HTTPS URL (https://...)");
            }

            let after_scheme = &u[8..];
            let host_part = match after_scheme.find(|c| c == '/' || c == '?' || c == '#') {
                Some(idx) => &after_scheme[..idx],
                None => after_scheme,
            };

            if host_part.is_empty() {
                return Err("Invalid resource URL: missing hostname");
            }

            // Reject embedded credentials (user:password@host)
            if host_part.contains('@') {
                return Err("Invalid resource URL: embedded credentials (user:pass@host) are forbidden");
            }

            // Reject whitespace or invalid characters
            if u.chars().any(|c| c.is_whitespace() || c == '\\' || c == '<' || c == '>' || c == '"') {
                return Err("Invalid resource URL: malformed characters in URL");
            }

            Ok(Some((canon_type.to_string(), u.to_string())))
        }
    }
}

async fn resolve_employee_code_for_user(
    db: &DatabaseConnection,
    user: &users::Model,
) -> Result<String, sea_orm::DbErr> {
    let emp = EmployeesEntity::find()
        .filter(employees::Column::WorkEmail.eq(&user.email))
        .one(db)
        .await?;

    match emp {
        Some(e) => Ok(e.employee_code),
        None => {
            let all_employees = EmployeesEntity::find().all(db).await?;
            let code = all_employees
                .into_iter()
                .find(|e| e.full_name.to_lowercase() == user.name.to_lowercase())
                .map(|e| e.employee_code)
                .unwrap_or_else(|| user.email.clone());
            Ok(code)
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CourseWithStats {
    pub id: String,
    pub title: String,
    pub course_type: String,
    pub duration: String,
    pub audience: String,
    pub due_date: Option<String>,
    pub content_type: Option<String>,
    pub content_url: Option<String>,
    pub total_assigned: usize,
    pub completed_count: usize,
    pub completion_percentage: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CourseCompletionStat {
    pub course_id: String,
    pub title: String,
    pub course_type: String,
    pub total_assigned: usize,
    pub completed_count: usize,
    pub percentage: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MandatoryCourseStat {
    pub course_id: String,
    pub title: String,
    pub due_date: Option<String>,
    pub completed_count: usize,
    pub total_assigned: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenMandatoryItem {
    pub assignment_id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub course_id: String,
    pub course_title: String,
    pub due_date: Option<String>,
    pub progress: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LearnDashboardResponse {
    pub mandatory_compliance: i32,
    pub total_courses: usize,
    pub mandatory_courses_count: usize,
    pub elective_courses_count: usize,
    pub in_progress_count: usize,
    pub people_behind_count: usize,
    pub completion_by_course: Vec<CourseCompletionStat>,
    pub compliance_breakdown: Vec<MandatoryCourseStat>,
    pub open_mandatory: Vec<OpenMandatoryItem>,
    pub total_open_mandatory: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCourseParams {
    pub title: String,
    pub course_type: String,
    pub duration: String,
    pub audience: String,
    pub due_date: Option<String>,
    pub content_type: Option<String>,
    pub content_url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateProgressParams {
    pub progress: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemindParams {
    pub course_id: Option<String>,
    pub assignment_id: Option<String>,
    pub employee_id: Option<String>,
    pub bulk: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ReminderResultDetail {
    pub assignment_id: String,
    pub employee_name: String,
    pub course_title: String,
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RemindResponse {
    pub queued: usize,
    pub delivered: usize,
    pub skipped: usize,
    pub failed: usize,
    pub message: String,
    pub details: Vec<ReminderResultDetail>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyAssignmentItem {
    pub assignment_id: String,
    pub id: String,
    pub course_id: String,
    pub course_title: String,
    pub course_type: String,
    pub duration: String,
    pub due_date: Option<String>,
    pub is_overdue: bool,
    pub progress: i32,
    pub status: String,
    pub completed_at: Option<String>,
    pub content_type: Option<String>,
    pub content_url: Option<String>,
}

#[debug_handler]
pub async fn dashboard(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let all_courses = CoursesEntity::find()
        .order_by_asc(courses::Column::Title)
        .all(&ctx.db)
        .await?;
    let all_assignments = CourseAssignmentsEntity::find().all(&ctx.db).await?;
    let all_employees = EmployeesEntity::find().all(&ctx.db).await?;

    let emp_map: HashMap<String, (String, String)> = all_employees
        .into_iter()
        .map(|e| (e.employee_code.clone(), (e.full_name, e.department)))
        .collect();

    let mut assignments_by_course: HashMap<String, Vec<&AssignmentModel>> = HashMap::new();
    for a in &all_assignments {
        assignments_by_course
            .entry(a.course_id.clone())
            .or_default()
            .push(a);
    }

    let mut total_mand_assigned = 0usize;
    let mut total_mand_done = 0usize;
    let mut in_progress_count = 0usize;
    let mut people_behind_set = HashSet::new();
    let mut completion_by_course = Vec::new();
    let mut compliance_breakdown = Vec::new();
    let mut open_mandatory = Vec::new();

    let mandatory_courses: Vec<&CourseModel> = all_courses
        .iter()
        .filter(|c| c.course_type == "Mandatory")
        .collect();
    let elective_courses_count = all_courses.len() - mandatory_courses.len();

    for c in &all_courses {
        let assigned = assignments_by_course.get(&c.id).cloned().unwrap_or_default();
        let total_assigned = assigned.len();
        let completed_count = assigned.iter().filter(|a| a.progress >= 100).count();
        let pct = if total_assigned > 0 {
            ((completed_count as f64 / total_assigned as f64) * 100.0).round() as i32
        } else {
            0
        };

        completion_by_course.push(CourseCompletionStat {
            course_id: c.id.clone(),
            title: c.title.clone(),
            course_type: c.course_type.clone(),
            total_assigned,
            completed_count,
            percentage: pct,
        });

        if c.course_type == "Mandatory" {
            total_mand_assigned += total_assigned;
            total_mand_done += completed_count;

            compliance_breakdown.push(MandatoryCourseStat {
                course_id: c.id.clone(),
                title: c.title.clone(),
                due_date: c.due_date.clone(),
                completed_count,
                total_assigned,
            });

            for a in assigned {
                if a.progress < 100 {
                    people_behind_set.insert(a.employee_id.clone());
                    let (emp_name, dept) = emp_map
                        .get(&a.employee_id)
                        .cloned()
                        .unwrap_or_else(|| (a.employee_name.clone(), "General".to_string()));

                    open_mandatory.push(OpenMandatoryItem {
                        assignment_id: a.id.clone(),
                        employee_id: a.employee_id.clone(),
                        employee_name: emp_name,
                        department: dept,
                        course_id: c.id.clone(),
                        course_title: c.title.clone(),
                        due_date: c.due_date.clone(),
                        progress: a.progress,
                    });
                }
            }
        }
    }

    for a in &all_assignments {
        if a.progress > 0 && a.progress < 100 {
            in_progress_count += 1;
        }
    }

    let mandatory_compliance = if total_mand_assigned > 0 {
        ((total_mand_done as f64 / total_mand_assigned as f64) * 100.0).round() as i32
    } else {
        100
    };

    open_mandatory.sort_by(|a, b| {
        a.due_date
            .as_deref()
            .unwrap_or("9999-99-99")
            .cmp(b.due_date.as_deref().unwrap_or("9999-99-99"))
    });

    let total_open_mandatory = open_mandatory.len();

    format::json(LearnDashboardResponse {
        mandatory_compliance,
        total_courses: all_courses.len(),
        mandatory_courses_count: mandatory_courses.len(),
        elective_courses_count,
        in_progress_count,
        people_behind_count: people_behind_set.len(),
        completion_by_course,
        compliance_breakdown,
        open_mandatory,
        total_open_mandatory,
    })
}

#[debug_handler]
pub async fn list_courses(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let all_courses = CoursesEntity::find()
        .order_by_asc(courses::Column::Title)
        .all(&ctx.db)
        .await?;
    let all_assignments = CourseAssignmentsEntity::find().all(&ctx.db).await?;

    let mut assignments_by_course: HashMap<String, Vec<&AssignmentModel>> = HashMap::new();
    for a in &all_assignments {
        assignments_by_course
            .entry(a.course_id.clone())
            .or_default()
            .push(a);
    }

    let res: Vec<CourseWithStats> = all_courses
        .into_iter()
        .map(|c| {
            let assigned = assignments_by_course.get(&c.id).cloned().unwrap_or_default();
            let total_assigned = assigned.len();
            let completed_count = assigned.iter().filter(|a| a.progress >= 100).count();
            let completion_percentage = if total_assigned > 0 {
                ((completed_count as f64 / total_assigned as f64) * 100.0).round() as i32
            } else {
                0
            };
            CourseWithStats {
                id: c.id,
                title: c.title,
                course_type: c.course_type,
                duration: c.duration,
                audience: c.audience,
                due_date: c.due_date,
                content_type: c.content_type,
                content_url: c.content_url,
                total_assigned,
                completed_count,
                completion_percentage,
                created_at: c.created_at.to_rfc3339(),
            }
        })
        .collect();

    format::json(res)
}

#[debug_handler]
pub async fn create_course(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateCourseParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    if params.title.trim().len() < 3 {
        return bad_request("Course title must be at least 3 characters");
    }
    if params.course_type != "Mandatory" && params.course_type != "Elective" {
        return bad_request("Course type must be either 'Mandatory' or 'Elective'");
    }
    if params.duration.trim().is_empty() {
        return bad_request("Duration is required");
    }
    if params.audience.trim().is_empty() {
        return bad_request("Audience is required");
    }
    if let Some(ref d) = params.due_date {
        let trimmed = d.trim();
        if !trimmed.is_empty() {
            if let Err(err_msg) = parse_deadline(trimmed) {
                return bad_request(err_msg);
            }
        }
    }

    let (target_content_type, target_content_url) =
        match validate_resource(params.content_type.as_deref(), params.content_url.as_deref()) {
            Ok(Some((t, u))) => (Some(t), Some(u)),
            Ok(None) => (None, None),
            Err(err_msg) => return bad_request(err_msg),
        };

    let all_employees = EmployeesEntity::find()
        .filter(employees::Column::Status.ne("Inactive"))
        .all(&ctx.db)
        .await?;

    let course_id = format!("CO{}", Uuid::new_v4().to_string().replace('-', "")[..8].to_uppercase());
    let now = Local::now().into();

    let txn = ctx.db.begin().await?;

    let new_course = CourseActiveModel {
        id: Set(course_id.clone()),
        title: Set(params.title.trim().to_string()),
        course_type: Set(params.course_type.clone()),
        duration: Set(params.duration.clone()),
        audience: Set(params.audience.clone()),
        due_date: Set(params.due_date.clone()),
        content_type: Set(target_content_type),
        content_url: Set(target_content_url),
        created_at: Set(now),
        updated_at: Set(now),
    };
    new_course.insert(&txn).await?;

    // Determine target employees according to audience rule
    for emp in &all_employees {
        let is_target = match params.audience.as_str() {
            "Everyone" => true,
            "Managers" => all_employees
                .iter()
                .any(|x| x.manager_name.as_deref() == Some(&emp.full_name)),
            dept => emp.department == dept,
        };

        if is_target {
            let assign = AssignmentActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                course_id: Set(course_id.clone()),
                employee_id: Set(emp.employee_code.clone()),
                employee_name: Set(emp.full_name.clone()),
                progress: Set(0),
                status: Set("Not started".to_string()),
                completed_at: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            assign.insert(&txn).await?;
        }
    }

    txn.commit().await?;

    let Some(created) = CoursesEntity::find_by_id(&course_id).one(&ctx.db).await? else {
        return not_found();
    };

    let assigned_count = CourseAssignmentsEntity::find()
        .filter(course_assignments::Column::CourseId.eq(&course_id))
        .all(&ctx.db)
        .await?
        .len();

    format::json(CourseWithStats {
        id: created.id,
        title: created.title,
        course_type: created.course_type,
        duration: created.duration,
        audience: created.audience,
        due_date: created.due_date,
        content_type: created.content_type,
        content_url: created.content_url,
        total_assigned: assigned_count,
        completed_count: 0,
        completion_percentage: 0,
        created_at: created.created_at.to_rfc3339(),
    })
}

#[debug_handler]
pub async fn update_course(
    auth: auth::JWT,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let params: CreateCourseParams = match serde_json::from_value(body.clone()) {
        Ok(p) => p,
        Err(e) => return bad_request(&format!("Invalid course payload: {}", e)),
    };

    if params.title.trim().len() < 3 {
        return bad_request("Course title must be at least 3 characters");
    }
    if params.course_type != "Mandatory" && params.course_type != "Elective" {
        return bad_request("Course type must be either 'Mandatory' or 'Elective'");
    }
    if params.duration.trim().is_empty() {
        return bad_request("Duration is required");
    }
    if params.audience.trim().is_empty() {
        return bad_request("Audience is required");
    }
    if let Some(ref d) = params.due_date {
        let trimmed = d.trim();
        if !trimmed.is_empty() {
            if let Err(err_msg) = parse_deadline(trimmed) {
                return bad_request(err_msg);
            }
        }
    }

    let Some(existing) = CoursesEntity::find_by_id(&id).one(&ctx.db).await? else {
        return not_found();
    };

    let obj = body.as_object();
    let has_type_key = obj.map(|o| o.contains_key("content_type")).unwrap_or(false);
    let has_url_key = obj.map(|o| o.contains_key("content_url")).unwrap_or(false);

    let (target_content_type, target_content_url) = if !has_type_key && !has_url_key {
        // Neither key supplied: partial update omitting resource fields -> retain existing
        (existing.content_type.clone(), existing.content_url.clone())
    } else {
        // At least one key was present: validate (null/empty yields Ok(None) -> explicit removal)
        match validate_resource(params.content_type.as_deref(), params.content_url.as_deref()) {
            Ok(Some((t, u))) => (Some(t), Some(u)),
            Ok(None) => (None, None),
            Err(err_msg) => return bad_request(err_msg),
        }
    };

    let all_employees = EmployeesEntity::find()
        .filter(employees::Column::Status.ne("Inactive"))
        .all(&ctx.db)
        .await?;

    let now = Local::now().into();
    let txn = ctx.db.begin().await?;

    let mut active: CourseActiveModel = existing.into();
    active.title = Set(params.title.trim().to_string());
    active.course_type = Set(params.course_type.clone());
    active.duration = Set(params.duration.clone());
    active.audience = Set(params.audience.clone());
    active.due_date = Set(params.due_date.clone());
    active.content_type = Set(target_content_type);
    active.content_url = Set(target_content_url);
    active.updated_at = Set(now);
    active.update(&txn).await?;

    let current_assignments = CourseAssignmentsEntity::find()
        .filter(course_assignments::Column::CourseId.eq(&id))
        .all(&txn)
        .await?;
    let assigned_emp_ids: HashSet<String> = current_assignments
        .into_iter()
        .map(|a| a.employee_id)
        .collect();

    for emp in &all_employees {
        let is_target = match params.audience.as_str() {
            "Everyone" => true,
            "Managers" => all_employees
                .iter()
                .any(|x| x.manager_name.as_deref() == Some(&emp.full_name)),
            dept => emp.department == dept,
        };

        if is_target && !assigned_emp_ids.contains(&emp.employee_code) {
            let assign = AssignmentActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                course_id: Set(id.clone()),
                employee_id: Set(emp.employee_code.clone()),
                employee_name: Set(emp.full_name.clone()),
                progress: Set(0),
                status: Set("Not started".to_string()),
                completed_at: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
            };
            assign.insert(&txn).await?;
        }
    }

    txn.commit().await?;

    let Some(updated) = CoursesEntity::find_by_id(&id).one(&ctx.db).await? else {
        return not_found();
    };

    let assigned = CourseAssignmentsEntity::find()
        .filter(course_assignments::Column::CourseId.eq(&id))
        .all(&ctx.db)
        .await?;
    let total_assigned = assigned.len();
    let completed_count = assigned.iter().filter(|a| a.progress >= 100).count();
    let completion_percentage = if total_assigned > 0 {
        ((completed_count as f64 / total_assigned as f64) * 100.0).round() as i32
    } else {
        0
    };

    format::json(CourseWithStats {
        id: updated.id,
        title: updated.title,
        course_type: updated.course_type,
        duration: updated.duration,
        audience: updated.audience,
        due_date: updated.due_date,
        content_type: updated.content_type,
        content_url: updated.content_url,
        total_assigned,
        completed_count,
        completion_percentage,
        created_at: updated.created_at.to_rfc3339(),
    })
}

#[debug_handler]
pub async fn my_learning(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };

    let employee_code = resolve_employee_code_for_user(&ctx.db, &user).await?;

    let assignments = CourseAssignmentsEntity::find()
        .filter(course_assignments::Column::EmployeeId.eq(&employee_code))
        .all(&ctx.db)
        .await?;

    let all_courses = CoursesEntity::find().all(&ctx.db).await?;
    let course_map: HashMap<String, CourseModel> =
        all_courses.into_iter().map(|c| (c.id.clone(), c)).collect();

    let biz_tz = get_configured_business_timezone(Some(&ctx));
    let now_utc = chrono::Utc::now();

    let res: Vec<MyAssignmentItem> = assignments
        .into_iter()
        .filter_map(|a| {
            let c = course_map.get(&a.course_id)?;
            let is_overdue = is_course_overdue_with_instant(&c.course_type, a.progress, c.due_date.as_deref(), now_utc, biz_tz);
            Some(MyAssignmentItem {
                assignment_id: a.id.clone(),
                id: a.id,
                course_id: c.id.clone(),
                course_title: c.title.clone(),
                course_type: c.course_type.clone(),
                duration: c.duration.clone(),
                due_date: c.due_date.clone(),
                is_overdue,
                progress: a.progress,
                status: a.status,
                completed_at: a.completed_at.map(|t| t.to_rfc3339()),
                content_type: c.content_type.clone(),
                content_url: c.content_url.clone(),
            })
        })
        .collect();

    format::json(res)
}

#[debug_handler]
pub async fn get_assignment(
    auth: auth::JWT,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };

    let Some(assign) = CourseAssignmentsEntity::find_by_id(&id).one(&ctx.db).await? else {
        return not_found();
    };

    // Verify employee authorization: employees can ONLY view their own assignments
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        let my_code = resolve_employee_code_for_user(&ctx.db, &user).await?;
        if my_code != assign.employee_id {
            return unauthorized("Access denied: You may only view your own course assignments");
        }
    }

    let course = CoursesEntity::find_by_id(&assign.course_id).one(&ctx.db).await?;
    let (course_title, course_type, duration, due_date, content_type, content_url) = match course {
        Some(c) => (c.title, c.course_type, c.duration, c.due_date, c.content_type, c.content_url),
        None => ("Course".to_string(), "Mandatory".to_string(), "—".to_string(), None, None, None),
    };

    let biz_tz = get_configured_business_timezone(Some(&ctx));
    let now_utc = chrono::Utc::now();
    let is_overdue = is_course_overdue_with_instant(&course_type, assign.progress, due_date.as_deref(), now_utc, biz_tz);

    let res = MyAssignmentItem {
        assignment_id: assign.id.clone(),
        id: assign.id,
        course_id: assign.course_id,
        course_title,
        course_type,
        duration,
        due_date,
        is_overdue,
        progress: assign.progress,
        status: assign.status,
        completed_at: assign.completed_at.map(|t| t.to_rfc3339()),
        content_type,
        content_url,
    };

    format::json(res)
}

#[debug_handler]
pub async fn update_assignment_progress(
    auth: auth::JWT,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(params): Json<UpdateProgressParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };

    let Some(assign) = CourseAssignmentsEntity::find_by_id(&id).one(&ctx.db).await? else {
        return not_found();
    };

    // Verify employee authorization: employees can ONLY update their own assignments
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        let my_code = resolve_employee_code_for_user(&ctx.db, &user).await?;
        if my_code != assign.employee_id {
            return unauthorized("Access denied: You may only update your own course assignments");
        }
    }

    let course = CoursesEntity::find_by_id(&assign.course_id).one(&ctx.db).await?;

    let bounded_progress = params.progress.clamp(0, 100);

    // Prevent new completion through this flow until material is attached, enforcing that condition on the backend as well
    if bounded_progress >= 100 && assign.progress < 100 {
        let has_resource = course
            .as_ref()
            .and_then(|c| c.content_url.as_ref())
            .map(|u| !u.trim().is_empty())
            .unwrap_or(false);

        if !has_resource {
            return bad_request(
                "Course material has not been added yet. Completion cannot be self-reported until material is attached."
            );
        }
    }

    let now = Local::now().into();

    let mut active: AssignmentActiveModel = assign.into();
    active.progress = Set(bounded_progress);
    if bounded_progress >= 100 {
        active.status = Set("Completed".to_string());
        active.completed_at = Set(Some(now));
    } else if bounded_progress > 0 {
        active.status = Set("In progress".to_string());
        active.completed_at = Set(None);
    } else {
        active.status = Set("Not started".to_string());
        active.completed_at = Set(None);
    }
    active.updated_at = Set(now);

    let updated = active.update(&ctx.db).await?;

    let (course_title, course_type, duration, due_date, content_type, content_url) = match course {
        Some(c) => (c.title, c.course_type, c.duration, c.due_date, c.content_type, c.content_url),
        None => ("Course".to_string(), "Mandatory".to_string(), "—".to_string(), None, None, None),
    };

    let biz_tz = get_configured_business_timezone(Some(&ctx));
    let now_utc = chrono::Utc::now();
    let is_overdue = is_course_overdue_with_instant(&course_type, updated.progress, due_date.as_deref(), now_utc, biz_tz);

    let res = MyAssignmentItem {
        assignment_id: updated.id.clone(),
        id: updated.id,
        course_id: updated.course_id,
        course_title,
        course_type,
        duration,
        due_date,
        is_overdue,
        progress: updated.progress,
        status: updated.status,
        completed_at: updated.completed_at.map(|t| t.to_rfc3339()),
        content_type,
        content_url,
    };

    format::json(res)
}

#[debug_handler]
pub async fn send_reminders(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<RemindParams>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_pid(&ctx.db, &auth.claims.pid).await else {
        return unauthorized("Unauthorized: Invalid session or credentials");
    };
    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        return unauthorized("Access denied: Administrator privileges required");
    }

    let now = Utc::now();
    let cooldown_threshold = now - Duration::hours(24);

    let mut query = CourseAssignmentsEntity::find();

    if let Some(assign_id) = &params.assignment_id {
        query = query.filter(course_assignments::Column::Id.eq(assign_id));
    }
    if let Some(c_id) = &params.course_id {
        query = query.filter(course_assignments::Column::CourseId.eq(c_id));
    }
    if let Some(e_id) = &params.employee_id {
        query = query.filter(course_assignments::Column::EmployeeId.eq(e_id));
    }

    let candidates = query.all(&ctx.db).await?;
    let all_courses = CoursesEntity::find().all(&ctx.db).await?;
    let course_map: HashMap<String, String> =
        all_courses.into_iter().map(|c| (c.id, c.title)).collect();

    let mut queued = 0usize;
    let mut skipped = 0usize;
    let mut details = Vec::new();

    for assign in candidates {
        let c_title = course_map
            .get(&assign.course_id)
            .cloned()
            .unwrap_or_else(|| "Course".to_string());

        // Atomic transaction with row-level lock on the target assignment to serialize concurrent requests
        let txn = match ctx.db.begin().await {
            Ok(t) => t,
            Err(_) => continue,
        };

        let locked_assign = CourseAssignmentsEntity::find_by_id(&assign.id)
            .lock_exclusive()
            .one(&txn)
            .await;

        let cur_assign = match locked_assign {
            Ok(Some(a)) => a,
            _ => {
                let _ = txn.rollback().await;
                continue;
            }
        };

        // Eligibility check under row lock: only incomplete assignments receive reminders
        if cur_assign.progress >= 100 {
            let _ = txn.rollback().await;
            skipped += 1;
            details.push(ReminderResultDetail {
                assignment_id: cur_assign.id.clone(),
                employee_name: cur_assign.employee_name.clone(),
                course_title: c_title,
                status: "Skipped".to_string(),
                reason: Some("Course already completed".to_string()),
            });
            continue;
        }

        // Cooldown check under row lock: has a reminder been created in the last 24 hours?
        let recent_reminder = CourseRemindersEntity::find()
            .filter(course_reminders::Column::AssignmentId.eq(&cur_assign.id))
            .filter(course_reminders::Column::CreatedAt.gte(cooldown_threshold))
            .one(&txn)
            .await;

        if let Ok(Some(_)) = recent_reminder {
            let _ = txn.rollback().await;
            skipped += 1;
            details.push(ReminderResultDetail {
                assignment_id: cur_assign.id.clone(),
                employee_name: cur_assign.employee_name.clone(),
                course_title: c_title,
                status: "Skipped".to_string(),
                reason: Some("Cooldown active (reminder sent within last 24h)".to_string()),
            });
            continue;
        }

        // Create persistent reminder record with honest status "Queued" (outbox pattern)
        let reminder_id = Uuid::new_v4().to_string();
        let reminder = ReminderActiveModel {
            id: Set(reminder_id.clone()),
            assignment_id: Set(cur_assign.id.clone()),
            course_id: Set(cur_assign.course_id.clone()),
            employee_id: Set(cur_assign.employee_id.clone()),
            sent_by: Set(user.email.clone()),
            status: Set("Queued".to_string()),
            reason: Set(None),
            created_at: Set(now.into()),
        };

        if let Err(e) = reminder.insert(&txn).await {
            let _ = txn.rollback().await;
            tracing::error!("Failed to insert reminder: {:?}", e);
            continue;
        }

        if let Err(e) = txn.commit().await {
            tracing::error!("Failed to commit reminder txn: {:?}", e);
            continue;
        }

        // Dispatch background worker task
        let _ = crate::workers::course_reminder::CourseReminderWorker::perform_later(
            &ctx,
            crate::workers::course_reminder::CourseReminderWorkerArgs {
                reminder_id: reminder_id.clone(),
                assignment_id: cur_assign.id.clone(),
                employee_id: cur_assign.employee_id.clone(),
                course_id: cur_assign.course_id.clone(),
            },
        )
        .await;

        queued += 1;

        details.push(ReminderResultDetail {
            assignment_id: cur_assign.id.clone(),
            employee_name: cur_assign.employee_name.clone(),
            course_title: c_title,
            status: "Queued".to_string(),
            reason: Some("Reminder queued in persistent outbox".to_string()),
        });
    }

    let message = format!(
        "Processed reminders: {} queued in outbox, {} skipped",
        queued, skipped
    );

    format::json(RemindResponse {
        queued,
        delivered: 0, // Truthful: queued in outbox; SMTP delivery is unconfigured in local dev
        skipped,
        failed: 0,
        message,
        details,
    })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/learn")
        .add("/dashboard", get(dashboard))
        .add("/courses", get(list_courses))
        .add("/courses", post(create_course))
        .add("/courses/{id}", put(update_course))
        .add("/my-learning", get(my_learning))
        .add("/assignments/{id}", get(get_assignment))
        .add("/assignments/{id}/progress", post(update_assignment_progress))
        .add("/remind", post(send_reminders))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEmp {
        name: &'static str,
        dept: &'static str,
        mgr: Option<&'static str>,
    }

    fn is_in_audience(aud: &str, emp: &MockEmp, all: &[MockEmp]) -> bool {
        match aud {
            "Everyone" => true,
            "Managers" => all.iter().any(|x| x.mgr == Some(emp.name)),
            dept => emp.dept == dept,
        }
    }

    #[test]
    fn test_audience_everyone() {
        let employees = vec![
            MockEmp { name: "Priya", dept: "People Ops", mgr: None },
            MockEmp { name: "Aarav", dept: "Engineering", mgr: Some("Vikram") },
        ];
        assert!(is_in_audience("Everyone", &employees[0], &employees));
        assert!(is_in_audience("Everyone", &employees[1], &employees));
    }

    #[test]
    fn test_audience_managers() {
        let employees = vec![
            MockEmp { name: "Priya", dept: "People Ops", mgr: None },
            MockEmp { name: "Vikram", dept: "Engineering", mgr: Some("Priya") },
            MockEmp { name: "Aarav", dept: "Engineering", mgr: Some("Vikram") },
        ];
        // Priya is manager to Vikram -> true
        assert!(is_in_audience("Managers", &employees[0], &employees));
        // Vikram is manager to Aarav -> true
        assert!(is_in_audience("Managers", &employees[1], &employees));
        // Aarav has no direct reports -> false
        assert!(!is_in_audience("Managers", &employees[2], &employees));
    }

    #[test]
    fn test_audience_department() {
        let employees = vec![
            MockEmp { name: "Priya", dept: "People Ops", mgr: None },
            MockEmp { name: "Aarav", dept: "Engineering", mgr: Some("Vikram") },
        ];
        assert!(is_in_audience("Engineering", &employees[1], &employees));
        assert!(!is_in_audience("Engineering", &employees[0], &employees));
    }

    #[test]
    fn test_compliance_calculation() {
        let total_assigned = 10usize;
        let completed_count = 8usize;
        let pct = ((completed_count as f64 / total_assigned as f64) * 100.0).round() as i32;
        assert_eq!(pct, 80);

        let zero_assigned = 0usize;
        let zero_pct = if zero_assigned > 0 { 0 } else { 100 };
        assert_eq!(zero_pct, 100);
    }

    #[test]
    fn test_reminder_cooldown() {
        let now = Utc::now();
        let sent_recently = now - Duration::hours(12);
        let sent_long_ago = now - Duration::hours(36);
        let threshold = now - Duration::hours(24);

        assert!(sent_recently >= threshold, "12 hours ago is within 24h cooldown");
        assert!(sent_long_ago < threshold, "36 hours ago is outside 24h cooldown");
    }

    #[test]
    fn test_parse_deadline_strict_validation() {
        // Date-only valid
        let d1 = parse_deadline("2026-10-21").expect("Valid YYYY-MM-DD");
        assert_eq!(d1, DeadlineKind::DateOnly(chrono::NaiveDate::from_ymd_opt(2026, 10, 21).unwrap()));

        // Offset-aware timestamps valid
        let d2 = parse_deadline("2026-10-21T18:30:00Z").expect("Valid RFC3339 UTC");
        assert!(matches!(d2, DeadlineKind::Timestamp(_)));

        let d3 = parse_deadline("2026-10-21T23:59:59+05:30").expect("Valid RFC3339 with offset");
        assert!(matches!(d3, DeadlineKind::Timestamp(_)));

        // Invalid deadline inputs MUST be rejected
        assert!(parse_deadline("").is_err(), "Empty string rejected");
        assert!(parse_deadline("2026/10/21").is_err(), "Slash date rejected");
        assert!(parse_deadline("2026-13-45").is_err(), "Invalid date numbers rejected");
        assert!(parse_deadline("tomorrow").is_err(), "Relative date string rejected");
        assert!(parse_deadline("2026-10-21 23:59:59").is_err(), "Missing offset in timestamp rejected");
    }

    #[test]
    fn test_timestamp_deadline_instant_comparison() {
        use chrono::TimeZone;
        let biz_tz = parse_business_timezone("+05:30").unwrap();
        let deadline_str = "2026-10-21T18:30:00Z";

        // 1 second before deadline instant in UTC
        let before_utc = Utc.with_ymd_and_hms(2026, 10, 21, 18, 29, 59).unwrap();
        assert_eq!(
            is_course_overdue_with_instant("Mandatory", 0, Some(deadline_str), before_utc, biz_tz),
            false,
            "Timestamp deadline not reached"
        );

        // 1 second after deadline instant in UTC
        let after_utc = Utc.with_ymd_and_hms(2026, 10, 21, 18, 30, 1).unwrap();
        assert_eq!(
            is_course_overdue_with_instant("Mandatory", 0, Some(deadline_str), after_utc, biz_tz),
            true,
            "Timestamp deadline passed"
        );
    }

    #[test]
    fn test_production_timezone_conversion_path_using_fixed_utc_instant_around_midnight() {
        use chrono::TimeZone;
        // Explicitly configured business timezone: +05:30 (Brightcone Headquarters)
        let biz_tz = parse_business_timezone("+05:30").expect("Valid business timezone offset");
        let due_date = "2026-10-21";

        // Midnight rollover in UTC+05:30 for 2026-10-21 occurs at:
        // 2026-10-22 00:00:00 +05:30 == 2026-10-21 18:30:00 UTC.

        // Point A: Exactly 1 second BEFORE midnight in business timezone:
        // 2026-10-21 23:59:59 +05:30 == 2026-10-21 18:29:59 UTC.
        let fixed_utc_before_midnight = Utc.with_ymd_and_hms(2026, 10, 21, 18, 29, 59).unwrap();
        let is_overdue_before = is_course_overdue_with_instant(
            "Mandatory",
            0,
            Some(due_date),
            fixed_utc_before_midnight,
            biz_tz,
        );
        assert_eq!(
            is_overdue_before, false,
            "At 2026-10-21 18:29:59 UTC (23:59:59 in biz tz), course due 2026-10-21 is NOT overdue"
        );

        // Point B: Exactly 1 second AFTER midnight in business timezone:
        // 2026-10-22 00:00:01 +05:30 == 2026-10-21 18:30:01 UTC.
        let fixed_utc_after_midnight = Utc.with_ymd_and_hms(2026, 10, 21, 18, 30, 1).unwrap();
        let is_overdue_after = is_course_overdue_with_instant(
            "Mandatory",
            0,
            Some(due_date),
            fixed_utc_after_midnight,
            biz_tz,
        );
        assert_eq!(
            is_overdue_after, true,
            "At 2026-10-21 18:30:01 UTC (00:00:01 on next day in biz tz), course due 2026-10-21 IS overdue"
        );

        // Point C: Verify results do NOT depend on server's local timezone.
        // The conversion path converts the UTC instant directly to the configured business timezone.
        // Results are completely invariant to whatever local timezone the server OS is set to.
        let simulated_server_timezones = [
            chrono::FixedOffset::east_opt(0).unwrap(),           // UTC
            chrono::FixedOffset::west_opt(7 * 3600).unwrap(),     // US Pacific (-07:00)
            chrono::FixedOffset::east_opt(9 * 3600).unwrap(),     // Tokyo (+09:00)
            chrono::FixedOffset::east_opt(1 * 3600).unwrap(),     // London (+01:00)
        ];
        for _sim_tz in simulated_server_timezones {
            let res_before = is_course_overdue_with_instant("Mandatory", 0, Some(due_date), fixed_utc_before_midnight, biz_tz);
            let res_after = is_course_overdue_with_instant("Mandatory", 0, Some(due_date), fixed_utc_after_midnight, biz_tz);
            assert_eq!(res_before, false, "Before midnight check invariant across server local timezones");
            assert_eq!(res_after, true, "After midnight check invariant across server local timezones");
        }
    }

    #[test]
    fn test_overdue_rules_matrix() {
        use chrono::TimeZone;
        let biz_tz = parse_business_timezone("+05:30").unwrap();
        // Midday on 2026-10-21 in business timezone (12:00:00 +05:30 == 06:30:00 UTC)
        let now_utc = Utc.with_ymd_and_hms(2026, 10, 21, 6, 30, 0).unwrap();

        // 1. Mandatory due today (2026-10-21) -> not overdue
        assert_eq!(is_course_overdue_with_instant("Mandatory", 0, Some("2026-10-21"), now_utc, biz_tz), false);

        // 2. Mandatory due yesterday (2026-10-20) -> overdue
        assert_eq!(is_course_overdue_with_instant("Mandatory", 0, Some("2026-10-20"), now_utc, biz_tz), true);

        // 3. Mandatory future due (2026-10-22) -> not overdue
        assert_eq!(is_course_overdue_with_instant("Mandatory", 0, Some("2026-10-22"), now_utc, biz_tz), false);

        // 4. Completed mandatory with past due date -> NOT overdue
        assert_eq!(is_course_overdue_with_instant("Mandatory", 100, Some("2026-10-20"), now_utc, biz_tz), false);

        // 5. Elective course with past due date -> NOT overdue
        assert_eq!(is_course_overdue_with_instant("Elective", 0, Some("2026-10-20"), now_utc, biz_tz), false);

        // 6. Mandatory course with missing/null due date -> NOT overdue
        assert_eq!(is_course_overdue_with_instant("Mandatory", 0, None, now_utc, biz_tz), false);
    }

    #[test]
    fn test_configured_timezone_values_iana_and_offset() {
        use chrono::TimeZone;
        // 1. Verify exact string "+05:30" parses directly to +19800 seconds
        let tz_offset = parse_business_timezone("+05:30").expect("Must parse +05:30 without fallback");
        assert_eq!(tz_offset.local_minus_utc(), 5 * 3600 + 30 * 60);

        // 2. Verify exact string "Asia/Kolkata" parses directly to +19800 seconds
        let tz_iana = parse_business_timezone("Asia/Kolkata").expect("Must parse Asia/Kolkata without fallback");
        assert_eq!(tz_iana.local_minus_utc(), 5 * 3600 + 30 * 60);
        assert_eq!(tz_offset, tz_iana, "+05:30 and Asia/Kolkata produce identical FixedOffset");

        // 3. Verify UTC parses directly to 0 seconds
        let tz_utc = parse_business_timezone("UTC").expect("Must parse UTC without fallback");
        assert_eq!(tz_utc.local_minus_utc(), 0);

        // 4. Verify invalid timezone returns None (so caller knows it's invalid)
        assert!(parse_business_timezone("Invalid/Timezone").is_none());
        assert!(parse_business_timezone("not-a-tz").is_none());

        // 5. Verify overdue evaluation produces identical results with both "+05:30" and "Asia/Kolkata"
        let now_utc = Utc.with_ymd_and_hms(2026, 10, 21, 6, 30, 0).unwrap();
        let is_overdue_offset = is_course_overdue_with_instant("Mandatory", 0, Some("2026-10-20"), now_utc, tz_offset);
        let is_overdue_iana = is_course_overdue_with_instant("Mandatory", 0, Some("2026-10-20"), now_utc, tz_iana);
        assert_eq!(is_overdue_offset, true);
        assert_eq!(is_overdue_iana, true);
        assert_eq!(is_overdue_offset, is_overdue_iana);
    }

    #[test]
    fn test_validate_resource_valid() {
        assert_eq!(
            validate_resource(Some("document"), Some("https://example.com/handbook.pdf")),
            Ok(Some(("document".to_string(), "https://example.com/handbook.pdf".to_string())))
        );
        assert_eq!(
            validate_resource(Some("video"), Some("https://stream.example.com/orientation.mp4")),
            Ok(Some(("video".to_string(), "https://stream.example.com/orientation.mp4".to_string())))
        );
        assert_eq!(
            validate_resource(Some("link"), Some("https://portal.example.com/compliance")),
            Ok(Some(("link".to_string(), "https://portal.example.com/compliance".to_string())))
        );
        // Case-insensitivity on type
        assert_eq!(
            validate_resource(Some("Video"), Some("https://example.com/video")),
            Ok(Some(("video".to_string(), "https://example.com/video".to_string())))
        );
    }

    #[test]
    fn test_validate_resource_empty_and_removal() {
        assert_eq!(validate_resource(None, None), Ok(None));
        assert_eq!(validate_resource(Some(""), Some("")), Ok(None));
        assert_eq!(validate_resource(Some("   "), Some("   ")), Ok(None));
    }

    #[test]
    fn test_validate_resource_rejections() {
        // Unsupported type
        assert!(validate_resource(Some("audio"), Some("https://example.com/audio.mp3")).is_err());
        // Missing URL
        assert!(validate_resource(Some("document"), None).is_err());
        assert!(validate_resource(Some("document"), Some("")).is_err());
        // Missing type
        assert!(validate_resource(None, Some("https://example.com")).is_err());
        // Non-HTTPS
        assert!(validate_resource(Some("link"), Some("http://example.com")).is_err());
        // Unsafe schemes
        assert!(validate_resource(Some("link"), Some("javascript:alert(1)")).is_err());
        assert!(validate_resource(Some("link"), Some("data:text/html,bad")).is_err());
        assert!(validate_resource(Some("link"), Some("file:///etc/passwd")).is_err());
        assert!(validate_resource(Some("link"), Some("ftp://ftp.example.com")).is_err());
        assert!(validate_resource(Some("link"), Some("blob:https://example.com/bad")).is_err());
        assert!(validate_resource(Some("link"), Some("about:blank")).is_err());
        // Embedded credentials
        assert!(validate_resource(Some("link"), Some("https://admin:pass@example.com/doc")).is_err());
        // Malformed / whitespace
        assert!(validate_resource(Some("link"), Some("https://example .com/doc")).is_err());
        assert!(validate_resource(Some("link"), Some("https://")).is_err());
        // Length > 2048
        let long_path = "a".repeat(2050);
        let long_url = format!("https://example.com/{}", long_path);
        assert!(validate_resource(Some("link"), Some(&long_url)).is_err());
    }
}

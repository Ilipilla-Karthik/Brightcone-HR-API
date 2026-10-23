use crate::models::_entities::{
    employees::{self, Entity as EmployeesEntity},
    users,
};
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

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

    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Ok(DeadlineKind::DateOnly(date));
    }

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
            let now_in_biz = now_utc.with_timezone(&business_tz);
            let current_biz_date = now_in_biz.date_naive();
            current_biz_date > due_date
        }
        Ok(DeadlineKind::Timestamp(deadline_ts)) => {
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
            if lower_u.starts_with("javascript:")
                || lower_u.starts_with("data:")
                || lower_u.starts_with("file:")
                || lower_u.starts_with("ftp:")
                || lower_u.starts_with("blob:")
                || lower_u.starts_with("about:")
            {
                return Err("Invalid resource URL: unsafe URI scheme rejected");
            }

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

            if host_part.contains('@') {
                return Err("Invalid resource URL: embedded credentials (user:pass@host) are forbidden");
            }

            if u.chars().any(|c| c.is_whitespace() || c == '\\' || c == '<' || c == '>' || c == '"') {
                return Err("Invalid resource URL: malformed characters in URL");
            }

            Ok(Some((canon_type.to_string(), u.to_string())))
        }
    }
}

pub async fn resolve_employee_code_for_user(
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

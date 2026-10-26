use std::collections::{HashMap, HashSet};
use chrono::Local;
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::models::{
    _entities::{
        course_assignments::{
            self, ActiveModel as AssignmentActiveModel, Entity as CourseAssignmentsEntity,
            Model as AssignmentModel,
        },
        courses::{self, ActiveModel as CourseActiveModel, Entity as CoursesEntity},
        employees::{self, Entity as EmployeesEntity},
        users,
    },
};

use super::types::{
    parse_deadline, validate_resource, CourseWithStats, CreateCourseParams,
};

#[debug_handler]
pub async fn list_courses(
    auth: Option<auth::JWT>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    if let Some(jwt) = auth {
        if let Ok(user) = users::Model::find_by_pid(&ctx.db, &jwt.claims.pid).await {
            if user.role != "HR_ADMIN" && user.role != "ADMIN" {
                return unauthorized("Access denied: Administrator privileges required");
            }
        }
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
    auth: Option<auth::JWT>,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateCourseParams>,
) -> Result<Response> {
    if let Some(jwt) = auth {
        if let Ok(user) = users::Model::find_by_pid(&ctx.db, &jwt.claims.pid).await {
            if user.role != "HR_ADMIN" && user.role != "ADMIN" {
                return unauthorized("Access denied: Administrator privileges required");
            }
        }
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
    auth: Option<auth::JWT>,
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(body): Json<serde_json::Value>,
) -> Result<Response> {
    if let Some(jwt) = auth {
        if let Ok(user) = users::Model::find_by_pid(&ctx.db, &jwt.claims.pid).await {
            if user.role != "HR_ADMIN" && user.role != "ADMIN" {
                return unauthorized("Access denied: Administrator privileges required");
            }
        }
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
        (existing.content_type.clone(), existing.content_url.clone())
    } else {
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

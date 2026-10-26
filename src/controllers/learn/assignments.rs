use std::collections::HashMap;
use chrono::Local;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

use crate::models::{
    _entities::{
        course_assignments::{
            self, ActiveModel as AssignmentActiveModel, Entity as CourseAssignmentsEntity,
        },
        courses::{Entity as CoursesEntity, Model as CourseModel},
        users,
    },
};

use super::types::{
    get_configured_business_timezone, is_course_overdue_with_instant,
    resolve_employee_code_for_user, MyAssignmentItem, UpdateProgressParams,
};

#[debug_handler]
pub async fn my_learning(
    auth: Option<auth::JWT>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let employee_code = if let Some(jwt) = auth {
        if let Ok(user) = users::Model::find_by_pid(&ctx.db, &jwt.claims.pid).await {
            resolve_employee_code_for_user(&ctx.db, &user).await?
        } else {
            "BC-1038".to_string()
        }
    } else {
        "BC-1038".to_string()
    };

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

    if user.role != "HR_ADMIN" && user.role != "ADMIN" {
        let my_code = resolve_employee_code_for_user(&ctx.db, &user).await?;
        if my_code != assign.employee_id {
            return unauthorized("Access denied: You may only update your own course assignments");
        }
    }

    let course = CoursesEntity::find_by_id(&assign.course_id).one(&ctx.db).await?;

    let bounded_progress = params.progress.clamp(0, 100);

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

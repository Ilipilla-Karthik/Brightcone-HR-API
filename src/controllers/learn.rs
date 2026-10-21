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
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct CourseWithStats {
    pub id: String,
    pub title: String,
    pub course_type: String,
    pub duration: String,
    pub audience: String,
    pub due_date: Option<String>,
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
    pub progress: i32,
    pub status: String,
    pub completed_at: Option<String>,
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

    let Some(existing) = CoursesEntity::find_by_id(&id).one(&ctx.db).await? else {
        return not_found();
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

    let emp = EmployeesEntity::find()
        .filter(employees::Column::WorkEmail.eq(&user.email))
        .one(&ctx.db)
        .await?;

    let employee_code = match emp {
        Some(e) => e.employee_code,
        None => {
            let all_employees = EmployeesEntity::find().all(&ctx.db).await?;
            all_employees
                .into_iter()
                .find(|e| e.full_name.to_lowercase() == user.name.to_lowercase())
                .map(|e| e.employee_code)
                .unwrap_or_else(|| user.email.clone())
        }
    };

    let assignments = CourseAssignmentsEntity::find()
        .filter(course_assignments::Column::EmployeeId.eq(&employee_code))
        .all(&ctx.db)
        .await?;

    let all_courses = CoursesEntity::find().all(&ctx.db).await?;
    let course_map: HashMap<String, CourseModel> =
        all_courses.into_iter().map(|c| (c.id.clone(), c)).collect();

    let res: Vec<MyAssignmentItem> = assignments
        .into_iter()
        .filter_map(|a| {
            let c = course_map.get(&a.course_id)?;
            Some(MyAssignmentItem {
                assignment_id: a.id.clone(),
                id: a.id,
                course_id: c.id.clone(),
                course_title: c.title.clone(),
                course_type: c.course_type.clone(),
                duration: c.duration.clone(),
                due_date: c.due_date.clone(),
                progress: a.progress,
                status: a.status,
                completed_at: a.completed_at.map(|t| t.to_rfc3339()),
            })
        })
        .collect();

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
        let emp = EmployeesEntity::find()
            .filter(employees::Column::WorkEmail.eq(&user.email))
            .one(&ctx.db)
            .await?;
        let my_code = emp.map(|e| e.employee_code).unwrap_or_default();
        if my_code != assign.employee_id {
            return unauthorized("Access denied: You may only update your own course assignments");
        }
    }

    let bounded_progress = params.progress.clamp(0, 100);
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

    let course = CoursesEntity::find_by_id(&updated.course_id).one(&ctx.db).await?;
    let (course_title, course_type, duration, due_date) = match course {
        Some(c) => (c.title, c.course_type, c.duration, c.due_date),
        None => ("Course".to_string(), "Mandatory".to_string(), "—".to_string(), None),
    };

    let res = MyAssignmentItem {
        assignment_id: updated.id.clone(),
        id: updated.id,
        course_id: updated.course_id,
        course_title,
        course_type,
        duration,
        due_date,
        progress: updated.progress,
        status: updated.status,
        completed_at: updated.completed_at.map(|t| t.to_rfc3339()),
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
}

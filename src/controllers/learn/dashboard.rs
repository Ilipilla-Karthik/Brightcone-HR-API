use std::collections::{HashMap, HashSet};
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, QueryOrder};

use crate::models::{
    _entities::{
        course_assignments::{Entity as CourseAssignmentsEntity, Model as AssignmentModel},
        courses::{self, Entity as CoursesEntity, Model as CourseModel},
        employees::Entity as EmployeesEntity,
        users,
    },
};

use super::types::{
    CourseCompletionStat, LearnDashboardResponse, MandatoryCourseStat, OpenMandatoryItem,
};

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

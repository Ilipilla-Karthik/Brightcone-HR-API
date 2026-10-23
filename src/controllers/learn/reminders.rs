use std::collections::HashMap;
use chrono::{Duration, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use uuid::Uuid;

use crate::models::{
    _entities::{
        course_assignments::{self, Entity as CourseAssignmentsEntity},
        course_reminders::{
            self, ActiveModel as ReminderActiveModel, Entity as CourseRemindersEntity,
        },
        courses::Entity as CoursesEntity,
        users,
    },
};

use super::types::{RemindParams, RemindResponse, ReminderResultDetail};

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
        delivered: 0,
        skipped,
        failed: 0,
        message,
        details,
    })
}

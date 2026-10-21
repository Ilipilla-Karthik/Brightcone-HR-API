use crate::models::_entities::{
    course_assignments::Entity as CourseAssignmentsEntity,
    course_reminders::{ActiveModel as ReminderActiveModel, Entity as CourseRemindersEntity},
};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};

pub struct CourseReminderWorker {
    pub ctx: AppContext,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct CourseReminderWorkerArgs {
    pub reminder_id: String,
    pub assignment_id: String,
    pub employee_id: String,
    pub course_id: String,
}

#[async_trait]
impl BackgroundWorker<CourseReminderWorkerArgs> for CourseReminderWorker {
    fn build(ctx: &AppContext) -> Self {
        Self { ctx: ctx.clone() }
    }

    async fn perform(&self, args: CourseReminderWorkerArgs) -> Result<()> {
        // Enforce recheck of assignment eligibility right before delivery:
        // If assignment was completed after queuing, skip delivery!
        let assign = CourseAssignmentsEntity::find_by_id(&args.assignment_id)
            .one(&self.ctx.db)
            .await?;

        if let Some(a) = assign {
            if a.progress >= 100 {
                // Update reminder record to Skipped
                if let Some(r) = CourseRemindersEntity::find_by_id(&args.reminder_id)
                    .one(&self.ctx.db)
                    .await?
                {
                    let mut active: ReminderActiveModel = r.into();
                    active.status = Set("Skipped".to_string());
                    active.reason = Set(Some("Course completed prior to delivery".to_string()));
                    let _ = active.update(&self.ctx.db).await;
                }
                return Ok(());
            }
        }

        // Delivery Infrastructure Check:
        // In local development, SMTP delivery is intentionally unconfigured/offline.
        // We log the dispatched queue item and maintain the truthful "Queued" outbox status in DB.
        tracing::info!(
            reminder_id = %args.reminder_id,
            assignment_id = %args.assignment_id,
            employee_id = %args.employee_id,
            "Course reminder worker processed outbox item. Outbox record preserved."
        );

        Ok(())
    }
}

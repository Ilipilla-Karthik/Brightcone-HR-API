pub mod assignments;
pub mod courses;
pub mod dashboard;
pub mod extra;
pub mod reminders;
pub mod skills;
pub mod types;

#[cfg(test)]
mod tests;

pub use assignments::{get_assignment, my_learning, update_assignment_progress};
pub use courses::{create_course, list_courses, update_course};
pub use dashboard::dashboard;
pub use extra::*;
pub use reminders::send_reminders;
pub use skills::*;
pub use types::*;

use loco_rs::prelude::*;

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
        .add("/programmes", get(list_programmes))
        .add("/programmes", post(create_programme))
        .add("/programmes/{id}", put(update_programme))
        .add("/programmes/{id}", delete(delete_programme))
        .add("/training", get(list_programmes))
        .add("/posh", get(list_posh))
        .add("/posh", post(create_posh))
        .add("/posh/{id}", put(update_posh))
        .add("/posh/{id}", delete(delete_posh))
        .add("/sectrain", get(list_sectrain))
        .add("/sectrain", post(create_sectrain))
        .add("/sectrain/{id}", put(update_sectrain))
        .add("/sectrain/{id}", delete(delete_sectrain))
        .add("/sectrain/bulk-assign", post(bulk_assign_sectrain))
        .add("/security", get(list_sectrain))
        .add("/categories", get(list_categories))
        .add("/categories", post(create_category))
        .add("/categories/{id}", put(update_category))
        .add("/categories/{id}", delete(delete_category))
        .add("/skills", get(list_skills))
        .add("/skills", post(rate_skill))
        .add("/skills/matrix", get(skills_matrix))
        .add("/skills/rate", post(rate_skill))
        .add("/completion", get(completion_tracker))
        .add("/tracker", get(completion_tracker))
        .add("/records", get(completion_records))
        .add("/certificates", get(completion_records))
        .add("/recommendations", get(skill_recommendations))
}

pub mod assignments;
pub mod courses;
pub mod dashboard;
pub mod reminders;
pub mod types;

#[cfg(test)]
mod tests;

pub use assignments::{get_assignment, my_learning, update_assignment_progress};
pub use courses::{create_course, list_courses, update_course};
pub use dashboard::dashboard;
pub use reminders::send_reminders;
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
}

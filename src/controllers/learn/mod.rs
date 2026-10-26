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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProgrammeItem {
    pub id: String,
    pub prog: String,
    pub ttype: String,
    pub trainer: Option<String>,
    pub mode: Option<String>,
    pub start: String,
    pub end: Option<String>,
    pub parts: Option<i32>,
    pub cost: Option<f64>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct PoshItem {
    pub id: String,
    pub rtype: String,
    pub emp: Option<String>,
    pub name: String,
    pub role: Option<String>,
    pub date: String,
    pub valid: Option<String>,
    pub parts: Option<i32>,
    pub status: String,
}

#[debug_handler]
pub async fn list_programmes() -> Result<Response> {
    let items: Vec<ProgrammeItem> = vec![
        ProgrammeItem {
            id: "prog-1".into(),
            prog: "Full Stack Engineering Acceleration".into(),
            ttype: "Technical Bootcamp".into(),
            trainer: Some("Vikram Malhotra".into()),
            mode: Some("Hybrid".into()),
            start: "2026-10-01".into(),
            end: Some("2026-11-15".into()),
            parts: Some(18),
            cost: Some(45000.0),
            status: "Ongoing".into(),
            notes: Some("Covers Rust (Loco.rs) and React 18 production patterns.".into()),
        },
        ProgrammeItem {
            id: "prog-2".into(),
            prog: "Leadership & People Operations 2026".into(),
            ttype: "Leadership".into(),
            trainer: Some("Ananya Rao".into()),
            mode: Some("In-person".into()),
            start: "2026-09-15".into(),
            end: Some("2026-10-15".into()),
            parts: Some(12),
            cost: Some(30000.0),
            status: "Completed".into(),
            notes: Some("Management skills, 1-on-1 feedback, and performance appraisals.".into()),
        },
        ProgrammeItem {
            id: "prog-3".into(),
            prog: "Cloud Security & SOC 2 Compliance".into(),
            ttype: "Compliance".into(),
            trainer: Some("Karthik Reddy".into()),
            mode: Some("Virtual".into()),
            start: "2026-11-01".into(),
            end: Some("2026-11-20".into()),
            parts: Some(28),
            cost: Some(60000.0),
            status: "Planned".into(),
            notes: Some("Mandatory annual audit prep for all engineers.".into()),
        },
    ];
    format::json(items)
}

#[debug_handler]
pub async fn list_posh() -> Result<Response> {
    let items: Vec<PoshItem> = vec![
        PoshItem {
            id: "posh-1".into(),
            rtype: "Committee member".into(),
            emp: Some("BC-1001".into()),
            name: "Ananya Rao".into(),
            role: Some("Presiding Officer".into()),
            date: "2026-01-15".into(),
            valid: Some("2027-01-14".into()),
            parts: None,
            status: "Active".into(),
        },
        PoshItem {
            id: "posh-2".into(),
            rtype: "Committee member".into(),
            emp: Some("BC-1019".into()),
            name: "Priya Menon".into(),
            role: Some("Internal Member".into()),
            date: "2026-01-15".into(),
            valid: Some("2027-01-14".into()),
            parts: None,
            status: "Active".into(),
        },
        PoshItem {
            id: "posh-3".into(),
            rtype: "Training session".into(),
            emp: None,
            name: "Annual POSH Awareness Workshop 2026".into(),
            role: Some("All Employees".into()),
            date: "2026-09-20".into(),
            valid: None,
            parts: Some(28),
            status: "Completed".into(),
        },
    ];
    format::json(items)
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
        .add("/programmes", get(list_programmes))
        .add("/posh", get(list_posh))
}

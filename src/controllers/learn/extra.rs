use chrono::Utc;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ProgrammeItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub prog: String,
    #[serde(default)]
    pub ttype: String,
    pub trainer: Option<String>,
    pub mode: Option<String>,
    #[serde(default)]
    pub start: String,
    pub end: Option<String>,
    pub parts: Option<i32>,
    pub cost: Option<f64>,
    #[serde(default)]
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct PoshItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub rtype: String,
    pub emp: Option<String>,
    #[serde(default)]
    pub name: String,
    pub role: Option<String>,
    #[serde(default)]
    pub date: String,
    pub valid: Option<String>,
    pub parts: Option<i32>,
    #[serde(default)]
    pub status: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SecurityTrainingItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub course: String,
    #[serde(default)]
    pub emp: String,
    #[serde(default)]
    pub assigned: String,
    pub due: Option<String>,
    pub done: Option<String>,
    pub score: Option<i32>,
    #[serde(default)]
    pub status: String,
    pub course_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct CategoryItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    #[serde(default)]
    pub course_count: i32,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

pub struct LearnExtraStore {
    pub programmes: Vec<ProgrammeItem>,
    pub posh: Vec<PoshItem>,
    pub sectrain: Vec<SecurityTrainingItem>,
    pub categories: Vec<CategoryItem>,
}

static LEARN_STORE: OnceLock<Arc<Mutex<LearnExtraStore>>> = OnceLock::new();

fn get_learn_store() -> &'static Arc<Mutex<LearnExtraStore>> {
    LEARN_STORE.get_or_init(|| {
        Arc::new(Mutex::new(LearnExtraStore {
            programmes: vec![
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
            ],
            posh: vec![
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
            ],
            sectrain: vec![
                SecurityTrainingItem {
                    id: "sec-1".into(),
                    course: "Information Security Essentials".into(),
                    emp: "BC-1001".into(),
                    assigned: "2026-10-01".into(),
                    due: Some("2026-10-29".into()),
                    done: Some("2026-10-20".into()),
                    score: Some(95),
                    status: "Completed".into(),
                    course_id: Some("c3".into()),
                },
                SecurityTrainingItem {
                    id: "sec-2".into(),
                    course: "Phishing & Social Engineering Prevention".into(),
                    emp: "BC-1012".into(),
                    assigned: "2026-10-05".into(),
                    due: Some("2026-10-31".into()),
                    done: None,
                    score: None,
                    status: "In progress".into(),
                    course_id: Some("c3".into()),
                },
            ],
            categories: vec![
                CategoryItem {
                    id: "cat-1".into(),
                    name: "Compliance & Legal".into(),
                    description: Some("Mandatory statutory and regulatory training for all staff.".into()),
                    color: Some("#7C5CFF".into()),
                    course_count: 3,
                    created_at: "2026-01-15T00:00:00Z".into(),
                    updated_at: "2026-01-15T00:00:00Z".into(),
                },
                CategoryItem {
                    id: "cat-2".into(),
                    name: "Engineering & Architecture".into(),
                    description: Some("Rust, React, cloud infrastructure, and distributed systems.".into()),
                    color: Some("#0BA6CC".into()),
                    course_count: 5,
                    created_at: "2026-01-15T00:00:00Z".into(),
                    updated_at: "2026-01-15T00:00:00Z".into(),
                },
                CategoryItem {
                    id: "cat-3".into(),
                    name: "People Ops & Leadership".into(),
                    description: Some("Management skills, performance management, and effective 1-on-1s.".into()),
                    color: Some("#12A58E".into()),
                    course_count: 2,
                    created_at: "2026-01-15T00:00:00Z".into(),
                    updated_at: "2026-01-15T00:00:00Z".into(),
                },
            ],
        }))
    })
}

#[debug_handler]
pub async fn list_programmes() -> Result<Response> {
    let store = get_learn_store().lock().unwrap();
    format::json(store.programmes.clone())
}

#[debug_handler]
pub async fn create_programme(Json(mut p): Json<ProgrammeItem>) -> Result<Response> {
    if p.id.is_empty() {
        p.id = format!("prog-{}", Utc::now().timestamp_millis());
    }
    let mut store = get_learn_store().lock().unwrap();
    store.programmes.push(p.clone());
    format::json(p)
}

#[debug_handler]
pub async fn update_programme(Path(id): Path<String>, Json(mut p): Json<ProgrammeItem>) -> Result<Response> {
    p.id = id.clone();
    let mut store = get_learn_store().lock().unwrap();
    if let Some(pos) = store.programmes.iter().position(|x| x.id == id) {
        store.programmes[pos] = p.clone();
    }
    format::json(p)
}

#[debug_handler]
pub async fn delete_programme(Path(id): Path<String>) -> Result<Response> {
    let mut store = get_learn_store().lock().unwrap();
    store.programmes.retain(|x| x.id != id);
    format::json(json!({ "success": true, "id": id }))
}

#[debug_handler]
pub async fn list_posh() -> Result<Response> {
    let store = get_learn_store().lock().unwrap();
    format::json(store.posh.clone())
}

#[debug_handler]
pub async fn create_posh(Json(mut p): Json<PoshItem>) -> Result<Response> {
    if p.id.is_empty() {
        p.id = format!("posh-{}", Utc::now().timestamp_millis());
    }
    let mut store = get_learn_store().lock().unwrap();
    store.posh.push(p.clone());
    format::json(p)
}

#[debug_handler]
pub async fn update_posh(Path(id): Path<String>, Json(mut p): Json<PoshItem>) -> Result<Response> {
    p.id = id.clone();
    let mut store = get_learn_store().lock().unwrap();
    if let Some(pos) = store.posh.iter().position(|x| x.id == id) {
        store.posh[pos] = p.clone();
    }
    format::json(p)
}

#[debug_handler]
pub async fn delete_posh(Path(id): Path<String>) -> Result<Response> {
    let mut store = get_learn_store().lock().unwrap();
    store.posh.retain(|x| x.id != id);
    format::json(json!({ "success": true, "id": id }))
}

#[debug_handler]
pub async fn list_sectrain() -> Result<Response> {
    let store = get_learn_store().lock().unwrap();
    format::json(store.sectrain.clone())
}

#[debug_handler]
pub async fn create_sectrain(Json(mut s): Json<SecurityTrainingItem>) -> Result<Response> {
    if s.id.is_empty() {
        s.id = format!("sec-{}", Utc::now().timestamp_millis());
    }
    let mut store = get_learn_store().lock().unwrap();
    store.sectrain.push(s.clone());
    format::json(s)
}

#[debug_handler]
pub async fn update_sectrain(Path(id): Path<String>, Json(mut s): Json<SecurityTrainingItem>) -> Result<Response> {
    s.id = id.clone();
    let mut store = get_learn_store().lock().unwrap();
    if let Some(pos) = store.sectrain.iter().position(|x| x.id == id) {
        store.sectrain[pos] = s.clone();
    }
    format::json(s)
}

#[debug_handler]
pub async fn delete_sectrain(Path(id): Path<String>) -> Result<Response> {
    let mut store = get_learn_store().lock().unwrap();
    store.sectrain.retain(|x| x.id != id);
    format::json(json!({ "success": true, "id": id }))
}

#[derive(Deserialize)]
pub struct BulkAssignReq {
    pub course: String,
    pub due: String,
}

#[debug_handler]
pub async fn bulk_assign_sectrain(Json(req): Json<BulkAssignReq>) -> Result<Response> {
    format::json(json!({
        "assigned": 28,
        "skipped": 0,
        "failed": 0,
        "message": format!("Assigned {} due on {} to all active employees", req.course, req.due)
    }))
}

#[debug_handler]
pub async fn list_categories() -> Result<Response> {
    let store = get_learn_store().lock().unwrap();
    format::json(store.categories.clone())
}

#[debug_handler]
pub async fn create_category(Json(mut c): Json<CategoryItem>) -> Result<Response> {
    if c.id.is_empty() {
        c.id = format!("cat-{}", Utc::now().timestamp_millis());
    }
    c.course_count = 0;
    c.created_at = Utc::now().to_rfc3339();
    c.updated_at = Utc::now().to_rfc3339();
    let mut store = get_learn_store().lock().unwrap();
    store.categories.push(c.clone());
    format::json(c)
}

#[debug_handler]
pub async fn update_category(Path(id): Path<String>, Json(mut c): Json<CategoryItem>) -> Result<Response> {
    c.id = id.clone();
    c.updated_at = Utc::now().to_rfc3339();
    let mut store = get_learn_store().lock().unwrap();
    if let Some(pos) = store.categories.iter().position(|x| x.id == id) {
        c.course_count = store.categories[pos].course_count;
        store.categories[pos] = c.clone();
    }
    format::json(c)
}

#[debug_handler]
pub async fn delete_category(Path(id): Path<String>) -> Result<Response> {
    let mut store = get_learn_store().lock().unwrap();
    store.categories.retain(|x| x.id != id);
    format::json(json!({ "success": true, "message": "Category removed" }))
}
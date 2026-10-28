use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::EntityTrait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::models::_entities::employees::Entity as Employees;

#[derive(Clone, Debug)]
pub struct SkillsStore {
    pub custom_skills: Vec<serde_json::Value>,
    pub ratings: HashMap<String, serde_json::Value>,
}

static SKILLS_STORE: OnceLock<Arc<Mutex<SkillsStore>>> = OnceLock::new();

fn get_skills_store() -> &'static Arc<Mutex<SkillsStore>> {
    SKILLS_STORE.get_or_init(|| {
        Arc::new(Mutex::new(SkillsStore {
            custom_skills: Vec::new(),
            ratings: HashMap::new(),
        }))
    })
}

#[debug_handler]
pub async fn completion_tracker(State(ctx): State<AppContext>) -> Result<Response> {
    let emps = Employees::find().all(&ctx.db).await.unwrap_or_default();
    let total = emps.len();

    let records: Vec<serde_json::Value> = emps
        .into_iter()
        .enumerate()
        .map(|(i, e)| {
            let pct = if i % 4 == 0 { 100 } else if i % 3 == 0 { 75 } else if i % 2 == 0 { 50 } else { 25 };
            let status = if pct == 100 { "Compliant" } else if pct >= 50 { "In progress" } else { "Not started" };
            json!({
                "employee_id": e.employee_code,
                "employee_name": e.full_name,
                "department": e.department,
                "assigned_count": 4,
                "completed_count": if pct == 100 { 4 } else if pct >= 50 { 2 } else { 1 },
                "pending_count": if pct == 100 { 0 } else { 2 },
                "overdue_count": 0,
                "progress_pct": pct,
                "status": status,
                "items": [
                    { "id": "t1", "title": "POSH & Workplace Ethics 2026", "training_type": "Course", "progress": pct, "is_assessment": false, "status": status },
                    { "id": "t2", "title": "Information Security Essentials", "training_type": "Security", "progress": pct, "is_assessment": true, "status": status }
                ]
            })
        })
        .collect();

    let fully_compliant = records.iter().filter(|r| r["status"] == "Compliant").count();
    let in_progress = records.iter().filter(|r| r["status"] == "In progress").count();

    format::json(json!({
        "total_employees": total,
        "fully_compliant_count": fully_compliant,
        "in_progress_count": in_progress,
        "overdue_count": 0,
        "not_assigned_count": 0,
        "overall_compliance_pct": 82.5,
        "records": records
    }))
}

#[debug_handler]
pub async fn skills_matrix(State(ctx): State<AppContext>) -> Result<Response> {
    let emps = Employees::find().all(&ctx.db).await.unwrap_or_default();
    let store = get_skills_store();
    let s = store.lock().unwrap();

    let mut skills = vec![
        json!({ "id": "sk-1", "name": "Rust (Loco.rs)", "category": "Backend", "description": "High-performance backend web framework" }),
        json!({ "id": "sk-2", "name": "React 18 & TypeScript", "category": "Frontend", "description": "Modern frontend single page apps" }),
        json!({ "id": "sk-3", "name": "PostgreSQL & SeaORM", "category": "Database", "description": "Relational data modeling and ORM migrations" }),
        json!({ "id": "sk-4", "name": "People Operations & HR Strategy", "category": "HR", "description": "Workforce planning and talent retention" })
    ];

    for cs in &s.custom_skills {
        skills.push(cs.clone());
    }

    let mut categories: Vec<String> = vec!["Backend".into(), "Frontend".into(), "Database".into(), "HR".into()];
    for sk in &skills {
        if let Some(cat) = sk.get("category").and_then(|c| c.as_str()) {
            if !categories.iter().any(|c| c == cat) {
                categories.push(cat.to_string());
            }
        }
    }

    let emp_rows: Vec<serde_json::Value> = emps
        .into_iter()
        .map(|e| {
            let mut map = HashMap::new();
            map.insert("sk-1".to_string(), json!({ "level": "Advanced", "verified": true, "updated_at": "2026-10-20" }));
            map.insert("sk-2".to_string(), json!({ "level": "Expert", "verified": true, "updated_at": "2026-10-22" }));
            map.insert("sk-3".to_string(), json!({ "level": "Intermediate", "verified": true, "updated_at": "2026-10-15" }));
            map.insert("sk-4".to_string(), json!({ "level": "Intermediate", "verified": false, "updated_at": "2026-10-10" }));

            // Overlay any custom skills and runtime ratings
            for cs in &s.custom_skills {
                if let Some(sk_id) = cs.get("id").and_then(|v| v.as_str()) {
                    map.insert(sk_id.to_string(), json!({ "level": "Intermediate", "verified": true, "updated_at": "2026-10-27" }));
                }
            }

            for ((_emp_code, sk_id), rating) in s.ratings.iter().filter_map(|(k, v)| {
                let parts: Vec<&str> = k.splitn(2, ':').collect();
                if parts.len() == 2 && parts[0] == e.employee_code {
                    Some(((parts[0], parts[1]), v))
                } else {
                    None
                }
            }) {
                map.insert(sk_id.to_string(), rating.clone());
            }

            json!({
                "employee_code": e.employee_code,
                "full_name": e.full_name,
                "department": e.department,
                "role": e.job_title,
                "skills": map
            })
        })
        .collect();

    format::json(json!({
        "skills": skills,
        "employees": emp_rows,
        "categories": categories,
        "levels": ["Beginner", "Intermediate", "Advanced", "Expert"]
    }))
}

#[debug_handler]
pub async fn list_skills() -> Result<Response> {
    let store = get_skills_store();
    let s = store.lock().unwrap();

    let mut skills = vec![
        json!({ "id": "sk-1", "name": "Rust (Loco.rs)", "category": "Backend", "description": "High-performance backend web framework" }),
        json!({ "id": "sk-2", "name": "React 18 & TypeScript", "category": "Frontend", "description": "Modern frontend single page apps" }),
        json!({ "id": "sk-3", "name": "PostgreSQL & SeaORM", "category": "Database", "description": "Relational data modeling and ORM migrations" }),
        json!({ "id": "sk-4", "name": "People Operations & HR Strategy", "category": "HR", "description": "Workforce planning and talent retention" })
    ];

    for cs in &s.custom_skills {
        skills.push(cs.clone());
    }

    format::json(skills)
}

#[debug_handler]
pub async fn create_skill(Json(payload): Json<serde_json::Value>) -> Result<Response> {
    let store = get_skills_store();
    let mut s = store.lock().unwrap();
    let id = format!("sk-{}", s.custom_skills.len() + 5);
    let skill = json!({
        "id": id,
        "name": payload.get("name").and_then(|v| v.as_str()).unwrap_or("New Skill"),
        "category": payload.get("category").and_then(|v| v.as_str()).unwrap_or("Core Competency"),
        "description": payload.get("description").and_then(|v| v.as_str()).unwrap_or("")
    });
    s.custom_skills.push(skill.clone());
    format::json(skill)
}

#[debug_handler]
pub async fn rate_skill(Json(payload): Json<serde_json::Value>) -> Result<Response> {
    let store = get_skills_store();
    let mut s = store.lock().unwrap();
    let emp = payload.get("employee_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let skill_id = payload.get("skill_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let rating = json!({
        "level": payload.get("level").and_then(|v| v.as_str()).unwrap_or("Intermediate"),
        "verified": payload.get("verified").and_then(|v| v.as_bool()).unwrap_or(true),
        "updated_at": Utc::now().to_rfc3339()
    });
    if !emp.is_empty() && !skill_id.is_empty() {
        s.ratings.insert(format!("{}:{}", emp, skill_id), rating.clone());
    }
    format::json(rating)
}

#[debug_handler]
pub async fn completion_records(State(ctx): State<AppContext>) -> Result<Response> {
    let emps = Employees::find().all(&ctx.db).await.unwrap_or_default();
    let records: Vec<serde_json::Value> = emps.into_iter().take(10).enumerate().map(|(i, e)| {
        json!({
            "id": format!("rec-{}", i + 1),
            "course_id": "c1",
            "course_title": "POSH & Workplace Ethics 2026",
            "course_type": "Mandatory",
            "category_name": "Compliance & Legal",
            "duration": "45 mins",
            "completed_at": "2026-10-24T10:00:00Z",
            "is_assessment": true,
            "score": 96,
            "record_type": "assessment_verified",
            "employee_name": e.full_name,
            "employee_id": e.employee_code
        })
    }).collect();
    format::json(records)
}

#[debug_handler]
pub async fn skill_recommendations() -> Result<Response> {
    format::json(json!({
        "recommendations": [
            {
                "id": "rec-1",
                "course_id": "c4",
                "title": "Rust Systems Programming Patterns",
                "course_type": "Elective",
                "duration": "120 mins",
                "category": "Engineering",
                "due_date": null,
                "rule_type": "proficiency_growth",
                "reason": "Recommended to level up from Intermediate to Advanced in Backend engineering.",
                "skill_name": "Rust (Loco.rs)"
            },
            {
                "id": "rec-2",
                "course_id": "c1",
                "title": "POSH & Workplace Ethics 2026",
                "course_type": "Mandatory",
                "duration": "45 mins",
                "category": "Compliance & Legal",
                "due_date": "2026-11-15",
                "rule_type": "mandatory_compliance",
                "reason": "Annual statutory compliance requirement for all Brightcone personnel.",
                "skill_name": "Workplace Conduct"
            },
            {
                "id": "rec-3",
                "course_id": "c2",
                "title": "React 18 Architecture & Performance",
                "course_type": "Elective",
                "duration": "90 mins",
                "category": "Frontend",
                "due_date": null,
                "rule_type": "department_skill",
                "reason": "Core frontend competency development for modern single page applications.",
                "skill_name": "React 18 & TypeScript"
            }
        ],
        "employee_department": "Engineering",
        "employee_name": "Karthik Reddy",
        "total_skills_mastered": 5
    }))
}

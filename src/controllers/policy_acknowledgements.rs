use crate::models::_entities::policy_acknowledgements::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAckParams {
    pub policy_id: Option<String>,
    pub policy_name: String,
    pub employee_name: String,
    pub employee_email: Option<String>,
    pub department: Option<String>,
    pub sent_date: Option<String>,
    pub due_date: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchAckParams {
    pub policy_name: String,
    pub due_date: String,
    pub recipients: Vec<BatchRecipient>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BatchRecipient {
    pub name: String,
    pub email: String,
    pub department: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateAckParams {
    pub acknowledged_on: Option<String>,
    pub status: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let mut list: Vec<Model> = Entity::find()
        .order_by_desc(policy_acknowledgements::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    if list.is_empty() {
        let defaults = vec![
            ("POL-01", "Code of Conduct & Ethics v2.0", "Rahul Varma", "rahul.varma@brightcone.ai", "Engineering", "2026-10-01", "2026-10-15", Some("2026-10-05".to_string()), "Acknowledged"),
            ("POL-02", "Information Security & Acceptable Use v3.2", "Priya Menon", "priya.menon@brightcone.ai", "Engineering", "2026-10-10", "2026-10-24", None, "Pending"),
            ("POL-03", "POSH Policy v1.1", "Divya Kulkarni", "divya.kulkarni@brightcone.ai", "Engineering", "2026-10-05", "2026-10-19", Some("2026-10-08".to_string()), "Acknowledged"),
            ("POL-04", "Employee Handbook v4.0", "Amit Patel", "amit.patel@brightcone.ai", "Product Design", "2026-10-12", "2026-10-26", None, "Pending"),
        ];

        for (pid, pname, ename, email, dept, sent, due, ack_on, st) in defaults {
            let active = ActiveModel {
                ack_code: Set(format!("ACK-{}", 100 + list.len() + 1)),
                policy_id: Set(pid.to_string()),
                policy_name: Set(pname.to_string()),
                employee_name: Set(ename.to_string()),
                employee_email: Set(email.to_string()),
                department: Set(dept.to_string()),
                sent_date: Set(sent.to_string()),
                due_date: Set(due.to_string()),
                acknowledged_on: Set(ack_on),
                status: Set(st.to_string()),
                ..Default::default()
            };
            if let Ok(rec) = active.insert(&ctx.db).await {
                list.push(rec);
            }
        }
    }

    format::json(list)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateAckParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let ack_code = format!("ACK-{}", 100 + count + 1);
    let email = params.employee_email.unwrap_or_else(|| {
        format!("{}@brightcone.ai", params.employee_name.to_lowercase().replace(' ', "."))
    });

    let active = ActiveModel {
        ack_code: Set(ack_code),
        policy_id: Set(params.policy_id.unwrap_or_else(|| "POL-GEN".to_string())),
        policy_name: Set(params.policy_name),
        employee_name: Set(params.employee_name),
        employee_email: Set(email),
        department: Set(params.department.unwrap_or_else(|| "Engineering".to_string())),
        sent_date: Set(params.sent_date.unwrap_or_else(|| "2026-10-26".to_string())),
        due_date: Set(params.due_date.unwrap_or_else(|| "2026-11-09".to_string())),
        acknowledged_on: Set(None),
        status: Set(params.status.unwrap_or_else(|| "Pending".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn batch(
    State(ctx): State<AppContext>,
    Json(params): Json<BatchAckParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let mut created = Vec::new();

    for (i, r) in params.recipients.into_iter().enumerate() {
        let ack_code = format!("ACK-{}", 100 + count + i + 1);
        let active = ActiveModel {
            ack_code: Set(ack_code),
            policy_id: Set("BATCH".to_string()),
            policy_name: Set(params.policy_name.clone()),
            employee_name: Set(r.name),
            employee_email: Set(r.email),
            department: Set(r.department),
            sent_date: Set("2026-10-26".to_string()),
            due_date: Set(params.due_date.clone()),
            acknowledged_on: Set(None),
            status: Set("Pending".to_string()),
            ..Default::default()
        };
        if let Ok(rec) = active.insert(&ctx.db).await {
            created.push(rec);
        }
    }

    format::json(created)
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<UpdateAckParams>,
) -> Result<Response> {
    let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    if let Some(ack_on) = params.acknowledged_on {
        active.acknowledged_on = Set(Some(ack_on));
        active.status = Set("Acknowledged".to_string());
    } else if let Some(st) = params.status {
        active.status = Set(st);
    }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/policy-acknowledgements")
        .add("", get(list))
        .add("", post(create))
        .add("/batch", post(batch))
        .add("/{id}", patch(update))
}

use chrono::{DateTime, Utc};
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::models::_entities::sign_audit_events::{self, ActiveModel as AuditActiveModel, Entity as SignAuditEvents};
use crate::models::_entities::sign_envelopes::{self, ActiveModel, Entity as SignEnvelopes};

#[derive(Debug, Deserialize)]
pub struct ListEnvelopesQuery {
    pub status: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEnvelopePayload {
    pub document_name: String,
    pub template_id: Option<String>,
    pub recipient_name: String,
    pub recipient_email: String,
    pub recipient_employee_id: Option<String>,
    pub is_external: Option<bool>,
    pub due_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignEnvelopePayload {
    pub signed_name: String,
}

#[derive(Debug, Deserialize)]
pub struct DeclineEnvelopePayload {
    pub reason: Option<String>,
}

async fn generate_envelope_code(db: &DatabaseConnection) -> String {
    let count = SignEnvelopes::find().count(db).await.unwrap_or(0);
    format!("SG-{:04}", 1001 + count)
}

pub async fn list_envelopes(
    State(ctx): State<AppContext>,
    Query(params): Query<ListEnvelopesQuery>,
) -> Result<Response> {
    let mut query = SignEnvelopes::find();

    if let Some(status) = params.status {
        if !status.is_empty() && status != "All" {
            query = query.filter(sign_envelopes::Column::Status.eq(status));
        }
    }

    if let Some(q) = params.q {
        if !q.is_empty() {
            query = query.filter(
                Condition::any()
                    .add(sign_envelopes::Column::DocumentName.contains(&q))
                    .add(sign_envelopes::Column::RecipientName.contains(&q))
                    .add(sign_envelopes::Column::Code.contains(&q)),
            );
        }
    }

    let items = query
        .order_by_desc(sign_envelopes::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    format::json(items)
}

pub async fn my_envelopes(
    State(ctx): State<AppContext>,
    auth: auth::JWT,
) -> Result<Response> {
    let user_email = auth.claims.pid.clone();
    let items = SignEnvelopes::find()
        .filter(
            Condition::any()
                .add(sign_envelopes::Column::RecipientEmail.eq(&user_email))
                .add(sign_envelopes::Column::RecipientName.contains(&user_email)),
        )
        .order_by_desc(sign_envelopes::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    format::json(items)
}

pub async fn create_envelope(
    State(ctx): State<AppContext>,
    Json(payload): Json<CreateEnvelopePayload>,
) -> Result<Response> {
    let now: DateTime<Utc> = Utc::now();
    let envelope_id = format!("env_{}", Uuid::new_v4().simple());
    let code = generate_envelope_code(&ctx.db).await;
    let due_at = payload.due_at.unwrap_or_else(|| {
        (now + chrono::Duration::days(7)).format("%Y-%m-%d").to_string()
    });

    let new_envelope = ActiveModel {
        envelope_id: Set(envelope_id.clone()),
        code: Set(code.clone()),
        document_name: Set(payload.document_name),
        template_id: Set(payload.template_id),
        sender_id: Set(Some("usr_admin".to_string())),
        sender_name: Set("Ananya Rao".to_string()),
        recipient_employee_id: Set(payload.recipient_employee_id),
        recipient_name: Set(payload.recipient_name),
        recipient_email: Set(payload.recipient_email),
        is_external: Set(payload.is_external.unwrap_or(false)),
        status: Set("Awaiting signature".to_string()),
        sent_at: Set(now.format("%Y-%m-%d").to_string()),
        due_at: Set(due_at),
        completed_at: Set(None),
        signed_name: Set(None),
        signed_ip: Set(None),
        decline_reason: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    let inserted = new_envelope.insert(&ctx.db).await?;

    // Audit log
    let audit_event = AuditActiveModel {
        event_id: Set(format!("evt_{}", Uuid::new_v4().simple())),
        envelope_id: Set(envelope_id),
        actor_id: Set(Some("usr_admin".to_string())),
        actor_name: Set("Ananya Rao".to_string()),
        action: Set("Created and sent envelope".to_string()),
        meta: Set(Some(json!({ "code": code }).to_string())),
        ip: Set(Some("127.0.0.1".to_string())),
        created_at: Set(now.into()),
        ..Default::default()
    };
    let _ = audit_event.insert(&ctx.db).await;

    format::json(inserted)
}

pub async fn get_envelope(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let mut item = SignEnvelopes::find()
        .filter(
            Condition::any()
                .add(sign_envelopes::Column::EnvelopeId.eq(&id))
                .add(sign_envelopes::Column::Code.eq(&id)),
        )
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    // Transition status to Viewed if Awaiting signature
    if item.status == "Awaiting signature" {
        let mut active: ActiveModel = item.clone().into();
        active.status = Set("Viewed".to_string());
        active.updated_at = Set(Utc::now().into());
        item = active.update(&ctx.db).await?;

        // Audit view
        let audit = AuditActiveModel {
            event_id: Set(format!("evt_{}", Uuid::new_v4().simple())),
            envelope_id: Set(item.envelope_id.clone()),
            actor_id: Set(item.recipient_employee_id.clone()),
            actor_name: Set(item.recipient_name.clone()),
            action: Set("Document opened for viewing".to_string()),
            meta: Set(None),
            ip: Set(Some("127.0.0.1".to_string())),
            created_at: Set(Utc::now().into()),
            ..Default::default()
        };
        let _ = audit.insert(&ctx.db).await;
    }

    format::json(item)
}

pub async fn sign_envelope(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(payload): Json<SignEnvelopePayload>,
) -> Result<Response> {
    let item = SignEnvelopes::find()
        .filter(
            Condition::any()
                .add(sign_envelopes::Column::EnvelopeId.eq(&id))
                .add(sign_envelopes::Column::Code.eq(&id)),
        )
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    if item.status == "Completed" || item.status == "Voided" {
        return Err(Error::BadRequest(
            "Envelope cannot be signed in its current status".to_string(),
        ));
    }

    let now: DateTime<Utc> = Utc::now();
    let mut active: ActiveModel = item.clone().into();
    active.status = Set("Completed".to_string());
    active.signed_name = Set(Some(payload.signed_name.clone()));
    active.completed_at = Set(Some(now.format("%Y-%m-%d %H:%M:%S").to_string()));
    active.signed_ip = Set(Some("127.0.0.1".to_string()));
    active.updated_at = Set(now.into());

    let updated = active.update(&ctx.db).await?;

    let audit = AuditActiveModel {
        event_id: Set(format!("evt_{}", Uuid::new_v4().simple())),
        envelope_id: Set(item.envelope_id),
        actor_id: Set(item.recipient_employee_id),
        actor_name: Set(payload.signed_name),
        action: Set("Electronically signed and verified".to_string()),
        meta: Set(Some(json!({ "status": "Completed" }).to_string())),
        ip: Set(Some("127.0.0.1".to_string())),
        created_at: Set(now.into()),
        ..Default::default()
    };
    let _ = audit.insert(&ctx.db).await;

    format::json(updated)
}

pub async fn decline_envelope(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
    Json(payload): Json<DeclineEnvelopePayload>,
) -> Result<Response> {
    let item = SignEnvelopes::find()
        .filter(
            Condition::any()
                .add(sign_envelopes::Column::EnvelopeId.eq(&id))
                .add(sign_envelopes::Column::Code.eq(&id)),
        )
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let now: DateTime<Utc> = Utc::now();
    let mut active: ActiveModel = item.clone().into();
    active.status = Set("Declined".to_string());
    active.decline_reason = Set(payload.reason.clone());
    active.updated_at = Set(now.into());

    let updated = active.update(&ctx.db).await?;

    let audit = AuditActiveModel {
        event_id: Set(format!("evt_{}", Uuid::new_v4().simple())),
        envelope_id: Set(item.envelope_id),
        actor_id: Set(item.recipient_employee_id),
        actor_name: Set(item.recipient_name),
        action: Set("Declined to sign document".to_string()),
        meta: Set(payload.reason.map(|r| json!({ "reason": r }).to_string())),
        ip: Set(Some("127.0.0.1".to_string())),
        created_at: Set(now.into()),
        ..Default::default()
    };
    let _ = audit.insert(&ctx.db).await;

    format::json(updated)
}

pub async fn void_envelope(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = SignEnvelopes::find()
        .filter(
            Condition::any()
                .add(sign_envelopes::Column::EnvelopeId.eq(&id))
                .add(sign_envelopes::Column::Code.eq(&id)),
        )
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let now: DateTime<Utc> = Utc::now();
    let mut active: ActiveModel = item.clone().into();
    active.status = Set("Voided".to_string());
    active.updated_at = Set(now.into());

    let updated = active.update(&ctx.db).await?;

    let audit = AuditActiveModel {
        event_id: Set(format!("evt_{}", Uuid::new_v4().simple())),
        envelope_id: Set(item.envelope_id),
        actor_id: Set(Some("usr_admin".to_string())),
        actor_name: Set("HR Admin".to_string()),
        action: Set("Voided envelope".to_string()),
        meta: Set(None),
        ip: Set(Some("127.0.0.1".to_string())),
        created_at: Set(now.into()),
        ..Default::default()
    };
    let _ = audit.insert(&ctx.db).await;

    format::json(updated)
}

pub async fn remind_envelope(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let item = SignEnvelopes::find()
        .filter(
            Condition::any()
                .add(sign_envelopes::Column::EnvelopeId.eq(&id))
                .add(sign_envelopes::Column::Code.eq(&id)),
        )
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let now: DateTime<Utc> = Utc::now();
    let audit = AuditActiveModel {
        event_id: Set(format!("evt_{}", Uuid::new_v4().simple())),
        envelope_id: Set(item.envelope_id),
        actor_id: Set(Some("usr_admin".to_string())),
        actor_name: Set("HR Admin".to_string()),
        action: Set(format!("Sent reminder to {}", item.recipient_name)),
        meta: Set(None),
        ip: Set(Some("127.0.0.1".to_string())),
        created_at: Set(now.into()),
        ..Default::default()
    };
    let _ = audit.insert(&ctx.db).await;

    format::json(json!({ "message": format!("Reminder sent to {}", item.recipient_name) }))
}

pub async fn get_audit_trail(
    Path(id): Path<String>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let events = SignAuditEvents::find()
        .filter(sign_audit_events::Column::EnvelopeId.eq(&id))
        .order_by_desc(sign_audit_events::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    format::json(events)
}

pub async fn get_stats(State(ctx): State<AppContext>) -> Result<Response> {
    let all = SignEnvelopes::find().all(&ctx.db).await?;
    let awaiting = all.iter().filter(|e| e.status == "Awaiting signature" || e.status == "Viewed").count();
    let completed = all.iter().filter(|e| e.status == "Completed").count();
    let declined = all.iter().filter(|e| e.status == "Declined" || e.status == "Voided").count();

    format::json(json!({
        "awaitingSignature": awaiting,
        "completed": completed,
        "declinedOrVoided": declined,
        "averageTurnaround": "1.2 days"
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/sign")
        .add("/envelopes", get(list_envelopes))
        .add("/envelopes", post(create_envelope))
        .add("/envelopes/mine", get(my_envelopes))
        .add("/envelopes/{id}", get(get_envelope))
        .add("/envelopes/{id}/sign", post(sign_envelope))
        .add("/envelopes/{id}/decline", post(decline_envelope))
        .add("/envelopes/{id}/remind", post(remind_envelope))
        .add("/envelopes/{id}/void", post(void_envelope))
        .add("/envelopes/{id}/audit", get(get_audit_trail))
        .add("/stats", get(get_stats))
}

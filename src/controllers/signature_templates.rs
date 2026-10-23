use chrono::Local;
use loco_rs::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::models::_entities::signature_templates::{self, ActiveModel, Entity, Model};

fn today_str() -> String {
    Local::now().format("%b %d, %Y").to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureTemplateView {
    pub id: String,
    pub template_id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub required_fields: i32,
    pub last_updated: String,
    pub usage_count: i32,
    pub status: String,
    pub body_template: String,
}

impl From<Model> for SignatureTemplateView {
    fn from(m: Model) -> Self {
        Self {
            id: m.template_id.clone(),
            template_id: m.template_id,
            title: m.title,
            category: m.category,
            description: m.description,
            required_fields: m.required_fields,
            last_updated: m.last_updated,
            usage_count: m.usage_count,
            status: m.status,
            body_template: m.body_template,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TemplateQueryParams {
    pub category: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateParams {
    pub title: String,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "required_fields")]
    pub required_fields: Option<i32>,
    #[serde(alias = "body_template")]
    pub body_template: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTemplateParams {
    pub title: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "required_fields")]
    pub required_fields: Option<i32>,
    #[serde(alias = "body_template")]
    pub body_template: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadTemplateParams {
    pub title: String,
    pub category: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "required_fields")]
    pub required_fields: Option<i32>,
    #[serde(alias = "body_template")]
    pub body_template: Option<String>,
}

pub async fn seed_defaults(db: &DatabaseConnection) -> Result<()> {
    let defaults = [
        ("tpl-1", "Policy acknowledgement", "Policies", "Standard 2-field company policy compliance and sign-off form.", 2, "Oct 14, 2026", 112, "I hereby confirm that I have read, understood, and agreed to adhere to the company policy outlined herein."),
        ("tpl-2", "Appraisal letter", "Agreements", "Formal increment, performance rating, and revised CTC notice.", 3, "Sep 30, 2026", 46, "Brightcone is pleased to announce your promotion and revised salary terms for the upcoming review cycle."),
        ("tpl-3", "NDA", "NDAs", "Standard 4-field unilateral or bilateral confidentiality agreement.", 4, "Sep 25, 2026", 31, "The Disclosing Party agrees to share proprietary information with the Receiving Party under strict confidentiality terms."),
        ("tpl-4", "Offer letter", "Offers", "6-field standard full-time employment offer with probation and benefits.", 6, "Oct 02, 2026", 24, "We are delighted to extend an offer of employment with Brightcone Information Systems for your designated role."),
        ("tpl-5", "Employment agreement", "Agreements", "9-field comprehensive permanent employment terms and covenants.", 9, "Aug 14, 2026", 19, "This Employment Agreement governs the terms, covenants, IP assignments, and confidentiality requirements between Brightcone and the Employee."),
    ];

    for (tid, title, cat, desc, fields, date, usage, body) in defaults {
        let exists = Entity::find()
            .filter(signature_templates::Column::TemplateId.eq(tid))
            .one(db)
            .await?;
        if exists.is_none() {
            let _ = ActiveModel {
                template_id: Set(tid.to_string()),
                title: Set(title.to_string()),
                category: Set(cat.to_string()),
                description: Set(desc.to_string()),
                required_fields: Set(fields),
                last_updated: Set(date.to_string()),
                usage_count: Set(usage),
                status: Set("active".to_string()),
                body_template: Set(body.to_string()),
                ..Default::default()
            }.insert(db).await;
        }
    }
    Ok(())
}

#[debug_handler]
pub async fn list_templates(
    State(ctx): State<AppContext>,
    Query(params): Query<TemplateQueryParams>,
) -> Result<Response> {
    if Entity::find().count(&ctx.db).await? == 0 {
        let _ = seed_defaults(&ctx.db).await;
    }

    let mut query = Entity::find();
    if let Some(cat) = params.category {
        if !cat.trim().is_empty() && cat.to_lowercase() != "all" {
            query = query.filter(signature_templates::Column::Category.eq(cat.trim()));
        }
    }

    if let Some(s) = params.search {
        let term = s.trim().to_lowercase();
        if !term.is_empty() {
            let cond = Condition::any()
                .add(signature_templates::Column::Title.contains(&term))
                .add(signature_templates::Column::Description.contains(&term))
                .add(signature_templates::Column::Category.contains(&term))
                .add(signature_templates::Column::TemplateId.contains(&term));
            query = query.filter(cond);
        }
    }

    let list: Vec<Model> = query
        .order_by_desc(signature_templates::Column::UsageCount)
        .all(&ctx.db)
        .await?;

    let views: Vec<SignatureTemplateView> = list.into_iter().map(Into::into).collect();
    format::json(views)
}

#[debug_handler]
pub async fn get_template(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    match Entity::find().filter(signature_templates::Column::TemplateId.eq(&id)).one(&ctx.db).await? {
        Some(m) => format::json(SignatureTemplateView::from(m)),
        None => not_found(),
    }
}

#[debug_handler]
pub async fn create_template(
    State(ctx): State<AppContext>,
    Json(params): Json<CreateTemplateParams>,
) -> Result<Response> {
    let tid = format!("tpl-{}", &Uuid::new_v4().simple().to_string()[..6]);
    let active = ActiveModel {
        template_id: Set(tid),
        title: Set(params.title.trim().to_string()),
        category: Set(params.category.unwrap_or_else(|| "Policies".to_string())),
        description: Set(params.description.unwrap_or_else(|| "Custom signature template created in portal.".to_string())),
        required_fields: Set(params.required_fields.unwrap_or(2)),
        last_updated: Set(today_str()),
        usage_count: Set(0),
        status: Set(params.status.unwrap_or_else(|| "active".to_string())),
        body_template: Set(params.body_template.unwrap_or_else(|| "I agree to the terms of this document.".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(SignatureTemplateView::from(record))
}

#[debug_handler]
pub async fn update_template(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<UpdateTemplateParams>,
) -> Result<Response> {
    let model = match Entity::find().filter(signature_templates::Column::TemplateId.eq(&id)).one(&ctx.db).await? {
        Some(m) => m,
        None => return not_found(),
    };

    let mut active: ActiveModel = model.into();
    if let Some(title) = params.title { active.title = Set(title.trim().to_string()); }
    if let Some(cat) = params.category { active.category = Set(cat); }
    if let Some(desc) = params.description { active.description = Set(desc); }
    if let Some(fields) = params.required_fields { active.required_fields = Set(fields); }
    if let Some(body) = params.body_template { active.body_template = Set(body); }
    if let Some(status) = params.status { active.status = Set(status); }
    active.last_updated = Set(today_str());

    let updated = active.update(&ctx.db).await?;
    format::json(SignatureTemplateView::from(updated))
}

#[debug_handler]
pub async fn delete_template(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    if let Some(model) = Entity::find().filter(signature_templates::Column::TemplateId.eq(&id)).one(&ctx.db).await? {
        let active: ActiveModel = model.into();
        active.delete(&ctx.db).await?;
    }
    format::empty_json()
}

#[debug_handler]
pub async fn use_template(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
) -> Result<Response> {
    let model = match Entity::find().filter(signature_templates::Column::TemplateId.eq(&id)).one(&ctx.db).await? {
        Some(m) => m,
        None => return not_found(),
    };

    let new_usage = model.usage_count + 1;
    let tid = model.template_id.clone();
    let title = model.title.clone();
    let cat = model.category.clone();
    let fields = model.required_fields;
    let body = model.body_template.clone();

    let mut active: ActiveModel = model.into();
    active.usage_count = Set(new_usage);
    let _ = active.update(&ctx.db).await?;

    let env_code = format!("ENV-{}", &Uuid::new_v4().simple().to_string()[..8].to_uppercase());
    format::json(json!({
        "success": true,
        "templateId": tid,
        "title": title,
        "category": cat,
        "usageCount": new_usage,
        "requiredFields": fields,
        "bodyTemplate": body,
        "draftEnvelopeId": env_code
    }))
}

#[debug_handler]
pub async fn upload_template(
    State(ctx): State<AppContext>,
    Json(params): Json<UploadTemplateParams>,
) -> Result<Response> {
    let tid = format!("tpl-up-{}", &Uuid::new_v4().simple().to_string()[..6]);
    let desc = params.description.unwrap_or_else(|| {
        format!(
            "Uploaded template: {} ({})",
            params.file_name.as_deref().unwrap_or("document.pdf"),
            params.file_size.as_deref().unwrap_or("PDF document")
        )
    });

    let active = ActiveModel {
        template_id: Set(tid),
        title: Set(params.title.trim().to_string()),
        category: Set(params.category.unwrap_or_else(|| "Policies".to_string())),
        description: Set(desc),
        required_fields: Set(params.required_fields.unwrap_or(2)),
        last_updated: Set(today_str()),
        usage_count: Set(0),
        status: Set("active".to_string()),
        body_template: Set(params.body_template.unwrap_or_else(|| "Uploaded PDF template placeholder.".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(SignatureTemplateView::from(record))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/signature-templates")
        .add("", get(list_templates))
        .add("", post(create_template))
        .add("/upload", post(upload_template))
        .add("/{id}", get(get_template))
        .add("/{id}", patch(update_template))
        .add("/{id}", delete(delete_template))
        .add("/{id}/use", post(use_template))
}

pub fn v1_routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/signature-templates")
        .add("", get(list_templates))
        .add("", post(create_template))
        .add("/upload", post(upload_template))
        .add("/{id}", get(get_template))
        .add("/{id}", patch(update_template))
        .add("/{id}", delete(delete_template))
        .add("/{id}/use", post(use_template))
}

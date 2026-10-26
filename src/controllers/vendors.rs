use crate::models::_entities::vendors::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateVendorParams {
    pub name: String,
    pub category: String,
    pub contact_person: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub contract_expiry: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateVendorParams {
    pub name: Option<String>,
    pub category: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub contract_expiry: Option<String>,
    pub status: Option<String>,
}

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Response> {
    let mut list: Vec<Model> = Entity::find()
        .order_by_desc(vendors::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    if list.is_empty() {
        let defaults = vec![
            ("Dell Technologies India", "IT Hardware", "Rajesh Khanna", "rkhanna@dell.com", "+91 98450 11223", "2027-03-31", "Active"),
            ("Amazon Web Services (AWS)", "SaaS & Cloud", "Sneha Rao", "sneha-aws@amazon.com", "+91 99887 65432", "2028-12-31", "Active"),
            ("SIS Security & Facility Services", "Facilities & Security", "Mahesh Verma", "mahesh.v@sisindia.com", "+91 98765 43210", "2026-11-30", "Under Renewal"),
            ("Blue Tokai Coffee & Pantry", "Catering & Supplies", "Arjun Nair", "enterprise@bluetokaicoffee.com", "+91 98111 22334", "2027-08-15", "Active"),
        ];

        for (vname, cat, cp, mail, ph, exp, st) in defaults {
            let active = ActiveModel {
                vendor_code: Set(format!("VEN-0{}", list.len() + 1)),
                name: Set(vname.to_string()),
                category: Set(cat.to_string()),
                contact_person: Set(cp.to_string()),
                email: Set(mail.to_string()),
                phone: Set(ph.to_string()),
                contract_expiry: Set(exp.to_string()),
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
    Json(params): Json<CreateVendorParams>,
) -> Result<Response> {
    let count = Entity::find().all(&ctx.db).await?.len();
    let vendor_code = format!("VEN-0{}", count + 1);

    let active = ActiveModel {
        vendor_code: Set(vendor_code),
        name: Set(params.name),
        category: Set(params.category),
        contact_person: Set(params.contact_person),
        email: Set(params.email.unwrap_or_else(|| "vendor@brightcone.ai".to_string())),
        phone: Set(params.phone.unwrap_or_else(|| "+91 98000 00000".to_string())),
        contract_expiry: Set(params.contract_expiry.unwrap_or_else(|| "2027-12-31".to_string())),
        status: Set(params.status.unwrap_or_else(|| "Active".to_string())),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<UpdateVendorParams>,
) -> Result<Response> {
    let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();
    if let Some(n) = params.name { active.name = Set(n); }
    if let Some(c) = params.category { active.category = Set(c); }
    if let Some(cp) = params.contact_person { active.contact_person = Set(cp); }
    if let Some(e) = params.email { active.email = Set(e); }
    if let Some(p) = params.phone { active.phone = Set(p); }
    if let Some(exp) = params.contract_expiry { active.contract_expiry = Set(exp); }
    if let Some(st) = params.status { active.status = Set(st); }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn remove(State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Response> {
    if let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? {
        let active: ActiveModel = existing.into();
        active.delete(&ctx.db).await?;
    }
    format::empty_json()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/vendors")
        .add("", get(list))
        .add("", post(create))
        .add("/{id}", patch(update))
        .add("/{id}", delete(remove))
}

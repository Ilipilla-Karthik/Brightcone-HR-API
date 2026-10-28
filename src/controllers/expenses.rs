use chrono::Utc;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpenseClaim {
    pub id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub category: String,
    pub amount: f64,
    pub date: String,
    pub description: String,
    pub receipt_url: Option<String>,
    pub status: String,
    pub paid_at: Option<String>,
    pub rejection_reason: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateExpenseClaimDto {
    pub employee_id: Option<String>,
    pub employee_name: Option<String>,
    pub category: String,
    pub amount: f64,
    pub date: String,
    pub description: String,
    pub receipt_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RejectExpenseClaimDto {
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OfficeExpense {
    pub id: String,
    pub vendor_name: String,
    pub category: String,
    pub invoice_no: String,
    pub invoice_date: String,
    pub base_amount: f64,
    pub gst_rate: f64,
    pub gst_amount: f64,
    pub total_amount: f64,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CreateOfficeExpenseDto {
    pub vendor_name: String,
    pub category: String,
    pub invoice_no: String,
    pub invoice_date: String,
    pub base_amount: f64,
    pub gst_rate: f64,
    pub status: Option<String>,
}

pub struct ExpensesStore {
    pub claims: Vec<ExpenseClaim>,
    pub office_expenses: Vec<OfficeExpense>,
}

static EXPENSES_STORE: OnceLock<Arc<Mutex<ExpensesStore>>> = OnceLock::new();

fn get_store() -> &'static Arc<Mutex<ExpensesStore>> {
    EXPENSES_STORE.get_or_init(|| {
        Arc::new(Mutex::new(ExpensesStore {
            claims: Vec::new(),
            office_expenses: Vec::new(),
        }))
    })
}

#[debug_handler]
pub async fn list_claims() -> Result<Response> {
    let store = get_store().lock().unwrap();
    let mut all = store.claims.clone();
    for oe in &store.office_expenses {
        all.push(ExpenseClaim {
            id: oe.id.clone(),
            employee_id: "OFFICE".to_string(),
            employee_name: format!("Vendor: {}", oe.vendor_name),
            category: oe.category.clone(),
            amount: oe.total_amount,
            date: oe.invoice_date.clone(),
            description: format!("Invoice #{}: {} (GST: {}%)", oe.invoice_no, oe.category, oe.gst_rate),
            receipt_url: None,
            status: oe.status.clone(),
            paid_at: None,
            rejection_reason: None,
            created_at: Utc::now().to_rfc3339(),
        });
    }
    format::json(all)
}

#[debug_handler]
pub async fn create_claim(Json(dto): Json<CreateExpenseClaimDto>) -> Result<Response> {
    let mut store = get_store().lock().unwrap();
    let new_claim = ExpenseClaim {
        id: format!("E-{}", &Uuid::new_v4().to_string()[..6].to_uppercase()),
        employee_id: dto.employee_id.unwrap_or_else(|| "BC-1001".to_string()),
        employee_name: dto.employee_name.unwrap_or_else(|| "Ananya Rao".to_string()),
        category: dto.category,
        amount: dto.amount,
        date: dto.date,
        description: dto.description,
        receipt_url: dto.receipt_url,
        status: "Pending".to_string(),
        paid_at: None,
        rejection_reason: None,
        created_at: Utc::now().to_rfc3339(),
    };
    store.claims.insert(0, new_claim.clone());
    format::json(new_claim)
}

#[debug_handler]
pub async fn approve_claim(Path(id): Path<String>) -> Result<Response> {
    let mut store = get_store().lock().unwrap();
    if let Some(claim) = store.claims.iter_mut().find(|c| c.id == id) {
        claim.status = "Approved".to_string();
        return format::json(claim.clone());
    }
    if let Some(oe) = store.office_expenses.iter_mut().find(|oe| oe.id == id) {
        oe.status = "Approved".to_string();
        return format::json(oe.clone());
    }
    not_found()
}

#[debug_handler]
pub async fn reject_claim(
    Path(id): Path<String>,
    Json(dto): Json<RejectExpenseClaimDto>,
) -> Result<Response> {
    let mut store = get_store().lock().unwrap();
    if let Some(claim) = store.claims.iter_mut().find(|c| c.id == id) {
        claim.status = "Rejected".to_string();
        claim.rejection_reason = Some(dto.reason);
        return format::json(claim.clone());
    }
    if let Some(oe) = store.office_expenses.iter_mut().find(|oe| oe.id == id) {
        oe.status = "Rejected".to_string();
        return format::json(oe.clone());
    }
    not_found()
}

#[debug_handler]
pub async fn pay_claim(Path(id): Path<String>) -> Result<Response> {
    let mut store = get_store().lock().unwrap();
    if let Some(claim) = store.claims.iter_mut().find(|c| c.id == id) {
        claim.status = "Paid".to_string();
        claim.paid_at = Some(Utc::now().to_rfc3339());
        return format::json(claim.clone());
    }
    if let Some(oe) = store.office_expenses.iter_mut().find(|oe| oe.id == id) {
        oe.status = "Paid".to_string();
        return format::json(oe.clone());
    }
    not_found()
}

#[debug_handler]
pub async fn list_office_expenses() -> Result<Response> {
    let store = get_store().lock().unwrap();
    format::json(store.office_expenses.clone())
}

#[debug_handler]
pub async fn create_office_expense(Json(dto): Json<CreateOfficeExpenseDto>) -> Result<Response> {
    let mut store = get_store().lock().unwrap();
    let gst_amount = (dto.base_amount * dto.gst_rate / 100.0).round();
    let total_amount = dto.base_amount + gst_amount;
    let new_expense = OfficeExpense {
        id: format!("OE-{}", &Uuid::new_v4().to_string()[..5].to_uppercase()),
        vendor_name: dto.vendor_name,
        category: dto.category,
        invoice_no: dto.invoice_no,
        invoice_date: dto.invoice_date,
        base_amount: dto.base_amount,
        gst_rate: dto.gst_rate,
        gst_amount,
        total_amount,
        status: dto.status.unwrap_or_else(|| "Pending".to_string()),
    };
    store.office_expenses.insert(0, new_expense.clone());
    format::json(new_expense)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/expenses")
        .add("", get(list_claims))
        .add("", post(create_claim))
        .add("/{id}/approve", post(approve_claim))
        .add("/{id}/reject", post(reject_claim))
        .add("/{id}/pay", post(pay_claim))
}

pub fn office_routes() -> Routes {
    Routes::new()
        .prefix("/api/office-expenses")
        .add("", get(list_office_expenses))
        .add("", post(create_office_expense))
}

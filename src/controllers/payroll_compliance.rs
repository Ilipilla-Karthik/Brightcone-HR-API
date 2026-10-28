use chrono::Utc;
use loco_rs::prelude::*;
use std::sync::{Arc, Mutex, OnceLock};

pub struct ComplianceStore {
    pub statutory_compliance: Vec<serde_json::Value>,
    pub pf_esi_compliance: Vec<serde_json::Value>,
    pub statutory_filings: Vec<serde_json::Value>,
    pub benefits: Vec<serde_json::Value>,
}

static COMPLIANCE_STORE: OnceLock<Arc<Mutex<ComplianceStore>>> = OnceLock::new();

fn get_store() -> &'static Arc<Mutex<ComplianceStore>> {
    COMPLIANCE_STORE.get_or_init(|| {
        let statutory_compliance = vec![
            serde_json::json!({ "id": "STAT-101", "item": "Form 24Q Q2 TDS Filing", "period": "Q2 FY26", "authority": "Income Tax Dept", "dueDate": "2026-10-31", "owner": "Meera Joshi", "filedOn": "2026-10-25", "status": "Completed", "ackNo": "ACK-IT-9821038", "isArchived": false }),
            serde_json::json!({ "id": "STAT-102", "item": "PF Monthly ECR Remittance", "period": "Sep 2026", "authority": "EPFO", "dueDate": "2026-10-15", "owner": "Meera Joshi", "filedOn": "2026-10-14", "status": "Completed", "ackNo": "TRRN-881293019", "isArchived": false }),
            serde_json::json!({ "id": "STAT-103", "item": "ESIC Monthly Contribution", "period": "Sep 2026", "authority": "ESIC", "dueDate": "2026-10-15", "owner": "Meera Joshi", "filedOn": "2026-10-14", "status": "Completed", "ackNo": "ESIC-CH-481920", "isArchived": false }),
            serde_json::json!({ "id": "STAT-104", "item": "Professional Tax Remittance", "period": "Sep 2026", "authority": "Telangana State Tax", "dueDate": "2026-10-10", "owner": "Meera Joshi", "filedOn": "2026-10-09", "status": "Completed", "ackNo": "PT-TS-2026-09", "isArchived": false }),
            serde_json::json!({ "id": "STAT-105", "item": "Labour Welfare Fund (Half-Yearly)", "period": "H1 FY26", "authority": "Labour Commissioner", "dueDate": "2026-11-15", "owner": "Ananya Rao", "filedOn": null, "status": "Pending", "ackNo": null, "isArchived": false })
        ];

        let pf_esi_compliance = vec![
            serde_json::json!({ "id": "PF-2026-09", "scheme": "EPF & EPS", "wageMonth": "September 2026", "employeeCount": 14, "totalAmount": 168000, "dueDate": "2026-10-15", "paidOn": "2026-10-14", "challanNo": "TRRN-102948201948", "status": "Paid", "isArchived": false }),
            serde_json::json!({ "id": "ESI-2026-09", "scheme": "ESIC", "wageMonth": "September 2026", "employeeCount": 14, "totalAmount": 42500, "dueDate": "2026-10-15", "paidOn": "2026-10-14", "challanNo": "CP-ESIC-99201920", "status": "Paid", "isArchived": false }),
            serde_json::json!({ "id": "PF-2026-10", "scheme": "EPF & EPS", "wageMonth": "October 2026", "employeeCount": 14, "totalAmount": 168000, "dueDate": "2026-11-15", "paidOn": null, "challanNo": null, "status": "Pending", "isArchived": false }),
            serde_json::json!({ "id": "ESI-2026-10", "scheme": "ESIC", "wageMonth": "October 2026", "employeeCount": 14, "totalAmount": 42500, "dueDate": "2026-11-15", "paidOn": null, "challanNo": null, "status": "Pending", "isArchived": false })
        ];

        let statutory_filings = vec![
            serde_json::json!({ "id": "FIL-01", "name": "Monthly PF ECR Electronic Challan Return", "period": "Sep 2026", "category": "PF", "dueDate": "2026-10-15", "totalEmployees": 14, "totalAmount": "₹1,68,000", "challanRef": "TRRN-102948201948", "status": "Filed" }),
            serde_json::json!({ "id": "FIL-02", "name": "Monthly ESIC Contribution Challan", "period": "Sep 2026", "category": "ESI", "dueDate": "2026-10-15", "totalEmployees": 14, "totalAmount": "₹42,500", "challanRef": "ESIC-CH-481920", "status": "Filed" }),
            serde_json::json!({ "id": "FIL-03", "name": "Form 24Q Q2 Quarterly Salary TDS Return", "period": "Q2 FY26", "category": "TDS", "dueDate": "2026-10-31", "totalEmployees": 14, "totalAmount": "₹3,45,200", "challanRef": "ACK-IT-9821038", "status": "Filed" }),
            serde_json::json!({ "id": "FIL-04", "name": "State Professional Tax Monthly Return", "period": "Sep 2026", "category": "PT", "dueDate": "2026-10-10", "totalEmployees": 14, "totalAmount": "₹2,800", "challanRef": "PT-TS-2026-09", "status": "Filed" }),
            serde_json::json!({ "id": "FIL-05", "name": "Monthly PF ECR Electronic Challan Return", "period": "Oct 2026", "category": "PF", "dueDate": "2026-11-15", "totalEmployees": 14, "totalAmount": "₹1,68,000", "challanRef": null, "status": "Ready to File" })
        ];

        let benefits = vec![
            serde_json::json!({ "id": "BP-01", "name": "Group health insurance", "provider": "Star Health", "coverAmount": "₹5,00,000 family floater", "cost": "Company paid" }),
            serde_json::json!({ "id": "BP-02", "name": "Group term life", "provider": "HDFC Life", "coverAmount": "3x annual CTC", "cost": "Company paid" }),
            serde_json::json!({ "id": "BP-03", "name": "Group personal accident", "provider": "ICICI Lombard", "coverAmount": "₹20,00,000", "cost": "Company paid" })
        ];

        Arc::new(Mutex::new(ComplianceStore {
            statutory_compliance,
            pf_esi_compliance,
            statutory_filings,
            benefits,
        }))
    })
}

#[debug_handler]
pub async fn list_statutory_compliance() -> Result<Response> {
    let s = get_store().lock().unwrap();
    format::json(s.statutory_compliance.clone())
}

#[debug_handler]
pub async fn create_statutory_compliance(Json(mut payload): Json<serde_json::Value>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    if payload.get("id").is_none() {
        let id = format!("STAT-{}", Utc::now().timestamp_subsec_millis());
        payload["id"] = serde_json::Value::String(id);
    }
    s.statutory_compliance.insert(0, payload.clone());
    format::json(payload)
}

#[debug_handler]
pub async fn update_statutory_compliance(Path(id): Path<String>, Json(payload): Json<serde_json::Value>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    for item in s.statutory_compliance.iter_mut() {
        if item.get("id").and_then(|v| v.as_str()) == Some(&id) {
            if let Some(obj) = payload.as_object() {
                for (k, v) in obj {
                    item[k] = v.clone();
                }
            }
            return format::json(item.clone());
        }
    }
    format::json(payload)
}

#[debug_handler]
pub async fn list_pf_esi() -> Result<Response> {
    let s = get_store().lock().unwrap();
    format::json(s.pf_esi_compliance.clone())
}

#[debug_handler]
pub async fn create_pf_esi(Json(mut payload): Json<serde_json::Value>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    if payload.get("id").is_none() {
        let scheme = payload.get("scheme").and_then(|v| v.as_str()).unwrap_or("PF");
        let id = format!("{}-{}", scheme, Utc::now().timestamp_subsec_millis());
        payload["id"] = serde_json::Value::String(id);
    }
    s.pf_esi_compliance.insert(0, payload.clone());
    format::json(payload)
}

#[debug_handler]
pub async fn update_pf_esi(Path(id): Path<String>, Json(payload): Json<serde_json::Value>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    for item in s.pf_esi_compliance.iter_mut() {
        if item.get("id").and_then(|v| v.as_str()) == Some(&id) {
            if let Some(obj) = payload.as_object() {
                for (k, v) in obj {
                    item[k] = v.clone();
                }
            }
            return format::json(item.clone());
        }
    }
    format::json(payload)
}

#[debug_handler]
pub async fn list_statutory_filings() -> Result<Response> {
    let s = get_store().lock().unwrap();
    format::json(s.statutory_filings.clone())
}

#[debug_handler]
pub async fn create_statutory_filings(Json(mut payload): Json<serde_json::Value>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    if payload.get("id").is_none() {
        let id = format!("FIL-{}", Utc::now().timestamp_subsec_millis());
        payload["id"] = serde_json::Value::String(id);
    }
    s.statutory_filings.insert(0, payload.clone());
    format::json(payload)
}

#[debug_handler]
pub async fn update_statutory_filings(Path(id): Path<String>, Json(payload): Json<serde_json::Value>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    for item in s.statutory_filings.iter_mut() {
        if item.get("id").and_then(|v| v.as_str()) == Some(&id) {
            if let Some(obj) = payload.as_object() {
                for (k, v) in obj {
                    item[k] = v.clone();
                }
            }
            return format::json(item.clone());
        }
    }
    format::json(payload)
}

#[debug_handler]
pub async fn list_benefits() -> Result<Response> {
    let s = get_store().lock().unwrap();
    format::json(s.benefits.clone())
}

#[debug_handler]
pub async fn create_benefit(Json(mut payload): Json<serde_json::Value>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    if payload.get("id").is_none() {
        let id = format!("BP-{}", Utc::now().timestamp_subsec_millis());
        payload["id"] = serde_json::Value::String(id);
    }
    s.benefits.push(payload.clone());
    format::json(payload)
}

#[debug_handler]
pub async fn delete_benefit(Path(id): Path<String>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    s.benefits.retain(|b| b.get("id").and_then(|v| v.as_str()) != Some(&id));
    format::json(serde_json::json!({ "success": true, "id": id }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/payroll")
        .add("/statutory-compliance", get(list_statutory_compliance))
        .add("/statutory-compliance", post(create_statutory_compliance))
        .add("/statutory-compliance/{id}", put(update_statutory_compliance))
        .add("/pf-esi", get(list_pf_esi))
        .add("/pf-esi", post(create_pf_esi))
        .add("/pf-esi/{id}", put(update_pf_esi))
        .add("/statutory-filings", get(list_statutory_filings))
        .add("/statutory-filings", post(create_statutory_filings))
        .add("/statutory-filings/{id}", put(update_statutory_filings))
        .add("/benefits", get(list_benefits))
        .add("/benefits", post(create_benefit))
        .add("/benefits/{id}", delete(delete_benefit))
}

pub fn v1_routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/payroll")
        .add("/statutory-compliance", get(list_statutory_compliance))
        .add("/statutory-compliance", post(create_statutory_compliance))
        .add("/statutory-compliance/{id}", put(update_statutory_compliance))
        .add("/pf-esi", get(list_pf_esi))
        .add("/pf-esi", post(create_pf_esi))
        .add("/pf-esi/{id}", put(update_pf_esi))
        .add("/statutory-filings", get(list_statutory_filings))
        .add("/statutory-filings", post(create_statutory_filings))
        .add("/statutory-filings/{id}", put(update_statutory_filings))
        .add("/benefits", get(list_benefits))
        .add("/benefits", post(create_benefit))
        .add("/benefits/{id}", delete(delete_benefit))
}

use chrono::Utc;
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};

use crate::modules::salary::calculator::compute_salary_in_paise;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PayrollRun {
    pub id: String,
    pub month: String,
    pub status: String,
    pub current_step: usize,
    pub total_employees: usize,
    pub total_gross: f64,
    pub total_net: f64,
    pub total_deductions: f64,
    pub is_locked: bool,
    pub is_supplementary: bool,
    pub supplementary_notes: Option<String>,
    pub disbursed_at: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AttendanceSyncItem {
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub total_days: usize,
    pub days_worked: usize,
    pub paid_leaves: usize,
    pub lop_days: usize,
    pub payable_days: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VarianceItem {
    pub employee_id: String,
    pub employee_name: String,
    pub department: String,
    pub previous_net: f64,
    pub current_net: f64,
    pub variance_amount: f64,
    pub variance_pct: f64,
    pub flagged: bool,
    pub reason: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RegisterItem {
    #[serde(rename = "empId")]
    pub emp_id: String,
    pub name: String,
    pub dept: String,
    pub gross: f64,
    pub pf: f64,
    pub pt: f64,
    pub tds: f64,
    pub net: f64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmployeeSalaryRecord {
    pub id: String,
    pub name: String,
    pub email: String,
    pub dept: String,
    pub gross: f64,
}

pub struct PayrollStore {
    pub active_run: PayrollRun,
    pub past_runs: Vec<PayrollRun>,
    pub employees: Vec<EmployeeSalaryRecord>,
    pub statutory_compliance: Vec<serde_json::Value>,
    pub pf_esi_compliance: Vec<serde_json::Value>,
    pub statutory_filings: Vec<serde_json::Value>,
    pub benefits: Vec<serde_json::Value>,
}

static PAYROLL_STORE: OnceLock<Arc<Mutex<PayrollStore>>> = OnceLock::new();

fn get_store() -> &'static Arc<Mutex<PayrollStore>> {
    PAYROLL_STORE.get_or_init(|| {
    let employees = vec![
        EmployeeSalaryRecord { id: "BC-1001".into(), name: "Ananya Rao".into(), email: "ananya.rao@brightcone.ai".into(), dept: "People Ops".into(), gross: 145000.0 },
        EmployeeSalaryRecord { id: "BC-1002".into(), name: "Naresh Vemparala".into(), email: "naresh.vemparala@brightcone.ai".into(), dept: "Operations".into(), gross: 180000.0 },
        EmployeeSalaryRecord { id: "BC-1003".into(), name: "Karthik Reddy".into(), email: "karthik.reddy@brightcone.ai".into(), dept: "Engineering".into(), gross: 240000.0 },
        EmployeeSalaryRecord { id: "BC-1004".into(), name: "Priya Menon".into(), email: "priya.menon@brightcone.ai".into(), dept: "Product".into(), gross: 170000.0 },
        EmployeeSalaryRecord { id: "BC-1005".into(), name: "Meera Joshi".into(), email: "meera.joshi@brightcone.ai".into(), dept: "Finance".into(), gross: 160000.0 },
        EmployeeSalaryRecord { id: "BC-1006".into(), name: "Rahul Varma".into(), email: "rahul.varma@brightcone.ai".into(), dept: "Engineering".into(), gross: 195000.0 },
        EmployeeSalaryRecord { id: "BC-1007".into(), name: "Sneha Iyer".into(), email: "sneha.iyer@brightcone.ai".into(), dept: "Design".into(), gross: 85000.0 },
        EmployeeSalaryRecord { id: "BC-1008".into(), name: "Arjun Nair".into(), email: "arjun.nair@brightcone.ai".into(), dept: "Sales".into(), gross: 90000.0 },
        EmployeeSalaryRecord { id: "BC-1009".into(), name: "Daniel Brooks".into(), email: "daniel.brooks@brightcone.ai".into(), dept: "Engineering".into(), gross: 105000.0 },
        EmployeeSalaryRecord { id: "BC-1010".into(), name: "Emily Carter".into(), email: "emily.carter@brightcone.ai".into(), dept: "Marketing".into(), gross: 95000.0 },
        EmployeeSalaryRecord { id: "BC-1011".into(), name: "Vikram Singh".into(), email: "vikram.singh@brightcone.ai".into(), dept: "Engineering".into(), gross: 110000.0 },
        EmployeeSalaryRecord { id: "BC-1012".into(), name: "Lakshmi Prasad".into(), email: "lakshmi.prasad@brightcone.ai".into(), dept: "Support".into(), gross: 80000.0 },
        EmployeeSalaryRecord { id: "BC-1013".into(), name: "Farhan Ali".into(), email: "farhan.ali@brightcone.ai".into(), dept: "Operations".into(), gross: 75000.0 },
        EmployeeSalaryRecord { id: "BC-1014".into(), name: "Divya Kulkarni".into(), email: "divya.kulkarni@brightcone.ai".into(), dept: "Marketing".into(), gross: 125000.0 },
    ];

    let mut total_gross: f64 = 0.0;
    let mut total_deductions: f64 = 0.0;
    let mut total_net: f64 = 0.0;

    for emp in &employees {
        let b = compute_salary_in_paise(emp.gross, true, "new");
        total_gross += b.gross_monthly_rupees;
        let ded = b.employee_pf_rupees + b.professional_tax_rupees + b.estimated_tds_rupees;
        total_deductions += ded;
        total_net += b.net_take_home_rupees;
    }

    let active_run = PayrollRun {
        id: "RUN-2026-10".into(),
        month: "October 2026".into(),
        status: "In Progress".into(),
        current_step: 1,
        total_employees: employees.len(),
        total_gross: (total_gross * 100.0_f64).round() / 100.0_f64,
        total_net: (total_net * 100.0_f64).round() / 100.0_f64,
        total_deductions: (total_deductions * 100.0_f64).round() / 100.0_f64,
        is_locked: false,
        is_supplementary: false,
        supplementary_notes: None,
        disbursed_at: None,
        created_at: "2026-10-01T00:00:00Z".into(),
    };

    let past_runs = vec![
        PayrollRun {
            id: "RUN-2026-09".into(),
            month: "September 2026".into(),
            status: "Locked & Disbursed".into(),
            current_step: 5,
            total_employees: employees.len(),
            total_gross: (total_gross * 100.0_f64).round() / 100.0_f64,
            total_net: (total_net * 100.0_f64).round() / 100.0_f64,
            total_deductions: (total_deductions * 100.0_f64).round() / 100.0_f64,
            is_locked: true,
            is_supplementary: false,
            supplementary_notes: None,
            disbursed_at: Some("2026-09-30T18:00:00Z".into()),
            created_at: "2026-09-01T00:00:00Z".into(),
        },
        PayrollRun {
            id: "RUN-2026-08".into(),
            month: "August 2026".into(),
            status: "Locked & Disbursed".into(),
            current_step: 5,
            total_employees: employees.len(),
            total_gross: ((total_gross - 75000.0_f64) * 100.0_f64).round() / 100.0_f64,
            total_net: ((total_net - 65000.0_f64) * 100.0_f64).round() / 100.0_f64,
            total_deductions: ((total_deductions - 10000.0_f64) * 100.0_f64).round() / 100.0_f64,
            is_locked: true,
            is_supplementary: false,
            supplementary_notes: None,
            disbursed_at: Some("2026-08-31T18:00:00Z".into()),
            created_at: "2026-08-01T00:00:00Z".into(),
        },
    ];

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

    Arc::new(Mutex::new(PayrollStore {
        active_run,
        past_runs,
        employees,
        statutory_compliance,
        pf_esi_compliance,
        statutory_filings,
        benefits,
    }))
    })
}

#[debug_handler]
pub async fn list_runs() -> Result<Response> {
    let s = get_store().lock().unwrap();
    let mut runs = vec![s.active_run.clone()];
    runs.extend(s.past_runs.clone());
    format::json(runs)
}

#[debug_handler]
pub async fn get_active_run() -> Result<Response> {
    let s = get_store().lock().unwrap();
    format::json(s.active_run.clone())
}

#[debug_handler]
pub async fn get_attendance_sync() -> Result<Response> {
    let s = get_store().lock().unwrap();
    let items: Vec<AttendanceSyncItem> = s.employees.iter().map(|e| AttendanceSyncItem {
        employee_id: e.id.clone(),
        employee_name: e.name.clone(),
        department: e.dept.clone(),
        total_days: 31,
        days_worked: 30,
        paid_leaves: 1,
        lop_days: 0,
        payable_days: 31,
    }).collect();
    format::json(items)
}

#[debug_handler]
pub async fn get_variances() -> Result<Response> {
    let s = get_store().lock().unwrap();
    let items: Vec<VarianceItem> = s.employees.iter().map(|e| {
        let b = compute_salary_in_paise(e.gross, true, "new");
        VarianceItem {
            employee_id: e.id.clone(),
            employee_name: e.name.clone(),
            department: e.dept.clone(),
            previous_net: b.net_take_home_rupees,
            current_net: b.net_take_home_rupees,
            variance_amount: 0.0,
            variance_pct: 0.0,
            flagged: false,
            reason: "Standard monthly salary without LOP".into(),
        }
    }).collect();
    format::json(items)
}

#[debug_handler]
pub async fn get_register(State(ctx): State<AppContext>) -> Result<Response> {
    let db_emps = crate::models::_entities::employees::Entity::find()
        .all(&ctx.db)
        .await
        .unwrap_or_default();

    let items: Vec<RegisterItem> = if !db_emps.is_empty() {
        db_emps.into_iter().map(|e| {
            let monthly_gross: f64 = e.ctc.unwrap_or(1200000.0) / 12.0;
            let b = compute_salary_in_paise(monthly_gross, true, "new");
            RegisterItem {
                emp_id: e.employee_code,
                name: e.full_name,
                dept: e.department,
                gross: b.gross_monthly_rupees,
                pf: b.employee_pf_rupees,
                pt: b.professional_tax_rupees,
                tds: b.estimated_tds_rupees,
                net: b.net_take_home_rupees,
            }
        }).collect()
    } else {
        let s = get_store().lock().unwrap();
        s.employees.iter().map(|e| {
            let b = compute_salary_in_paise(e.gross, true, "new");
            RegisterItem {
                emp_id: e.id.clone(),
                name: e.name.clone(),
                dept: e.dept.clone(),
                gross: b.gross_monthly_rupees,
                pf: b.employee_pf_rupees,
                pt: b.professional_tax_rupees,
                tds: b.estimated_tds_rupees,
                net: b.net_take_home_rupees,
            }
        }).collect()
    };
    format::json(items)
}

#[debug_handler]
pub async fn advance_step(Path(_id): Path<String>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    if s.active_run.current_step < 5 {
        s.active_run.current_step += 1;
    }
    format::json(s.active_run.clone())
}

#[derive(Serialize)]
pub struct PublishResponse {
    pub success: bool,
    pub run: PayrollRun,
    pub message: String,
}

#[debug_handler]
pub async fn publish_and_lock(Path(_id): Path<String>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    s.active_run.is_locked = true;
    s.active_run.status = "Locked & Disbursed".into();
    s.active_run.current_step = 5;
    s.active_run.disbursed_at = Some(Utc::now().to_rfc3339());
    format::json(PublishResponse {
        success: true,
        run: s.active_run.clone(),
        message: format!("{} payroll run locked and disbursed successfully", s.active_run.month),
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
        .add("/runs", get(list_runs))
        .add("/runs/active", get(get_active_run))
        .add("/attendance-sync", get(get_attendance_sync))
        .add("/variances", get(get_variances))
        .add("/register", get(get_register))
        .add("/runs/{id}/advance-step", post(advance_step))
        .add("/runs/{id}/publish-and-lock", post(publish_and_lock))
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
        .add("/runs", get(list_runs))
        .add("/runs/active", get(get_active_run))
        .add("/attendance-sync", get(get_attendance_sync))
        .add("/variances", get(get_variances))
        .add("/register", get(get_register))
        .add("/runs/{id}/advance-step", post(advance_step))
        .add("/runs/{id}/publish-and-lock", post(publish_and_lock))
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

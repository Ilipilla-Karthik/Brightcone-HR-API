use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::{ConnectionTrait, Statement};
use serde_json::Value;

use crate::controllers::payroll_store::{
    get_store, AttendanceSyncItem, RegisterItem, VarianceItem,
};
use crate::modules::salary::calculator::compute_salary_in_paise;

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
pub async fn get_attendance_sync(State(ctx): State<AppContext>) -> Result<Response> {
    let db_emps = crate::models::_entities::employees::Entity::find()
        .all(&ctx.db)
        .await
        .unwrap_or_default();

    let items: Vec<AttendanceSyncItem> = db_emps
        .into_iter()
        .map(|e| AttendanceSyncItem {
            employee_id: e.employee_code,
            employee_name: e.full_name,
            department: e.department,
            total_days: 31,
            days_worked: 30,
            paid_leaves: 1,
            lop_days: 0,
            payable_days: 31,
        })
        .collect();
    format::json(items)
}

#[debug_handler]
pub async fn get_variances(State(ctx): State<AppContext>) -> Result<Response> {
    let db_emps = crate::models::_entities::employees::Entity::find()
        .all(&ctx.db)
        .await
        .unwrap_or_default();

    let items: Vec<VarianceItem> = db_emps
        .into_iter()
        .map(|e| {
            let monthly_gross: f64 = e.ctc.unwrap_or(1800000.0) / 12.0;
            let b = compute_salary_in_paise(monthly_gross, true, "new");
            VarianceItem {
                employee_id: e.employee_code,
                employee_name: e.full_name,
                department: e.department,
                previous_net: b.net_take_home_rupees,
                current_net: b.net_take_home_rupees,
                variance_amount: 0.0,
                variance_pct: 0.0,
                flagged: false,
                reason: "Standard monthly salary without LOP".into(),
            }
        })
        .collect();
    format::json(items)
}

#[debug_handler]
pub async fn get_register(State(ctx): State<AppContext>) -> Result<Response> {
    let db_emps = crate::models::_entities::employees::Entity::find()
        .all(&ctx.db)
        .await
        .unwrap_or_default();

    let items: Vec<RegisterItem> = db_emps
        .into_iter()
        .map(|e| {
            let monthly_gross: f64 = e.ctc.unwrap_or(1800000.0) / 12.0;
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
        })
        .collect();
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

#[debug_handler]
pub async fn publish_and_lock(Path(_id): Path<String>) -> Result<Response> {
    let mut s = get_store().lock().unwrap();
    s.active_run.is_locked = true;
    s.active_run.status = "Locked & Disbursed".into();
    s.active_run.current_step = 5;
    s.active_run.disbursed_at = Some(Utc::now().to_rfc3339());
    format::json(serde_json::json!({
        "success": true,
        "run": s.active_run.clone(),
        "message": format!("{} payroll run locked and disbursed successfully", s.active_run.month),
    }))
}

// Salary Structures API
#[debug_handler]
pub async fn list_structures(State(ctx): State<AppContext>) -> Result<Response> {
    let sql = "SELECT id, employee_id, employee_name, dept, effective_from, annual_ctc, basic, basic_monthly, hra, hra_monthly, special_allowance, other_allowance, employer_pf, monthly_gross, pf_deduction, pt_deduction, status, is_archived FROM payroll_structures ORDER BY created_at DESC";
    let stmt = Statement::from_sql_and_values(ctx.db.get_database_backend(), sql, []);
    if let Ok(rows) = ctx.db.query_all_raw(stmt).await {
        if !rows.is_empty() {
            let mut res = Vec::new();
            for r in rows {
                let id: String = r.try_get("", "id").unwrap_or_default();
                let employee_id: String = r.try_get("", "employee_id").unwrap_or_default();
                let employee_name: String = r.try_get("", "employee_name").unwrap_or_default();
                let dept: String = r.try_get("", "dept").unwrap_or_default();
                let effective_from: String = r.try_get("", "effective_from").unwrap_or_default();
                let annual_ctc: f64 = r.try_get("", "annual_ctc").unwrap_or(0.0);
                let basic: f64 = r.try_get("", "basic").unwrap_or(0.0);
                let monthly_gross: f64 = r.try_get("", "monthly_gross").unwrap_or(0.0);
                let status: String = r.try_get("", "status").unwrap_or_else(|_| "Active".into());
                let is_archived: bool = r.try_get("", "is_archived").unwrap_or(false);
                res.push(serde_json::json!({
                    "id": id,
                    "employeeId": employee_id,
                    "employeeName": employee_name,
                    "dept": dept,
                    "effectiveFrom": effective_from,
                    "annualCtc": annual_ctc,
                    "basic": basic,
                    "basicMonthly": basic,
                    "hra": basic * 0.4,
                    "hraMonthly": basic * 0.4,
                    "specialAllowance": monthly_gross - (basic * 1.4),
                    "otherAllowance": 0,
                    "employerPf": 1800,
                    "monthlyGross": monthly_gross,
                    "pfDeduction": 1800,
                    "ptDeduction": 200,
                    "status": status,
                    "isArchived": is_archived
                }));
            }
            return format::json(res);
        }
    }
    let s = get_store().lock().unwrap();
    format::json(s.structures.clone())
}

#[debug_handler]
pub async fn create_structure(State(ctx): State<AppContext>, Json(mut payload): Json<Value>) -> Result<Response> {
    let id = payload.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
        .unwrap_or_else(|| format!("SS-{}", Utc::now().timestamp_subsec_millis()));
    payload["id"] = Value::String(id.clone());

    let emp_id = payload.get("employeeId").and_then(|v| v.as_str()).unwrap_or("BC-1006");
    let emp_name = payload.get("employeeName").and_then(|v| v.as_str()).unwrap_or("Rahul Varma");
    let dept = payload.get("dept").and_then(|v| v.as_str()).unwrap_or("Engineering");
    let eff = payload.get("effectiveFrom").and_then(|v| v.as_str()).unwrap_or("2026-10-01");
    let ctc = payload.get("annualCtc").and_then(|v| v.as_f64()).unwrap_or(2400000.0);
    let gross = payload.get("monthlyGross").and_then(|v| v.as_f64()).unwrap_or(ctc / 12.0);
    let basic = payload.get("basic").and_then(|v| v.as_f64()).unwrap_or(gross * 0.5);
    let hra = payload.get("hra").and_then(|v| v.as_f64()).unwrap_or(basic * 0.4);
    let special = payload.get("specialAllowance").and_then(|v| v.as_f64()).unwrap_or(gross - basic - hra);
    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("Active");

    let sql = format!(
        "INSERT INTO payroll_structures (id, employee_id, employee_name, dept, effective_from, annual_ctc, basic, basic_monthly, hra, hra_monthly, special_allowance, monthly_gross, status) VALUES ('{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}, {}, {}, {}, '{}') ON CONFLICT (id) DO UPDATE SET annual_ctc = {}, monthly_gross = {}, status = '{}'",
        id, emp_id, emp_name.replace('\'', "''"), dept, eff, ctc, basic, basic, hra, hra, special, gross, status, ctc, gross, status
    );
    let _ = ctx.db.execute_unprepared(&sql).await;

    let mut s = get_store().lock().unwrap();
    s.structures.insert(0, payload.clone());
    format::json(payload)
}

// Salary Revisions API
#[debug_handler]
pub async fn list_revisions(State(ctx): State<AppContext>) -> Result<Response> {
    let sql = "SELECT id, employee_id, employee_name, dept, appraisal_cycle, rating, current_ctc, revised_ctc, change_pct, effective_date, approver, status, is_archived FROM payroll_revisions ORDER BY created_at DESC";
    let stmt = Statement::from_sql_and_values(ctx.db.get_database_backend(), sql, []);
    if let Ok(rows) = ctx.db.query_all_raw(stmt).await {
        if !rows.is_empty() {
            let mut res = Vec::new();
            for r in rows {
                let id: String = r.try_get("", "id").unwrap_or_default();
                let employee_id: String = r.try_get("", "employee_id").unwrap_or_default();
                let employee_name: String = r.try_get("", "employee_name").unwrap_or_default();
                let dept: String = r.try_get("", "dept").unwrap_or_default();
                let cycle: String = r.try_get("", "appraisal_cycle").unwrap_or_default();
                let rating: i32 = r.try_get("", "rating").unwrap_or(4);
                let current_ctc: f64 = r.try_get("", "current_ctc").unwrap_or(0.0);
                let revised_ctc: f64 = r.try_get("", "revised_ctc").unwrap_or(0.0);
                let change_pct: f64 = r.try_get("", "change_pct").unwrap_or(0.0);
                let eff: String = r.try_get("", "effective_date").unwrap_or_default();
                let approver: String = r.try_get("", "approver").unwrap_or_default();
                let status: String = r.try_get("", "status").unwrap_or_else(|_| "Pending".into());
                let is_archived: bool = r.try_get("", "is_archived").unwrap_or(false);
                res.push(serde_json::json!({
                    "id": id,
                    "employeeId": employee_id,
                    "employeeName": employee_name,
                    "dept": dept,
                    "appraisalCycle": cycle,
                    "rating": rating,
                    "currentCtc": current_ctc,
                    "revisedCtc": revised_ctc,
                    "changePct": change_pct,
                    "effectiveDate": eff,
                    "approver": approver,
                    "status": status,
                    "isArchived": is_archived
                }));
            }
            return format::json(res);
        }
    }
    let s = get_store().lock().unwrap();
    format::json(s.revisions.clone())
}

#[debug_handler]
pub async fn create_revision(State(ctx): State<AppContext>, Json(mut payload): Json<Value>) -> Result<Response> {
    let id = payload.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())
        .unwrap_or_else(|| format!("SR-{}", Utc::now().timestamp_subsec_millis()));
    payload["id"] = Value::String(id.clone());

    let emp_id = payload.get("employeeId").and_then(|v| v.as_str()).unwrap_or("BC-1006");
    let emp_name = payload.get("employeeName").and_then(|v| v.as_str()).unwrap_or("Rahul Varma");
    let dept = payload.get("dept").and_then(|v| v.as_str()).unwrap_or("Engineering");
    let cycle = payload.get("appraisalCycle").and_then(|v| v.as_str()).unwrap_or("Annual Appraisal 2026-27");
    let rating = payload.get("rating").and_then(|v| v.as_i64()).unwrap_or(4);
    let cur_ctc = payload.get("currentCtc").and_then(|v| v.as_f64()).unwrap_or(2100000.0);
    let rev_ctc = payload.get("revisedCtc").and_then(|v| v.as_f64()).unwrap_or(2400000.0);
    let change = payload.get("changePct").and_then(|v| v.as_f64()).unwrap_or(14.29);
    let eff = payload.get("effectiveDate").and_then(|v| v.as_str()).unwrap_or("2026-11-01");
    let approver = payload.get("approver").and_then(|v| v.as_str()).unwrap_or("Naresh Vemparala");
    let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("Pending");

    let sql = format!(
        "INSERT INTO payroll_revisions (id, employee_id, employee_name, dept, appraisal_cycle, rating, current_ctc, revised_ctc, change_pct, effective_date, approver, status) VALUES ('{}', '{}', '{}', '{}', '{}', {}, {}, {}, {}, '{}', '{}', '{}') ON CONFLICT (id) DO UPDATE SET revised_ctc = {}, status = '{}'",
        id, emp_id, emp_name.replace('\'', "''"), dept, cycle.replace('\'', "''"), rating, cur_ctc, rev_ctc, change, eff, approver.replace('\'', "''"), status, rev_ctc, status
    );
    let _ = ctx.db.execute_unprepared(&sql).await;

    let mut s = get_store().lock().unwrap();
    s.revisions.insert(0, payload.clone());
    format::json(payload)
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
        .add("/structures", get(list_structures))
        .add("/structures", post(create_structure))
        .add("/revisions", get(list_revisions))
        .add("/revisions", post(create_revision))
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
        .add("/structures", get(list_structures))
        .add("/structures", post(create_structure))
        .add("/revisions", get(list_revisions))
        .add("/revisions", post(create_revision))
}

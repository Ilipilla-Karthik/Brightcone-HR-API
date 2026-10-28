use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};

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

pub struct PayrollStore {
    pub active_run: PayrollRun,
    pub past_runs: Vec<PayrollRun>,
    pub structures: Vec<serde_json::Value>,
    pub revisions: Vec<serde_json::Value>,
    pub payslips: Vec<serde_json::Value>,
}

static PAYROLL_STORE: OnceLock<Arc<Mutex<PayrollStore>>> = OnceLock::new();

pub fn get_store() -> &'static Arc<Mutex<PayrollStore>> {
    PAYROLL_STORE.get_or_init(|| {
        let active_run = PayrollRun {
            id: "RUN-2026-10".into(),
            month: "October 2026".into(),
            status: "In Progress".into(),
            current_step: 1,
            total_employees: 14,
            total_gross: 4942467.0,
            total_net: 3885467.0,
            total_deductions: 1057000.0,
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
                total_employees: 14,
                total_gross: 4942467.0,
                total_net: 3885467.0,
                total_deductions: 1057000.0,
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
                total_employees: 14,
                total_gross: 4867467.0,
                total_net: 3820467.0,
                total_deductions: 1047000.0,
                is_locked: true,
                is_supplementary: false,
                supplementary_notes: None,
                disbursed_at: Some("2026-08-31T18:00:00Z".into()),
                created_at: "2026-08-01T00:00:00Z".into(),
            },
        ];

        let structures = vec![
            serde_json::json!({ "id": "SS-101", "employeeId": "BC-1001", "employeeName": "Ananya Rao", "dept": "People Ops", "effectiveFrom": "2026-04-01", "annualCtc": 3200000, "basic": 133333, "basicMonthly": 133333, "hra": 53333, "hraMonthly": 53333, "specialAllowance": 80001, "otherAllowance": 0, "employerPf": 1800, "monthlyGross": 266667, "pfDeduction": 1800, "ptDeduction": 200, "status": "Active" }),
            serde_json::json!({ "id": "SS-102", "employeeId": "BC-1002", "employeeName": "Naresh Vemparala", "dept": "Operations", "effectiveFrom": "2026-04-01", "annualCtc": 9600000, "basic": 400000, "basicMonthly": 400000, "hra": 160000, "hraMonthly": 160000, "specialAllowance": 240000, "otherAllowance": 0, "employerPf": 1800, "monthlyGross": 800000, "pfDeduction": 1800, "ptDeduction": 200, "status": "Active" }),
            serde_json::json!({ "id": "SS-103", "employeeId": "BC-1006", "employeeName": "Rahul Varma", "dept": "Engineering", "effectiveFrom": "2026-04-01", "annualCtc": 2400000, "basic": 100000, "basicMonthly": 100000, "hra": 40000, "hraMonthly": 40000, "specialAllowance": 60000, "otherAllowance": 0, "employerPf": 1800, "monthlyGross": 200000, "pfDeduction": 1800, "ptDeduction": 200, "status": "Active" }),
        ];

        let revisions = vec![
            serde_json::json!({ "id": "SR-201", "employeeId": "BC-1001", "employeeName": "Ananya Rao", "dept": "People Ops", "appraisalCycle": "Annual Appraisal 2026-27", "rating": 5, "currentCtc": 2800000, "revisedCtc": 3200000, "changePct": 14.29, "effectiveDate": "2026-04-01", "approver": "Naresh Vemparala", "status": "Approved" }),
            serde_json::json!({ "id": "SR-202", "employeeId": "BC-1006", "employeeName": "Rahul Varma", "dept": "Engineering", "appraisalCycle": "Mid-Year Merit 2026", "rating": 4, "currentCtc": 2100000, "revisedCtc": 2400000, "changePct": 14.29, "effectiveDate": "2026-10-01", "approver": "Naresh Vemparala", "status": "Approved" }),
        ];

        let payslips = vec![
            serde_json::json!({ "id": "PS-2026-09-01", "employeeId": "BC-1006", "employeeName": "Rahul Varma", "dept": "Engineering", "month": "September 2026", "issuedOn": "2026-09-30", "grossPay": 200000, "deductions": 28400, "netPay": 171600, "payslipFile": "payslip_rahul_sep26.pdf", "status": "Generated" }),
            serde_json::json!({ "id": "PS-2026-09-02", "employeeId": "BC-1001", "employeeName": "Ananya Rao", "dept": "People Ops", "month": "September 2026", "issuedOn": "2026-09-30", "grossPay": 266667, "deductions": 39500, "netPay": 227167, "payslipFile": "payslip_ananya_sep26.pdf", "status": "Generated" }),
        ];

        Arc::new(Mutex::new(PayrollStore {
            active_run,
            past_runs,
            structures,
            revisions,
            payslips,
        }))
    })
}

//! GENERATED file (split for the 400-line source limit). Source of truth: the HTML spec.
//! Regenerate with: node D:/pip-temp/opencode/bcgen/gen.js
use super::{DynCol, DynTable};
pub static PROFILE_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "pemail", sql: "str", req: 0 },
  DynCol { name: "marital", sql: "str", req: 0 },
  DynCol { name: "address", sql: "text", req: 0 },
  DynCol { name: "ec_name", sql: "str", req: 0 },
  DynCol { name: "ec_phone", sql: "str", req: 0 },
  DynCol { name: "blood", sql: "str", req: 0 },
  DynCol { name: "pan", sql: "str", req: 0 },
  DynCol { name: "aadhaar", sql: "str", req: 0 },
  DynCol { name: "uan", sql: "str", req: 0 },
  DynCol { name: "bank", sql: "str", req: 0 },
  DynCol { name: "acct", sql: "str", req: 0 },
  DynCol { name: "ifsc", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static PROFILE_TABLE: DynTable = DynTable { table: "employee_profiles", kind: "profile", code: "undefined", ext: false, prefix: "PRO-", pad: 3, map: &[("ecName","ec_name"),("ecPhone","ec_phone")], cols: PROFILE_COLS };

pub static PROBATION_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "start", sql: "date", req: 1 },
  DynCol { name: "end", sql: "date", req: 1 },
  DynCol { name: "reviewer", sql: "str", req: 0 },
  DynCol { name: "outcome", sql: "str", req: 0 },
  DynCol { name: "ext_till", sql: "date", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "remarks", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static PROBATION_TABLE: DynTable = DynTable { table: "probations", kind: "probation", code: "undefined", ext: false, prefix: "PRO-", pad: 3, map: &[("extTill","ext_till")], cols: PROBATION_COLS };

pub static APPRAISAL_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "cycle", sql: "str", req: 1 },
  DynCol { name: "appraiser", sql: "str", req: 0 },
  DynCol { name: "due", sql: "date", req: 1 },
  DynCol { name: "rating", sql: "str", req: 0 },
  DynCol { name: "inc_pct", sql: "double", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "notes", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static APPRAISAL_TABLE: DynTable = DynTable { table: "appraisals", kind: "appraisal", code: "undefined", ext: false, prefix: "APP-", pad: 3, map: &[("incPct","inc_pct")], cols: APPRAISAL_COLS };

pub static PROBREV_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "reviewer", sql: "str", req: 1 },
  DynCol { name: "p_end", sql: "date", req: 1 },
  DynCol { name: "perf", sql: "str", req: 0 },
  DynCol { name: "conduct", sql: "str", req: 0 },
  DynCol { name: "rec", sql: "str", req: 1 },
  DynCol { name: "rdate", sql: "date", req: 0 },
  DynCol { name: "comments", sql: "text", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static PROBREV_TABLE: DynTable = DynTable { table: "probation_reviews", kind: "probrev", code: "undefined", ext: false, prefix: "PRO-", pad: 3, map: &[("pEnd","p_end")], cols: PROBREV_COLS };

pub static INCREMENTS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "eff", sql: "date", req: 1 },
  DynCol { name: "itype", sql: "str", req: 1 },
  DynCol { name: "prev", sql: "double", req: 1 },
  DynCol { name: "next", sql: "double", req: 1 },
  DynCol { name: "pct", sql: "double", req: 0 },
  DynCol { name: "approver", sql: "str", req: 0 },
  DynCol { name: "letter", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static INCREMENTS_TABLE: DynTable = DynTable { table: "increments", kind: "increments", code: "undefined", ext: false, prefix: "INC-", pad: 3, map: &[], cols: INCREMENTS_COLS };

pub static CARDS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "card_no", sql: "str", req: 1 },
  DynCol { name: "ctype", sql: "str", req: 1 },
  DynCol { name: "emp", sql: "str", req: 0 },
  DynCol { name: "holder", sql: "str", req: 0 },
  DynCol { name: "issued", sql: "date", req: 1 },
  DynCol { name: "valid", sql: "date", req: 0 },
  DynCol { name: "zones", sql: "str", req: 0 },
  DynCol { name: "returned", sql: "date", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static CARDS_TABLE: DynTable = DynTable { table: "access_cards", kind: "cards", code: "undefined", ext: false, prefix: "AC-", pad: 3, map: &[("cardNo","card_no")], cols: CARDS_COLS };

pub static FACILITIES_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "facility", sql: "str", req: 1 },
  DynCol { name: "location", sql: "str", req: 0 },
  DynCol { name: "raised_by", sql: "str", req: 0 },
  DynCol { name: "priority", sql: "str", req: 1 },
  DynCol { name: "desc", sql: "text", req: 1 },
  DynCol { name: "owner", sql: "str", req: 0 },
  DynCol { name: "due", sql: "date", req: 1 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static FACILITIES_TABLE: DynTable = DynTable { table: "facility_requests", kind: "facilities", code: "undefined", ext: false, prefix: "FR-", pad: 3, map: &[("raisedBy","raised_by")], cols: FACILITIES_COLS };

pub static MANPOWER_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "department", sql: "str", req: 1 },
  DynCol { name: "designation", sql: "str", req: 1 },
  DynCol { name: "positions", sql: "double", req: 1 },
  DynCol { name: "etype", sql: "str", req: 1 },
  DynCol { name: "location", sql: "str", req: 0 },
  DynCol { name: "exp", sql: "str", req: 0 },
  DynCol { name: "budget", sql: "str", req: 0 },
  DynCol { name: "reason", sql: "str", req: 1 },
  DynCol { name: "replacing", sql: "str", req: 0 },
  DynCol { name: "hm", sql: "str", req: 0 },
  DynCol { name: "by", sql: "date", req: 1 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "jd", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static MANPOWER_TABLE: DynTable = DynTable { table: "manpower_requirements", kind: "manpower", code: "undefined", ext: false, prefix: "MR-", pad: 3, map: &[], cols: MANPOWER_COLS };

pub static JOINING_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "doj", sql: "date", req: 1 },
  DynCol { name: "docs", sql: "str", req: 0 },
  DynCol { name: "idcard", sql: "str", req: 0 },
  DynCol { name: "mail", sql: "str", req: 0 },
  DynCol { name: "laptop", sql: "str", req: 0 },
  DynCol { name: "bankd", sql: "str", req: 0 },
  DynCol { name: "pf", sql: "str", req: 0 },
  DynCol { name: "appt", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "notes", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static JOINING_TABLE: DynTable = DynTable { table: "joinings", kind: "joining", code: "undefined", ext: false, prefix: "JOI-", pad: 3, map: &[], cols: JOINING_COLS };

pub static ONBOARD2_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "task", sql: "str", req: 1 },
  DynCol { name: "owner", sql: "str", req: 0 },
  DynCol { name: "due", sql: "date", req: 1 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "done", sql: "date", req: 0 },
  DynCol { name: "notes", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static ONBOARD2_TABLE: DynTable = DynTable { table: "onboarding_tasks", kind: "onboard2", code: "undefined", ext: false, prefix: "ONB-", pad: 3, map: &[], cols: ONBOARD2_COLS };

pub static LOGS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "date", sql: "date", req: 1 },
  DynCol { name: "source", sql: "str", req: 1 },
  DynCol { name: "login", sql: "time", req: 1 },
  DynCol { name: "logout", sql: "time", req: 0 },
  DynCol { name: "hours", sql: "str", req: 0 },
  DynCol { name: "place", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static LOGS_TABLE: DynTable = DynTable { table: "login_logs", kind: "logs", code: "undefined", ext: false, prefix: "LOG-", pad: 3, map: &[], cols: LOGS_COLS };

pub static WFH_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "date", sql: "date", req: 1 },
  DynCol { name: "req_on", sql: "date", req: 1 },
  DynCol { name: "reason", sql: "text", req: 0 },
  DynCol { name: "approver", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "notice", sql: "double", req: 0 },
  DynCol { name: "flag", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static WFH_TABLE: DynTable = DynTable { table: "wfh_requests", kind: "wfh", code: "undefined", ext: false, prefix: "WFH-", pad: 3, map: &[("reqOn","req_on")], cols: WFH_COLS };

pub static COMPOFF_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "worked", sql: "date", req: 1 },
  DynCol { name: "hours", sql: "double", req: 1 },
  DynCol { name: "reason", sql: "text", req: 1 },
  DynCol { name: "approver", sql: "str", req: 0 },
  DynCol { name: "use_by", sql: "date", req: 1 },
  DynCol { name: "availed", sql: "date", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static COMPOFF_TABLE: DynTable = DynTable { table: "comp_offs", kind: "compoff", code: "undefined", ext: false, prefix: "CO-", pad: 3, map: &[("useBy","use_by")], cols: COMPOFF_COLS };

pub static PAYROLL2_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "month", sql: "month", req: 1 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "wdays", sql: "double", req: 1 },
  DynCol { name: "lop", sql: "double", req: 0 },
  DynCol { name: "pdays", sql: "double", req: 0 },
  DynCol { name: "gross", sql: "double", req: 1 },
  DynCol { name: "ded", sql: "double", req: 1 },
  DynCol { name: "net", sql: "double", req: 0 },
  DynCol { name: "paid_on", sql: "date", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static PAYROLL2_TABLE: DynTable = DynTable { table: "payroll_lines", kind: "payroll2", code: "undefined", ext: false, prefix: "PR-", pad: 3, map: &[("paidOn","paid_on")], cols: PAYROLL2_COLS };

pub static STRUCTURE_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "eff", sql: "date", req: 1 },
  DynCol { name: "ctc", sql: "double", req: 1 },
  DynCol { name: "basic", sql: "double", req: 1 },
  DynCol { name: "hra", sql: "double", req: 0 },
  DynCol { name: "special", sql: "double", req: 0 },
  DynCol { name: "other", sql: "double", req: 0 },
  DynCol { name: "pf_er", sql: "double", req: 0 },
  DynCol { name: "mgross", sql: "double", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static STRUCTURE_TABLE: DynTable = DynTable { table: "salary_structures", kind: "structure", code: "undefined", ext: false, prefix: "STR-", pad: 3, map: &[("pfEr","pf_er")], cols: STRUCTURE_COLS };

pub static REVISIONS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "cycle", sql: "str", req: 0 },
  DynCol { name: "rating", sql: "str", req: 0 },
  DynCol { name: "cur", sql: "double", req: 1 },
  DynCol { name: "rev", sql: "double", req: 1 },
  DynCol { name: "pct", sql: "double", req: 0 },
  DynCol { name: "eff", sql: "date", req: 1 },
  DynCol { name: "approver", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "inc_id", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static REVISIONS_TABLE: DynTable = DynTable { table: "salary_revisions", kind: "revisions", code: "undefined", ext: false, prefix: "SR-", pad: 3, map: &[("incId","inc_id")], cols: REVISIONS_COLS };

pub static PAYSLIPS2_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "month", sql: "month", req: 1 },
  DynCol { name: "issued", sql: "date", req: 0 },
  DynCol { name: "gross", sql: "double", req: 1 },
  DynCol { name: "ded", sql: "double", req: 0 },
  DynCol { name: "net", sql: "double", req: 0 },
  DynCol { name: "file", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static PAYSLIPS2_TABLE: DynTable = DynTable { table: "payslips", kind: "payslips2", code: "undefined", ext: false, prefix: "PS-", pad: 3, map: &[], cols: PAYSLIPS2_COLS };

pub static PFESI_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "month", sql: "month", req: 1 },
  DynCol { name: "scheme", sql: "str", req: 1 },
  DynCol { name: "count", sql: "double", req: 0 },
  DynCol { name: "ee", sql: "double", req: 0 },
  DynCol { name: "er", sql: "double", req: 0 },
  DynCol { name: "total", sql: "double", req: 0 },
  DynCol { name: "due", sql: "date", req: 1 },
  DynCol { name: "challan", sql: "str", req: 0 },
  DynCol { name: "paid_on", sql: "date", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static PFESI_TABLE: DynTable = DynTable { table: "pf_esi_filings", kind: "pfesi", code: "undefined", ext: false, prefix: "PFE-", pad: 3, map: &[("paidOn","paid_on")], cols: PFESI_COLS };


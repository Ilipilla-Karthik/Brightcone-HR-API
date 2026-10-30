//! GENERATED file (split for the 400-line source limit). Source of truth: the HTML spec.
//! Regenerate with: node D:/pip-temp/opencode/bcgen/gen.js
use super::{DynCol, DynTable};
pub static STATUTORY_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "item", sql: "str", req: 1 },
  DynCol { name: "period", sql: "str", req: 1 },
  DynCol { name: "authority", sql: "str", req: 0 },
  DynCol { name: "due", sql: "date", req: 1 },
  DynCol { name: "owner", sql: "str", req: 0 },
  DynCol { name: "filed", sql: "date", req: 0 },
  DynCol { name: "ack", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "notes", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static STATUTORY_TABLE: DynTable = DynTable { table: "statutory_filings", kind: "statutory", code: "undefined", ext: false, prefix: "STA-", pad: 3, map: &[], cols: STATUTORY_COLS };

pub static OFFICEEXP_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "date", sql: "date", req: 1 },
  DynCol { name: "cat", sql: "str", req: 1 },
  DynCol { name: "vendor", sql: "str", req: 0 },
  DynCol { name: "inv", sql: "str", req: 0 },
  DynCol { name: "desc", sql: "text", req: 1 },
  DynCol { name: "amount", sql: "double", req: 1 },
  DynCol { name: "gst", sql: "double", req: 0 },
  DynCol { name: "total", sql: "double", req: 0 },
  DynCol { name: "mode", sql: "str", req: 0 },
  DynCol { name: "paid_by", sql: "str", req: 0 },
  DynCol { name: "approver", sql: "str", req: 0 },
  DynCol { name: "receipt", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static OFFICEEXP_TABLE: DynTable = DynTable { table: "office_expenses", kind: "officeexp", code: "undefined", ext: false, prefix: "OX-", pad: 3, map: &[("paidBy","paid_by")], cols: OFFICEEXP_COLS };

pub static TRAINING_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "prog", sql: "str", req: 1 },
  DynCol { name: "ttype", sql: "str", req: 1 },
  DynCol { name: "trainer", sql: "str", req: 0 },
  DynCol { name: "mode", sql: "str", req: 0 },
  DynCol { name: "start", sql: "date", req: 1 },
  DynCol { name: "end", sql: "date", req: 0 },
  DynCol { name: "parts", sql: "double", req: 0 },
  DynCol { name: "cost", sql: "double", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "notes", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static TRAINING_TABLE: DynTable = DynTable { table: "trainings", kind: "training", code: "undefined", ext: false, prefix: "TR-", pad: 3, map: &[], cols: TRAINING_COLS };

pub static POSH_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "rtype", sql: "str", req: 1 },
  DynCol { name: "emp", sql: "str", req: 0 },
  DynCol { name: "name", sql: "str", req: 1 },
  DynCol { name: "role", sql: "str", req: 0 },
  DynCol { name: "date", sql: "date", req: 1 },
  DynCol { name: "valid", sql: "date", req: 0 },
  DynCol { name: "parts", sql: "double", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static POSH_TABLE: DynTable = DynTable { table: "posh_records", kind: "posh", code: "undefined", ext: false, prefix: "POS-", pad: 3, map: &[], cols: POSH_COLS };

pub static SECTRAIN_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "course", sql: "str", req: 1 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "assigned", sql: "date", req: 1 },
  DynCol { name: "due", sql: "date", req: 1 },
  DynCol { name: "done", sql: "date", req: 0 },
  DynCol { name: "score", sql: "double", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static SECTRAIN_TABLE: DynTable = DynTable { table: "security_trainings", kind: "sectrain", code: "undefined", ext: false, prefix: "SEC-", pad: 3, map: &[], cols: SECTRAIN_COLS };

pub static CHECKINS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "mgr", sql: "str", req: 0 },
  DynCol { name: "date", sql: "date", req: 1 },
  DynCol { name: "well", sql: "text", req: 0 },
  DynCol { name: "block", sql: "text", req: 0 },
  DynCol { name: "acts", sql: "text", req: 0 },
  DynCol { name: "next", sql: "date", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static CHECKINS_TABLE: DynTable = DynTable { table: "checkins", kind: "checkins", code: "undefined", ext: false, prefix: "CI-", pad: 3, map: &[], cols: CHECKINS_COLS };

pub static PIPS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "mgr", sql: "str", req: 0 },
  DynCol { name: "hrp", sql: "str", req: 0 },
  DynCol { name: "start", sql: "date", req: 1 },
  DynCol { name: "end", sql: "date", req: 1 },
  DynCol { name: "reason", sql: "text", req: 1 },
  DynCol { name: "targets", sql: "text", req: 1 },
  DynCol { name: "support", sql: "text", req: 0 },
  DynCol { name: "reviews", sql: "text", req: 0 },
  DynCol { name: "outcome", sql: "str", req: 0 },
  DynCol { name: "shared", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static PIPS_TABLE: DynTable = DynTable { table: "pips", kind: "pips", code: "undefined", ext: false, prefix: "PIP-", pad: 3, map: &[], cols: PIPS_COLS };

pub static PROMOS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "cur_des", sql: "str", req: 0 },
  DynCol { name: "cur_grade", sql: "str", req: 0 },
  DynCol { name: "cur_ctc", sql: "double", req: 0 },
  DynCol { name: "rating", sql: "str", req: 0 },
  DynCol { name: "prop_des", sql: "str", req: 1 },
  DynCol { name: "prop_grade", sql: "str", req: 0 },
  DynCol { name: "prop_ctc", sql: "double", req: 0 },
  DynCol { name: "pct", sql: "double", req: 0 },
  DynCol { name: "elig", sql: "str", req: 0 },
  DynCol { name: "just", sql: "text", req: 1 },
  DynCol { name: "override", sql: "text", req: 0 },
  DynCol { name: "nom_by", sql: "str", req: 0 },
  DynCol { name: "eff", sql: "date", req: 1 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static PROMOS_TABLE: DynTable = DynTable { table: "promos", kind: "promos", code: "undefined", ext: false, prefix: "PRM-", pad: 3, map: &[("curDes","cur_des"),("curGrade","cur_grade"),("curCtc","cur_ctc"),("propDes","prop_des"),("propGrade","prop_grade"),("propCtc","prop_ctc"),("nomBy","nom_by")], cols: PROMOS_COLS };

pub static IDPS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "target", sql: "str", req: 1 },
  DynCol { name: "time", sql: "str", req: 0 },
  DynCol { name: "skills", sql: "text", req: 0 },
  DynCol { name: "courses", sql: "str", req: 0 },
  DynCol { name: "mentor", sql: "str", req: 0 },
  DynCol { name: "review", sql: "date", req: 0 },
  DynCol { name: "progress", sql: "double", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static IDPS_TABLE: DynTable = DynTable { table: "idps", kind: "idps", code: "undefined", ext: false, prefix: "IDP-", pad: 3, map: &[], cols: IDPS_COLS };

pub static SUCCESSION_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "role", sql: "str", req: 1 },
  DynCol { name: "inc", sql: "str", req: 1 },
  DynCol { name: "risk", sql: "str", req: 1 },
  DynCol { name: "s1", sql: "str", req: 0 },
  DynCol { name: "r1", sql: "str", req: 0 },
  DynCol { name: "s2", sql: "str", req: 0 },
  DynCol { name: "r2", sql: "str", req: 0 },
  DynCol { name: "s3", sql: "str", req: 0 },
  DynCol { name: "r3", sql: "str", req: 0 },
  DynCol { name: "notes", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static SUCCESSION_TABLE: DynTable = DynTable { table: "successions", kind: "succession", code: "undefined", ext: false, prefix: "SUC-", pad: 3, map: &[], cols: SUCCESSION_COLS };

pub static SURVEYS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "title", sql: "str", req: 1 },
  DynCol { name: "period", sql: "str", req: 0 },
  DynCol { name: "aud", sql: "str", req: 1 },
  DynCol { name: "stmts", sql: "text", req: 1 },
  DynCol { name: "open", sql: "date", req: 1 },
  DynCol { name: "close", sql: "date", req: 1 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static SURVEYS_TABLE: DynTable = DynTable { table: "surveys", kind: "surveys", code: "undefined", ext: false, prefix: "SV-", pad: 3, map: &[], cols: SURVEYS_COLS };

pub static LICENCES_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "name", sql: "str", req: 1 },
  DynCol { name: "no", sql: "str", req: 1 },
  DynCol { name: "auth", sql: "str", req: 0 },
  DynCol { name: "loc", sql: "str", req: 0 },
  DynCol { name: "valid", sql: "date", req: 0 },
  DynCol { name: "owner", sql: "str", req: 0 },
  DynCol { name: "file", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static LICENCES_TABLE: DynTable = DynTable { table: "licences", kind: "licences", code: "undefined", ext: false, prefix: "LIC-", pad: 3, map: &[], cols: LICENCES_COLS };

pub static ACTIONS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "title", sql: "str", req: 1 },
  DynCol { name: "cat", sql: "str", req: 1 },
  DynCol { name: "pri", sql: "str", req: 1 },
  DynCol { name: "emp", sql: "str", req: 0 },
  DynCol { name: "owner", sql: "str", req: 0 },
  DynCol { name: "due", sql: "date", req: 1 },
  DynCol { name: "remind", sql: "str", req: 0 },
  DynCol { name: "repeat", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "src", sql: "str", req: 0 },
  DynCol { name: "notes", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static ACTIONS_TABLE: DynTable = DynTable { table: "hr_actions", kind: "actions", code: "undefined", ext: false, prefix: "ACT-", pad: 3, map: &[], cols: ACTIONS_COLS };

pub static EXPENSE_CLAIMS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "edate", sql: "date", req: 1 },
  DynCol { name: "cat", sql: "str", req: 1 },
  DynCol { name: "amt", sql: "double", req: 1 },
  DynCol { name: "gst", sql: "double", req: 0 },
  DynCol { name: "total", sql: "double", req: 0 },
  DynCol { name: "inv", sql: "str", req: 0 },
  DynCol { name: "descp", sql: "text", req: 1 },
  DynCol { name: "vendor", sql: "str", req: 0 },
  DynCol { name: "mode", sql: "str", req: 0 },
  DynCol { name: "paid_by", sql: "str", req: 0 },
  DynCol { name: "bill", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static EXPENSE_CLAIMS_TABLE: DynTable = DynTable { table: "expense_claims", kind: "expense_claims", code: "undefined", ext: false, prefix: "EX-", pad: 3, map: &[("paidBy","paid_by")], cols: EXPENSE_CLAIMS_COLS };

pub static BENEFIT_PLANS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "name", sql: "str", req: 1 },
  DynCol { name: "btype", sql: "str", req: 1 },
  DynCol { name: "insurer", sql: "str", req: 0 },
  DynCol { name: "cover_amt", sql: "double", req: 0 },
  DynCol { name: "premium", sql: "double", req: 0 },
  DynCol { name: "bstart", sql: "date", req: 0 },
  DynCol { name: "bend", sql: "date", req: 0 },
  DynCol { name: "elig", sql: "text", req: 0 },
  DynCol { name: "descp2", sql: "text", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static BENEFIT_PLANS_TABLE: DynTable = DynTable { table: "benefit_plans", kind: "benefit_plans", code: "undefined", ext: false, prefix: "BPL-", pad: 3, map: &[("coverAmt","cover_amt")], cols: BENEFIT_PLANS_COLS };

pub static BENEFIT_ENROLLMENTS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "plan", sql: "str", req: 1 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "bdate", sql: "date", req: 0 },
  DynCol { name: "bdependents", sql: "text", req: 0 },
  DynCol { name: "bnote", sql: "text", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static BENEFIT_ENROLLMENTS_TABLE: DynTable = DynTable { table: "benefit_enrollments", kind: "benefit_enrollments", code: "undefined", ext: false, prefix: "BEN-", pad: 3, map: &[], cols: BENEFIT_ENROLLMENTS_COLS };

pub static LETTER_REQUESTS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "letter", sql: "str", req: 1 },
  DynCol { name: "lang", sql: "text", req: 0 },
  DynCol { name: "rdate", sql: "date", req: 1 },
  DynCol { name: "rfor", sql: "date", req: 0 },
  DynCol { name: "notes", sql: "text", req: 0 },
  DynCol { name: "lfile", sql: "str", req: 0 },
  DynCol { name: "lstatus", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static LETTER_REQUESTS_TABLE: DynTable = DynTable { table: "letter_requests", kind: "letter_requests", code: "undefined", ext: false, prefix: "LTR-", pad: 3, map: &[], cols: LETTER_REQUESTS_COLS };

pub static SURVEY_RESP_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "survey", sql: "str", req: 1 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "answers", sql: "text", req: 1 },
  DynCol { name: "submitted", sql: "date", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static SURVEY_RESP_TABLE: DynTable = DynTable { table: "survey_responses", kind: "survey_resp", code: "undefined", ext: false, prefix: "SRE-", pad: 3, map: &[], cols: SURVEY_RESP_COLS };


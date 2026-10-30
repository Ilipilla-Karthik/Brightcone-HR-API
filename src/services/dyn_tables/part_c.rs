//! GENERATED file (split for the 400-line source limit). Source of truth: the HTML spec.
//! Regenerate with: node D:/pip-temp/opencode/bcgen/gen.js
use super::{DynCol, DynTable};
pub static EMP_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "first", sql: "str", req: 1 },
  DynCol { name: "last", sql: "str", req: 1 },
  DynCol { name: "gender", sql: "str", req: 0 },
  DynCol { name: "type_value", sql: "str", req: 1 },
  DynCol { name: "prob_end", sql: "date", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static EMP_TABLE: DynTable = DynTable { table: "employees", kind: "emp", code: "employee_code", ext: true, prefix: "EMP-", pad: 4, map: &[("email","work_email"),("phone","phone"),("dob","dob"),("department","department"),("designation","job_title"),("grade","grade"),("location","location"),("manager","manager_name"),("doj","joined"),("status","status"),("type","type_value"),("probEnd","prob_end")], cols: EMP_COLS };

pub static DOCS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "dno", sql: "str", req: 0 },
  DynCol { name: "file", sql: "str", req: 0 },
  DynCol { name: "expiry", sql: "date", req: 0 },
  DynCol { name: "verified_by", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static DOCS_TABLE: DynTable = DynTable { table: "employee_documents", kind: "docs", code: "code", ext: true, prefix: "DOC-", pad: 3, map: &[("emp","employee_code"),("dtype","document_type"),("issued","issued_date"),("verifiedBy","verified_by")], cols: DOCS_COLS };

pub static REVIEWS_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "reviewer", sql: "str", req: 1 },
  DynCol { name: "from_date", sql: "date", req: 1 },
  DynCol { name: "to_date", sql: "date", req: 1 },
  DynCol { name: "goals", sql: "text", req: 0 },
  DynCol { name: "improve", sql: "text", req: 0 },
  DynCol { name: "rdate", sql: "date", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static REVIEWS_TABLE: DynTable = DynTable { table: "performance_reviews", kind: "reviews", code: "code", ext: true, prefix: "REV-", pad: 3, map: &[("strengths","strengths"),("status","status"),("rating","rating"),("from","from_date"),("to","to_date")], cols: REVIEWS_COLS };

pub static ASSETS2_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "tag", sql: "str", req: 1 },
  DynCol { name: "purchase", sql: "date", req: 0 },
  DynCol { name: "warranty", sql: "date", req: 0 },
  DynCol { name: "condition", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static ASSETS2_TABLE: DynTable = DynTable { table: "employee_assets", kind: "assets2", code: "code", ext: true, prefix: "AS-", pad: 3, map: &[("emp","employee_code"),("atype","asset_type"),("model","model_name"),("serial","serial_number"),("assigned","assigned_date"),("status","status")], cols: ASSETS2_COLS };

pub static CANDS_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "req", sql: "str", req: 0 },
  DynCol { name: "exp", sql: "double", req: 0 },
  DynCol { name: "cctc", sql: "double", req: 0 },
  DynCol { name: "ectc", sql: "double", req: 0 },
  DynCol { name: "notice", sql: "double", req: 0 },
  DynCol { name: "loc", sql: "str", req: 0 },
  DynCol { name: "resume", sql: "str", req: 0 },
  DynCol { name: "consent", sql: "str", req: 1 },
  DynCol { name: "emp_id", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static CANDS_TABLE: DynTable = DynTable { table: "candidates", kind: "cands", code: "candidate_code", ext: true, prefix: "CAN-", pad: 3, map: &[("name","full_name"),("email","email"),("phone","phone"),("source","source"),("stage","stage"),("empId","emp_id")], cols: CANDS_COLS };

pub static INTERVIEWS2_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "cand", sql: "str", req: 1 },
  DynCol { name: "mode", sql: "str", req: 1 },
  DynCol { name: "date", sql: "date", req: 1 },
  DynCol { name: "time", sql: "time", req: 1 },
  DynCol { name: "venue", sql: "str", req: 0 },
  DynCol { name: "rating", sql: "str", req: 0 },
  DynCol { name: "result", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static INTERVIEWS2_TABLE: DynTable = DynTable { table: "interviews", kind: "interviews2", code: "code", ext: true, prefix: "INT-", pad: 3, map: &[("round","round_name"),("panel","panel"),("feedback","notes"),("status","status")], cols: INTERVIEWS2_COLS };

pub static OFFERS2_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "cand", sql: "str", req: 1 },
  DynCol { name: "department", sql: "str", req: 1 },
  DynCol { name: "odate", sql: "date", req: 1 },
  DynCol { name: "valid", sql: "date", req: 1 },
  DynCol { name: "approver", sql: "str", req: 0 },
  DynCol { name: "file", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static OFFERS2_TABLE: DynTable = DynTable { table: "offers", kind: "offers2", code: "code", ext: true, prefix: "OF-", pad: 3, map: &[("designation","job_title"),("ctc","offered_ctc"),("doj","joining_date"),("status","status")], cols: OFFERS2_COLS };

pub static INTERNS_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "email", sql: "str", req: 1 },
  DynCol { name: "phone", sql: "str", req: 0 },
  DynCol { name: "department", sql: "str", req: 0 },
  DynCol { name: "start", sql: "date", req: 1 },
  DynCol { name: "end", sql: "date", req: 1 },
  DynCol { name: "project", sql: "str", req: 0 },
  DynCol { name: "eval", sql: "str", req: 0 },
  DynCol { name: "status", sql: "str", req: 1 },
  DynCol { name: "emp_id", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static INTERNS_TABLE: DynTable = DynTable { table: "internships", kind: "interns", code: "intern_id", ext: true, prefix: "IN-", pad: 3, map: &[("name","name"),("college","college"),("course","track"),("mentor","mentor"),("stipend","stipend"),("empId","emp_id")], cols: INTERNS_COLS };

pub static ATT_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "date", sql: "date", req: 1 },
  DynCol { name: "mark", sql: "str", req: 1 },
  DynCol { name: "remarks", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static ATT_TABLE: DynTable = DynTable { table: "attendance_records", kind: "att", code: "code", ext: true, prefix: "ATT-", pad: 3, map: &[("emp","employee_code"),("in","clock_in"),("out","clock_out"),("hours","hours_worked"),("status","status")], cols: ATT_COLS };

pub static LEAVES2_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "half", sql: "str", req: 0 },
  DynCol { name: "approver", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static LEAVES2_TABLE: DynTable = DynTable { table: "leave_requests", kind: "leaves2", code: "code", ext: true, prefix: "LV-", pad: 3, map: &[("emp","employee_code"),("ltype","leave_type"),("from","from_date"),("to","to_date"),("days","days"),("reason","reason"),("status","status")], cols: LEAVES2_COLS };

pub static HOLIDAYS2_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "day", sql: "str", req: 0 },
  DynCol { name: "htype", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static HOLIDAYS2_TABLE: DynTable = DynTable { table: "holidays", kind: "holidays2", code: "code", ext: true, prefix: "HOL-", pad: 3, map: &[("date","holiday_date"),("name","name"),("loc","applicable_location")], cols: HOLIDAYS2_COLS };

pub static VENDORS_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "gstin", sql: "str", req: 0 },
  DynCol { name: "pan", sql: "str", req: 0 },
  DynCol { name: "terms", sql: "str", req: 0 },
  DynCol { name: "start", sql: "date", req: 0 },
  DynCol { name: "value", sql: "double", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static VENDORS_TABLE: DynTable = DynTable { table: "vendors", kind: "vendors", code: "vendor_code", ext: true, prefix: "VN-", pad: 3, map: &[("name","name"),("cat","category"),("contact","contact_person"),("phone","phone"),("email","email"),("end","contract_expiry"),("status","status")], cols: VENDORS_COLS };

pub static HANDBOOK_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static HANDBOOK_TABLE: DynTable = DynTable { table: "handbooks", kind: "handbook", code: "handbook_id", ext: true, prefix: "HB-", pad: 3, map: &[("ver","version"),("eff","effective_date"),("title","title"),("owner","owner"),("file","file_name"),("changes","changes_summary"),("status","status")], cols: HANDBOOK_COLS };

pub static POLICIES_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static POLICIES_TABLE: DynTable = DynTable { table: "hr_policies", kind: "policies", code: "policy_id", ext: true, prefix: "POL-", pad: 3, map: &[("name","name"),("cat","category"),("ver","version"),("eff","effective_date"),("review","next_review_date"),("owner","owner"),("file","file_name"),("ackReq","ack_required"),("status","status")], cols: POLICIES_COLS };

pub static ACKS_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static ACKS_TABLE: DynTable = DynTable { table: "policy_acknowledgements", kind: "acks", code: "ack_code", ext: true, prefix: "ACK-", pad: 3, map: &[("policy","policy_id"),("sent","sent_date"),("due","due_date"),("ackOn","acknowledged_on"),("status","status")], cols: ACKS_COLS };

pub static EVENTS_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static EVENTS_TABLE: DynTable = DynTable { table: "company_events", kind: "events", code: "event_code", ext: true, prefix: "EV-", pad: 3, map: &[("name","name"),("etype","etype"),("date","event_date"),("time","event_time"),("venue","venue"),("organiser","organiser"),("budget","budget"),("att","attendees_count"),("status","status"),("notes","notes")], cols: EVENTS_COLS };

pub static FEEDBACK_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "anon", sql: "str", req: 1 },
  DynCol { name: "emp", sql: "str", req: 0 },
  DynCol { name: "rating", sql: "str", req: 0 },
  DynCol { name: "date", sql: "date", req: 0 },
  DynCol { name: "owner", sql: "str", req: 0 },
  DynCol { name: "action", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static FEEDBACK_TABLE: DynTable = DynTable { table: "connect_feedback", kind: "feedback", code: "feedback_id", ext: true, prefix: "FB-", pad: 3, map: &[("cat","category"),("text","body"),("status","status")], cols: FEEDBACK_COLS };

pub static TOWNHALL_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static TOWNHALL_TABLE: DynTable = DynTable { table: "connect_townhalls", kind: "townhall", code: "townhall_code", ext: true, prefix: "TH-", pad: 3, map: &[("title","title"),("date","meeting_date"),("time","meeting_time"),("mode","mode"),("host","host"),("agenda","agenda"),("att","attendees_count"),("status","status"),("minutes","minutes"),("actions","actions")], cols: TOWNHALL_COLS };

pub static GOALS_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "emp", sql: "str", req: 1 },
  DynCol { name: "cycle", sql: "str", req: 1 },
  DynCol { name: "kra", sql: "str", req: 1 },
  DynCol { name: "measure", sql: "str", req: 0 },
  DynCol { name: "weight", sql: "double", req: 1 },
  DynCol { name: "ec", sql: "text", req: 0 },
  DynCol { name: "mc", sql: "text", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static GOALS_TABLE: DynTable = DynTable { table: "performance_goals", kind: "goals", code: "goal_id", ext: true, prefix: "GL-", pad: 3, map: &[("title","title"),("due","target_date"),("progress","progress_percentage"),("status","status")], cols: GOALS_COLS };

pub static RECOG_COLS: &[DynCol] = &[
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "reward", sql: "double", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static RECOG_TABLE: DynTable = DynTable { table: "recognitions", kind: "recog", code: "recognition_code", ext: true, prefix: "REC-", pad: 3, map: &[("emp","employee_code"),("award","award"),("by","nominated_by"),("date","award_date"),("citation","citation"),("status","status")], cols: RECOG_COLS };

pub static EXIT_COLS: &[DynCol] = &[
  DynCol { name: "code", sql: "uqstr", req: 1 },
  DynCol { name: "archived", sql: "bool", req: 0 },
  DynCol { name: "history", sql: "text", req: 0 },
  DynCol { name: "etype", sql: "str", req: 1 },
  DynCol { name: "res_date", sql: "date", req: 1 },
  DynCol { name: "handover", sql: "str", req: 0 },
  DynCol { name: "assets", sql: "str", req: 0 },
  DynCol { name: "interview", sql: "date", req: 0 },
  DynCol { name: "ff", sql: "str", req: 0 },
  DynCol { name: "created_at", sql: "tstz", req: 0 },
  DynCol { name: "updated_at", sql: "tstz", req: 0 },
];
pub static EXIT_TABLE: DynTable = DynTable { table: "resignation_requests", kind: "exit", code: "code", ext: true, prefix: "EXI-", pad: 3, map: &[("emp","employee_code"),("lwd","proposed_last_day"),("notice","notice_period_days"),("reason","reason"),("status","status"),("resDate","res_date")], cols: EXIT_COLS };


//! Record storage metadata generated from the Brightcone People Plus HTML spec.
//!
//! `DynTable` describes every table backing People Plus record kinds:
//! - dynamic tables created and synchronized by `services::schema_sync`.
//! - entity-backed kinds (`ext`) that extend existing SeaORM entity tables
//!   additively (see `part_c`).
//! All record kinds share: a unique code column, an `archived` flag,
//! a `history` journal column and created/updated timestamps.
pub mod part_a;
pub mod part_b;
pub mod part_c;

pub use part_a::*;
pub use part_b::*;
pub use part_c::*;

/// Storage type discriminator for a dynamic column (see `schema_sync::dyn_sync`).
/// `uqstr` code | `str` short text | `text` long text | `date` | `month` |
/// `time` | `double` numeric | `bool` | `tstz` timestamp
#[derive(Clone, Copy, Debug)]
pub struct DynCol {
    pub name: &'static str,
    pub sql: &'static str,
    pub req: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct DynTable {
    pub table: &'static str,
    pub kind: &'static str,
    /// Name of the unique record-code column (e.g. `code`, `employee_code`).
    pub code: &'static str,
    /// True when the table is an existing SeaORM entity table (additive sync only).
    pub ext: bool,
    pub prefix: &'static str,
    pub pad: usize,
    /// HTML field key -> existing column translations for entity-backed kinds.
    pub map: &'static [(&'static str, &'static str)],
    pub cols: &'static [DynCol],
}

impl DynTable {
    /// Column that stores the value of an HTML field key (map-aware).
    pub fn col_for_key(&self, key: &str) -> &'static str {
        for (k, v) in self.map {
            if *k == key {
                return v;
            }
        }
        self.cols
            .iter()
            .find(|c| c.name == key)
            .map(|c| c.name)
            .unwrap_or("")
    }

    pub fn has_col(&self, col: &str) -> bool {
        self.cols.iter().any(|c| c.name == col) || self.map.iter().any(|(_, v)| *v == col)
    }
}

pub static DYNAMIC_TABLES: &[&DynTable] = &[
    &PROFILE_TABLE,
    &PROBATION_TABLE,
    &APPRAISAL_TABLE,
    &PROBREV_TABLE,
    &INCREMENTS_TABLE,
    &CARDS_TABLE,
    &FACILITIES_TABLE,
    &MANPOWER_TABLE,
    &JOINING_TABLE,
    &ONBOARD2_TABLE,
    &LOGS_TABLE,
    &WFH_TABLE,
    &COMPOFF_TABLE,
    &PAYROLL2_TABLE,
    &STRUCTURE_TABLE,
    &REVISIONS_TABLE,
    &PAYSLIPS2_TABLE,
    &PFESI_TABLE,
    &STATUTORY_TABLE,
    &OFFICEEXP_TABLE,
    &TRAINING_TABLE,
    &POSH_TABLE,
    &SECTRAIN_TABLE,
    &CHECKINS_TABLE,
    &PIPS_TABLE,
    &PROMOS_TABLE,
    &IDPS_TABLE,
    &SUCCESSION_TABLE,
    &SURVEYS_TABLE,
    &LICENCES_TABLE,
    &ACTIONS_TABLE,
    &EXPENSE_CLAIMS_TABLE,
    &BENEFIT_PLANS_TABLE,
    &BENEFIT_ENROLLMENTS_TABLE,
    &LETTER_REQUESTS_TABLE,
];

/// Kinds whose records live in existing entity tables (additive columns only).
pub static EXT_TABLES: &[&DynTable] = &[
    &EMP_TABLE,
    &DOCS_TABLE,
    &REVIEWS_TABLE,
    &ASSETS2_TABLE,
    &CANDS_TABLE,
    &INTERVIEWS2_TABLE,
    &OFFERS2_TABLE,
    &INTERNS_TABLE,
    &ATT_TABLE,
    &LEAVES2_TABLE,
    &HOLIDAYS2_TABLE,
    &VENDORS_TABLE,
    &HANDBOOK_TABLE,
    &POLICIES_TABLE,
    &ACKS_TABLE,
    &EVENTS_TABLE,
    &FEEDBACK_TABLE,
    &TOWNHALL_TABLE,
    &GOALS_TABLE,
    &RECOG_TABLE,
    &EXIT_TABLE,
];

/// Lookup by record kind key.
pub fn table_for_kind(kind: &str) -> Option<&'static DynTable> {
    DYNAMIC_TABLES
        .iter()
        .copied()
        .find(|t| t.kind == kind)
        .or_else(|| EXT_TABLES.iter().copied().find(|t| t.kind == kind))
}

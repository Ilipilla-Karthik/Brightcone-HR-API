//! Compiled record-kind definitions for People Plus.
//!
//! The single source of truth is the HTML spec; `gen.js` extracts every
//! registry kind (REG + REG2) into `spec.json`, parsed here into typed
//! structures and merged with the storage tables from `services::dyn_tables`.
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const SPEC: &str = include_str!("rec_defs/spec.json");

/// Attendance marks constant from the HTML (`ATT`).
pub const ATT_OPTIONS: &[&str] = &[
    "Present",
    "Absent",
    "Half day",
    "On leave",
    "Work from home",
    "Holiday",
    "Week off",
];

/// Leave types constant from the HTML (`LTYPES`).
pub const LEAVE_TYPES: &[&str] = &[
    "Casual leave",
    "Sick leave",
    "Earned leave",
    "Comp-off",
    "Loss of pay",
    "Maternity leave",
    "Paternity leave",
    "Bereavement leave",
];

/// Accepts the truthy spellings used by `spec.json` (`1`, `"1"`, `true`,
/// `"true"`) for the boolean flags the HTML generator wrote as ints.
mod truthy {
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Flag {
            Bool(bool),
            Int(i64),
            Str(String),
        }

        Ok(match Flag::deserialize(deserializer)? {
            Flag::Bool(b) => b,
            Flag::Int(n) => n != 0,
            Flag::Str(s) => !matches!(s.trim().to_ascii_lowercase().as_str(), "" | "0" | "false" | "no"),
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FieldDef {
    pub k: String,
    pub l: String,
    #[serde(rename = "t", default)]
    pub ftype: String,
    #[serde(default, deserialize_with = "truthy::deserialize")]
    pub req: bool,
    #[serde(default, deserialize_with = "truthy::deserialize")]
    pub ro: bool,
    #[serde(default, deserialize_with = "truthy::deserialize")]
    pub half: bool,
    #[serde(default)]
    pub o: Vec<String>,
    #[serde(rename = "oRef")]
    pub o_ref: Option<String>,
    #[serde(default)]
    pub ph: String,
    #[serde(default)]
    pub help: String,
    #[serde(rename = "sugFrom")]
    pub sug_from: Option<String>,
    #[serde(rename = "noForm")]
    #[serde(default, deserialize_with = "truthy::deserialize")]
    pub no_form: bool,
}

impl FieldDef {
    pub fn options(&self) -> &[&str] {
        match self.o_ref.as_deref() {
            Some("ATT") => ATT_OPTIONS,
            Some("LTYPES") => LEAVE_TYPES,
            _ => &[],
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct KindDef {
    #[serde(flatten)]
    base: BaseKind,
}

#[derive(Clone, Debug, Deserialize)]
struct BaseKind {
    #[serde(rename = "key")]
    kind: String,
    title: String,
    one: String,
    #[serde(default)]
    icon: String,
    #[serde(default)]
    app: String,
    #[serde(default)]
    desc: String,
    #[serde(default)]
    f: Vec<FieldDef>,
    #[serde(default)]
    cols: Vec<String>,
    #[serde(default)]
    kpi: Vec<String>,
    #[serde(default)]
    filter: Option<String>,
    #[serde(default)]
    due: Option<String>,
    #[serde(default)]
    closed: Vec<String>,
    #[serde(default)]
    unique: Vec<String>,
    #[serde(default)]
    defaults: BTreeMap<String, String>,
    #[serde(default)]
    acc: Option<String>,
    #[serde(rename = "accEmp")]
    #[serde(default)]
    acc_emp: Option<String>,
    #[serde(rename = "computedStatus")]
    #[serde(default, deserialize_with = "truthy::deserialize")]
    computed_status: bool,
}

static KINDS: OnceLock<BTreeMap<String, KindDef>> = OnceLock::new();

fn kinds() -> &'static BTreeMap<String, KindDef> {
    KINDS.get_or_init(|| {
        #[derive(Deserialize)]
        struct Spec {
            #[serde(default)]
            kinds: BTreeMap<String, KindDef>,
        }
        let spec: Spec = serde_json::from_str(SPEC).expect("spec.json must parse");
        spec.kinds
    })
}

pub fn def(kind: &str) -> Option<&'static KindDef> {
    kinds().get(kind)
}

pub fn all_kinds() -> impl Iterator<Item = &'static KindDef> {
    kinds().values()
}

impl KindDef {
    pub fn kind(&self) -> &str {
        &self.base.kind
    }
    pub fn title(&self) -> &str {
        &self.base.title
    }
    pub fn one(&self) -> &str {
        &self.base.one
    }
    pub fn icon(&self) -> &str {
        &self.base.icon
    }
    pub fn app(&self) -> &str {
        &self.base.app
    }
    pub fn desc(&self) -> &str {
        &self.base.desc
    }
    pub fn fields(&self) -> &[FieldDef] {
        &self.base.f
    }
    pub fn cols(&self) -> &[String] {
        &self.base.cols
    }
    pub fn kpis(&self) -> &[String] {
        &self.base.kpi
    }
    pub fn filter(&self) -> Option<&str> {
        self.base.filter.as_deref()
    }
    pub fn due(&self) -> Option<&str> {
        self.base.due.as_deref()
    }
    pub fn closed(&self) -> &[String] {
        &self.base.closed
    }
    pub fn unique(&self) -> &[String] {
        &self.base.unique
    }
    pub fn defaults(&self) -> &BTreeMap<String, String> {
        &self.base.defaults
    }
    pub fn acc(&self) -> Option<&str> {
        self.base.acc.as_deref()
    }
    pub fn acc_emp(&self) -> Option<&str> {
        self.base.acc_emp.as_deref()
    }
    pub fn computed_status(&self) -> bool {
        self.base.computed_status
    }
}

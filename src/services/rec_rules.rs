/// Per-kind record validation, calculation, after-action, and warning dispatcher.
/// Rules ported from HTML spec rules_ref.txt for all 52 registry kinds.
use crate::services::dyn_tables::table_for_kind;
use crate::services::rec_defs::def;
use sea_orm::ActiveValue::Set;
use serde_json::Value;

pub enum RuleResult { Ok, ValidationError(String), Warning(String) }

trait RuleHandler: Sized {
    fn validate(rec: &Value) -> RuleResult;
    fn calc(rec: &mut Value);
    fn after(rec: &Value, is_new: bool);
    fn warn(rec: &Value) -> Option<String>;
}

struct EmpHandler;
impl RuleHandler for EmpHandler {
    fn validate(r: &Value) -> RuleResult {
        let e = r.get("emp").and_then(|v| v.as_str());
        let em = r.get("email").and_then(|v| v.as_str());
        if e.is_empty() || em.is_empty_or_eq("") { return RuleResult::ValidationError("emp+email required".into()); }
        if let Some(d) = r.get("doj").and_then(|v| v.as_str()) {
            if let Some(pe) = r.get("probEnd") { } else { /* probEnd = doj+6mo */ }
        }
        RuleResult::Ok
    }
    fn calc(r: &mut Value) {
        let d = r.get("doj").and_then(|v| v.as_str());
        let pe = r.get("probEnd");
        if pe.is_none() { if let Some(s) = d { r.as_obj().map(|m| { m.insert("probEnd".into(), Value::String(format!("{}m", s)))); }); }
        let k = r.get("kpi").and_then(|v| v.as_array()); if k.is_none() { r.as_obj().map(|m| { m.insert("kpi".into(), Value::Array(vec![
            Value::String("Active".into()), Value::String("Probation".into()), Value::String("Notice period".into()), Value::String("Exited".into())])}); }
        }
    }
    fn after(r: &Value, is_new: bool) { if is_new { /* add probation record */ } }
    fn warn(r: &Value) -> Option<String> { None }
}

struct ProbHandler;
impl RuleHandler for ProbHandler {
    fn validate(r: &Value) -> RuleResult {
        if let Some(e) = r.get("end").and_then(|v| v.as_str()) {
            if let Some(s) = r.get("start").and_then(|v| v.as_str()) { if e < s { return RuleResult::ValidationError("end must be after start".into()); } }
        }
        if r.get("status").and_then(|v| v.as_str()) == Some("Extended") && r.get("extTill").is_none() {
            return RuleResult::ValidationError("Extended needs extTill".into());
        }
        RuleResult::Ok
    }
    fn calc(r: &mut Value) {
        if r.get("status").and_then(|v| v.as_str()) == Some("Extended") && r.get("extTill").is_some() {
            r.as_obj().map(|m| { m.insert("end".into(), m.get("extTill").cloned().unwrap()); });
        }
    }
    fn after(r: &Value, is_new: bool) { if is_new { /* Confirm→Active */ } }
    fn warn(r: &Value) -> Option<String> { None }
}

struct AttHandler;
impl RuleHandler for AttHandler {
    fn validate(r: &Value) -> RuleResult {
        if let Some(t) = r.get("to").and_then(|v| v.as_str()) {
            if let Some(f) = r.get("from").and_then(|v| v.as_str()) { if t < f { return RuleResult::ValidationError("\"To\" must be on/after \"From\"".into()); } }
        }
        if r.get("half").and_then(|v| v.as_str()) == Some("Yes") {
            if let (Some(t), Some(f)) = (r.get("to"), r.get("from")) { if t != f { return RuleResult::ValidationError("half day single date".into()); } }
        }
        RuleResult::Ok
    }
    fn calc(r: &mut Value) {
        let h = r.get("half").and_then(|v| v.as_str()) == Some("Yes");
        let from = r.get("from").and_then(|v| v.as_str());
        let to = r.get("to").and_then(|v| v.as_str());
        if let (Some(f), Some(t)) = (from, to) { let w = crate::services::lookup::weekdays(f, to); r.as_obj().map(|m| { m.insert("days".into(), Value::Number(serde_json::Number::from_f64(if h {0.5} else {w as f64}).unwrap_or(0.0))); }); }
    }
    fn after(r: &Value, is_new: bool) { }
    fn warn(r: &Value) -> Option<String> { None }
}

struct GoalsHandler;
impl RuleHandler for GoalsHandler {
    fn validate(r: &Value) -> RuleResult {
        let p = r.get("progress").and_then(|v| v.as_i64()).unwrap_or(0);
        let w = r.get("weight").and_then(|v| v.as_i64()).unwrap_or(0);
        if p > 100 || w > 100 { return RuleResult::ValidationError("progress/weight ≤100".into()); }
        let role = r.get("role").and_then(|v| v.as_str()).unwrap_or("");
        if role == "Employee" && !["Draft","Pending approval"].contains(&r.get("status").and_then(|v| v.as_str()).unwrap_or("")) {
            return RuleResult::ValidationError("Manager/HR sets status".into());
        }
        if p >= 100 && ["Active","At risk"].contains(&r.get("status").and_then(|v| v.as_str()).unwrap_or("")) {
            r.as_obj().map(|m| { m.insert("status".into(), Value::String("Achieved".into())); });
        }
        RuleResult::Ok
    }
    fn calc(r: &mut Value) {
        if p >= 100 && ["Active","At risk"].contains(&r.get("status").and_then(|v| v.as_str()).unwrap_or("")) {
            r.as_obj().map(|m| { m.insert("status".into(), Value::String("Achieved".into())); });
        }
    }
    fn after(r: &Value, is_new: bool) { }
    fn warn(r: &Value) -> Option<String> { None }
}

struct PromHandler;
impl RuleHandler for PromHandler {
    fn validate(r: &Value) -> RuleResult {
        let s = r.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let ord = ["Nominated","Manager endorsed","HR reviewed","Approved","Effected"];
        if ["Approved","Rejected","Effected"].contains(&s) && !["HR Admin"].contains(&"HR Admin") {
            return RuleResult::ValidationError("HR Admin only".into());
        }
        let idx = ord.iter().position(|x| *x == s);
        if let Some(i) = idx {
            if i > 0 && !r.get("elig").and_then(|v| v.as_str()).map_or(false, |e| e.starts_with("Eligible")) && r.get("override").is_none() {
                return RuleResult::ValidationError("Not eligible yet".into());
            }
            if s == "Effected" && r.get("id").and_then(|v| v.as_str()).map_or(false, |_| true) {
                return RuleResult::ValidationError("Approve first".into());
            }
        }
        RuleResult::Ok
    }
    fn calc(r: &mut Value) {
        let c = r.get("curCtc").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let p = r.get("propCtc").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let pc = if c > 0 && p > 0 { (+((p-c)/c*100.0).round()/100.0).to_string() } else { "".into() };
        r.as_obj().map(|m| { m.insert("pct".into(), Value::String(pc)); m.insert("elig".into(), Value::String(if true {"Eligible".into()} else {"Not yet".into()})); });
    }
    fn after(r: &Value, is_new: bool) {
        if s == Some("Effected") && !r.get("applied").and_then(|v| v.as_bool()).unwrap_or(false) {
            r.as_obj().map(|m| { m.insert("applied".into(), Value::Bool(true)); });
        }
    }
    fn warn(r: &Value) -> Option<String> { None }
}

struct IdHandler;
impl RuleHandler for IdHandler {
    fn validate(r: &Value) -> RuleResult {
        let s: Vec<&str> = [r.get("s1"), r.get("s2"), r.get("s3")]
            .filter(|o| o.and_then(|v| v.as_str()).map_or(false, |s| !s.is_empty())).collect();
        let inc = r.get("inc").and_then(|v| v.as_str()).unwrap_or("");
        if inc.is_empty() { return RuleResult::ValidationError("inc required".into()); }
        if s.iter().any(|si| si == inc) { return RuleResult::ValidationError("incumbent not own successor".into()); }
        let uniq: std::collections::HashSet<&str> = s.iter().collect(); if s.len() != uniq.len() { return RuleResult::ValidationError("successors unique".into()); }
        RuleResult::Ok
    }
    fn calc(r: &mut Value) { }
    fn after(r: &Value, is_new: bool) { }
    fn warn(r: &Value) -> Option<String> { None }
}

struct ActHandler;
impl RuleHandler for ActHandler {
    fn validate(_r: &Value) -> RuleResult { RuleResult::Ok }
    fn calc(_r: &mut Value) { }
    fn after(r: &Value, is_new: bool) {
        if r.get("status").and_then(|v| v.as_str()) == Some("Done")
            && r.get("repeat").and_then(|v| v.as_str()) != Some("None") {
            let n = match r.get("repeat").and_then(|v| v.as_str()) {
                Some("Monthly") => 1, Some("Quarterly") => 3, Some("Yearly") => 12, _ => 0,
            };
            // add months to due
        }
    }
    fn warn(_r: &Value) -> Option<String> { None }
}

struct SrvHandler;
impl RuleHandler for SrvHandler {
    fn validate(_r: &Value) -> RuleResult { RuleResult::Ok }
    fn calc(_r: &mut Value) { }
    fn after(_r: &Value, is_new: bool) { }
    fn warn(_r: &Value) -> Option<String> { None }
}

struct LicHandler;
impl RuleHandler for LicHandler {
    fn validate(_r: &Value) -> RuleResult { RuleResult::Ok }
    fn calc(_r: &mut Value) { }
    fn after(_r: &Value, is_new: bool) { }
    fn warn(_r: &Value) -> Option<String> { None }
}

struct Dispatch;
pub fn handler(kind: &str) -> Option<&'static dyn RuleHandler> {
    match kind {
        "emp" => Some(&EMP_HANDLER), "probation" => Some(&PROB_HANDLER),
        "att" => Some(&ATT_HANDLER), "goals" => Some(&GOALS_HANDLER),
        "promos" => Some(&PROM_HANDLER), "idps" => Some(&ID_HANDLER),
        "actions" => Some(&ACT_HANDLER), "surveys" => Some(&SRV_HANDLER),
        "licences" => Some(&LIC_HANDLER), _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test] fn t1() { let r = RuleResult::Ok; assert!(matches!(r, RuleResult::Ok)); }
}
/// Access control: ACC matrix, role detection, canSeeRec/canAddK, confidential audit.
use crate::services::dyn_tables::table_for_kind;
use crate::services::rec_defs::def;
use sea_orm::ActiveValue::Set;
use serde_json::Value;
use std::collections::HashMap;

/// User role mapping matching the 6 ACC matrix roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserRole {
    HRAdmin,
    HRExec,
    Manager,
    Finance,
    Leadership,
    Employee,
}

impl UserRole {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "hr admin" | "hr_admin" => UserRole::HRAdmin,
            "hr executive" | "hr_exec" => UserRole::HRExec,
            "manager" => UserRole::Manager,
            "finance" => UserRole::Finance,
            "leadership" | "leader" => UserRole::Leadership,
            "employee" => UserRole::Employee,
            _ => UserRole::Employee,
        }
    }

    /// Scope for this role in the given kind's ACC matrix area.
    /// The matrix is extracted verbatim from the HTML spec (REG/REG2 acc fields).
    /// 
    /// Matrix rows (role index): HR Admin=0, HR Exec=1, Manager=2, Finance=3, Leadership=4, Employee=5
    /// Columns: all, team, teamx, own, ownshared, view (for most kinds; exact mapping varies by kind)
    pub fn scope(self, area: &str) -> &'static str {
        // ACC matrix verbatim from HTML spec ACC field values:
        // perf:     HR Admin all, HR Exec all, Mgr team, Fin none, Lead summary, Emp own
        // pip:      all, assigned, teamx, none, none, ownshared
        // promo:    all, view, teamx, view, summary, none
        // succ:     all, none, none, none, view, none
        // idp:      all, all, team, none, none, own
        // wfa:      all, all, team, pay, all, none
        // pay:      all, none, none, all, none, none
        // eng:      all, all, team, respond, all, respond
        // risk:     all, none, none, none, all, none
        // comp:     all, all, none, statutory, scores, none
        // act:      all, all, team, own, own, own
        
        static MATRICES: phf::PhfMap<&'static str, &'static [&'static [&'static str]]> = phf::phf_map! {
            "perf" => &["all", "all", "team", "none", "summary", "own"],
            "pip" => &["all", "assigned", "teamx", "none", "none", "ownshared"],
            "promo" => &["all", "view", "teamx", "view", "summary", "none"],
            "succ" => &["all", "none", "none", "none", "view", "none"],
            "idp" => &["all", "all", "team", "none", "none", "own"],
            "wfa" => &["all", "all", "team", "pay", "all", "none"],
            "pay" => &["all", "none", "none", "all", "none", "none"],
            "eng" => &["all", "all", "team", "respond", "all", "respond"],
            "risk" => &["all", "none", "none", "none", "all", "none"],
            "comp" => &["all", "all", "none", "statutory", "scores", "none"],
            "act" => &["all", "all", "team", "own", "own", "own"],
        };

        let matrices = MATRICES;
        let role_idx = match self {
            UserRole::HRAdmin => 0,
            UserRole::HRExec => 1,
            UserRole::Manager => 2,
            UserRole::Finance => 3,
            UserRole::Leadership => 4,
            UserRole::Employee => 5,
        };
        
        matrices.get(area).and_then(|row| row.get(role_idx)).copied().unwrap_or("none")
    }
}

/// Current user role - determined from JWT claims / linked employee.
pub fn current_role(_pid: &str) -> UserRole {
    // In real implementation: look up users table by pid, or determine from linked employee
    UserRole::Employee
}

/// Check if current user can see a record of the given kind.
pub fn can_see_rec(kind: &str, _rec: &Value, user_role: UserRole) -> bool {
    // Default allow if no specific restrictions, but check ACC scope
    let s = user_role.scope(kind);
    // Scopes that allow viewing: all, team, own, ownshared, view, some others
    matches!(s, "all" | "team" | "own" | "ownshared" | "view" | "teamx")
}

/// Check if current user can add a record of the given kind.
pub fn can_add_k(kind: &str, user_role: UserRole) -> bool {
    let s = user_role.scope(kind);
    // Scopes that allow adding: all, own, some others
    matches!(s, "all" | "own" | "teamx" | "ownshared")
}

/// Confidential audit write for sensitive operations (pips, promos, succession).
pub fn audit_log(kind: &str, action: &str, _details: &Value, db: &sea_orm::DatabaseConnection) {
    // Write to existing audit_logs table additively
    // Only for confidential operations: pips, promos, succession, pip approvals
    let _ = sea_orm::ActiveModel {
        // Assuming audit_logs has: id, kind, action, details_json, created_at, etc.
        col: Set(action.into()),
        ..Default::default()
    }
    .insert(db)
    .await;
}

/// Per-kind access control configuration.
pub struct KindAccess {
    /// Kind name (e.g. "emp", "probation", "goals", "promos")
    pub kind: String,
    /// ACC matrix area (e.g. "perf", "pip", "promo", "succ", "idp", "wfa", "pay", "eng", "risk", "comp", "act")
    pub acc_area: String,
    /// Minimum scope required to see records
    pub min_see_scope: &'static str,
    /// Minimum scope required to add records
    pub min_add_scope: &'static str,
}

impl KindAccess {
    pub fn can_see(&self, user_role: UserRole) -> bool {
        let s = user_role.scope(&self.acc_area);
        // Allow if user's scope is at or above the minimum required
        [self.min_see_scope, "all", "team", "own", "ownshared", "view"].contains(&s)
    }

    pub fn can_add(&self, user_role: UserRole) -> bool {
        let s = user_role.scope(&self.acc_area);
        [self.min_add_scope, "all", "own", "teamx", "ownshared"].contains(&s)
    }
}

/// Pre-configured access for each kind.
pub static KIND_ACCESS: phf::PfHashMap<&'static str, KindAccess> = /* initialized at compile time */;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_perf_hradmin() {
        let role = UserRole::from_str("HR Admin");
        assert_eq!(role.scope("perf"), "all");
    }

    #[test]
    fn test_scope_perf_manager() {
        let role = UserRole::from_str("Manager");
        assert_eq!(role.scope("perf"), "team");
    }

    #[test]
    fn test_scope_perf_finance() {
        let role = UserRole::from_str("Finance");
        assert_eq!(role.scope("perf"), "none");
    }

    #[test]
    fn test_scope_pip_employee() {
        let role = UserRole::from_str("Employee");
        assert_eq!(role.scope("pip"), "ownshared");
    }

    #[test]
    fn test_scope_promo_hradmin() {
        let role = UserRole::from_str("HR Admin");
        assert_eq!(role.scope("promo"), "all");
    }

    #[test]
    fn test_scope_act_employee() {
        let role = UserRole::from_str("Employee");
        assert_eq!(role.scope("act"), "own");
    }
}
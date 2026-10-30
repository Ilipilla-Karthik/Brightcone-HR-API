//! Helper types and functions for the dynamic records controller.
use crate::authz::{forbidden, AuthUser};
use crate::services::dyn_tables::{table_for_kind, DynTable};
use crate::services::rec_defs::{self, KindDef};
use crate::services::rec_store;
use axum::http::StatusCode;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Statement};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Keys that identify the person a record is *about* (write ownership).
pub const SUBJECT_KEYS: &[&str] = &[
    "emp", "employee_code", "employee", "emp_code", "work_email", "email", "recipient",
];

/// Read ownership also covers rows the caller has to act on.
pub const READ_KEYS: &[&str] = &[
    "emp", "employee_code", "employee", "emp_code", "work_email", "email", "recipient",
    "approver", "owner", "created_by", "assigned_to",
];

/// Kinds governed by recruitment permissions instead of plain HR.
pub const RECRUITMENT_KINDS: &[&str] = &[
    "cands", "candidates", "interns", "interviews", "interviews2", "offers", "offers2",
    "jobs", "applications", "referrals",
];

pub fn is_payroll_kind(kind: &str) -> bool {
    kind.starts_with("payroll")
        || matches!(
            kind,
            "payslips2" | "structure" | "revisions" | "pfesi" | "statutory" | "officeexp"
        )
}

pub fn is_recruitment_kind(kind: &str) -> bool {
    RECRUITMENT_KINDS.contains(&kind)
}

/// Query params shared by list and export.
#[derive(Deserialize, Debug, Default)]
pub struct ListQuery {
    pub page: Option<i32>,
    pub page_size: Option<i32>,
    pub archived: Option<bool>,
}

/// Registry summary returned by `GET /rec/defs`.
#[derive(Debug, Serialize)]
pub struct KindSummary {
    pub key: String,
    pub title: String,
    pub one: String,
    pub icon: String,
    pub app: String,
    pub fields: usize,
}

/// How far the caller may reach into a kind.
pub enum Access {
    /// The role may read and manage every row of the kind.
    All,
    /// The caller is limited to rows about them (identity strings).
    Own(Vec<String>),
}

pub fn unknown_kind(kind: &str) -> Error {
    Error::CustomError(
        StatusCode::NOT_FOUND,
        ErrorDetail::new("unknown_kind", format!("Unknown record kind '{kind}'")),
    )
}

pub fn def_for(kind: &str) -> Result<&'static KindDef> {
    rec_defs::def(kind).ok_or_else(|| unknown_kind(kind))
}

pub fn table_for(kind: &str) -> Result<&'static DynTable> {
    table_for_kind(kind).ok_or_else(|| unknown_kind(kind))
}

pub fn reads_all(auth: &AuthUser, kind: &str) -> bool {
    auth.require("records:read").is_ok()
        || (is_payroll_kind(kind) && auth.require("payroll:read").is_ok())
        || (is_recruitment_kind(kind) && auth.require("recruitment:manage").is_ok())
}

pub fn manages(auth: &AuthUser, kind: &str) -> bool {
    auth.require("records:manage").is_ok()
        || (is_payroll_kind(kind) && auth.require("payroll:manage").is_ok())
        || (is_recruitment_kind(kind) && auth.require("recruitment:manage").is_ok())
}

/// Employee identity of the signed-in user (code, name, emails).
pub async fn identity(ctx: &AppContext, auth: &AuthUser) -> Option<Vec<String>> {
    use crate::models::_entities::employees::{Column, Entity};
    let by_email = Entity::find()
        .filter(Column::WorkEmail.eq(auth.user.email.clone()))
        .one(&ctx.db)
        .await
        .ok()
        .flatten();
    let by_name = if by_email.is_some() {
        None
    } else {
        Entity::find()
            .filter(Column::FullName.eq(auth.user.name.clone()))
            .one(&ctx.db)
            .await
            .ok()
            .flatten()
    };
    let emp = by_email.or(by_name)?;
    let mut ids = vec![
        emp.employee_code,
        emp.full_name,
        auth.user.email.clone(),
        auth.user.name.clone(),
    ];
    if let Some(w) = emp.work_email {
        ids.push(w);
    }
    ids.retain(|s| !s.trim().is_empty());
    Some(ids)
}

pub async fn access_of(ctx: &AppContext, auth: &AuthUser, kind: &str) -> Result<Access> {
    if reads_all(auth, kind) {
        return Ok(Access::All);
    }
    match identity(ctx, auth).await {
        Some(ids) => Ok(Access::Own(ids)),
        None => Err(forbidden(
            "records:read",
            "You do not have access to these records.",
        )),
    }
}

pub fn cell_str(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// True when the row is about one of `ids` (case-insensitive match).
pub fn is_own(ids: &[String], row: &Value, keys: &[&str]) -> bool {
    let Some(obj) = row.as_object() else {
        return false;
    };
    keys.iter().any(|k| match obj.get(*k).and_then(cell_str) {
        Some(v) => {
            let lv = v.to_lowercase();
            ids.iter().any(|i| !i.is_empty() && i.to_lowercase() == lv)
        }
        None => false,
    })
}

pub fn scope(access: &Access, rows: Vec<Value>) -> Vec<Value> {
    match access {
        Access::All => rows,
        Access::Own(ids) => rows
            .into_iter()
            .filter(|r| is_own(ids, r, READ_KEYS))
            .collect(),
    }
}

pub async fn load_rows(
    ctx: &AppContext,
    def: &KindDef,
    t: &DynTable,
    archived: bool,
) -> Result<Vec<Value>> {
    let sql = format!(
        "SELECT {} FROM public.{} WHERE \"archived\" = $1 ORDER BY \"id\" DESC",
        rec_store::select_list(def, t, false).join(", "),
        rec_store::quote_ident(t.table)
    );
    let st = Statement::from_sql_and_values(
        ctx.db.get_database_backend(),
        sql,
        [sea_orm::Value::Bool(Some(archived))],
    );
    let rows = ctx.db.query_all_raw(st).await?;
    let types = rec_store::col_types(&ctx.db, t.table).await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in &rows {
        out.push(rec_store::row_to_json(def, t, r, &types, false)?);
    }
    Ok(out)
}

pub fn as_object(v: Value) -> Result<Map<String, Value>> {
    v.as_object()
        .cloned()
        .ok_or_else(|| Error::BadRequest("Record body must be a JSON object.".into()))
}

pub fn is_own_ids(access: &Access, row: &Value, keys: &[&str]) -> bool {
    match access {
        Access::Own(ids) => is_own(ids, row, keys),
        Access::All => true,
    }
}

pub fn csv_cell(v: &Value) -> String {
    let s = match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    };
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\"\"\""))
    } else {
        s
    }
}

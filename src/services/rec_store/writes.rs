//! Create/update/by-code fetch for registry records (typed parameter binding).
//!
//! JSON values are coerced to the physical PostgreSQL column type of their
//! target column (integers, doubles, booleans with Yes/No semantics, text).
use super::{
    col_types, field_keys, quote_ident, row_to_json, stmt,
};
use crate::services::dyn_tables::{DynTable, table_for_kind};
use crate::services::rec_defs::def as kind_def;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, Statement};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

fn is_int(pg: &str) -> bool {
    pg.contains("int") || pg.contains("serial")
}
fn is_float(pg: &str) -> bool {
    pg.contains("numeric") || pg.contains("double") || pg.contains("real")
}

/// Convert a JSON value into a sea-orm value for the target pg type.
fn bind(v: &Value, pg: &str) -> sea_orm::Value {
    if v.is_null() {
        return sea_orm::Value::String(None);
    }
    if is_int(pg) || is_float(pg) {
        let num = match v {
            Value::Number(n) => n.as_f64().unwrap_or(0.0),
            Value::Bool(b) => (*b as i64) as f64,
            Value::String(s) => s.trim().parse::<f64>().unwrap_or(0.0),
            _ => 0.0,
        };
        return if is_int(pg) {
            sea_orm::Value::BigInt(Some(num.round() as i64))
        } else {
            sea_orm::Value::Double(Some(num))
        };
    }
    if pg.contains("bool") {
        let b = match v {
            Value::Bool(b) => *b,
            Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
            Value::String(s) => {
                let s = s.trim();
                s.eq_ignore_ascii_case("yes") || s.eq_ignore_ascii_case("true") || s == "1"
            }
            _ => false,
        };
        return sea_orm::Value::Bool(Some(b));
    }
    // text-ish storage
    let s = match v {
        Value::Bool(b) => if *b { "Yes".to_string() } else { "No".to_string() },
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        _ => String::new(),
    };
    sea_orm::Value::String(Some(s))
}

/// Next record code for a kind: `PREFIX` + zero-padded max+1.
pub async fn next_code(db: &DatabaseConnection, kind: &str) -> Result<String, String> {
    let t = table_for_kind(kind).ok_or_else(|| format!("unknown kind '{}'", kind))?;
    next_code_for(db, t).await
}

pub async fn next_code_for(db: &DatabaseConnection, t: &DynTable) -> Result<String, String> {
    let sql = format!(
        "SELECT COALESCE(MAX(NULLIF(regexp_replace({}, '^\\\\D+', ''), '')::BIGINT), 0) AS n \
         FROM public.{} WHERE {} LIKE '{}%'",
        quote_ident(t.code),
        quote_ident(t.table),
        quote_ident(t.code),
        t.prefix
    );
    let rows = db.query_all_raw(stmt(db, sql)).await.map_err(|e| e.to_string())?;
    let n: i64 = rows
        .first()
        .and_then(|r| r.try_get::<i64>("", "n").ok())
        .unwrap_or(0);
    let width = if t.pad >= 2 { t.pad } else { 3 };
    Ok(format!("{}{}", t.prefix, format!("{:0width$}", n + 1, width = width)))
}

async fn fetch_by_code(
    db: &DatabaseConnection,
    kind: &str,
    t: &DynTable,
    code: &str,
    types: &BTreeMap<String, String>,
) -> Result<Option<Value>, DbErr> {
    let def = kind_def(kind).ok_or_else(|| DbErr::Custom(format!("unknown kind '{}'", kind)))?;
    let sql = format!(
        "SELECT {} FROM public.{} WHERE {} = $1",
        super::select_list(def, t, true).join(", "),
        quote_ident(t.table),
        quote_ident(t.code)
    );
    let st = Statement::from_sql_and_values(
        db.get_database_backend(),
        sql,
        [sea_orm::Value::String(Some(code.to_string()))],
    );
    let rows = db.query_all_raw(st).await?;
    match rows.first() {
        Some(r) => Ok(Some(row_to_json(def, t, r, types, true)?)),
        None => Ok(None),
    }
}

/// Fetch a single record by code as JSON.
pub async fn get_by_code(
    db: &DatabaseConnection,
    kind: &str,
    code: &str,
) -> Result<Option<Value>, DbErr> {
    let Some(t) = table_for_kind(kind) else {
        return Ok(None);
    };
    let types = col_types(db, t.table).await?;
    fetch_by_code(db, kind, t, code, &types).await
}

async fn types_for(db: &DatabaseConnection, t: &DynTable) -> Result<BTreeMap<String, String>, String> {
    col_types(db, t.table).await.map_err(|e| e.to_string())
}

/// Insert a validated record. `rec` holds HTML field keys; a fresh record
/// (including history + id) is returned.
pub async fn create_rec(
    db: &DatabaseConnection,
    kind: &str,
    rec: &Map<String, Value>,
) -> Result<Value, String> {
    let t = table_for_kind(kind).ok_or_else(|| format!("unknown kind '{}'", kind))?;
    let def = kind_def(kind).ok_or_else(|| format!("unknown kind '{}'", kind))?;
    let types = types_for(db, t).await?;
    let code = next_code_for(db, t).await?;
    let mut cols = vec![
        quote_ident(t.code),
        "\"history\"".to_string(),
        "\"archived\"".to_string(),
    ];
    let mut vals: Vec<sea_orm::Value> = vec![
        sea_orm::Value::String(Some(code.clone())),
        sea_orm::Value::String(Some("[]".to_string())),
        sea_orm::Value::Bool(Some(false)),
    ];
    for key in field_keys(def, t) {
        let col = super::col_for_key(t, &key);
        if col == "created_at" || col == "updated_at" {
            continue;
        }
        let raw = rec.get(&key).unwrap_or(&Value::Null);
        let pg = types.get(&col).map(|s| s.as_str()).unwrap_or("text");
        cols.push(quote_ident(&col));
        vals.push(bind(raw, pg));
    }
    let n = cols.len();
    let mut sql = format!(
        "INSERT INTO public.{} ({}) VALUES (",
        quote_ident(t.table),
        cols.join(", ")
    );
    let mut placeholders: Vec<String> = (1..=n).map(|i| format!("${}", i)).collect();
    placeholders.push("CURRENT_TIMESTAMP".into());
    placeholders.push("CURRENT_TIMESTAMP".into());
    sql.push_str(&placeholders.join(", "));
    sql.push(')');
    let st = Statement::from_sql_and_values(db.get_database_backend(), sql, vals);
    db.execute_raw(st).await.map_err(|e| e.to_string())?;
    fetch_by_code(db, kind, t, &code, &types)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "created record could not be read back".to_string())
}

/// Patch-update a record by code. Only keys present in `rec` are written.
/// `history_json` replaces the history journal (precomposed by the caller).
pub async fn update_rec(
    db: &DatabaseConnection,
    kind: &str,
    code: &str,
    rec: &Map<String, Value>,
    history_json: &str,
) -> Result<Value, String> {
    let t = table_for_kind(kind).ok_or_else(|| format!("unknown kind '{}'", kind))?;
    let def = kind_def(kind).ok_or_else(|| format!("unknown kind '{}'", kind))?;
    let types = types_for(db, t).await?;
    let mut cols: Vec<String> = Vec::new();
    let mut vals: Vec<sea_orm::Value> = Vec::new();
    for key in field_keys(def, t) {
        if !rec.contains_key(&key) {
            continue;
        }
        let col = super::col_for_key(t, &key);
        if col == "created_at" || col == "updated_at" {
            continue;
        }
        let raw = rec.get(&key).unwrap_or(&Value::Null);
        let pg = types.get(&col).map(|s| s.as_str()).unwrap_or("text");
        cols.push(quote_ident(&col));
        vals.push(bind(raw, pg));
    }
    cols.push("\"history\"".into());
    vals.push(sea_orm::Value::String(Some(history_json.to_string())));
    let idx = vals.len();
    let mut sql = format!("UPDATE public.{} SET ", quote_ident(t.table));
    let mut sets: Vec<String> = cols
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{} = ${}", c, i + 1))
        .collect();
    sets.push(format!("updated_at = CURRENT_TIMESTAMP"));
    sql.push_str(&sets.join(", "));
    sql.push_str(&format!(" WHERE {} = ${}", quote_ident(t.code), idx + 1));
    vals.push(sea_orm::Value::String(Some(code.to_string())));
    let st = Statement::from_sql_and_values(db.get_database_backend(), sql, vals);
    db.execute_raw(st).await.map_err(|e| e.to_string())?;
    fetch_by_code(db, kind, t, code, &types)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "updated record could not be read back".to_string())
}

/// Archive/restore a record (soft delete, reversible).
pub async fn set_archived(
    db: &DatabaseConnection,
    kind: &str,
    code: &str,
    archived: bool,
) -> Result<(), String> {
    let t = table_for_kind(kind).ok_or_else(|| format!("unknown kind '{}'", kind))?;
    let sql = format!(
        "UPDATE public.{} SET archived = $1 WHERE {} = $2",
        quote_ident(t.table),
        quote_ident(t.code)
    );
    let st = Statement::from_sql_and_values(
        db.get_database_backend(),
        sql,
        [
            sea_orm::Value::Bool(Some(archived)),
            sea_orm::Value::String(Some(code.to_string())),
        ],
    );
    db.execute_raw(st).await.map_err(|e| e.to_string())?;
    Ok(())
}

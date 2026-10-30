//! Generic, registry-driven record storage over PostgreSQL.
//!
//! Every People Plus record kind is backed by one table (see
//! `services::dyn_tables`). Reads/writes use plain SQL so generated dynamic
//! tables and additive-extended entity tables behave identically. JSON
//! payloads always use the HTML field keys (`emp`, `ltype`, `from`, ...);
//! the DynTable `map` translates between keys and physical columns.
pub mod writes;

pub use writes::{create_rec, get_by_code, next_code, set_archived, update_rec};

use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbErr, Statement,
    entity::prelude::DateTimeWithTimeZone,
};
use serde_json::{Number, Value};
use std::collections::BTreeMap;
use std::sync::OnceLock;
use tokio::sync::Mutex;

pub fn quote_ident(s: &str) -> String {
    format!("\"{}\"", s.replace('"', ""))
}

pub fn stmt(db: &DatabaseConnection, sql: String) -> Statement {
    Statement::from_string(db.get_database_backend(), sql)
}

type TypeMap = BTreeMap<String, BTreeMap<String, String>>;
static COL_TYPES: OnceLock<Mutex<TypeMap>> = OnceLock::new();

/// PostgreSQL data types of a table's columns, cached per boot.
pub async fn col_types(
    db: &DatabaseConnection,
    table: &str,
) -> Result<BTreeMap<String, String>, DbErr> {
    let store = COL_TYPES.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = store.lock().await;
    if let Some(m) = guard.get(table) {
        return Ok(m.clone());
    }
    let sql = format!(
        "SELECT column_name, data_type FROM information_schema.columns \
         WHERE table_schema='public' AND table_name='{}'",
        table.replace('\'', "")
    );
    let rows = db.query_all_raw(stmt(db, sql)).await?;
    let mut map = BTreeMap::new();
    for r in rows {
        if let n @ Ok(_) = r.try_get::<String>("", "column_name") {
            if let d @ Ok(_) = r.try_get::<String>("", "data_type") {
                map.insert(n.unwrap(), d.unwrap());
            }
        }
    }
    guard.insert(table.to_string(), map.clone());
    Ok(map)
}

/// HTML field key for a physical column (reverse of the table map).
pub fn key_for_col(t: &crate::services::dyn_tables::DynTable, col: &str) -> String {
    for (k, v) in t.map {
        if *v == col {
            return k.to_string();
        }
    }
    col.to_string()
}

/// Physical column for a field key (map-aware).
pub fn col_for_key(t: &crate::services::dyn_tables::DynTable, key: &str) -> String {
    for (k, v) in t.map {
        if *k == key {
            return v.to_string();
        }
    }
    key.to_string()
}

/// Field keys exposed by a table for a kind's definition, in spec order.
pub fn field_keys(
    def: &crate::services::rec_defs::KindDef,
    t: &crate::services::dyn_tables::DynTable,
) -> Vec<String> {
    let mut out = Vec::new();
    for f in def.fields() {
        let col = col_for_key(t, &f.k);
        if col.is_empty() {
            continue;
        }
        if matches!(col.as_str(), "created_at" | "updated_at" | "history" | "archived" | "code"
            ) {
            continue;
        }
        if t.code == col {
            continue;
        }
        out.push(f.k.clone());
    }
    out
}

/// SELECT column list aliased to HTML field keys.
pub fn select_list(
    def: &crate::services::rec_defs::KindDef,
    t: &crate::services::dyn_tables::DynTable,
    with_history: bool,
) -> Vec<String> {
    let mut out = vec![
        "\"id\"".to_string(),
        format!("{} AS \"code\"", quote_ident(t.code)),
        "\"archived\"".to_string(),
        "\"created_at\" AS \"__created\"".to_string(),
        "\"updated_at\" AS \"__updated\"".to_string(),
    ];
    if with_history {
        out.push("\"history\"".to_string());
    }
    for key in field_keys(def, t) {
        let col = col_for_key(t, &key);
        if col == key {
            out.push(quote_ident(&col));
        } else {
            out.push(format!("{} AS {}", quote_ident(&col), quote_ident(&key)));
        }
    }
    out
}

/// One SELECT row -> JSON object with HTML field keys.
pub fn row_to_json(
    def: &crate::services::rec_defs::KindDef,
    t: &crate::services::dyn_tables::DynTable,
    row: &sea_orm::QueryResult,
    types: &BTreeMap<String, String>,
    with_history: bool,
) -> Result<Value, DbErr> {
    let mut m = serde_json::Map::new();
    m.insert("id".into(), Value::from(row.try_get::<i64>("", "id")?));
    m.insert(
        "code".into(),
        row.try_get::<Option<String>>("", "code")?
            .map(Value::String)
            .unwrap_or(Value::Null),
    );
    m.insert(
        "archived".into(),
        row.try_get::<bool>("", "archived").unwrap_or(false).into(),
    );
    m.insert(
        "created_at".into(),
        ts_json(row, "__created")?,
    );
    m.insert("updated_at".into(), ts_json(row, "__updated")?);
    if with_history {
        if let Ok(h) = row.try_get::<Option<String>>("", "history") {
            m.insert(
                "history".into(),
                h.and_then(|s| serde_json::from_str::<Value>(&s).ok())
                    .unwrap_or(Value::Array(vec![])),
            );
        }
    }
    for key in field_keys(def, t) {
        let col = col_for_key(t, &key);
        let pg_type = types.get(&col).map(|s| s.as_str()).unwrap_or("text");
        m.insert(key.clone(), pg_cell_to_json(row, &key, pg_type));
    }
    Ok(Value::Object(m))
}

fn ts_json(row: &sea_orm::QueryResult, alias: &str) -> Result<Value, DbErr> {
    let v = row.try_get::<Option<DateTimeWithTimeZone>>("", alias)?;
    Ok(v.map(|d| Value::String(d.to_rfc3339())).unwrap_or(Value::Null))
}

fn pg_cell_to_json(row: &sea_orm::QueryResult, col: &str, pg_type: &str) -> Value {
    if pg_type.contains("int") || pg_type.contains("serial") {
        return row
            .try_get::<Option<i64>>("", col)
            .ok()
            .flatten()
            .map(|n| Value::Number(n.into()))
            .unwrap_or(Value::Null);
    }
    if pg_type.contains("numeric") || pg_type.contains("double") || pg_type.contains("real") {
        return row
            .try_get::<Option<f64>>("", col)
            .ok()
            .flatten()
            .and_then(Number::from_f64)
            .map(Value::Number)
            .unwrap_or(Value::Null);
    }
    if pg_type.contains("bool") {
        return Value::Bool(row.try_get::<bool>("", col).unwrap_or(false));
    }
    if pg_type.contains("timestamp") {
        return row
            .try_get::<Option<DateTimeWithTimeZone>>("", col)
            .ok()
            .flatten()
            .map(|d| Value::String(d.to_rfc3339()))
            .unwrap_or(Value::Null);
    }
    if pg_type.contains("json") {
        return row
            .try_get::<Option<String>>("", col)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
            .unwrap_or(Value::Null);
    }
    row.try_get::<Option<String>>("", col)
        .ok()
        .flatten()
        .map(Value::String)
        .unwrap_or(Value::Null)
}

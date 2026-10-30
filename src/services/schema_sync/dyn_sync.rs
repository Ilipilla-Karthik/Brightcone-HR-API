//! Additive, migration-free synchronization for every record table
//! (`services::dyn_tables::DYNAMIC_TABLES` + `EXT_TABLES`).
//!
//! Behaviour:
//! - missing dynamic tables are created (entity-backed tables always exist
//!   already, they are created by the entity sync loop in `mod.rs`);
//! - missing columns are added with `ADD COLUMN IF NOT EXISTS` (data kept);
//! - `created_at`/`updated_at` receive sane defaults;
//! - a unique index is ensured on each table's code column;
//! - every table is verified with a trivial probe query.
//! Nothing is dropped, reset or recreated, ever.
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use std::collections::HashSet;
use tracing::{info, warn};

use crate::services::dyn_tables::{DynTable, EXT_TABLES, DYNAMIC_TABLES};

fn stmt(db: &DatabaseConnection, sql: String) -> Statement {
    Statement::from_string(db.get_database_backend(), sql)
}

fn sql_type(disc: &str) -> &'static str {
    match disc {
        "uqstr" => "varchar(64)",
        "str" => "text",
        "text" => "text",
        "date" => "varchar(10)",
        "month" => "varchar(7)",
        "time" => "varchar(8)",
        "double" => "double precision",
        "bool" => "boolean",
        "tstz" => "timestamp with time zone",
        _ => "text",
    }
}

fn quote_ident(s: &str) -> String {
    let cleaned = s.replace('"', "");
    format!("\"{}\"", cleaned)
}

async fn existing_columns(
    db: &DatabaseConnection,
    table: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error + Send + Sync>> {
    let sql = format!(
        "SELECT column_name FROM information_schema.columns WHERE table_schema='public' AND table_name='{}'",
        table.replace('\'', "")
    );
    let rows = db.query_all_raw(stmt(db, sql)).await?;
    let mut out = HashSet::new();
    for r in rows {
        let name: String = r.try_get("", "column_name")?;
        out.insert(name);
    }
    Ok(out)
}

async fn table_exists(db: &DatabaseConnection, table: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let sql = format!(
        "SELECT 1 FROM information_schema.tables WHERE table_schema='public' AND table_name='{}'",
        table.replace('\'', "")
    );
    let rows = db.query_all_raw(stmt(db, sql)).await?;
    Ok(!rows.is_empty())
}

fn create_table_sql(t: &DynTable) -> String {
    let mut cols = vec![
        "\"id\" BIGSERIAL PRIMARY KEY".to_string(),
        format!("{} VARCHAR(64) NOT NULL", quote_ident(t.code)),
        "\"archived\" BOOLEAN NOT NULL DEFAULT FALSE".to_string(),
        "\"history\" TEXT".to_string(),
    ];
    for c in t.cols {
        if matches!(c.sql, "tstz") || c.name == "code" || c.name == "archived" || c.name == "history" {
            // created_at/updated_at handled below; primary code columns already added.
            continue;
        }
        cols.push(format!(
            "{} {}{}",
            quote_ident(c.name),
            sql_type(c.sql),
            if c.req == 1 { " NOT NULL" } else { "" }
        ));
    }
    cols.push("\"created_at\" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP".to_string());
    cols.push("\"updated_at\" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP".to_string());
    format!(
        "CREATE TABLE IF NOT EXISTS public.{} ({})",
        quote_ident(t.table),
        cols.join(", ")
    )
}

pub async fn sync_tables(db: &DatabaseConnection) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let tables: Vec<&DynTable> = DYNAMIC_TABLES.iter().copied().chain(EXT_TABLES.iter().copied()).collect();
    info!(
        "[SCHEMA-SYNC] Synchronizing {} dynamic/ext record tables (create-if-missing + ADD COLUMN only)...",
        tables.len()
    );

    for t in &tables {
        if !table_exists(db, t.table).await? {
            if t.ext {
                warn!(
                    "[SCHEMA-SYNC] Entity-owned table '{}' is missing; creating it defensively from its record definition.",
                    t.table
                );
            }
            db.execute_unprepared(&create_table_sql(t)).await?;
            info!("[SCHEMA-SYNC] Created dynamic table '{}'.", t.table);
        }

        let existing = existing_columns(db, t.table).await?;
        for c in t.cols {
            if c.sql == "tstz" {
                continue; // created_at/updated_at handled below (defaults)
            }
            if existing.contains(c.name) {
                continue;
            }
            let sql = format!(
                "ALTER TABLE public.{} ADD COLUMN IF NOT EXISTS {} {}{};",
                quote_ident(t.table),
                quote_ident(c.name),
                sql_type(c.sql),
                if c.req == 1 && c.name == t.code { " DEFAULT ''" } else { "" }
            );
            db.execute_unprepared(&sql).await?;
            info!("[SCHEMA-SYNC] Added column '{}.{}' ({}).", t.table, c.name, c.sql);
        }

        for (col, default) in [
            ("created_at", "CURRENT_TIMESTAMP"),
            ("updated_at", "CURRENT_TIMESTAMP"),
        ] {
            let sql = format!(
                "ALTER TABLE public.{} ALTER COLUMN {} SET DEFAULT {};",
                quote_ident(t.table),
                quote_ident(col),
                default
            );
            let _ = db.execute_unprepared(&sql).await;
        }
        if existing.contains("archived") || !t.ext {
            let sql = format!(
                "ALTER TABLE public.{} ALTER COLUMN archived SET DEFAULT FALSE;",
                quote_ident(t.table)
            );
            let _ = db.execute_unprepared(&sql).await;
        }

        // unique code index (non-fatal when legacy data already violates it)
        let idx = format!("idx_{}_{}_uq", t.table, t.code);
        let sql = format!(
            "CREATE UNIQUE INDEX IF NOT EXISTS {} ON public.{} ({});",
            quote_ident(&idx),
            quote_ident(t.table),
            quote_ident(t.code)
        );
        if let Err(e) = db.execute_unprepared(&sql).await {
            warn!(
                "[SCHEMA-SYNC] Unique index on '{}.{}' not applied (existing data conflict kept intact): {}",
                t.table, t.code, e
            );
        }

        // verify
        let probe = format!("SELECT 1 FROM public.{} LIMIT 1", quote_ident(t.table));
        db.execute_unprepared(&probe)
            .await
            .map_err(|e| format!("Verification failed for dynamic table '{}': {}", t.table, e))?;
    }

    info!("[SCHEMA-SYNC] Verified {} dynamic/ext record tables.", tables.len());
    Ok(())
}

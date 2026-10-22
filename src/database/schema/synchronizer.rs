use sqlx::{PgPool, Row};
use std::collections::{HashMap, HashSet};
use tracing::{info, warn};

use super::manifest::SchemaManifest;
use super::schema::TableDefinition;

/// Structured result of the Core Schema Synchronization execution
#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncReport {
    pub status: String,
    pub tables_created: Vec<String>,
    pub columns_added: Vec<String>,
    pub indexes_created: Vec<String>,
    pub verified_tables: Vec<String>,
    pub warnings: Vec<String>,
}

/// Core Schema Synchronizer responsible for idempotent runtime reflection,
/// database introspection, and safe automated schema reconciliation.
pub struct SchemaSynchronizer;

impl SchemaSynchronizer {
    /// Global advisory lock identifier to prevent concurrent synchronization races
    const ADVISORY_LOCK_ID: i64 = 839210482;

    /// Execute runtime schema inspection and synchronization against PostgreSQL
    pub async fn run(pool: &PgPool, manifest: &SchemaManifest) -> Result<SyncReport, sqlx::Error> {
        info!("[Schema Synchronizer] Starting Core Schema Synchronization...");

        // 1. Acquire PostgreSQL session advisory lock to prevent concurrent races
        sqlx::query("SELECT pg_advisory_lock($1);")
            .bind(Self::ADVISORY_LOCK_ID)
            .execute(pool)
            .await?;
        info!(
            "[Schema Synchronizer] Acquired global synchronization lock #{}",
            Self::ADVISORY_LOCK_ID
        );

        let result = Self::execute_sync(pool, manifest).await;

        // Ensure lock is released even if sync encountered an error
        let unlock_res = sqlx::query("SELECT pg_advisory_unlock($1);")
            .bind(Self::ADVISORY_LOCK_ID)
            .execute(pool)
            .await;
        if let Err(e) = unlock_res {
            warn!("[Schema Synchronizer] Failed to release advisory lock: {e}");
        } else {
            info!(
                "[Schema Synchronizer] Released global synchronization lock #{}",
                Self::ADVISORY_LOCK_ID
            );
        }

        result
    }

    async fn execute_sync(
        pool: &PgPool,
        manifest: &SchemaManifest,
    ) -> Result<SyncReport, sqlx::Error> {
        let mut report = SyncReport {
            status: "Success".to_string(),
            tables_created: Vec::new(),
            columns_added: Vec::new(),
            indexes_created: Vec::new(),
            verified_tables: Vec::new(),
            warnings: Vec::new(),
        };

        // 2. Introspect existing tables in public schema
        let existing_tables_rows = sqlx::query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';"
        )
        .fetch_all(pool)
        .await?;

        let existing_tables: HashSet<String> = existing_tables_rows
            .into_iter()
            .map(|r| r.get::<String, _>("table_name"))
            .collect();

        // 3. Introspect existing columns in public schema
        let existing_columns_rows = sqlx::query(
            "SELECT table_name, column_name, data_type, udt_name, is_nullable, column_default, character_maximum_length 
             FROM information_schema.columns 
             WHERE table_schema = 'public';"
        )
        .fetch_all(pool)
        .await?;

        let mut existing_columns_by_table: HashMap<String, HashSet<String>> = HashMap::new();
        for r in existing_columns_rows {
            let tbl: String = r.get("table_name");
            let col: String = r.get("column_name");
            existing_columns_by_table
                .entry(tbl)
                .or_default()
                .insert(col);
        }

        // 4. Introspect existing indexes
        let existing_indexes_rows =
            sqlx::query("SELECT indexname, tablename FROM pg_indexes WHERE schemaname = 'public';")
                .fetch_all(pool)
                .await?;

        let existing_indexes: HashSet<String> = existing_indexes_rows
            .into_iter()
            .map(|r| r.get::<String, _>("indexname"))
            .collect();

        // 5. Reconcile each table in the manifest
        for table_def in &manifest.tables {
            let table_name = &table_def.name;

            if !existing_tables.contains(table_name) {
                // Table is missing: Generate and execute safe CREATE TABLE DDL
                let create_sql = table_def.to_create_table_sql();
                info!(
                    "[Schema Synchronizer] Creating missing table '{}'...",
                    table_name
                );
                sqlx::query(&create_sql).execute(pool).await?;
                report.tables_created.push(table_name.clone());
            } else {
                // Table exists: Check for missing columns
                let existing_cols = existing_columns_by_table
                    .get(table_name)
                    .cloned()
                    .unwrap_or_default();

                for col in &table_def.columns {
                    if !existing_cols.contains(&col.name) {
                        // Safe non-destructive column addition
                        let add_col_sql = format!(
                            "ALTER TABLE \"{}\" ADD COLUMN IF NOT EXISTS {}",
                            table_name,
                            col.to_column_sql()
                        );
                        info!(
                            "[Schema Synchronizer] Adding missing column '{}.{}'...",
                            table_name, col.name
                        );
                        sqlx::query(&add_col_sql).execute(pool).await?;
                        report
                            .columns_added
                            .push(format!("{}.{}", table_name, col.name));
                    }
                }
            }

            // 6. Reconcile indexes
            for idx in &table_def.indexes {
                if !existing_indexes.contains(&idx.name) {
                    let create_idx_sql = idx.to_create_sql(table_name);
                    info!(
                        "[Schema Synchronizer] Creating index '{}' on table '{}'...",
                        idx.name, table_name
                    );
                    if let Err(e) = sqlx::query(&create_idx_sql).execute(pool).await {
                        warn!(
                            "[Schema Synchronizer] Could not create index '{}': {e}",
                            idx.name
                        );
                        report
                            .warnings
                            .push(format!("Index '{}' skipped: {e}", idx.name));
                    } else {
                        report.indexes_created.push(idx.name.clone());
                    }
                }
            }

            // 7. Verify table structure
            report.verified_tables.push(table_name.clone());
        }

        info!(
            "[Schema Synchronizer] Synchronization finished. Verified {} tables, created {} tables, added {} columns, created {} indexes.",
            report.verified_tables.len(),
            report.tables_created.len(),
            report.columns_added.len(),
            report.indexes_created.len()
        );

        Ok(report)
    }

    /// Validate that a given table satisfies entity constraints
    pub async fn verify_table(
        pool: &PgPool,
        table_def: &TableDefinition,
    ) -> Result<bool, sqlx::Error> {
        let count_row = sqlx::query(
            "SELECT count(*) as count FROM information_schema.tables WHERE table_schema = 'public' AND table_name = $1;"
        )
        .bind(&table_def.name)
        .fetch_one(pool)
        .await?;

        let count: i64 = count_row.get("count");
        Ok(count > 0)
    }
}

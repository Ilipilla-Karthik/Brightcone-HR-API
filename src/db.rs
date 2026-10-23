use loco_rs::app::AppContext;
use sqlx::PgPool;

/// The master SQL schema definition embedded directly into the binary.
pub const MASTER_SCHEMA: &str = include_str!("../db/schema.sql");

/// Synchronizes the PostgreSQL database schema with `db/schema.sql`.
///
/// This function executes idempotent DDL:
/// - Ensures the `pgcrypto` extension is installed.
/// - Ensures `shifts`, `holidays`, `leave_types`, and `leave_quotas` tables exist with correct types and constraints.
/// - Ensures all indexes exist.
/// - Preserves all existing table rows and data without dropping or truncating.
pub async fn sync_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    tracing::info!("Synchronizing database schema via db/schema.sql...");
    println!("Synchronizing database schema via db/schema.sql...");

    sqlx::raw_sql(sqlx::AssertSqlSafe(MASTER_SCHEMA))
        .execute(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database schema synchronization failed: {}", e);
            eprintln!("Database schema synchronization failed: {}", e);
            e
        })?;

    tracing::info!("Database schema synchronization completed successfully.");
    println!("Database schema synchronization completed successfully.");
    Ok(())
}

/// Abstraction to obtain a PostgreSQL connection pool from either Loco's AppContext
/// or directly from a sqlx::PgPool.
pub trait AsPgPool: Send + Sync + 'static {
    fn as_pg_pool(&self) -> &PgPool;
}

impl AsPgPool for PgPool {
    fn as_pg_pool(&self) -> &PgPool {
        self
    }
}

impl AsPgPool for AppContext {
    fn as_pg_pool(&self) -> &PgPool {
        self.db.get_postgres_connection_pool()
    }
}

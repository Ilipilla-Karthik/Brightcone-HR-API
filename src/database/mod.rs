pub mod connection;
pub mod repository;
pub mod schema;
pub mod seed;

pub use connection::{create_connection_pool, test_connection};
pub use repository::DatabaseRepository;
pub use schema::{
    ColumnDefinition, IndexDefinition, PgDataType, SchemaManifest, SchemaSynchronizer, SyncReport,
    TableDefinition,
};
pub use seed::{hash_password, verify_password};

use sqlx::PgPool;
use std::sync::Arc;
use tracing::info;

/// Core Database Subsystem Handle
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
    repo: Arc<DatabaseRepository>,
    manifest: Arc<SchemaManifest>,
}

impl Database {
    /// Initialize the PostgreSQL connection pool, run the Core Schema Synchronizer,
    /// and prepare the repository layer.
    pub async fn init() -> Result<(Self, SyncReport), Box<dyn std::error::Error + Send + Sync>> {
        info!("[Database] Booting Database Subsystem...");

        // 1. Establish connection pool
        let pool = create_connection_pool().await?;

        // 2. Load centralized Entity Schema Manifest
        let manifest = Arc::new(SchemaManifest::new());

        // 3. Run Core Schema Synchronizer with advisory locking & reflection
        let sync_report = SchemaSynchronizer::run(&pool, &manifest).await?;

        // 4. Seed initial records if tables are fresh, preserving all existing records
        seed::seed_missing_data(&pool).await?;

        // 5. Initialize Database Repository
        let repo = Arc::new(DatabaseRepository::new(pool.clone()));

        info!("[Database] Subsystem initialized successfully.");
        Ok((
            Self {
                pool,
                repo,
                manifest,
            },
            sync_report,
        ))
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn repo(&self) -> &DatabaseRepository {
        &self.repo
    }

    pub fn repository(&self) -> &DatabaseRepository {
        &self.repo
    }

    pub fn manifest(&self) -> &SchemaManifest {
        &self.manifest
    }
}

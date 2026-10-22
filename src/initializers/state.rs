use async_trait::async_trait;
use axum::{Extension, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::database::Database;
use crate::models::{AppState, SharedState};

pub struct StateInitializer {
    state: SharedState,
}

impl StateInitializer {
    /// Initialize the Database subsystem and Core Schema Synchronizer during Loco startup.
    /// Halts startup with an informative diagnostic if PostgreSQL is unavailable.
    pub async fn init() -> Result<Self> {
        info!("[Startup] Initializing Brightcone PostgreSQL Database Subsystem...");
        let (db, sync_report) = Database::init().await.map_err(|e| {
            error!("[Startup] Fatal: Database initialization failed: {e}");
            loco_rs::Error::string(&format!("Database initialization failed: {e}"))
        })?;

        info!(
            "[Startup] Schema synchronization complete. Created: {} tables, {} columns, {} indexes. Warnings: {}",
            sync_report.tables_created.len(),
            sync_report.columns_added.len(),
            sync_report.indexes_created.len(),
            sync_report.warnings.len()
        );

        let repo = db.repository().clone();
        let app_state = AppState::new(repo);

        info!("[Startup] PostgreSQL connection pool established and verified.");
        Ok(Self {
            state: Arc::new(app_state),
        })
    }
}

#[async_trait]
impl Initializer for StateInitializer {
    fn name(&self) -> String {
        "app-state".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let bg_state = self.state.clone();

        // Background service: real 30-second heartbeat checking PostgreSQL connection latency
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                match bg_state.repo.check_database_health().await {
                    Ok(latency_ms) => {
                        if let Ok(mut t) = bg_state.transient.write() {
                            t.wakeup_count += 1;
                            t.last_wakeup_time = "Just now".to_string();
                            t.last_db_latency_ms = latency_ms.round() as u64;
                            info!(
                                "[Heartbeat] Wake-up pulse recorded (#{}); PostgreSQL ping: {:.2}ms; connection pool healthy.",
                                t.wakeup_count, latency_ms
                            );
                        }
                    }
                    Err(err) => {
                        error!("[Heartbeat] PostgreSQL heartbeat ping failed: {err}");
                    }
                }
            }
        });

        Ok(router.layer(Extension(self.state.clone())))
    }
}

use async_trait::async_trait;
use loco_rs::app::{AppContext, Initializer};
use loco_rs::Result;

pub struct SchemaSyncInitializer;

#[async_trait]
impl Initializer for SchemaSyncInitializer {
    fn name(&self) -> String {
        "schema_sync".to_string()
    }

    async fn after_routes(&self, router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
        if let Err(e) = crate::services::schema_sync::run_schema_synchronization(&ctx.db).await {
            tracing::error!("[SCHEMA-SYNC] Fatal synchronization error: {}", e);
            return Err(loco_rs::Error::string(&format!(
                "Schema synchronization failed: {}",
                e
            )));
        }
        Ok(router)
    }
}

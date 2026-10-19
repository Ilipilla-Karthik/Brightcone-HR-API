use async_trait::async_trait;
use loco_rs::app::{AppContext, Initializer};
use loco_rs::Result;

pub struct SeedInitializer;

#[async_trait]
impl Initializer for SeedInitializer {
    fn name(&self) -> String {
        "db_seed".to_string()
    }

    async fn after_routes(&self, router: axum::Router, ctx: &AppContext) -> Result<axum::Router> {
        if let Err(e) = crate::services::seed::run_seed(&ctx.db).await {
            tracing::error!("Seed error: {}", e);
        }
        Ok(router)
    }
}

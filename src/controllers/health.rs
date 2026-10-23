use loco_rs::prelude::*;
use serde_json::json;

pub async fn health() -> Result<Response> {
    format::json(json!({
        "status": "healthy",
        "service": "Brightcone HR API",
        "framework": "loco",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/health", get(health))
        .add("/api/health", get(health))
}

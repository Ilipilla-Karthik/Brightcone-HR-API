use axum::{extract::State, http::StatusCode, response::Json};
use loco_rs::prelude::*;
use serde_json::json;
use sqlx::PgPool;

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

async fn generic_handler<S: AsPgPool>(
    State(db): State<S>,
) -> (StatusCode, Json<serde_json::Value>) {
    let _pool = db.as_pg_pool();
    (StatusCode::OK, Json(json!({"ok": true})))
}

fn test_routes() -> Routes {
    Routes::new().add("/generic", get(generic_handler::<AppContext>))
}

#[tokio::test]
async fn test_generic_with_pool() {
    // We can't connect without DB in unit test, but we can check type check
    let _fn = generic_handler::<PgPool>;
}

#[test]
fn test_loco_routes() {
    let _r = test_routes();
}

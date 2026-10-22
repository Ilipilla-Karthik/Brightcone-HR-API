use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info};

/// Establish a connection pool to PostgreSQL using DATABASE_URL
pub async fn create_connection_pool() -> Result<PgPool, sqlx::Error> {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        let host = std::env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("DB_PORT").unwrap_or_else(|_| "5433".to_string());
        let user = std::env::var("DB_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = std::env::var("DB_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
        let db = std::env::var("DB_NAME").unwrap_or_else(|_| "brightcone_recruitment".to_string());
        format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, db)
    });

    info!("[Database] Initializing PostgreSQL connection pool...");

    let pool = PgPoolOptions::new()
        .max_connections(15)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await
        .map_err(|err| {
            error!("[Database] Failed to establish PostgreSQL pool: {err}");
            err
        })?;

    // Verify immediate connection responsiveness
    test_connection(&pool).await?;

    info!("[Database] PostgreSQL connection pool established successfully.");
    Ok(pool)
}

/// Ping the database to verify active connectivity and pool responsiveness
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1;").execute(pool).await?;
    Ok(())
}

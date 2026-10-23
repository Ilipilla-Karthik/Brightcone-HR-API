pub mod recruitment_and_perf;
pub mod users_and_org;

use sea_orm::DatabaseConnection;

pub async fn run_seed(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Running schema synchronization before seed...");
    crate::services::schema_sync::run_schema_synchronization(db).await?;

    tracing::info!("Checking and running idempotent database seed...");
    users_and_org::seed_users_and_org(db).await?;
    recruitment_and_perf::seed_recruitment_and_perf(db).await?;

    tracing::info!("Database seed completed successfully.");
    Ok(())
}

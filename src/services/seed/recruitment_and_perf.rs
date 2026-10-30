use sea_orm::DatabaseConnection;

/// Recruitment and performance seed data.
///
/// This module is intentionally kept empty - all recruitment and performance
/// data is dynamic and database-backed through the People Plus record kinds.
/// No demo candidates, interviews, goals, or reviews are seeded here.
///
/// Seed behavior: idempotent, no-op. Only the two spec accounts are seeded
/// via `seed_users_and_org`.
pub async fn seed_recruitment_and_perf(
    _db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Intentionally empty - no demo data seeded
    Ok(())
}
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Shift {
    pub id: i64,
    pub pid: String,
    pub name: String,
    pub code: String,
    pub start_time: String,
    pub end_time: String,
    pub break_duration_minutes: i32,
    pub grace_period_minutes: i32,
    pub is_overnight: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Holiday {
    pub id: i64,
    pub pid: String,
    pub name: String,
    pub holiday_date: String,
    pub description: Option<String>,
    pub applicable_location: Option<String>,
    pub is_recurring: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct LeaveType {
    pub id: i64,
    pub pid: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_paid: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct LeaveQuota {
    pub id: i64,
    pub pid: String,
    pub leave_type_id: i64,
    pub leave_type_name: Option<String>,
    pub leave_type_code: Option<String>,
    pub year: i32,
    pub entitled_days: f64,
    pub applicable_group: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tokio::test]
async fn test_full_postgres_persistence_crud_and_guards() {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .connect(&db_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    brightcone_hr_api::db::sync_schema(&pool)
        .await
        .expect("Failed to synchronize database schema");

    // 1. SHIFTS CRUD
    let test_shift_code = format!("TEST-{}", chrono::Utc::now().timestamp_subsec_millis());
    let inserted_shift: Shift = sqlx::query_as(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active) \
         VALUES ($1, $2, $3::TIME, $4::TIME, $5, $6, $7, $8) \
         RETURNING id, pid::TEXT as pid, name, code, start_time::TEXT as start_time, end_time::TEXT as end_time, \
                   break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at"
    )
    .bind("Integration Test Shift")
    .bind(&test_shift_code)
    .bind("08:00:00")
    .bind("17:00:00")
    .bind(45)
    .bind(10)
    .bind(false)
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("Create shift failed");

    assert_eq!(inserted_shift.name, "Integration Test Shift");
    assert_eq!(inserted_shift.code, test_shift_code);

    // Get shift by ID
    let fetched_shift: Shift = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, start_time::TEXT as start_time, end_time::TEXT as end_time, \
                break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM shifts WHERE id = $1"
    )
    .bind(inserted_shift.id)
    .fetch_one(&pool)
    .await
    .expect("Get shift failed");

    assert_eq!(fetched_shift.id, inserted_shift.id);

    // Update shift
    let updated_shift: Shift = sqlx::query_as(
        "UPDATE shifts \
         SET name = $1, start_time = $2::TIME, end_time = $3::TIME, updated_at = CURRENT_TIMESTAMP \
         WHERE id = $4 \
         RETURNING id, pid::TEXT as pid, name, code, start_time::TEXT as start_time, end_time::TEXT as end_time, \
                   break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at"
    )
    .bind("Updated Test Shift")
    .bind("08:30:00")
    .bind("17:30:00")
    .bind(inserted_shift.id)
    .fetch_one(&pool)
    .await
    .expect("Update shift failed");

    assert_eq!(updated_shift.name, "Updated Test Shift");
    assert_eq!(updated_shift.start_time, "08:30:00");

    // Delete shift
    let delete_shift_res = sqlx::query("DELETE FROM shifts WHERE id = $1")
        .bind(inserted_shift.id)
        .execute(&pool)
        .await
        .expect("Delete shift query failed");
    assert_eq!(delete_shift_res.rows_affected(), 1);

    // 2. HOLIDAYS CRUD
    let inserted_holiday: Holiday = sqlx::query_as(
        "INSERT INTO holidays (name, holiday_date, description, applicable_location, is_recurring) \
         VALUES ($1, $2::DATE, $3, $4, $5) \
         RETURNING id, pid::TEXT as pid, name, holiday_date::TEXT as holiday_date, description, applicable_location, is_recurring, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at"
    )
    .bind("Integration Test Holiday")
    .bind("2026-12-25")
    .bind(Some("Christmas Holiday"))
    .bind(Some("Global HQ"))
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("Create holiday failed");

    assert_eq!(inserted_holiday.holiday_date, "2026-12-25");

    // Update holiday
    let updated_holiday: Holiday = sqlx::query_as(
        "UPDATE holidays \
         SET name = $1, description = $2, updated_at = CURRENT_TIMESTAMP \
         WHERE id = $3 \
         RETURNING id, pid::TEXT as pid, name, holiday_date::TEXT as holiday_date, description, applicable_location, is_recurring, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at"
    )
    .bind("Updated Holiday Name")
    .bind(Some("Updated description"))
    .bind(inserted_holiday.id)
    .fetch_one(&pool)
    .await
    .expect("Update holiday failed");

    assert_eq!(updated_holiday.name, "Updated Holiday Name");

    // Delete holiday
    let delete_holiday_res = sqlx::query("DELETE FROM holidays WHERE id = $1")
        .bind(inserted_holiday.id)
        .execute(&pool)
        .await
        .expect("Delete holiday query failed");
    assert_eq!(delete_holiday_res.rows_affected(), 1);

    // 3. LEAVE TYPES & LEAVE QUOTAS WITH RELATIONSHIP GUARDS
    let test_lt_code = format!("LT{}", chrono::Utc::now().timestamp_subsec_millis());
    let inserted_lt: LeaveType = sqlx::query_as(
        "INSERT INTO leave_types (name, code, description, is_paid, is_active) \
         VALUES ($1, $2, $3, $4, $5) \
         RETURNING id, pid::TEXT as pid, name, code, description, is_paid, is_active, \
                   to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                   to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at",
    )
    .bind("Integration Leave Type")
    .bind(&test_lt_code)
    .bind(Some("Integration testing type"))
    .bind(true)
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("Create leave type failed");

    // Create a leave quota referencing this leave type
    let inserted_quota: LeaveQuota = sqlx::query_as(
        "WITH inserted AS ( \
             INSERT INTO leave_quotas (leave_type_id, year, entitled_days, applicable_group) \
             VALUES ($1, $2, $3::NUMERIC, $4) \
             RETURNING * \
         ) \
         SELECT i.id, i.pid::TEXT as pid, i.leave_type_id, \
                lt.name as leave_type_name, lt.code as leave_type_code, \
                i.year, i.entitled_days::FLOAT8 as entitled_days, i.applicable_group, \
                to_char(i.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                to_char(i.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM inserted i \
         LEFT JOIN leave_types lt ON i.leave_type_id = lt.id",
    )
    .bind(inserted_lt.id)
    .bind(2027)
    .bind(25.5)
    .bind("DEV_TEAM")
    .fetch_one(&pool)
    .await
    .expect("Create leave quota failed");

    assert_eq!(inserted_quota.leave_type_id, inserted_lt.id);
    assert_eq!(
        inserted_quota.leave_type_name,
        Some("Integration Leave Type".to_string())
    );
    assert_eq!(inserted_quota.entitled_days, 25.5);

    // Requirement 5 guard test: Attempting to delete leave type referenced by active quota MUST be blocked
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM leave_quotas WHERE leave_type_id = $1")
            .bind(inserted_lt.id)
            .fetch_one(&pool)
            .await
            .expect("Query quota count failed");
    assert!(count.0 > 0, "Leave type must have active quota references");

    // Database foreign key constraint also prevents this delete
    let delete_lt_res = sqlx::query("DELETE FROM leave_types WHERE id = $1")
        .bind(inserted_lt.id)
        .execute(&pool)
        .await;
    assert!(
        delete_lt_res.is_err(),
        "PostgreSQL foreign key RESTRICT constraint must block deleting referenced leave type"
    );

    // Delete the quota first
    let delete_quota_res = sqlx::query("DELETE FROM leave_quotas WHERE id = $1")
        .bind(inserted_quota.id)
        .execute(&pool)
        .await
        .expect("Delete leave quota query failed");
    assert_eq!(delete_quota_res.rows_affected(), 1);

    // Now deleting the leave type succeeds
    let delete_lt_res2 = sqlx::query("DELETE FROM leave_types WHERE id = $1")
        .bind(inserted_lt.id)
        .execute(&pool)
        .await
        .expect("Delete leave type after quota removal failed");
    assert_eq!(delete_lt_res2.rows_affected(), 1);

    println!("All PostgreSQL CRUD and relationship tests passed successfully!");
}

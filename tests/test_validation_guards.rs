use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_shift_validation_and_constraints() {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    brightcone_hr_api::db::sync_schema(&pool)
        .await
        .expect("Failed to synchronize database schema");

    let unique_code = format!("VAL-{}", chrono::Utc::now().timestamp_subsec_nanos());

    // 1. Successful Shift Insert
    let insert_res = sqlx::query(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active) \
         VALUES ($1, $2, '09:00:00'::TIME, '17:00:00'::TIME, 60, 15, false, true)"
    )
    .bind("Valid Shift")
    .bind(&unique_code)
    .execute(&pool)
    .await;
    assert!(insert_res.is_ok(), "Initial shift insert must succeed");

    // 2. Duplicate Code Constraint
    let duplicate_res = sqlx::query(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active) \
         VALUES ($1, $2, '10:00:00'::TIME, '18:00:00'::TIME, 60, 15, false, true)"
    )
    .bind("Duplicate Code Shift")
    .bind(&unique_code)
    .execute(&pool)
    .await;
    assert!(
        duplicate_res.is_err(),
        "Duplicate shift code must be rejected by PostgreSQL UNIQUE constraint"
    );

    // 3. Invalid Timing: Daytime shift with start >= end
    let invalid_time_res = sqlx::query(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active) \
         VALUES ($1, $2, '18:00:00'::TIME, '09:00:00'::TIME, 60, 15, false, true)"
    )
    .bind("Invalid Hours Shift")
    .bind(format!("INV-{}", chrono::Utc::now().timestamp_subsec_nanos()))
    .execute(&pool)
    .await;
    assert!(
        invalid_time_res.is_err(),
        "Daytime shift with start >= end must violate chk_shifts_times"
    );

    // 4. Invalid Break Duration (< 0)
    let negative_break_res = sqlx::query(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active) \
         VALUES ($1, $2, '09:00:00'::TIME, '17:00:00'::TIME, -10, 15, false, true)"
    )
    .bind("Negative Break Shift")
    .bind(format!("BRK-{}", chrono::Utc::now().timestamp_subsec_nanos()))
    .execute(&pool)
    .await;
    assert!(
        negative_break_res.is_err(),
        "Negative break duration must violate chk_shifts_break_duration"
    );
}

#[tokio::test]
async fn test_holiday_validation_and_duplicate_handling() {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    let unique_loc = format!("LOC-{}", chrono::Utc::now().timestamp_subsec_nanos());

    // 1. Initial Holiday Insert
    let insert_res = sqlx::query(
        "INSERT INTO holidays (name, holiday_date, description, applicable_location, is_recurring) \
         VALUES ($1, '2026-10-25'::DATE, 'Test Holiday', $2, false)"
    )
    .bind("Original Holiday")
    .bind(&unique_loc)
    .execute(&pool)
    .await;
    assert!(insert_res.is_ok(), "Initial holiday insert must succeed");

    // 2. Duplicate Date + Location
    let duplicate_res = sqlx::query(
        "INSERT INTO holidays (name, holiday_date, description, applicable_location, is_recurring) \
         VALUES ($1, '2026-10-25'::DATE, 'Second Holiday', $2, false)"
    )
    .bind("Conflicting Holiday")
    .bind(&unique_loc)
    .execute(&pool)
    .await;
    assert!(
        duplicate_res.is_err(),
        "Duplicate holiday (date + location) must violate idx_holidays_date_location"
    );

    // 3. Missing record query returns no rows
    let missing_res = sqlx::query("SELECT id FROM holidays WHERE id = -99999")
        .fetch_optional(&pool)
        .await
        .expect("Query for non-existent record must not error");
    assert!(
        missing_res.is_none(),
        "Non-existent record must return None"
    );
}

#[tokio::test]
async fn test_leave_type_and_quota_constraints() {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    let unique_lt_code = format!("LT-{}", chrono::Utc::now().timestamp_subsec_nanos());

    // 1. Insert Leave Type
    let lt_row: (i64,) = sqlx::query_as(
        "INSERT INTO leave_types (name, code, description, is_paid, is_active) \
         VALUES ('Constraint Leave Type', $1, 'Test', true, true) \
         RETURNING id",
    )
    .bind(&unique_lt_code)
    .fetch_one(&pool)
    .await
    .expect("Leave type insert must succeed");

    let lt_id = lt_row.0;

    // 2. Duplicate Leave Type Code
    let dup_lt = sqlx::query(
        "INSERT INTO leave_types (name, code, description, is_paid, is_active) \
         VALUES ('Duplicate Code LT', $1, 'Test', true, true)",
    )
    .bind(&unique_lt_code)
    .execute(&pool)
    .await;
    assert!(
        dup_lt.is_err(),
        "Duplicate leave type code must violate unique constraint"
    );

    // 3. Insert Leave Quota
    let quota_res = sqlx::query(
        "INSERT INTO leave_quotas (leave_type_id, year, entitled_days, applicable_group) \
         VALUES ($1, 2026, 15.0, 'SALES_TEAM')",
    )
    .bind(lt_id)
    .execute(&pool)
    .await;
    assert!(quota_res.is_ok(), "Quota insert must succeed");

    // 4. Duplicate Leave Quota: (leave_type_id, year, applicable_group)
    let dup_quota = sqlx::query(
        "INSERT INTO leave_quotas (leave_type_id, year, entitled_days, applicable_group) \
         VALUES ($1, 2026, 20.0, 'SALES_TEAM')",
    )
    .bind(lt_id)
    .execute(&pool)
    .await;
    assert!(
        dup_quota.is_err(),
        "Duplicate quota allocation must violate uq_leave_quotas_type_year_group"
    );

    // 5. Deletion Protection: Deleting leave type referenced by active quota must fail
    let delete_lt_res = sqlx::query("DELETE FROM leave_types WHERE id = $1")
        .bind(lt_id)
        .execute(&pool)
        .await;
    assert!(
        delete_lt_res.is_err(),
        "FK constraint fk_leave_quotas_leave_type must block deleting leave type assigned to active quotas"
    );

    // 6. Delete quota first, then leave type succeeds
    let clean_quota = sqlx::query("DELETE FROM leave_quotas WHERE leave_type_id = $1")
        .bind(lt_id)
        .execute(&pool)
        .await
        .expect("Quota delete query must succeed");
    assert!(clean_quota.rows_affected() > 0);

    let clean_lt = sqlx::query("DELETE FROM leave_types WHERE id = $1")
        .bind(lt_id)
        .execute(&pool)
        .await
        .expect("Leave type delete query must now succeed");
    assert_eq!(clean_lt.rows_affected(), 1);
}

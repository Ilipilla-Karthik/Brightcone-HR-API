use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_fresh_schema_sync_and_seeding() {
    dotenv().ok();

    let test_db_name = "brightcone_schema_sync_sandbox";
    let test_db_url = std::env::var("TEST_DATABASE_URL")
        .expect("Set TEST_DATABASE_URL explicitly before running this test");

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test database");

    // 1. Strict database-name safety guard
    let (current_db,): (String,) = sqlx::query_as("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("Failed to query current_database()");

    assert_eq!(
        current_db, test_db_name,
        "CRITICAL SAFETY ABORT: Connected database must be exactly '{}', but was '{}'",
        test_db_name, current_db
    );
    println!(
        "[SAFETY CHECK PASSED] Connected strictly to: {}",
        current_db
    );

    // 2. Precondition check: verify test database is completely empty
    let (table_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to query public tables count");

    assert_eq!(
        table_count, 0,
        "CRITICAL PRECONDITION FAILED: '{}' is not empty (found {} tables). Aborting test.",
        test_db_name, table_count
    );
    println!("[PRECONDITION PASSED] Database is empty (0 base tables).");

    // 3. Phase 1: Run schema synchronization against empty database
    println!("[PHASE 1] Executing sync_schema() on fresh database...");
    brightcone_hr_api::db::sync_schema(&pool)
        .await
        .expect("Fresh database schema synchronization failed");
    println!("[PHASE 1 PASSED] sync_schema() completed.");

    // 4. Verify required tables exist
    let tables: Vec<(String,)> = sqlx::query_as(
        "SELECT table_name FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_type = 'BASE TABLE' \
         ORDER BY table_name",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query created tables");

    let table_names: Vec<String> = tables.into_iter().map(|(t,)| t).collect();
    println!("Created tables: {:?}", table_names);

    for required_table in &["shifts", "holidays", "leave_types", "leave_quotas"] {
        assert!(
            table_names.contains(&required_table.to_string()),
            "Required table '{}' was not created by sync_schema",
            required_table
        );
    }
    println!("[TABLES CHECK PASSED] All 4 required tables exist.");

    // 5. Verify indexes exist
    let indexes: Vec<(String,)> = sqlx::query_as(
        "SELECT indexname FROM pg_indexes WHERE schemaname = 'public' ORDER BY indexname",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query indexes");

    let index_names: Vec<String> = indexes.into_iter().map(|(i,)| i).collect();
    println!("Created indexes: {:?}", index_names);

    let required_indexes = [
        "idx_shifts_code",
        "idx_shifts_active",
        "idx_holidays_date_location",
        "idx_holidays_date",
        "idx_leave_types_code",
        "idx_leave_types_active",
        "idx_leave_quotas_type",
        "idx_leave_quotas_year_group",
    ];

    for idx in &required_indexes {
        assert!(
            index_names.contains(&idx.to_string()),
            "Required index '{}' was not created",
            idx
        );
    }
    println!("[INDEXES CHECK PASSED] All required indexes exist.");

    // 6. Verify constraints exist
    let constraints: Vec<(String,)> = sqlx::query_as(
        "SELECT conname FROM pg_constraint \
         WHERE connamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'public') \
         ORDER BY conname",
    )
    .fetch_all(&pool)
    .await
    .expect("Failed to query constraints");

    let constraint_names: Vec<String> = constraints.into_iter().map(|(c,)| c).collect();
    println!("Created constraints: {:?}", constraint_names);

    let required_constraints = [
        "chk_shifts_break_duration",
        "chk_shifts_grace_period",
        "chk_shifts_times",
        "fk_leave_quotas_leave_type",
        "chk_leave_quotas_days",
        "chk_leave_quotas_year",
        "uq_leave_quotas_type_year_group",
    ];

    for con in &required_constraints {
        assert!(
            constraint_names.contains(&con.to_string()),
            "Required constraint '{}' was not created",
            con
        );
    }
    println!("[CONSTRAINTS CHECK PASSED] All required constraints exist.");

    // 7. Verify schema creation did NOT insert data (separate from seeding)
    let (s_cnt,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM shifts")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (h_cnt,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holidays")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (lt_cnt,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_types")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (lq_cnt,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_quotas")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(s_cnt, 0, "shifts must be empty after sync_schema alone");
    assert_eq!(h_cnt, 0, "holidays must be empty after sync_schema alone");
    assert_eq!(
        lt_cnt, 0,
        "leave_types must be empty after sync_schema alone"
    );
    assert_eq!(
        lq_cnt, 0,
        "leave_quotas must be empty after sync_schema alone"
    );
    println!("[SEPARATION CHECK PASSED] Tables are unseeded (0 records).");

    // 8. Phase 2: Insert a test record, run sync_schema a second time, verify record preserved
    println!("[PHASE 2] Inserting test record before second sync_schema...");
    sqlx::query(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active) \
         VALUES ('Idempotency Test Shift', 'IDEMP-01', '08:30:00'::TIME, '17:30:00'::TIME, 45, 10, false, true)"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test shift record");

    println!("[PHASE 2] Executing sync_schema() second time (idempotency test)...");
    brightcone_hr_api::db::sync_schema(&pool)
        .await
        .expect("Second sync_schema execution failed");

    let (preserved_count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM shifts WHERE code = 'IDEMP-01'")
            .fetch_one(&pool)
            .await
            .expect("Failed to query preserved shift");

    assert_eq!(
        preserved_count, 1,
        "Test record was not preserved after second sync_schema execution"
    );
    println!(
        "[IDEMPOTENCY CHECK PASSED] Second sync completed successfully and preserved test record."
    );

    // 9. Phase 3: Test seeding separately
    println!("[PHASE 3] Executing time::seed_if_empty() separately...");
    brightcone_hr_api::modules::time::seed_if_empty(&pool)
        .await
        .expect("seed_if_empty failed on synchronized test database");

    let (seeded_h,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holidays")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (seeded_lt,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_types")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (seeded_lq,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_quotas")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(seeded_h > 0, "Holidays should be seeded by seed_if_empty");
    assert!(
        seeded_lt > 0,
        "Leave types should be seeded by seed_if_empty"
    );
    assert!(
        seeded_lq > 0,
        "Leave quotas should be seeded by seed_if_empty"
    );
    println!(
        "[SEEDING CHECK PASSED] Separate seeding successful. Holidays: {}, Leave Types: {}, Quotas: {}",
        seeded_h, seeded_lt, seeded_lq
    );
}

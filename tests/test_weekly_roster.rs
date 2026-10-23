use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_weekly_roster_endpoints_and_guards() {
    dotenv().ok();
    let raw_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be defined");

    // Connect strictly to disposable test database brightcone_schema_sync_test
    let test_db_name = "brightcone_schema_sync_test";
    let base = match raw_url.rfind('/') {
        Some(idx) => &raw_url[..=idx],
        None => panic!("Invalid DATABASE_URL format"),
    };
    let test_db_url = format!("{}{}", base, test_db_name);

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&test_db_url)
        .await
        .expect("Failed to connect to test database");

    // Safety guard: Must NOT be development database
    let (current_db,): (String,) = sqlx::query_as("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("Failed to query current database");

    assert_eq!(
        current_db, test_db_name,
        "CRITICAL SAFETY ABORT: Must connect to test database, got: {}",
        current_db
    );

    // Synchronize schema (creates employees, roster_assignments, roster_publications, swap_requests)
    brightcone_hr_api::db::sync_schema(&pool)
        .await
        .expect("Schema sync failed");

    // Seed data
    brightcone_hr_api::modules::time::seed_if_empty(&pool)
        .await
        .expect("Seed failed");

    // 1. Verify employees exist
    let (emp_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM employees")
        .fetch_one(&pool)
        .await
        .expect("Fetch emp count failed");
    assert!(
        emp_count >= 14,
        "Expected at least 14 employees, found {}",
        emp_count
    );

    // 2. Fetch employee list via list_employees
    let employees_res = brightcone_hr_api::modules::time::roster::list_employees(
        axum::extract::State(pool.clone()),
    )
    .await
    .expect("list_employees failed");
    assert!(
        !employees_res.0.is_empty(),
        "Employees list should not be empty"
    );

    let first_emp = &employees_res.0[0];
    let emp_id = first_emp.id;

    // 3. Test roster retrieval for week Oct 12, 2026
    let query = brightcone_hr_api::modules::time::models::WeeklyRosterQuery {
        week_start: Some("2026-10-12".to_string()),
    };
    let roster_res = brightcone_hr_api::modules::time::roster::get_weekly_roster(
        axum::extract::State(pool.clone()),
        axum::extract::Query(query),
    )
    .await
    .expect("get_weekly_roster failed");

    let roster_data = roster_res.0;
    assert_eq!(roster_data.week_start, "2026-10-12");
    assert_eq!(roster_data.week_end, "2026-10-18");
    assert_eq!(roster_data.summary.total_employees, emp_count);
    assert!(!roster_data.summary.shift_codes.is_empty());
    assert!(!roster_data.employees.is_empty());
    assert_eq!(
        roster_data.employees[0].days.len(),
        7,
        "Each employee must have 7 day columns"
    );

    // 4. Test validation: invalid date format in get_weekly_roster
    let invalid_query = brightcone_hr_api::modules::time::models::WeeklyRosterQuery {
        week_start: Some("invalid-date".to_string()),
    };
    let invalid_roster_res = brightcone_hr_api::modules::time::roster::get_weekly_roster(
        axum::extract::State(pool.clone()),
        axum::extract::Query(invalid_query),
    )
    .await;
    assert!(
        invalid_roster_res.is_err(),
        "Invalid week_start must return an error"
    );

    // 5. Test validation: assign shift with non-existent employee ID
    let invalid_emp_payload = brightcone_hr_api::modules::time::models::AssignShiftPayload {
        employee_id: 99999999,
        date: "2026-10-14".to_string(),
        shift_id: None,
        is_off: Some(true),
        notes: None,
    };
    let invalid_emp_res = brightcone_hr_api::modules::time::roster::assign_shift(
        axum::extract::State(pool.clone()),
        axum::extract::Json(invalid_emp_payload),
    )
    .await;
    assert!(
        invalid_emp_res.is_err(),
        "Non-existent employee must return NOT_FOUND error"
    );

    // 6. Test successful shift assignment
    let (shift_id,): (i64,) = sqlx::query_as("SELECT id FROM shifts LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("Fetch shift id failed");

    let assign_payload = brightcone_hr_api::modules::time::models::AssignShiftPayload {
        employee_id: emp_id,
        date: "2026-10-14".to_string(),
        shift_id: Some(shift_id),
        is_off: Some(false),
        notes: Some("Test assignment".to_string()),
    };
    let assign_res = brightcone_hr_api::modules::time::roster::assign_shift(
        axum::extract::State(pool.clone()),
        axum::extract::Json(assign_payload),
    )
    .await
    .expect("assign_shift failed");

    assert_eq!(assign_res.0.employee_id, emp_id);
    assert_eq!(assign_res.0.roster_date, "2026-10-14");
    assert_eq!(assign_res.0.shift_id, Some(shift_id));
    assert_eq!(assign_res.0.is_off, false);

    // 7. Test updating assignment (e.g. setting to Day Off)
    let update_off_payload = brightcone_hr_api::modules::time::models::AssignShiftPayload {
        employee_id: emp_id,
        date: "2026-10-14".to_string(),
        shift_id: None,
        is_off: Some(true),
        notes: Some("Updated to Day Off".to_string()),
    };
    let update_res = brightcone_hr_api::modules::time::roster::assign_shift(
        axum::extract::State(pool.clone()),
        axum::extract::Json(update_off_payload),
    )
    .await
    .expect("update assignment to off failed");

    assert_eq!(update_res.0.is_off, true);
    assert_eq!(update_res.0.shift_id, None);

    // 8. Test publish roster
    let pub_payload = brightcone_hr_api::modules::time::models::PublishRosterPayload {
        week_start_date: "2026-10-12".to_string(),
        published_by: Some("Ananya Rao".to_string()),
        notes: Some("Finalized schedule".to_string()),
    };
    let pub_res = brightcone_hr_api::modules::time::roster::publish_roster(
        axum::extract::State(pool.clone()),
        axum::extract::Json(pub_payload),
    )
    .await
    .expect("publish_roster failed");

    assert_eq!(pub_res.0, axum::http::StatusCode::OK);

    // Verify roster retrieval now indicates is_published = true
    let verified_roster = brightcone_hr_api::modules::time::roster::get_weekly_roster(
        axum::extract::State(pool.clone()),
        axum::extract::Query(
            brightcone_hr_api::modules::time::models::WeeklyRosterQuery {
                week_start: Some("2026-10-12".to_string()),
            },
        ),
    )
    .await
    .expect("get_weekly_roster failed");

    assert_eq!(
        verified_roster.0.is_published, true,
        "Roster should be marked published"
    );
}

use axum::extract::{Json, State};
use brightcone_hr_api::modules::time::leave_calc::{
    count_leave_days_service, preview_leave_days_post, CountLeaveDaysRequest,
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_leave_day_count_service_and_api_integration() {
    dotenv().ok();
    let raw_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be defined");

    // Strictly connect to disposable test database brightcone_schema_sync_test
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

    // Strict safety guard
    let (current_db,): (String,) = sqlx::query_as("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("Failed to query current database");

    assert_eq!(
        current_db, test_db_name,
        "CRITICAL SAFETY ABORT: Must connect to test database, got: {}",
        current_db
    );

    // Sync schema & seed data
    brightcone_hr_api::db::sync_schema(&pool)
        .await
        .expect("Schema sync failed");
    brightcone_hr_api::modules::time::seed_if_empty(&pool)
        .await
        .expect("Seeding failed");

    // 1. Fetch Casual Leave (CL) and Maternity Leave (ML) from database
    let (cl_id,): (i64,) = sqlx::query_as("SELECT id FROM leave_types WHERE code = 'CL'")
        .fetch_one(&pool)
        .await
        .expect("CL leave type not found");

    let (ml_id,): (i64,) = sqlx::query_as("SELECT id FROM leave_types WHERE code = 'ML'")
        .fetch_one(&pool)
        .await
        .expect("ML leave type not found");

    // Fetch an employee from database
    let (emp_id,): (i64,) = sqlx::query_as("SELECT id FROM employees LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("Employee not found");

    // 2. Test Casual Leave across Republic Day 2026 (Mon Jan 26, 2026 is configured holiday in DB)
    // Range: Fri 2026-01-23 to Tue 2026-01-27 (5 days: Fri work, Sat/Sun weekend, Mon holiday, Tue work)
    let cl_req = CountLeaveDaysRequest {
        start_date: "2026-01-23".to_string(),
        end_date: "2026-01-27".to_string(),
        employee_id: Some(emp_id),
        leave_type_id: Some(cl_id),
        leave_type_code: None,
        is_half_day: None,
        count_calendar_days: None,
        exclude_weekends: None,
        exclude_holidays: None,
        applicable_location: None,
    };

    let cl_result = count_leave_days_service(&pool, &cl_req)
        .await
        .expect("count_leave_days_service failed for CL");

    assert_eq!(cl_result.total_calendar_days, 5);
    assert_eq!(cl_result.weekend_days, 2);
    assert_eq!(cl_result.holiday_days, 1);
    assert_eq!(cl_result.chargeable_days, 2.0); // Fri and Tue only
    assert_eq!(cl_result.policy_applied, "WORKING_DAYS");

    // 3. Test Maternity Leave across the exact same date range (statutory calendar days)
    let ml_req = CountLeaveDaysRequest {
        start_date: "2026-01-23".to_string(),
        end_date: "2026-01-27".to_string(),
        employee_id: Some(emp_id),
        leave_type_id: Some(ml_id),
        leave_type_code: None,
        is_half_day: None,
        count_calendar_days: None,
        exclude_weekends: None,
        exclude_holidays: None,
        applicable_location: None,
    };

    let ml_result = count_leave_days_service(&pool, &ml_req)
        .await
        .expect("count_leave_days_service failed for ML");

    assert_eq!(ml_result.total_calendar_days, 5);
    assert_eq!(ml_result.chargeable_days, 5.0); // Calendar days count consecutive days
    assert_eq!(ml_result.policy_applied, "CALENDAR_DAYS");

    // 4. Test HTTP Handler with valid request
    let api_res = preview_leave_days_post(State(pool.clone()), Json(cl_req))
        .await
        .expect("preview_leave_days_post handler failed");

    assert_eq!(api_res.0.chargeable_days, 2.0);

    // 5. Test validation error: End date earlier than start date returns 400 Bad Request
    let invalid_range_req = CountLeaveDaysRequest {
        start_date: "2026-01-27".to_string(),
        end_date: "2026-01-23".to_string(),
        employee_id: None,
        leave_type_id: None,
        leave_type_code: None,
        is_half_day: None,
        count_calendar_days: None,
        exclude_weekends: None,
        exclude_holidays: None,
        applicable_location: None,
    };

    let (status, Json(err)) = preview_leave_days_post(State(pool.clone()), Json(invalid_range_req))
        .await
        .expect_err("Should return error for invalid date range");

    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert!(err.error.contains("cannot be earlier than start date"));

    // 6. Test non-existent employee ID returns 404 Not Found
    let non_existent_emp_req = CountLeaveDaysRequest {
        start_date: "2026-02-01".to_string(),
        end_date: "2026-02-05".to_string(),
        employee_id: Some(999999),
        leave_type_id: None,
        leave_type_code: None,
        is_half_day: None,
        count_calendar_days: None,
        exclude_weekends: None,
        exclude_holidays: None,
        applicable_location: None,
    };

    let (status, Json(err)) =
        preview_leave_days_post(State(pool.clone()), Json(non_existent_emp_req))
            .await
            .expect_err("Should return 404 for non-existent employee");

    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
    assert!(err.error.contains("Employee with ID 999999 not found"));
}

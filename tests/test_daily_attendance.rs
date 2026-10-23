use axum::Json;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_daily_attendance_endpoints_and_logic() {
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

    // Synchronize schema (creates daily_attendance, employees, etc.)
    brightcone_hr_api::db::sync_schema(&pool)
        .await
        .expect("Schema sync failed");

    // Seed employees & shifts
    brightcone_hr_api::modules::time::seed_if_empty(&pool)
        .await
        .expect("Seed failed");

    let (emp_id,): (i64,) = sqlx::query_as("SELECT id FROM employees ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("Failed to get employee ID");

    // Clear any test attendance records in disposable database for clean testing
    sqlx::query("DELETE FROM daily_attendance WHERE employee_id = $1")
        .bind(emp_id)
        .execute(&pool)
        .await
        .ok();

    // 1. Test worked-hours calculation unit logic
    let day_hrs = brightcone_hr_api::modules::time::attendance::calc_worked_hours(
        "PRESENT",
        Some("09:00"),
        Some("17:30"),
    )
    .expect("Day hours calculation failed");
    assert_eq!(day_hrs, 8.5);

    let overnight_hrs = brightcone_hr_api::modules::time::attendance::calc_worked_hours(
        "PRESENT",
        Some("21:00"),
        Some("06:00"),
    )
    .expect("Overnight hours calculation failed");
    assert_eq!(overnight_hrs, 9.0);

    let absent_hrs =
        brightcone_hr_api::modules::time::attendance::calc_worked_hours("ABSENT", None, None)
            .expect("Absent hours calculation failed");
    assert_eq!(absent_hrs, 0.0);

    let half_day_hrs =
        brightcone_hr_api::modules::time::attendance::calc_worked_hours("HALF_DAY", None, None)
            .expect("Half day hours calculation failed");
    assert_eq!(half_day_hrs, 4.0);

    // 2. Test create attendance entry
    let create_payload = brightcone_hr_api::modules::time::models::CreateAttendancePayload {
        employee_id: emp_id,
        date: "2026-10-21".to_string(),
        attendance_status: "PRESENT".to_string(),
        in_time: Some("09:00:00".to_string()),
        out_time: Some("17:30:00".to_string()),
        entry_status: Some("RECORDED".to_string()),
        notes: Some("On time regular shift".to_string()),
    };

    let (status, Json(created)) = brightcone_hr_api::modules::time::attendance::create_attendance(
        axum::extract::State(pool.clone()),
        axum::extract::Json(create_payload),
    )
    .await
    .expect("create_attendance failed");

    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(created.employee_id, emp_id);
    assert_eq!(created.attendance_date, "2026-10-21");
    assert_eq!(created.attendance_status, "PRESENT");
    assert_eq!(created.hours_worked, 8.5);
    assert_eq!(created.entry_status, "RECORDED");
    assert_eq!(created.is_archived, false);

    // 3. Test duplicate prevention: same employee and date must return 409 Conflict
    let dup_payload = brightcone_hr_api::modules::time::models::CreateAttendancePayload {
        employee_id: emp_id,
        date: "2026-10-21".to_string(),
        attendance_status: "ABSENT".to_string(),
        in_time: None,
        out_time: None,
        entry_status: Some("PENDING".to_string()),
        notes: None,
    };

    let dup_res = brightcone_hr_api::modules::time::attendance::create_attendance(
        axum::extract::State(pool.clone()),
        axum::extract::Json(dup_payload),
    )
    .await;

    assert!(
        dup_res.is_err(),
        "Duplicate entry for same employee and date must fail"
    );
    let (err_status, _) = dup_res.err().unwrap();
    assert_eq!(err_status, axum::http::StatusCode::CONFLICT);

    // 4. Test invalid employee: non-existent employee ID must return 404 Not Found
    let inv_emp_payload = brightcone_hr_api::modules::time::models::CreateAttendancePayload {
        employee_id: 99999999,
        date: "2026-10-22".to_string(),
        attendance_status: "PRESENT".to_string(),
        in_time: Some("09:00".to_string()),
        out_time: Some("17:00".to_string()),
        entry_status: None,
        notes: None,
    };

    let inv_emp_res = brightcone_hr_api::modules::time::attendance::create_attendance(
        axum::extract::State(pool.clone()),
        axum::extract::Json(inv_emp_payload),
    )
    .await;

    assert!(
        inv_emp_res.is_err(),
        "Non-existent employee must return error"
    );
    let (err_status, _) = inv_emp_res.err().unwrap();
    assert_eq!(err_status, axum::http::StatusCode::NOT_FOUND);

    // 5. Test invalid times: Present without times must return 400 Bad Request
    let inv_time_payload = brightcone_hr_api::modules::time::models::CreateAttendancePayload {
        employee_id: emp_id,
        date: "2026-10-22".to_string(),
        attendance_status: "PRESENT".to_string(),
        in_time: None,
        out_time: None,
        entry_status: None,
        notes: None,
    };

    let inv_time_res = brightcone_hr_api::modules::time::attendance::create_attendance(
        axum::extract::State(pool.clone()),
        axum::extract::Json(inv_time_payload),
    )
    .await;

    assert!(
        inv_time_res.is_err(),
        "Present status without clock times must fail"
    );
    let (err_status, _) = inv_time_res.err().unwrap();
    assert_eq!(err_status, axum::http::StatusCode::BAD_REQUEST);

    // 6. Test update entry: change status to Half day and update times
    let update_payload = brightcone_hr_api::modules::time::models::UpdateAttendancePayload {
        attendance_status: Some("HALF_DAY".to_string()),
        in_time: Some("09:00:00".to_string()),
        out_time: Some("13:00:00".to_string()),
        entry_status: Some("REGULARISED".to_string()),
        is_archived: None,
        notes: Some("Half day approved by lead".to_string()),
    };

    let Json(updated) = brightcone_hr_api::modules::time::attendance::update_attendance(
        axum::extract::State(pool.clone()),
        axum::extract::Path(created.id),
        axum::extract::Json(update_payload),
    )
    .await
    .expect("update_attendance failed");

    assert_eq!(updated.attendance_status, "HALF_DAY");
    assert_eq!(updated.hours_worked, 4.0);
    assert_eq!(updated.entry_status, "REGULARISED");
    assert_eq!(updated.notes, Some("Half day approved by lead".to_string()));

    // 7. Test listing and summary counts
    let list_res = brightcone_hr_api::modules::time::attendance::list_attendance(
        axum::extract::State(pool.clone()),
        axum::extract::Query(brightcone_hr_api::modules::time::models::AttendanceQuery {
            search: None,
            status: Some("HALF_DAY".to_string()),
            entry_status: None,
            archived: Some(false),
            date: None,
        }),
    )
    .await
    .expect("list_attendance failed");

    assert!(
        !list_res.0.records.is_empty(),
        "Filtered records should contain the updated entry"
    );
    assert!(
        list_res.0.summary.total_records >= 1,
        "Total live records must be >= 1"
    );
    assert!(
        list_res.0.summary.regularised_count >= 1,
        "Regularised count must be >= 1"
    );

    // 8. Test archive toggle
    let Json(archived_rec) =
        brightcone_hr_api::modules::time::attendance::toggle_archive_attendance(
            axum::extract::State(pool.clone()),
            axum::extract::Path(created.id),
        )
        .await
        .expect("toggle_archive failed");

    assert_eq!(
        archived_rec.is_archived, true,
        "Record should now be archived"
    );

    // 9. Test CSV Export endpoint
    let csv_res = brightcone_hr_api::modules::time::attendance::export_attendance_csv(
        axum::extract::State(pool.clone()),
        axum::extract::Query(brightcone_hr_api::modules::time::models::AttendanceQuery {
            search: None,
            status: None,
            entry_status: None,
            archived: Some(true),
            date: None,
        }),
    )
    .await
    .expect("export_attendance_csv failed");

    use axum::response::IntoResponse;
    let resp = csv_res.into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::OK);
}

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
async fn test_db_connection() {
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
    println!("Database schema synchronized successfully!");

    // Test insert and query shifts
    sqlx::query(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active) \
         VALUES ($1, $2, $3::TIME, $4::TIME, $5, $6, $7, $8) \
         ON CONFLICT (code) DO NOTHING"
    )
    .bind("General Day Shift")
    .bind("GEN-01")
    .bind("09:00:00")
    .bind("18:00:00")
    .bind(60)
    .bind(15)
    .bind(false)
    .bind(true)
    .execute(&pool)
    .await
    .expect("Insert shift failed");

    let shifts: Vec<Shift> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, start_time::TEXT as start_time, end_time::TEXT as end_time, \
         break_duration_minutes, grace_period_minutes, is_overnight, is_active, \
         to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
         to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM shifts ORDER BY id ASC"
    )
    .fetch_all(&pool)
    .await
    .expect("Fetch shifts failed");

    println!("Fetched shifts: {:?}", shifts);
    assert!(!shifts.is_empty());

    // Test insert and query leave types
    sqlx::query(
        "INSERT INTO leave_types (name, code, description, is_paid, is_active) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (code) DO NOTHING",
    )
    .bind("Casual Leave")
    .bind("CL")
    .bind("For personal errands")
    .bind(true)
    .bind(true)
    .execute(&pool)
    .await
    .expect("Insert leave type failed");

    let leave_types: Vec<LeaveType> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, code, description, is_paid, is_active, \
         to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
         to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM leave_types ORDER BY id ASC",
    )
    .fetch_all(&pool)
    .await
    .expect("Fetch leave types failed");

    println!("Fetched leave types: {:?}", leave_types);
    assert!(!leave_types.is_empty());

    // Test insert and query leave quotas
    let lt_id = leave_types[0].id;
    sqlx::query(
        "INSERT INTO leave_quotas (leave_type_id, year, entitled_days, applicable_group) \
         VALUES ($1, $2, $3::NUMERIC, $4) \
         ON CONFLICT (leave_type_id, year, applicable_group) DO NOTHING",
    )
    .bind(lt_id)
    .bind(2026)
    .bind(12.0)
    .bind("ALL")
    .execute(&pool)
    .await
    .expect("Insert leave quota failed");

    let quotas: Vec<LeaveQuota> = sqlx::query_as(
        "SELECT lq.id, lq.pid::TEXT as pid, lq.leave_type_id, lt.name as leave_type_name, lt.code as leave_type_code, \
         lq.year, lq.entitled_days::FLOAT8 as entitled_days, lq.applicable_group, \
         to_char(lq.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
         to_char(lq.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM leave_quotas lq \
         LEFT JOIN leave_types lt ON lq.leave_type_id = lt.id \
         ORDER BY lq.id ASC"
    )
    .fetch_all(&pool)
    .await
    .expect("Fetch leave quotas failed");

    println!("Fetched leave quotas: {:?}", quotas);
    assert!(!quotas.is_empty());

    // Test insert and query holidays
    sqlx::query(
        "INSERT INTO holidays (name, holiday_date, description, applicable_location, is_recurring) \
         VALUES ($1, $2::DATE, $3, $4, $5) \
         ON CONFLICT DO NOTHING"
    )
    .bind("Republic Day")
    .bind("2026-01-26")
    .bind("National Holiday")
    .bind(None::<String>)
    .bind(true)
    .execute(&pool)
    .await
    .expect("Insert holiday failed");

    let holidays: Vec<Holiday> = sqlx::query_as(
        "SELECT id, pid::TEXT as pid, name, holiday_date::TEXT as holiday_date, description, applicable_location, is_recurring, \
         to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
         to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at \
         FROM holidays ORDER BY holiday_date ASC"
    )
    .fetch_all(&pool)
    .await
    .expect("Fetch holidays failed");

    println!("Fetched holidays: {:?}", holidays);
    assert!(!holidays.is_empty());
}

pub mod attendance;
pub mod corrections;
pub mod holidays;
pub mod leave_calc;
pub mod leave_quotas;
pub mod leave_types;
pub mod models;
pub mod roster;
pub mod shifts;

use sqlx::PgPool;

#[allow(unused_imports)]
pub use attendance::*;
#[allow(unused_imports)]
pub use corrections::*;

#[allow(unused_imports)]
pub use holidays::*;
#[allow(unused_imports)]
pub use leave_calc::*;
#[allow(unused_imports)]
pub use leave_quotas::*;
#[allow(unused_imports)]
pub use leave_types::*;
#[allow(unused_imports)]
pub use models::*;
#[allow(unused_imports)]
pub use roster::*;
#[allow(unused_imports)]
pub use shifts::*;

pub type SharedTimeState = PgPool;

// =========================================================================
// Database Seed Helper
// =========================================================================

pub async fn seed_if_empty(pool: &PgPool) -> Result<(), sqlx::Error> {
    let (shift_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM shifts")
        .fetch_one(pool)
        .await?;

    if shift_count == 0 {
        sqlx::query(
            "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active)
             VALUES 
             ('General Day Shift', 'GEN-01', '09:00:00'::TIME, '18:00:00'::TIME, 60, 15, false, true),
             ('Early Morning Shift', 'MOR-01', '06:00:00'::TIME, '14:30:00'::TIME, 45, 10, false, true),
             ('Night Support Shift', 'NGT-01', '21:00:00'::TIME, '06:00:00'::TIME, 60, 15, true, true),
             ('Weekend Standby Shift', 'WKD-01', '10:00:00'::TIME, '16:00:00'::TIME, 30, 20, false, false)
             ON CONFLICT (code) DO NOTHING"
        )
        .execute(pool)
        .await?;
    }

    // Ensure USP and USC shifts exist for full screenshot fidelity
    sqlx::query(
        "INSERT INTO shifts (name, code, start_time, end_time, break_duration_minutes, grace_period_minutes, is_overnight, is_active)
         VALUES 
         ('US Pacific Shift', 'USP', '18:30:00'::TIME, '03:30:00'::TIME, 60, 15, true, true),
         ('US Central Shift', 'USC', '19:30:00'::TIME, '04:30:00'::TIME, 60, 15, true, true)
         ON CONFLICT (code) DO NOTHING"
    )
    .execute(pool)
    .await?;

    let (holiday_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM holidays")
        .fetch_one(pool)
        .await?;

    if holiday_count == 0 {
        sqlx::query(
            "INSERT INTO holidays (name, holiday_date, description, applicable_location, is_recurring)
             VALUES 
             ('Republic Day', '2026-01-26'::DATE, 'National holiday commemorating the Constitution', NULL, true),
             ('Independence Day', '2026-08-15'::DATE, 'National Independence celebration', NULL, true),
             ('Diwali', '2026-11-08'::DATE, 'Festival of Lights holiday', NULL, false),
             ('Telangana Formation Day', '2026-06-02'::DATE, 'State holiday for Hyderabad branch office', 'Hyderabad', true),
             ('Karnataka Rajyotsava', '2026-11-01'::DATE, 'State holiday for Bengaluru technology centre', 'Bengaluru', true)
             ON CONFLICT DO NOTHING"
        )
        .execute(pool)
        .await?;
    }

    let (leave_type_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_types")
        .fetch_one(pool)
        .await?;

    if leave_type_count == 0 {
        sqlx::query(
            "INSERT INTO leave_types (name, code, description, is_paid, is_active)
             VALUES 
             ('Casual Leave', 'CL', 'For personal errands, family obligations, and unplanned short absences.', true, true),
             ('Sick Leave', 'SL', 'For recovery from personal illnesses, medical appointments, and healthcare needs.', true, true),
             ('Earned Leave / Annual Vacation', 'EL', 'Planned vacation days accumulated monthly throughout the active service year.', true, true),
             ('Maternity Leave', 'ML', 'Statutory paid leave for expectant mothers and post-delivery care.', true, true),
             ('Loss of Pay', 'LOP', 'Unpaid leaves taken after exhaust of eligible paid quotas.', false, true)
             ON CONFLICT (code) DO NOTHING"
        )
        .execute(pool)
        .await?;
    }

    let (quota_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM leave_quotas")
        .fetch_one(pool)
        .await?;

    if quota_count == 0 {
        sqlx::query(
            "INSERT INTO leave_quotas (leave_type_id, year, entitled_days, applicable_group)
             SELECT id, 2026, 12.0, 'ALL' FROM leave_types WHERE code = 'CL'
             UNION ALL
             SELECT id, 2026, 10.5, 'ALL' FROM leave_types WHERE code = 'SL'
             UNION ALL
             SELECT id, 2026, 18.0, 'ALL' FROM leave_types WHERE code = 'EL'
             UNION ALL
             SELECT id, 2026, 90.0, 'FEMALE_EMPLOYEES' FROM leave_types WHERE code = 'ML'
             ON CONFLICT (leave_type_id, year, applicable_group) DO NOTHING",
        )
        .execute(pool)
        .await?;
    }

    // Seed Employees if empty (matching Weekly Roster reference)
    let (emp_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM employees")
        .fetch_one(pool)
        .await?;

    if emp_count == 0 {
        sqlx::query(
            "INSERT INTO employees (name, email, role, department, default_shift_id, avatar_color)
             VALUES
             ('Naresh Vemparala', 'naresh.v@brightcone.com', 'Senior Engineer', 'Platform', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'purple'),
             ('Ananya Rao', 'ananya.rao@brightcone.com', 'People Ops Lead', 'Human Resources', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'purple'),
             ('Karthik Reddy', 'karthik.r@brightcone.com', 'Full Stack Developer', 'Engineering', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'teal'),
             ('Priya Menon', 'priya.m@brightcone.com', 'UI/UX Designer', 'Product', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'purple'),
             ('Daniel Brooks', 'daniel.b@brightcone.com', 'Support Specialist', 'Customer Success', (SELECT id FROM shifts WHERE code = 'USP' LIMIT 1), 'pink'),
             ('Vikram Seth', 'vikram.s@brightcone.com', 'DevOps Lead', 'Infrastructure', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'blue'),
             ('Aisha Khan', 'aisha.k@brightcone.com', 'QA Engineer', 'Quality', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'indigo'),
             ('Rahul Sharma', 'rahul.s@brightcone.com', 'Backend Developer', 'Engineering', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'emerald'),
             ('Sneha Patel', 'sneha.p@brightcone.com', 'Product Manager', 'Product', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'amber'),
             ('David Miller', 'david.m@brightcone.com', 'Cloud Architect', 'Infrastructure', (SELECT id FROM shifts WHERE code = 'USP' LIMIT 1), 'rose'),
             ('Samantha Reed', 'samantha.r@brightcone.com', 'Technical Writer', 'Documentation', (SELECT id FROM shifts WHERE code = 'USC' LIMIT 1), 'cyan'),
             ('Tanvi Joshi', 'tanvi.j@brightcone.com', 'Security Analyst', 'Security', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'violet'),
             ('Arjun Kapoor', 'arjun.k@brightcone.com', 'Operations Engineer', 'Operations', (SELECT id FROM shifts WHERE code LIKE 'NGT%' LIMIT 1), 'fuchsia'),
             ('Elena Rostova', 'elena.r@brightcone.com', 'Frontend Engineer', 'Web Team', (SELECT id FROM shifts WHERE code LIKE 'GEN%' LIMIT 1), 'teal')
             ON CONFLICT (email) DO NOTHING"
        )
        .execute(pool)
        .await?;
    }

    // Seed swap requests if empty
    let (swap_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM swap_requests")
        .fetch_one(pool)
        .await?;

    if swap_count == 0 {
        sqlx::query(
            "INSERT INTO swap_requests (requester_id, target_employee_id, request_date, status, reason)
             SELECT e1.id, e2.id, '2026-10-15'::DATE, 'PENDING', 'Need morning shift due to doctor appointment'
             FROM employees e1, employees e2
             WHERE e1.name = 'Naresh Vemparala' AND e2.name = 'Karthik Reddy'
             LIMIT 1"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT INTO swap_requests (requester_id, target_employee_id, request_date, status, reason)
             SELECT e1.id, e2.id, '2026-10-16'::DATE, 'PENDING', 'Family event on Friday afternoon'
             FROM employees e1, employees e2
             WHERE e1.name = 'Daniel Brooks' AND e2.name = 'David Miller'
             LIMIT 1"
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

use sea_orm::{ConnectionTrait, DatabaseConnection};
use tracing::warn;

pub async fn sync_suite_tables(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let suite_tables_sql = [
        r#"
        CREATE TABLE IF NOT EXISTS suite_approvals (
            id VARCHAR PRIMARY KEY,
            module VARCHAR NOT NULL,
            title VARCHAR NOT NULL,
            requester_name VARCHAR NOT NULL,
            requester_role VARCHAR,
            requester_dept VARCHAR,
            details TEXT,
            amount_or_duration VARCHAR,
            submitted_at VARCHAR,
            status VARCHAR NOT NULL DEFAULT 'Pending',
            priority VARCHAR NOT NULL DEFAULT 'Normal',
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS career_promotions (
            id VARCHAR PRIMARY KEY,
            employee_name VARCHAR NOT NULL,
            "current_role" VARCHAR NOT NULL,
            proposed_role VARCHAR NOT NULL,
            department VARCHAR NOT NULL,
            requested_by VARCHAR NOT NULL,
            status VARCHAR NOT NULL DEFAULT 'pending',
            date VARCHAR NOT NULL,
            justification TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS hr_compliance_rules (
            id VARCHAR PRIMARY KEY,
            name VARCHAR NOT NULL,
            category VARCHAR NOT NULL,
            description TEXT,
            is_active BOOLEAN NOT NULL DEFAULT true,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
        "#,
        r#"
        CREATE TABLE IF NOT EXISTS hr_actions_reminders (
            id VARCHAR PRIMARY KEY,
            title VARCHAR NOT NULL,
            message TEXT,
            priority VARCHAR NOT NULL DEFAULT 'medium',
            due_date VARCHAR,
            status VARCHAR NOT NULL DEFAULT 'pending',
            category VARCHAR,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );
        "#,
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS action VARCHAR(255) DEFAULT '';",
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS related_employee VARCHAR(255) DEFAULT '';",
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS owner VARCHAR(255) DEFAULT '';",
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS due VARCHAR(50) DEFAULT '';",
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS remind_me VARCHAR(100) DEFAULT '1 day before';",
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS repeat_frequency VARCHAR(50) DEFAULT 'None';",
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS source VARCHAR(255) DEFAULT 'Manual';",
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS notes TEXT DEFAULT '';",
        "ALTER TABLE hr_actions_reminders ADD COLUMN IF NOT EXISTS archived BOOLEAN DEFAULT FALSE;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS grade VARCHAR DEFAULT 'L1';",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS location VARCHAR DEFAULT 'Hyderabad';",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS manager_name VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS work_email VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS phone VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS shift VARCHAR DEFAULT 'General';",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS joined VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS ctc DOUBLE PRECISION;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS address VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS dob VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS blood_group VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS emergency_contact_name VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS emergency_contact_relation VARCHAR;",
        "ALTER TABLE employees ADD COLUMN IF NOT EXISTS emergency_contact_phone VARCHAR;",
    ];

    for sql in suite_tables_sql {
        if let Err(e) = db.execute_unprepared(sql).await {
            warn!("[SCHEMA-SYNC] Notice on suite table/column: {}", e);
        }
    }
    Ok(())
}

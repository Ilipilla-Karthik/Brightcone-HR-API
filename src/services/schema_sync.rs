use sea_orm::{
    sea_query::{
        Alias, Expr, ExprTrait, Iden, IndexCreateStatement, Query, StringLen, TableCreateStatement,
    },
    ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityName, EntityTrait, Iterable,
    Schema,
};
use std::collections::HashSet;
use tracing::{info, warn};

use crate::models::_entities::prelude::*;

pub struct RegisteredEntity {
    pub table_name: &'static str,
    pub create_table: Box<dyn Fn(DbBackend) -> TableCreateStatement + Send + Sync>,
    pub create_indexes: Box<dyn Fn(DbBackend) -> Vec<IndexCreateStatement> + Send + Sync>,
    pub columns: Box<dyn Fn() -> Vec<RegisteredColumn> + Send + Sync>,
}

pub struct RegisteredColumn {
    pub name: String,
    pub col_type_sql: String,
    pub nullable: bool,
}

fn register_entity<E>() -> RegisteredEntity
where
    E: EntityTrait + EntityName + Default + 'static,
    <E as EntityTrait>::Column: Iterable + Iden,
{
    let table_name = <E as EntityName>::table_name(&E::default());
    RegisteredEntity {
        table_name,
        create_table: Box::new(|backend| {
            let schema = Schema::new(backend);
            let mut stmt = schema.create_table_from_entity(E::default());
            stmt.if_not_exists();
            stmt
        }),
        create_indexes: Box::new(|backend| {
            let schema = Schema::new(backend);
            schema.create_index_from_entity(E::default())
        }),
        columns: Box::new(|| {
            let mut cols = Vec::new();
            for col in <E as EntityTrait>::Column::iter() {
                let name = col.to_string();
                let def = col.def();
                let col_type = def.get_column_type();
                let type_sql = match col_type {
                    sea_orm::sea_query::ColumnType::Char(len) => match len {
                        Some(l) => format!("character({})", l),
                        None => "character(1)".to_string(),
                    },
                    sea_orm::sea_query::ColumnType::String(len) => match len {
                        StringLen::N(l) => format!("varchar({})", l),
                        StringLen::None => "varchar".to_string(),
                        StringLen::Max => "text".to_string(),
                    },
                    sea_orm::sea_query::ColumnType::Text => "text".to_string(),
                    sea_orm::sea_query::ColumnType::TinyInteger => "smallint".to_string(),
                    sea_orm::sea_query::ColumnType::SmallInteger => "smallint".to_string(),
                    sea_orm::sea_query::ColumnType::Integer => "integer".to_string(),
                    sea_orm::sea_query::ColumnType::BigInteger => "bigint".to_string(),
                    sea_orm::sea_query::ColumnType::TinyUnsigned => "smallint".to_string(),
                    sea_orm::sea_query::ColumnType::SmallUnsigned => "integer".to_string(),
                    sea_orm::sea_query::ColumnType::Unsigned => "bigint".to_string(),
                    sea_orm::sea_query::ColumnType::BigUnsigned => "numeric(20, 0)".to_string(),
                    sea_orm::sea_query::ColumnType::Float => "real".to_string(),
                    sea_orm::sea_query::ColumnType::Double => "double precision".to_string(),
                    sea_orm::sea_query::ColumnType::Decimal(precision) => match precision {
                        Some((p, s)) => format!("numeric({}, {})", p, s),
                        None => "numeric".to_string(),
                    },
                    sea_orm::sea_query::ColumnType::DateTime => {
                        "timestamp without time zone".to_string()
                    }
                    sea_orm::sea_query::ColumnType::Timestamp => {
                        "timestamp without time zone".to_string()
                    }
                    sea_orm::sea_query::ColumnType::TimestampWithTimeZone => {
                        "timestamp with time zone".to_string()
                    }
                    sea_orm::sea_query::ColumnType::Time => "time without time zone".to_string(),
                    sea_orm::sea_query::ColumnType::Date => "date".to_string(),
                    sea_orm::sea_query::ColumnType::Year => "smallint".to_string(),
                    sea_orm::sea_query::ColumnType::Interval(_, _) => "interval".to_string(),
                    sea_orm::sea_query::ColumnType::Binary(_) => "bytea".to_string(),
                    sea_orm::sea_query::ColumnType::VarBinary(_) => "bytea".to_string(),
                    sea_orm::sea_query::ColumnType::Bit(_) => "bit".to_string(),
                    sea_orm::sea_query::ColumnType::VarBit(_) => "varbit".to_string(),
                    sea_orm::sea_query::ColumnType::Boolean => "boolean".to_string(),
                    sea_orm::sea_query::ColumnType::Money(precision) => match precision {
                        Some((p, s)) => format!("numeric({}, {})", p, s),
                        None => "numeric".to_string(),
                    },
                    sea_orm::sea_query::ColumnType::Json => "json".to_string(),
                    sea_orm::sea_query::ColumnType::JsonBinary => "jsonb".to_string(),
                    sea_orm::sea_query::ColumnType::Uuid => "uuid".to_string(),
                    sea_orm::sea_query::ColumnType::Array(_) => "text[]".to_string(),
                    sea_orm::sea_query::ColumnType::Enum { .. } => "varchar".to_string(),
                    _ => "text".to_string(),
                };
                let nullable = def.is_null();
                cols.push(RegisteredColumn {
                    name,
                    col_type_sql: type_sql,
                    nullable,
                });
            }
            cols
        }),
    }
}

pub fn get_entity_registry() -> Vec<RegisteredEntity> {
    vec![
        register_entity::<Users>(),
        register_entity::<Departments>(),
        register_entity::<Employees>(),
        register_entity::<Jobs>(),
        register_entity::<Candidates>(),
        register_entity::<Applications>(),
        register_entity::<ApplicationHistories>(),
        register_entity::<Interviews>(),
        register_entity::<Offers>(),
        register_entity::<Referrals>(),
        register_entity::<AttendanceRecords>(),
        register_entity::<LeaveRequests>(),
        register_entity::<HelpdeskTickets>(),
        register_entity::<ResignationRequests>(),
        register_entity::<EmployeeAssets>(),
        register_entity::<EmployeeDocuments>(),
        register_entity::<PerformanceGoals>(),
        register_entity::<PerformanceReviews>(),
        register_entity::<AuditLogs>(),
        register_entity::<Courses>(),
        register_entity::<CourseAssignments>(),
        register_entity::<CourseReminders>(),
        register_entity::<ReviewCycles>(),
        register_entity::<CycleReviews>(),
    ]
}

pub async fn run_schema_synchronization(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("[SCHEMA-SYNC] Starting automatic entity-first database schema synchronization...");

    // Acquire PostgreSQL advisory lock to ensure safe atomic sync across concurrent backend instances
    let _ = db
        .execute_unprepared("SELECT pg_advisory_lock(88421038);")
        .await?;
    info!("[SCHEMA-SYNC] Acquired advisory lock 88421038.");

    let sync_res = run_sync_inner(db).await;

    // Release advisory lock
    let _ = db
        .execute_unprepared("SELECT pg_advisory_unlock(88421038);")
        .await;
    info!("[SCHEMA-SYNC] Released advisory lock 88421038.");

    sync_res
}

async fn run_sync_inner(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let backend = db.get_database_backend();
    let registry = get_entity_registry();
    info!(
        "[SCHEMA-SYNC] Registered {} domain entities for synchronization.",
        registry.len()
    );

    // 1. Inspect existing tables in the database
    let mut table_query = Query::select();
    table_query
        .column(Alias::new("table_name"))
        .from((Alias::new("information_schema"), Alias::new("tables")))
        .and_where(Expr::col(Alias::new("table_schema")).eq("public"))
        .and_where(Expr::col(Alias::new("table_type")).eq("BASE TABLE"));

    let table_rows = db.query_all(&table_query).await?;
    let mut existing_tables = HashSet::new();
    for row in table_rows {
        let name: String = row.try_get("", "table_name")?;
        existing_tables.insert(name);
    }
    info!(
        "[SCHEMA-SYNC] Found {} existing base tables in PostgreSQL public schema.",
        existing_tables.len()
    );

    // 2. Process each registered entity
    for entity in &registry {
        let table_name = entity.table_name;
        if !existing_tables.contains(table_name) {
            info!(
                "[SCHEMA-SYNC] Table '{}' does not exist. Creating table from entity definition...",
                table_name
            );
            let create_stmt = (entity.create_table)(backend);
            db.execute(&create_stmt).await?;
            info!("[SCHEMA-SYNC] Successfully created table '{}'.", table_name);
        } else {
            // Table exists. Check for missing columns and synchronize safely
            let mut col_query = Query::select();
            col_query
                .column(Alias::new("column_name"))
                .column(Alias::new("data_type"))
                .column(Alias::new("is_nullable"))
                .from((Alias::new("information_schema"), Alias::new("columns")))
                .and_where(Expr::col(Alias::new("table_schema")).eq("public"))
                .and_where(Expr::col(Alias::new("table_name")).eq(table_name));

            let col_rows = db.query_all(&col_query).await?;
            let mut existing_cols = HashSet::new();
            for r in col_rows {
                let col_name: String = r.try_get("", "column_name")?;
                existing_cols.insert(col_name);
            }

            let entity_cols = (entity.columns)();
            for col in entity_cols {
                if !existing_cols.contains(&col.name) {
                    info!(
                        "[SCHEMA-SYNC] Column '{}.{}' missing in existing table. Adding safely (type: {}, nullable: {})...",
                        table_name, col.name, col.col_type_sql, col.nullable
                    );
                    // Add column safely without dropping or breaking existing data
                    let add_sql = if col.nullable {
                        format!(
                            "ALTER TABLE \"{}\" ADD COLUMN IF NOT EXISTS \"{}\" {};",
                            table_name, col.name, col.col_type_sql
                        )
                    } else {
                        // For non-nullable columns added to existing tables, keep nullable during sync to avoid breaking existing rows
                        format!(
                            "ALTER TABLE \"{}\" ADD COLUMN IF NOT EXISTS \"{}\" {};",
                            table_name, col.name, col.col_type_sql
                        )
                    };
                    db.execute_unprepared(&add_sql).await?;
                    info!("[SCHEMA-SYNC] Added column '{}.{}'.", table_name, col.name);
                }
            }
        }

        // Ensure default timestamps in PostgreSQL if table has created_at / updated_at
        let _ = db
            .execute_unprepared(&format!(
                "ALTER TABLE \"{}\" ALTER COLUMN \"created_at\" SET DEFAULT CURRENT_TIMESTAMP;",
                table_name
            ))
            .await;
        let _ = db
            .execute_unprepared(&format!(
                "ALTER TABLE \"{}\" ALTER COLUMN \"updated_at\" SET DEFAULT CURRENT_TIMESTAMP;",
                table_name
            ))
            .await;

        // Apply entity indexes safely (using CREATE INDEX IF NOT EXISTS)
        let index_stmts = (entity.create_indexes)(backend);
        for mut idx in index_stmts {
            idx.if_not_exists();
            if let Err(e) = db.execute(&idx).await {
                // If index already exists or fails non-critically, log warning
                warn!(
                    "[SCHEMA-SYNC] Note on index for table '{}': {}",
                    table_name, e
                );
            }
        }
    }

    // 3. Ensure required composite constraints and domain indexes for REC-09 and REC-12
    let domain_indexes = [
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_course_assignments_unique ON course_assignments (course_id, employee_id);",
        "CREATE INDEX IF NOT EXISTS idx_course_assignments_emp ON course_assignments (employee_id);",
        "CREATE INDEX IF NOT EXISTS idx_course_reminders_assign ON course_reminders (assignment_id);",
        "CREATE INDEX IF NOT EXISTS idx_course_reminders_created ON course_reminders (created_at);",
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_cycle_reviews_unique ON cycle_reviews (cycle_id, employee_id);",
        "CREATE INDEX IF NOT EXISTS idx_cycle_reviews_cycle ON cycle_reviews (cycle_id);",
        "CREATE INDEX IF NOT EXISTS idx_cycle_reviews_emp ON cycle_reviews (employee_id);",
        "CREATE INDEX IF NOT EXISTS idx_review_cycles_status ON review_cycles (status);",
    ];
    for sql in domain_indexes {
        if let Err(e) = db.execute_unprepared(sql).await {
            warn!("[SCHEMA-SYNC] Domain index notice: {}", e);
        }
    }

    // 4. Final verification of all registered tables
    for entity in &registry {
        let verify_sql = format!("SELECT 1 FROM \"{}\" LIMIT 1;", entity.table_name);
        db.execute_unprepared(&verify_sql).await.map_err(|e| {
            format!(
                "Verification failed for table '{}': {}",
                entity.table_name, e
            )
        })?;
    }

    info!(
        "[SCHEMA-SYNC] Verification passed for all {} registered entities.",
        registry.len()
    );

    // 5. Seed initial review cycle if none exist
    let seed_cycle_sql = "INSERT INTO review_cycles (id, name, stage, self_review_due, status, created_at, updated_at) VALUES ('CYC-H2-2026', 'H2 2026 review', 2, '2026-11-15', 'Active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT (id) DO NOTHING;";
    let _ = db.execute_unprepared(seed_cycle_sql).await;

    let seed_reviews_sql = r#"
        INSERT INTO cycle_reviews (id, cycle_id, employee_id, employee_name, manager_name, department, self_review_status, self_rating, goals_count, rating, recommended_increment_percentage, created_at, updated_at)
        SELECT
            'REV-' || e.employee_code || '-H2-2026',
            'CYC-H2-2026',
            e.employee_code,
            e.full_name,
            COALESCE(e.manager_name, '—'),
            e.department,
            CASE 
                WHEN e.full_name IN ('Rahul Varma', 'Sneha Iyer', 'Arjun Mehta', 'Kavita Nair') THEN 'Submitted'
                ELSE 'Not submitted'
            END,
            CASE
                WHEN e.full_name = 'Rahul Varma' THEN '4.2'
                WHEN e.full_name = 'Sneha Iyer' THEN '4.8'
                WHEN e.full_name = 'Arjun Mehta' THEN '4.0'
                WHEN e.full_name = 'Kavita Nair' THEN '3.9'
                ELSE NULL
            END,
            CASE
                WHEN e.full_name = 'Rahul Varma' THEN 3
                WHEN e.full_name = 'Ananya Rao' THEN 1
                ELSE 2
            END,
            CASE
                WHEN e.full_name = 'Rahul Varma' THEN 'Exceeds expectations'
                WHEN e.full_name = 'Sneha Iyer' THEN 'Outstanding'
                ELSE NULL
            END,
            CASE
                WHEN e.full_name = 'Rahul Varma' THEN 10.0
                WHEN e.full_name = 'Sneha Iyer' THEN 15.0
                ELSE NULL
            END,
            CURRENT_TIMESTAMP,
            CURRENT_TIMESTAMP
        FROM employees e
        WHERE e.status != 'Inactive'
        ON CONFLICT (cycle_id, employee_id) DO NOTHING;
    "#;
    let _ = db.execute_unprepared(seed_reviews_sql).await;

    info!("[SCHEMA-SYNC] Schema synchronization completed successfully.");
    Ok(())
}

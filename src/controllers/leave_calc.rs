//! Quota lookup and weekday calculations for employee leave requests.
use crate::services::lookup::weekdays;
use chrono::Datelike;
use loco_rs::prelude::*;

/// Resolve the signed-in user's employee row via their login email.
pub async fn own_employee(
    db: &DatabaseConnection,
    email: &str,
) -> Result<(String, String)> {
    let row = sea_orm::Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT employee_code, full_name FROM employees \
         WHERE lower(work_email) = lower($1) LIMIT 1",
        [email.to_owned().into()],
    );
    let rows = db.query_all_raw(row).await?;
    match rows.first() {
        Some(r) => {
            let code: String = r.try_get("", "employee_code").unwrap_or_default();
            let name: String = r.try_get("", "full_name").unwrap_or_default();
            Ok((code, name))
        }
        None => Err(Error::BadRequest(
            "Your account is not linked to an employee record. Contact HR to be added to the \
             employee master before applying for leave."
                .to_string(),
        )),
    }
}

/// Weekdays in [from, to], minus holidays falling on weekdays.
pub async fn leave_days(db: &DatabaseConnection, from: &str, to: &str) -> f64 {
    let mut n = weekdays(from, to) as f64;
    let st = sea_orm::Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT count(*) AS n FROM holidays \
         WHERE holiday_date >= $1 AND holiday_date <= $2 \
           AND extract(dow FROM holiday_date::date) NOT IN (0, 6)",
        [from.to_owned().into(), to.to_owned().into()],
    );
    if let Ok(rows) = db.query_all_raw(st).await {
        if let Some(h) = rows.first().and_then(|r| r.try_get::<i64>("", "n").ok()) {
            n -= h as f64;
        }
    }
    n.max(0.0)
}

/// Map a UI leave-type name (or code) to the `leave_types` row.
pub async fn resolve_type(
    db: &DatabaseConnection,
    asked: &str,
) -> Option<(String, String, f64)> {
    let st = sea_orm::Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT code, name, days_allowed::float8 AS days_allowed FROM leave_types \
         WHERE lower(code) = lower($1) OR lower(name) = lower($1) \
            OR lower(name) = lower($1 || ' leave') LIMIT 1",
        [asked.to_owned().into()],
    );
    let rows = db.query_all_raw(st).await.ok()?;
    let r = rows.first()?;
    Some((
        r.try_get("", "code").unwrap_or_default(),
        r.try_get("", "name").unwrap_or_default(),
        r.try_get::<f64>("", "days_allowed").unwrap_or(0.0),
    ))
}

/// Approved + pending days already booked for this employee/type/year.
pub async fn booked_days(
    db: &DatabaseConnection,
    emp: &str,
    type_name: &str,
    year: i32,
) -> (f64, f64) {
    let like = format!("{year}%");
    let st = sea_orm::Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT \
           COALESCE(sum(CASE WHEN status LIKE 'Approved%' \
             THEN regexp_replace(days, '[^0-9.]', '', 'g')::numeric ELSE 0 END), 0)::float8 AS used, \
           COALESCE(sum(CASE WHEN status NOT LIKE 'Approved%' AND status NOT LIKE 'Rejected%' \
             THEN regexp_replace(days, '[^0-9.]', '', 'g')::numeric ELSE 0 END), 0)::float8 AS pending \
         FROM leave_requests \
         WHERE employee_code = $1 AND lower(leave_type) = lower($2) \
           AND (from_date LIKE $3 OR to_date LIKE $3)",
        [
            emp.to_owned().into(),
            type_name.to_owned().into(),
            like.into(),
        ],
    );
    match db.query_all_raw(st).await {
        Ok(rows) => {
            let r = rows.first();
            let used = r
                .and_then(|x| x.try_get::<f64>("", "used").ok())
                .unwrap_or(0.0);
            let pending = r
                .and_then(|x| x.try_get::<f64>("", "pending").ok())
                .unwrap_or(0.0);
            (used, pending)
        }
        Err(_) => (0.0, 0.0),
    }
}

/// Allocated days for one employee/type/year: `leave_quotas` first, then the
/// `leave_types` default.
pub async fn allocated_days(
    db: &DatabaseConnection,
    emp: &str,
    code: &str,
    fallback: f64,
    year: i32,
) -> f64 {
    let st = sea_orm::Statement::from_sql_and_values(
        db.get_database_backend(),
        "SELECT allocated::float8 AS allocated FROM leave_quotas \
         WHERE employee_code = $1 AND upper(leave_type_code) = upper($2) AND year = $3 \
         LIMIT 1",
        [
            emp.to_owned().into(),
            code.to_owned().into(),
            (year as i32).into(),
        ],
    );
    if let Ok(rows) = db.query_all_raw(st).await {
        if let Some(v) = rows
            .first()
            .and_then(|r| r.try_get::<f64>("", "allocated").ok())
        {
            return v;
        }
    }
    fallback
}

pub fn current_year() -> i32 {
    chrono::Utc::now().date_naive().year()
}

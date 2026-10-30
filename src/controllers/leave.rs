//! Employee leave: list, apply with real quota enforcement, HR approval.
//!
//! Quotas come from the real `leave_quotas` table (allocated per employee and
//! leave type per year); `leave_types` supplies the defaults. Applied days are
//! counted as weekdays (Mon-Fri), excluding holidays stored in `holidays`.
use crate::authz::AuthUser;
use crate::models::_entities::leave_requests::{self, ActiveModel, Entity, Model};
use crate::services::lookup::weekdays;
use chrono::Datelike;
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LeaveApplyParams {
    pub employee_code: Option<String>,
    pub employee_name: Option<String>,
    pub leave_type: String,
    pub from_date: String,
    pub to_date: String,
    pub days: Option<String>,
    pub reason: String,
    pub half: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeaveStatusParams {
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct LeaveBalance {
    pub leave_type: String,
    pub code: String,
    pub allocated: f64,
    pub used: f64,
    pub pending: f64,
    pub balance: f64,
}

use super::leave_calc::{
    allocated_days, booked_days, current_year, leave_days, own_employee, resolve_type,
};

#[debug_handler]
pub async fn list(auth: AuthUser, State(ctx): State<AppContext>) -> Result<Response> {
    let all = auth.require("approvals:decide").is_ok();
    let query = Entity::find().order_by_desc(leave_requests::Column::CreatedAt);
    let list: Vec<Model> = if all {
        query.all(&ctx.db).await?
    } else {
        // A user with no linked employee record simply has no requests yet.
        match own_employee(&ctx.db, &auth.user.email).await {
            Ok((code, _)) => {
                query
                    .filter(leave_requests::Column::EmployeeCode.eq(code))
                    .all(&ctx.db)
                    .await?
            }
            Err(_) => Vec::new(),
        }
    };
    format::json(list)
}

#[debug_handler]
pub async fn balance(auth: AuthUser, State(ctx): State<AppContext>) -> Result<Response> {
    let (code, _) = own_employee(&ctx.db, &auth.user.email).await?;
    let year = current_year();
    let types_st = sea_orm::Statement::from_sql_and_values(
        ctx.db.get_database_backend(),
        "SELECT code, name, days_allowed::float8 AS days_allowed FROM leave_types WHERE is_active ORDER BY id",
        Vec::<sea_orm::Value>::new(),
    );
    let mut out: Vec<LeaveBalance> = Vec::new();
    for r in ctx.db.query_all_raw(types_st).await? {
        let code_t: String = r.try_get("", "code").unwrap_or_default();
        let name: String = r.try_get("", "name").unwrap_or_default();
        let days: f64 = r.try_get("", "days_allowed").unwrap_or(0.0);
        let allocated =
            allocated_days(&ctx.db, &code, &code_t, days, year).await;
        let (used, pending) = booked_days(&ctx.db, &code, &name, year).await;
        out.push(LeaveBalance {
            leave_type: name,
            code: code_t,
            allocated,
            used,
            pending,
            balance: (allocated - used - pending).max(0.0),
        });
    }
    format::json(out)
}

#[debug_handler]
pub async fn apply(
    auth: AuthUser,
    State(ctx): State<AppContext>,
    Json(params): Json<LeaveApplyParams>,
) -> Result<Response> {
    // HR may apply on behalf of an employee; everyone else applies for self.
    let (code, name) = if auth.require("approvals:decide").is_ok() {
        match params.employee_code {
            Some(c) if !c.trim().is_empty() => {
                let n = params.employee_name.unwrap_or_default();
                (c, n)
            }
            _ => own_employee(&ctx.db, &auth.user.email).await?,
        }
    } else {
        own_employee(&ctx.db, &auth.user.email).await?
    };

    let (type_code, type_name, days_allowed) = match resolve_type(&ctx.db, &params.leave_type).await
    {
        Some(t) => t,
        None => {
            return Err(Error::BadRequest(format!(
                "Unknown leave type '{}'.",
                params.leave_type
            )))
        }
    };

    let days = if params.half.is_some() {
        0.5
    } else {
        leave_days(&ctx.db, &params.from_date, &params.to_date).await
    };
    if days <= 0.0 && params.half.is_none() {
        return Err(Error::BadRequest(
            "The selected range has no working days.".to_string(),
        ));
    }

    // Quota gate: "Loss of pay" / unpaid types skip it; the rest must fit in
    // the remaining balance for the year.
    if type_code != "LWP" {
        let year = current_year();
        let allocated =
            allocated_days(&ctx.db, &code, &type_code, days_allowed, year).await;
        let (used, pending) = booked_days(&ctx.db, &code, &type_name, year).await;
        if allocated <= 0.0 && days_allowed <= 0.0 {
            return Err(Error::BadRequest(format!(
                "{type_name} has no annual allocation for you. Contact HR."
            )));
        }
        if allocated > 0.0 && used + pending + days > allocated + 0.001 {
            return Err(Error::BadRequest(format!(
                "Not enough {type_name} balance: allocated {allocated}, used {used}, \
                 pending {pending}, requested {days}."
            )));
        }
    }

    let active = ActiveModel {
        request_id: Set(Uuid::new_v4()),
        employee_code: Set(code),
        employee_name: Set(name),
        leave_type: Set(type_name),
        from_date: Set(params.from_date),
        to_date: Set(params.to_date),
        days: Set(format!("{}", days)),
        reason: Set(params.reason),
        status: Set("Pending HR Approval".to_string()),
        ..Default::default()
    };

    let record = active.insert(&ctx.db).await?;
    // The `half` marker column exists in the table (additive sync) but not in
    // the compiled entity; persist it raw so half-day requests stay explicit.
    if let Some(h) = params.half {
        let st = sea_orm::Statement::from_sql_and_values(
            ctx.db.get_database_backend(),
            "UPDATE leave_requests SET half = $2 WHERE id = $1",
            [record.id.into(), h.into()],
        );
        let _ = ctx.db.execute_raw(st).await;
    }
    format::json(record)
}

#[debug_handler]
pub async fn update_status(
    auth: AuthUser,
    State(ctx): State<AppContext>,
    Path(id): Path<i64>,
    Json(params): Json<LeaveStatusParams>,
) -> Result<Response> {
    auth.require("approvals:decide")?;

    let Some(existing) = Entity::find_by_id(id).one(&ctx.db).await? else {
        return not_found();
    };

    let approving = params.status.starts_with("Approved");
    let days: f64 = existing
        .days
        .replace(',', ".")
        .parse::<f64>()
        .unwrap_or(0.0);

    if approving && existing.leave_type != "Loss of pay" {
        let year = current_year();
        if let Some((code, _, _)) = resolve_type(&ctx.db, &existing.leave_type).await {
            let allocated = allocated_days(
                &ctx.db,
                &existing.employee_code,
                &code,
                0.0,
                year,
            )
            .await;
            let (used, _) = booked_days(
                &ctx.db,
                &existing.employee_code,
                &existing.leave_type,
                year,
            )
            .await;
            if allocated > 0.0 && used + days > allocated + 0.001 {
                return Err(Error::BadRequest(format!(
                    "Cannot approve: {} already used {} of {} allocated {} days.",
                    existing.employee_name, used, allocated, existing.leave_type
                )));
            }
        }
    }

    // Reflect the decision in the quota ledger so HR balances stay real.
    if days > 0.0 {
        let used_delta = if approving { days } else { 0.0 };
        let pend_delta = if approving || params.status.starts_with("Rejected") {
            days
        } else {
            0.0
        };
        let st = sea_orm::Statement::from_sql_and_values(
            ctx.db.get_database_backend(),
            "UPDATE leave_quotas SET used = used + $3, pending = GREATEST(pending - $4, 0), \
             balance = GREATEST(allocated - used - $3 + GREATEST(pending - $4, 0), 0), \
             updated_at = now() \
             WHERE employee_code = $1 AND lower(leave_type_code) = (SELECT lower(code) \
               FROM leave_types WHERE lower(name) = lower($2)) AND year = $5",
            [
                existing.employee_code.to_owned().into(),
                existing.leave_type.to_owned().into(),
                used_delta.into(),
                pend_delta.into(),
                current_year().into(),
            ],
        );
        let _ = ctx.db.execute_raw(st).await;
    }

    let mut active: ActiveModel = existing.into();
    active.status = Set(params.status);
    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/leave")
        .add("", get(list))
        .add("/my", get(list))
        .add("/balance", get(balance))
        .add("/apply", post(apply))
        .add("/{id}/status", patch(update_status))
}

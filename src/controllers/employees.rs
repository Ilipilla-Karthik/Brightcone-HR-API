use crate::models::_entities::employees::{self, ActiveModel, Entity};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct EmployeeFilter {
    pub q: Option<String>,
    pub department: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmployeeParams {
    pub employee_code: String,
    pub full_name: String,
    pub job_title: String,
    pub department: String,
    pub grade: Option<String>,
    pub location: Option<String>,
    pub manager_name: Option<String>,
    pub status: Option<String>,
    pub work_email: Option<String>,
    pub phone: Option<String>,
    pub shift: Option<String>,
    pub joined: Option<String>,
    pub ctc: Option<f64>,
    pub address: Option<String>,
    pub dob: Option<String>,
    pub blood_group: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_relation: Option<String>,
    pub emergency_contact_phone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmployeeUpdateParams {
    pub full_name: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub grade: Option<String>,
    pub location: Option<String>,
    pub manager_name: Option<String>,
    pub status: Option<String>,
    pub work_email: Option<String>,
    pub phone: Option<String>,
    pub shift: Option<String>,
    pub joined: Option<String>,
    pub ctc: Option<f64>,
    pub address: Option<String>,
    pub dob: Option<String>,
    pub blood_group: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_relation: Option<String>,
    pub emergency_contact_phone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImportSummary {
    pub success_count: usize,
    pub error_count: usize,
    pub errors: Vec<String>,
}

#[debug_handler]
pub async fn list(
    State(ctx): State<AppContext>,
    Query(filter): Query<EmployeeFilter>,
) -> Result<Response> {
    let mut query = Entity::find().order_by_asc(employees::Column::EmployeeCode);

    if let Some(status) = filter.status {
        if !status.is_empty() && status != "All" {
            query = query.filter(employees::Column::Status.eq(status));
        }
    }

    if let Some(dept) = filter.department {
        if !dept.is_empty() && dept != "All" {
            query = query.filter(employees::Column::Department.eq(dept));
        }
    }

    let records = query.all(&ctx.db).await?;

    let filtered = if let Some(q) = filter.q {
        let q_lower = q.to_lowercase();
        records
            .into_iter()
            .filter(|r| {
                r.full_name.to_lowercase().contains(&q_lower)
                    || r.employee_code.to_lowercase().contains(&q_lower)
                    || r.job_title.to_lowercase().contains(&q_lower)
                    || r.department.to_lowercase().contains(&q_lower)
            })
            .collect()
    } else {
        records
    };

    format::json(filtered)
}

#[debug_handler]
pub async fn create(
    State(ctx): State<AppContext>,
    Json(params): Json<EmployeeParams>,
) -> Result<Response> {
    let emp_code = if params.employee_code.trim().is_empty() {
        let all = Entity::find().all(&ctx.db).await?;
        let mut max_id = 1000;
        for emp in &all {
            if let Some(num_str) = emp.employee_code.strip_prefix("BC-") {
                if let Ok(num) = num_str.parse::<i32>() {
                    if num < 9000 && num > max_id {
                        max_id = num;
                    }
                }
            }
        }
        format!("BC-{}", max_id + 1)
    } else {
        let exists = Entity::find()
            .filter(employees::Column::EmployeeCode.eq(&params.employee_code))
            .one(&ctx.db)
            .await?;
        if exists.is_some() {
            let all = Entity::find().all(&ctx.db).await?;
            let mut max_id = 1000;
            for emp in &all {
                if let Some(num_str) = emp.employee_code.strip_prefix("BC-") {
                    if let Ok(num) = num_str.parse::<i32>() {
                        if num < 9000 && num > max_id {
                            max_id = num;
                        }
                    }
                }
            }
            format!("BC-{}", max_id + 1)
        } else {
            params.employee_code
        }
    };

    let active_model = ActiveModel {
        emp_id: Set(Uuid::new_v4()),
        employee_code: Set(emp_code),
        full_name: Set(params.full_name),
        job_title: Set(params.job_title),
        department: Set(params.department),
        grade: Set(params.grade.unwrap_or_else(|| "L1".to_string())),
        location: Set(params.location.unwrap_or_else(|| "Hyderabad".to_string())),
        manager_name: Set(params.manager_name),
        status: Set(params.status.unwrap_or_else(|| "Active".to_string())),
        work_email: Set(params.work_email),
        phone: Set(params.phone),
        shift: Set(params.shift),
        joined: Set(params.joined),
        ctc: Set(params.ctc),
        address: Set(params.address),
        dob: Set(params.dob),
        blood_group: Set(params.blood_group),
        emergency_contact_name: Set(params.emergency_contact_name),
        emergency_contact_relation: Set(params.emergency_contact_relation),
        emergency_contact_phone: Set(params.emergency_contact_phone),
        ..Default::default()
    };

    let record = active_model.insert(&ctx.db).await?;
    format::json(record)
}

#[debug_handler]
pub async fn get_one(State(ctx): State<AppContext>, Path(id): Path<String>) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(employees::Column::EmpId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else if let Ok(num_id) = id.parse::<i64>() {
        Entity::find()
            .filter(employees::Column::Id.eq(num_id))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(employees::Column::EmployeeCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    match record {
        Some(e) => format::json(e),
        None => not_found(),
    }
}

#[debug_handler]
pub async fn update(
    State(ctx): State<AppContext>,
    Path(id): Path<String>,
    Json(params): Json<EmployeeUpdateParams>,
) -> Result<Response> {
    let record = if let Ok(uuid) = Uuid::parse_str(&id) {
        Entity::find()
            .filter(employees::Column::EmpId.eq(uuid))
            .one(&ctx.db)
            .await?
    } else if let Ok(num_id) = id.parse::<i64>() {
        Entity::find()
            .filter(employees::Column::Id.eq(num_id))
            .one(&ctx.db)
            .await?
    } else {
        Entity::find()
            .filter(employees::Column::EmployeeCode.eq(&id))
            .one(&ctx.db)
            .await?
    };

    let Some(existing) = record else {
        return not_found();
    };

    let mut active: ActiveModel = existing.into();

    if let Some(name) = params.full_name {
        active.full_name = Set(name);
    }
    if let Some(title) = params.job_title {
        active.job_title = Set(title);
    }
    if let Some(dept) = params.department {
        active.department = Set(dept);
    }
    if let Some(grade) = params.grade {
        active.grade = Set(grade);
    }
    if let Some(loc) = params.location {
        active.location = Set(loc);
    }
    if let Some(mgr) = params.manager_name {
        active.manager_name = Set(Some(mgr));
    }
    if let Some(st) = params.status {
        active.status = Set(st);
    }
    if let Some(email) = params.work_email {
        active.work_email = Set(Some(email));
    }
    if let Some(ph) = params.phone {
        active.phone = Set(Some(ph));
    }
    if let Some(sh) = params.shift {
        active.shift = Set(Some(sh));
    }
    if let Some(j) = params.joined {
        active.joined = Set(Some(j));
    }
    if let Some(c) = params.ctc {
        active.ctc = Set(Some(c));
    }
    if let Some(addr) = params.address {
        active.address = Set(Some(addr));
    }
    if let Some(dob) = params.dob {
        active.dob = Set(Some(dob));
    }
    if let Some(bg) = params.blood_group {
        active.blood_group = Set(Some(bg));
    }
    if let Some(ecn) = params.emergency_contact_name {
        active.emergency_contact_name = Set(Some(ecn));
    }
    if let Some(ecr) = params.emergency_contact_relation {
        active.emergency_contact_relation = Set(Some(ecr));
    }
    if let Some(ecp) = params.emergency_contact_phone {
        active.emergency_contact_phone = Set(Some(ecp));
    }

    let updated = active.update(&ctx.db).await?;
    format::json(updated)
}

#[debug_handler]
pub async fn me_profile(
    auth: Option<auth::JWT>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let email = if let Some(a) = auth {
        if let Ok(u) = crate::models::_entities::users::Entity::find()
            .filter(crate::models::_entities::users::Column::Pid.eq(a.claims.pid))
            .one(&ctx.db)
            .await
        {
            u.map(|user| user.email)
                .unwrap_or_else(|| "rahul.varma@brightcone.ai".to_string())
        } else {
            "rahul.varma@brightcone.ai".to_string()
        }
    } else {
        "rahul.varma@brightcone.ai".to_string()
    };

    let record = Entity::find()
        .filter(employees::Column::WorkEmail.eq(&email))
        .one(&ctx.db)
        .await?;

    match record {
        Some(e) => format::json(e),
        None => {
            let fallback = Entity::find()
                .filter(employees::Column::EmployeeCode.eq("BC-1038"))
                .one(&ctx.db)
                .await?;
            match fallback {
                Some(e) => format::json(e),
                None => not_found(),
            }
        }
    }
}

pub use super::employees_csv::{export_csv, import_csv, ImportRequest};

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api/v1/employees")
        .add("/me/profile", get(me_profile))
        .add("/me", get(me_profile))
        .add("", get(list))
        .add("", post(create))
        .add("/import", post(import_csv))
        .add("/export", get(export_csv))
        .add("/{id}", get(get_one))
        .add("/{id}", patch(update))
        .add("/{id}", put(update))
}

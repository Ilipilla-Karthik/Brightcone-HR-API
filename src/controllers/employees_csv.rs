use crate::models::_entities::employees::{self, ActiveModel, Entity, Model};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::Deserialize;
use uuid::Uuid;

use super::employees::{EmployeeFilter, ImportSummary};

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub csv_content: String,
}

#[debug_handler]
pub async fn import_csv(
    State(ctx): State<AppContext>,
    Json(req): Json<ImportRequest>,
) -> Result<Response> {
    let mut success_count = 0;
    let mut errors = Vec::new();

    let lines: Vec<&str> = req.csv_content.lines().collect();
    if lines.is_empty() {
        return bad_request("CSV content is empty");
    }

    let header = lines[0];
    let headers: Vec<&str> = header.split(',').map(|s| s.trim()).collect();
    if headers.len() < 4 {
        return bad_request(
            "Invalid CSV format: Missing required columns (Employee ID, Name, Job Title, Department)",
        );
    }

    for (idx, line) in lines.iter().skip(1).enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 4 {
            errors.push(format!("Row {}: Insufficient columns", idx + 2));
            continue;
        }

        let code = parts[0];
        let name = parts[1];
        let title = parts[2];
        let dept = parts[3];
        let grade = parts.get(4).unwrap_or(&"L4");
        let loc = parts.get(5).unwrap_or(&"Hyderabad");
        let mgr = parts.get(6).cloned().unwrap_or("");
        let status = parts.get(7).unwrap_or(&"Active");

        if code.is_empty() || name.is_empty() {
            errors.push(format!("Row {}: Employee code or Name missing", idx + 2));
            continue;
        }

        let exists = Entity::find()
            .filter(employees::Column::EmployeeCode.eq(code))
            .one(&ctx.db)
            .await?;

        if exists.is_some() {
            errors.push(format!("Row {}: Duplicate employee ID {}", idx + 2, code));
            continue;
        }

        let active = ActiveModel {
            emp_id: Set(Uuid::new_v4()),
            employee_code: Set(code.to_string()),
            full_name: Set(name.to_string()),
            job_title: Set(title.to_string()),
            department: Set(dept.to_string()),
            grade: Set(grade.to_string()),
            location: Set(loc.to_string()),
            manager_name: Set(if mgr.is_empty() {
                None
            } else {
                Some(mgr.to_string())
            }),
            status: Set(status.to_string()),
            ..Default::default()
        };

        if active.insert(&ctx.db).await.is_ok() {
            success_count += 1;
        } else {
            errors.push(format!("Row {}: Failed to insert into database", idx + 2));
        }
    }

    format::json(ImportSummary {
        success_count,
        error_count: errors.len(),
        errors,
    })
}

#[debug_handler]
pub async fn export_csv(
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

    let records: Vec<Model> = query.all(&ctx.db).await?;

    let mut csv = String::from(
        "Employee ID,Name,Job Title,Department,Grade,Location,Manager,Status,Work Email,Phone\n",
    );
    for r in records {
        let mgr = r.manager_name.unwrap_or_default();
        let email = r.work_email.unwrap_or_default();
        let phone = r.phone.unwrap_or_default();
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            r.employee_code.replace('"', "\"\""),
            r.full_name.replace('"', "\"\""),
            r.job_title.replace('"', "\"\""),
            r.department.replace('"', "\"\""),
            r.grade.replace('"', "\"\""),
            r.location.replace('"', "\"\""),
            mgr.replace('"', "\"\""),
            r.status.replace('"', "\"\""),
            email.replace('"', "\"\""),
            phone.replace('"', "\"\"")
        ));
    }

    let response = axum::response::Response::builder()
        .header("Content-Type", "text/csv; charset=utf-8")
        .header(
            "Content-Disposition",
            "attachment; filename=\"employees_export.csv\"",
        )
        .body(axum::body::Body::from(csv))
        .map_err(|e| Error::string(&e.to_string()))?;

    Ok(response)
}

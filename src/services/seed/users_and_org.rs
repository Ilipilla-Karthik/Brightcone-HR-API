use crate::models::_entities::{
    departments::{self, ActiveModel as DeptActiveModel},
    employees::{self, ActiveModel as EmployeeActiveModel},
    users::{self, ActiveModel as UserActiveModel},
};
use chrono::Local;
use loco_rs::hash;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn seed_users_and_org(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Seed standard test users for automated test suite idempotently
    let test_users = vec![
        (
            "user1@example.com",
            "11111111-1111-1111-1111-111111111111",
            "lo-95ec80d7-cb60-4b70-9b4b-9ef74cb88758",
            "user1",
            "12341234",
            false,
        ),
        (
            "user2@example.com",
            "22222222-2222-2222-2222-222222222222",
            "lo-d4d42044-ee73-4ea0-98cc-9cf0f6974720",
            "user2",
            "12341234",
            true,
        ),
    ];

    for (email, pid_str, api_key, name, pwd, verified) in test_users {
        let exists = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await?;

        if exists.is_none() {
            let password_hash = hash::hash_password(pwd)?;
            let pid = Uuid::parse_str(pid_str)?;
            let verified_at = if verified {
                Some(Local::now().into())
            } else {
                None
            };
            let user = UserActiveModel {
                pid: Set(pid),
                email: Set(email.to_string()),
                password: Set(password_hash),
                api_key: Set(api_key.to_string()),
                name: Set(name.to_string()),
                role: Set("EMPLOYEE".to_string()),
                email_verified_at: Set(verified_at),
                ..Default::default()
            };
            user.insert(db).await?;
            tracing::info!("Seeded test user: {}", email);
        }
    }

    // 1. Seed Demo Users (2 Roles: HR Admin & Employee)
    let demo_users = vec![
        ("ananya.rao@brightcone.ai", "Ananya Rao", "HR_ADMIN"),
        ("rahul.varma@brightcone.ai", "Rahul Varma", "EMPLOYEE"),
    ];

    for (email, name, role) in demo_users {
        let exists = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await?;

        if exists.is_none() {
            let password_hash = hash::hash_password("Password123!")?;
            let user = UserActiveModel {
                pid: Set(Uuid::new_v4()),
                email: Set(email.to_string()),
                password: Set(password_hash),
                api_key: Set(format!("lo-{}", Uuid::new_v4())),
                name: Set(name.to_string()),
                role: Set(role.to_string()),
                email_verified_at: Set(Some(Local::now().into())),
                ..Default::default()
            };
            user.insert(db).await?;
            tracing::info!("Seeded user: {}", email);
        }
    }

    // 2. Seed Departments
    let depts = vec![
        "Leadership",
        "People Ops",
        "AI Research",
        "Engineering",
        "Sales",
        "Design",
        "Finance",
        "Customer Success",
    ];

    for dept_name in depts {
        let exists = departments::Entity::find()
            .filter(departments::Column::Name.eq(dept_name))
            .one(db)
            .await?;

        if exists.is_none() {
            let dept = DeptActiveModel {
                dept_id: Set(Uuid::new_v4()),
                name: Set(dept_name.to_string()),
                code: Set(Some(
                    dept_name.chars().take(3).collect::<String>().to_uppercase(),
                )),
                description: Set(Some(format!("{} department at Brightcone", dept_name))),
                ..Default::default()
            };
            dept.insert(db).await?;
        }
    }

    // 3. Seed Exactly the 8 Employees Mandated by Section 8 and 18
    let demo_employees = vec![
        (
            "BC-1001",
            "Naresh Vemparala",
            "Founder & CEO",
            "Leadership",
            "L9",
            "Hyderabad",
            None,
            "Active",
            Some(9600000.0),
        ),
        (
            "BC-1004",
            "Ananya Rao",
            "People Ops Lead",
            "People Ops",
            "L6",
            "Hyderabad",
            Some("Naresh Vemparala"),
            "Active",
            Some(3200000.0),
        ),
        (
            "BC-1012",
            "Karthik Reddy",
            "Principal AI Engineer",
            "AI Research",
            "L8",
            "Hyderabad",
            Some("Naresh Vemparala"),
            "Active",
            Some(6400000.0),
        ),
        (
            "BC-1019",
            "Priya Menon",
            "Engineering Manager",
            "Engineering",
            "L7",
            "Bengaluru",
            Some("Karthik Reddy"),
            "Active",
            Some(5200000.0),
        ),
        (
            "BC-1024",
            "Arjun Nair",
            "Senior Frontend Engineer",
            "Engineering",
            "L5",
            "Bengaluru",
            Some("Priya Menon"),
            "Active",
            Some(3800000.0),
        ),
        (
            "BC-1031",
            "Sneha Iyer",
            "Product Designer",
            "Design",
            "L5",
            "Hyderabad",
            Some("Ananya Rao"),
            "Active",
            Some(3400000.0),
        ),
        (
            "BC-1038",
            "Rahul Varma",
            "Senior Backend Engineer",
            "Engineering",
            "L5",
            "Hyderabad",
            Some("Priya Menon"),
            "Active",
            Some(4200000.0),
        ),
        (
            "BC-1045",
            "Vikram Malhotra",
            "DevOps Architect",
            "Engineering",
            "L6",
            "Pune",
            Some("Priya Menon"),
            "Active",
            Some(4600000.0),
        ),
    ];

    for (code, name, title, dept, grade, loc, mgr, status, ctc) in demo_employees {
        let exists = employees::Entity::find()
            .filter(employees::Column::EmployeeCode.eq(code))
            .one(db)
            .await?;

        if exists.is_none() {
            let email = format!("{}.{}@brightcone.ai", name.split(' ').next().unwrap().to_lowercase(), name.split(' ').last().unwrap().to_lowercase());
            let emp = EmployeeActiveModel {
                emp_id: Set(Uuid::new_v4()),
                employee_code: Set(code.to_string()),
                full_name: Set(name.to_string()),
                job_title: Set(title.to_string()),
                department: Set(dept.to_string()),
                grade: Set(grade.to_string()),
                location: Set(loc.to_string()),
                manager_name: Set(mgr.map(|s| s.to_string())),
                status: Set(status.to_string()),
                work_email: Set(Some(email)),
                phone: Set(Some("+91 98765 43210".to_string())),
                shift: Set(Some("General".to_string())),
                joined: Set(Some("2022-01-15".to_string())),
                ctc: Set(ctc),
                ..Default::default()
            };
            emp.insert(db).await?;
            tracing::info!("Seeded employee: {} ({})", name, code);
        }
    }

    Ok(())
}

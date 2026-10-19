use crate::models::_entities::{
    candidates::{self, ActiveModel as CandidateActiveModel},
    departments::{self, ActiveModel as DeptActiveModel},
    employees::{self, ActiveModel as EmployeeActiveModel},
    interviews::{self, ActiveModel as InterviewActiveModel},
    jobs::{self, ActiveModel as JobActiveModel},
    performance_goals::{self, ActiveModel as GoalActiveModel},
    performance_reviews::{self, ActiveModel as ReviewActiveModel},
    referrals::{self, ActiveModel as ReferralActiveModel},
    users::{self, ActiveModel as UserActiveModel},
};
use chrono::Local;
use loco_rs::hash;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn run_seed(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Running schema synchronization before seed...");
    crate::services::schema_sync::run_schema_synchronization(db).await?;

    tracing::info!("Checking and running idempotent database seed...");

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

    // 1. Seed Users (2 Roles: HR Admin & Employee)
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
            "BC-1023",
            "Daniel Brooks",
            "Account Executive",
            "Sales",
            "L6",
            "San Francisco",
            Some("Naresh Vemparala"),
            "Active",
            Some(11500000.0),
        ),
        (
            "BC-1031",
            "Sneha Iyer",
            "Product Designer",
            "Design",
            "L5",
            "Hyderabad",
            Some("Priya Menon"),
            "On leave",
            Some(2400000.0),
        ),
        (
            "BC-1038",
            "Rahul Varma",
            "Backend Engineer",
            "Engineering",
            "L4",
            "Hyderabad",
            Some("Priya Menon"),
            "Active",
            Some(2200000.0),
        ),
        (
            "BC-1042",
            "Meera Joshi",
            "Finance Manager",
            "Finance",
            "L6",
            "Hyderabad",
            Some("Naresh Vemparala"),
            "Active",
            Some(3000000.0),
        ),
    ];

    for (code, name, title, dept, grade, loc, mgr, status, ctc) in demo_employees {
        let exists = employees::Entity::find()
            .filter(employees::Column::EmployeeCode.eq(code))
            .one(db)
            .await?;

        if exists.is_none() {
            let emp = EmployeeActiveModel {
                emp_id: Set(Uuid::new_v4()),
                employee_code: Set(code.to_string()),
                full_name: Set(name.to_string()),
                job_title: Set(title.to_string()),
                department: Set(dept.to_string()),
                grade: Set(grade.to_string()),
                location: Set(loc.to_string()),
                manager_name: Set(mgr.map(|m| m.to_string())),
                status: Set(status.to_string()),
                work_email: Set(Some(format!(
                    "{}.{}@brightcone.ai",
                    name.split_whitespace().next().unwrap_or("").to_lowercase(),
                    name.split_whitespace().last().unwrap_or("").to_lowercase()
                ))),
                phone: Set(Some("+91 98480 11000".to_string())),
                shift: Set(Some("General".to_string())),
                joined: Set(Some("2022-01-15".to_string())),
                ctc: Set(ctc),
                ..Default::default()
            };
            emp.insert(db).await?;
            tracing::info!("Seeded employee: {} ({})", name, code);
        }
    }

    // 4. Seed Jobs
    let demo_jobs = vec![
        (
            "J1",
            "Senior ML Engineer",
            "AI Research",
            "Hyderabad",
            "Full-time",
            "Open",
            50000,
        ),
        (
            "J2",
            "Solutions Architect",
            "Sales",
            "San Francisco",
            "Full-time",
            "Open",
            75000,
        ),
        (
            "J3",
            "Product Designer",
            "Design",
            "Hyderabad",
            "Full-time",
            "Open",
            40000,
        ),
        (
            "J4",
            "QA Automation Engineer",
            "Engineering",
            "Pune",
            "Full-time",
            "Open",
            30000,
        ),
    ];

    for (code, title, dept, loc, emp_type, status, bonus) in demo_jobs {
        let exists = jobs::Entity::find()
            .filter(jobs::Column::JobCode.eq(code))
            .one(db)
            .await?;

        if exists.is_none() {
            let job = JobActiveModel {
                job_id: Set(Uuid::new_v4()),
                job_code: Set(code.to_string()),
                title: Set(title.to_string()),
                department: Set(dept.to_string()),
                location: Set(loc.to_string()),
                employment_type: Set(emp_type.to_string()),
                status: Set(status.to_string()),
                referral_bonus: Set(bonus),
                description: Set(Some(format!(
                    "Exciting opportunity for {} at Brightcone.",
                    title
                ))),
                ..Default::default()
            };
            job.insert(db).await?;
        }
    }

    // 5. Seed Candidates & Pipeline Stages
    let demo_candidates = vec![
        (
            "C1",
            "Aditi Sharma",
            "Senior ML Engineer",
            "LinkedIn",
            82,
            "Applied",
        ),
        (
            "C2",
            "Rohan Gupta",
            "Senior ML Engineer",
            "Careers page",
            74,
            "Applied",
        ),
        (
            "C3",
            "Kavya Pillai",
            "Product Designer",
            "Referral",
            88,
            "Screening",
        ),
        (
            "C4",
            "Marcus Lee",
            "Solutions Architect",
            "Indeed",
            79,
            "Screening",
        ),
        (
            "C5",
            "Neha Bansal",
            "Senior ML Engineer",
            "LinkedIn",
            91,
            "Interview",
        ),
        (
            "C6",
            "Siddharth Rao",
            "QA Automation Engineer",
            "Referral",
            84,
            "Interview",
        ),
        (
            "C7",
            "Olivia Martin",
            "Solutions Architect",
            "LinkedIn",
            93,
            "Offer",
        ),
        (
            "C8",
            "Tejas Patil",
            "QA Automation Engineer",
            "Careers page",
            87,
            "Hired",
        ),
    ];

    for (code, name, job_t, src, score, stage) in demo_candidates {
        let exists = candidates::Entity::find()
            .filter(candidates::Column::CandidateCode.eq(code))
            .one(db)
            .await?;

        if exists.is_none() {
            let cand = CandidateActiveModel {
                cand_id: Set(Uuid::new_v4()),
                candidate_code: Set(code.to_string()),
                full_name: Set(name.to_string()),
                email: Set(Some(format!(
                    "{}.cand@example.com",
                    name.replace(' ', ".").to_lowercase()
                ))),
                phone: Set(Some("+91 98765 43210".to_string())),
                source: Set(src.to_string()),
                job_title: Set(Some(job_t.to_string())),
                match_score: Set(Some(score)),
                stage: Set(stage.to_string()),
                ..Default::default()
            };
            cand.insert(db).await?;
        }
    }

    // 6. Seed Interviews
    let demo_interviews = vec![
        (
            "Neha Bansal",
            "Senior ML Engineer",
            "Technical, round 2",
            "2026-10-20T11:00",
            "Karthik Reddy, Arjun Nair",
        ),
        (
            "Siddharth Rao",
            "QA Automation Engineer",
            "Hiring manager",
            "2026-10-22T15:30",
            "Priya Menon",
        ),
    ];

    for (cand_name, job_t, round, when, panel) in demo_interviews {
        let exists = interviews::Entity::find()
            .filter(interviews::Column::CandidateName.eq(cand_name))
            .one(db)
            .await?;

        if exists.is_none() {
            let interview = InterviewActiveModel {
                interview_id: Set(Uuid::new_v4()),
                candidate_name: Set(cand_name.to_string()),
                job_title: Set(job_t.to_string()),
                round_name: Set(round.to_string()),
                scheduled_at: Set(when.to_string()),
                panel: Set(panel.to_string()),
                status: Set("Scheduled".to_string()),
                ..Default::default()
            };
            interview.insert(db).await?;
        }
    }

    // 7. Seed Referrals
    let demo_referrals = vec![(
        "Rahul Varma",
        "Siddharth Rao",
        "QA Automation Engineer",
        "Interview",
        30000,
    )];

    for (referred_by, cand_name, job_t, status, bonus) in demo_referrals {
        let exists = referrals::Entity::find()
            .filter(referrals::Column::CandidateName.eq(cand_name))
            .one(db)
            .await?;

        if exists.is_none() {
            let r = ReferralActiveModel {
                referral_id: Set(Uuid::new_v4()),
                referred_by: Set(referred_by.to_string()),
                candidate_name: Set(cand_name.to_string()),
                job_title: Set(job_t.to_string()),
                status: Set(status.to_string()),
                bonus_amount: Set(bonus),
                ..Default::default()
            };
            r.insert(db).await?;
        }
    }

    // 8. Seed Performance Goals
    let demo_goals = vec![
        (
            "Rahul Varma",
            "Ship leave & payslip APIs for self-service",
            "2026-12-15",
            72,
            "In Progress",
        ),
        (
            "Rahul Varma",
            "Cut p95 latency of the agent gateway to 300 ms",
            "2026-11-30",
            45,
            "In Progress",
        ),
        (
            "Rahul Varma",
            "Mentor one new backend engineer",
            "2026-12-31",
            30,
            "In Progress",
        ),
        (
            "Ananya Rao",
            "Reduce time-to-hire to 30 days",
            "2026-12-31",
            55,
            "In Progress",
        ),
    ];

    for (emp, title, due, p, status) in demo_goals {
        let exists = performance_goals::Entity::find()
            .filter(performance_goals::Column::Title.eq(title))
            .one(db)
            .await?;

        if exists.is_none() {
            let g = GoalActiveModel {
                goal_id: Set(Uuid::new_v4()),
                employee_name: Set(emp.to_string()),
                title: Set(title.to_string()),
                target_date: Set(due.to_string()),
                progress_percentage: Set(p),
                status: Set(status.to_string()),
                ..Default::default()
            };
            g.insert(db).await?;
        }
    }

    // 9. Seed Performance Reviews
    let demo_reviews = vec![
        (
            "Rahul Varma",
            "Priya Menon",
            "H2 2026",
            4,
            "Strong backend quality and rate-limiter design",
            "Document internal architecture earlier",
            "Exceeds expectations",
            "Submitted",
        ),
        (
            "Sneha Iyer",
            "Priya Menon",
            "H2 2026",
            5,
            "Remarkable UX polish and design system consistency",
            "Keep mentoring junior designers",
            "Outstanding work",
            "Approved",
        ),
    ];

    for (emp, rev, period, rating, strn, imp, comm, status) in demo_reviews {
        let exists = performance_reviews::Entity::find()
            .filter(performance_reviews::Column::EmployeeName.eq(emp))
            .one(db)
            .await?;

        if exists.is_none() {
            let r = ReviewActiveModel {
                review_id: Set(Uuid::new_v4()),
                employee_name: Set(emp.to_string()),
                reviewer_name: Set(rev.to_string()),
                review_period: Set(period.to_string()),
                rating: Set(rating),
                strengths: Set(Some(strn.to_string())),
                improvement_areas: Set(Some(imp.to_string())),
                comments: Set(Some(comm.to_string())),
                status: Set(status.to_string()),
                ..Default::default()
            };
            r.insert(db).await?;
        }
    }

    tracing::info!("Database seed completed successfully.");
    Ok(())
}

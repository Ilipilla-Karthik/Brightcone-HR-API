use crate::models::_entities::{
    candidates::{self, ActiveModel as CandidateActiveModel},
    interviews::{self, ActiveModel as InterviewActiveModel},
    jobs::{self, ActiveModel as JobActiveModel},
    performance_goals::{self, ActiveModel as GoalActiveModel},
    performance_reviews::{self, ActiveModel as ReviewActiveModel},
    referrals::{self, ActiveModel as ReferralActiveModel},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn seed_recruitment_and_perf(
    db: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 4. Seed Jobs
    let demo_jobs = vec![
        ("J1", "Senior ML Engineer", "AI Research", "Hyderabad", "Full-time", "Open", 50000),
        ("J2", "Solutions Architect", "Sales", "San Francisco", "Full-time", "Open", 75000),
        ("J3", "Product Designer", "Design", "Hyderabad", "Full-time", "Open", 40000),
        ("J4", "QA Automation Engineer", "Engineering", "Pune", "Full-time", "Open", 30000),
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
                description: Set(Some(format!("Exciting opportunity for {} at Brightcone.", title))),
                ..Default::default()
            };
            job.insert(db).await?;
        }
    }

    // 5. Seed Candidates & Pipeline Stages
    let demo_candidates = vec![
        ("C1", "Aditi Sharma", "Senior ML Engineer", "LinkedIn", 82, "Applied"),
        ("C2", "Rohan Gupta", "Senior ML Engineer", "Careers page", 74, "Applied"),
        ("C3", "Kavya Pillai", "Product Designer", "Referral", 88, "Screening"),
        ("C4", "Marcus Lee", "Solutions Architect", "Indeed", 79, "Screening"),
        ("C5", "Neha Bansal", "Senior ML Engineer", "LinkedIn", 91, "Interview"),
        ("C6", "Siddharth Rao", "QA Automation Engineer", "Referral", 84, "Interview"),
        ("C7", "Olivia Martin", "Solutions Architect", "LinkedIn", 93, "Offer"),
        ("C8", "Tejas Patil", "QA Automation Engineer", "Careers page", 87, "Hired"),
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
                email: Set(Some(format!("{}.cand@example.com", name.replace(' ', ".").to_lowercase()))),
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
        ("Neha Bansal", "Senior ML Engineer", "Technical, round 2", "2026-10-20T11:00", "Karthik Reddy, Arjun Nair"),
        ("Siddharth Rao", "QA Automation Engineer", "Hiring manager", "2026-10-22T15:30", "Priya Menon"),
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
        ("Rahul Varma", "Ship leave & payslip APIs for self-service", "2026-12-15", 72, "In Progress"),
        ("Rahul Varma", "Cut p95 latency of the agent gateway to 300 ms", "2026-11-30", 45, "In Progress"),
        ("Rahul Varma", "Mentor one new backend engineer", "2026-12-31", 30, "In Progress"),
        ("Ananya Rao", "Reduce time-to-hire to 30 days", "2026-12-31", 55, "In Progress"),
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
        ("Rahul Varma", "Priya Menon", "H2 2026", 4, "Strong backend quality and rate-limiter design", "Document internal architecture earlier", "Exceeds expectations", "Submitted"),
        ("Sneha Iyer", "Priya Menon", "H2 2026", 5, "Remarkable UX polish and design system consistency", "Keep mentoring junior designers", "Outstanding work", "Approved"),
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

    Ok(())
}

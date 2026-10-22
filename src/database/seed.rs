use sqlx::{PgPool, Row};
use tracing::info;

/// Salted password hashing using SHA-256
pub fn hash_password(password: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(b"brightcone_salt_v1:");
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    format!("$sha256${}", hex::encode(result))
}

/// Verify password against stored hash or fallback to legacy plaintext
pub fn verify_password(password: &str, stored: &str) -> bool {
    if stored.starts_with("$sha256$") {
        hash_password(password) == stored
    } else {
        // Legacy plaintext match
        password == stored
    }
}

/// Seed initial records idempotently without overwriting existing data
pub async fn seed_missing_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("[Database Seed] Verifying initial seed records...");

    // 1. Initial Users - safely check email before insert to prevent ID collisions
    let admin_hash = hash_password("admin123");
    let ananya_hash = hash_password("Admin@123");
    let rahul_hash = hash_password("Welcome@123");

    // Upgrade unhashed legacy passwords
    let _ = sqlx::query("UPDATE users SET password = $1 WHERE email = 'admin@brightcone.ai' AND password NOT LIKE '$sha256$%';")
        .bind(&admin_hash)
        .execute(pool)
        .await;

    // Ensure admin user exists if table is empty
    let admin_exists =
        sqlx::query("SELECT id FROM users WHERE lower(email) = 'admin@brightcone.ai';")
            .fetch_optional(pool)
            .await?;
    if admin_exists.is_none() {
        let max_id: i32 = sqlx::query("SELECT coalesce(max(id), 0) + 1 as m FROM users;")
            .fetch_one(pool)
            .await?
            .get("m");
        let _ = sqlx::query("INSERT INTO users (id, name, email, password, role, created_at) VALUES ($1, 'Recruitment Admin', 'admin@brightcone.ai', $2, 'Senior Talent Lead', 'Oct 01, 2026');")
            .bind(max_id)
            .bind(&admin_hash)
            .execute(pool)
            .await;
    }

    // Ensure Ananya Rao exists
    let ananya_exists =
        sqlx::query("SELECT id FROM users WHERE lower(email) = 'ananya.rao@brightcone.ai';")
            .fetch_optional(pool)
            .await?;
    if ananya_exists.is_none() {
        let max_id: i32 = sqlx::query("SELECT coalesce(max(id), 0) + 1 as m FROM users;")
            .fetch_one(pool)
            .await?
            .get("m");
        let _ = sqlx::query("INSERT INTO users (id, name, email, password, role, created_at) VALUES ($1, 'Ananya Rao', 'ananya.rao@brightcone.ai', $2, 'HR Admin', 'Oct 01, 2026');")
            .bind(max_id)
            .bind(&ananya_hash)
            .execute(pool)
            .await;
    }

    // Ensure Rahul Varma exists
    let rahul_exists =
        sqlx::query("SELECT id FROM users WHERE lower(email) = 'rahul.varma@brightcone.ai';")
            .fetch_optional(pool)
            .await?;
    if rahul_exists.is_none() {
        let max_id: i32 = sqlx::query("SELECT coalesce(max(id), 0) + 1 as m FROM users;")
            .fetch_one(pool)
            .await?
            .get("m");
        let _ = sqlx::query("INSERT INTO users (id, name, email, password, role, created_at) VALUES ($1, 'Rahul Varma', 'rahul.varma@brightcone.ai', $2, 'Employee', 'Oct 01, 2026');")
            .bind(max_id)
            .bind(&rahul_hash)
            .execute(pool)
            .await;
    }

    // 2. Initial Jobs (if empty)
    sqlx::query(
        "INSERT INTO jobs (id, title, department, location, skills, status, posted_date) VALUES 
         (1, 'Senior Rust Backend Engineer', 'Engineering', 'San Francisco, CA (Hybrid)', 'Rust, Axum, Tokio, Distributed Systems', 'Active', 'Oct 01, 2026'),
         (2, 'Full-Stack Product Developer', 'Product', 'New York, NY (Remote)', 'TypeScript, React, Rust, UI/UX', 'Active', 'Oct 04, 2026'),
         (3, 'Technical Talent Acquisition Lead', 'Human Resources', 'Bangalore, India (On-site)', 'Tech Recruiting, ATS, Sourcing, Hiring Strategy', 'Active', 'Oct 08, 2026'),
         (4, 'Cloud Infrastructure & DevOps Lead', 'Infrastructure', 'Remote', 'Kubernetes, CI/CD, AWS, Terraform', 'Active', 'Oct 11, 2026')
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 3. Initial Candidates (if empty)
    sqlx::query(
        "INSERT INTO candidates (id, name, email, job_id, job_title, skills, status, applied_date) VALUES 
         (1, 'Aarav Sharma', 'aarav.sharma@example.com', 1, 'Senior Rust Backend Engineer', 'Rust, Tokio, Axum, Postgres, Docker', 'Interview', 'Oct 12, 2026'),
         (2, 'Elena Rostova', 'elena.rostova@example.com', 2, 'Full-Stack Product Developer', 'React, TypeScript, Node.js, GraphQL', 'Interview', 'Oct 13, 2026'),
         (3, 'Marcus Vance', 'marcus.vance@workmail.net', 1, 'Senior Rust Backend Engineer', 'C++, Rust, Linux Kernel, eBPF', 'Shortlisted', 'Oct 14, 2026')
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 4. Initial Interviews (if empty)
    sqlx::query(
        "INSERT INTO interviews (id, candidate_id, candidate_name, candidate_email, job_title, interview_date, formatted_date, interviewer, interview_type, notes, status, round, mode, time, rating, result, feedback, archived) VALUES 
         (1, 2, 'Elena Rostova', 'elena.rostova@example.com', 'Full-Stack Product Developer', '2026-10-16T14:00', 'Oct 16, 2026 at 02:00 PM', 'Sarah Jenkins (Engineering Manager)', 'System Design', 'Discuss component modularity and client-server sync patterns.', 'Scheduled', 'Technical 1', 'Video call', '02:00 PM', 4, 'Pass', 'Good system architecture concepts', FALSE),
         (2, 1, 'Aarav Sharma', 'aarav.sharma@example.com', 'Senior Rust Backend Engineer', '2026-10-17T11:30', 'Oct 17, 2026 at 11:30 AM', 'Vikram Malhotra (Staff Architect)', 'Technical Screening', 'Focus on memory management, async runtime, and error handling.', 'Scheduled', 'Technical 2', 'Video call', '11:30 AM', 5, 'Pass', 'Strong proficiency in Rust and concurrency primitives.', FALSE)
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 5. Initial Employees (if empty)
    sqlx::query(
        "INSERT INTO employees (id, name, role, department, grade, location, manager, status, email, joined_date) VALUES 
         ('BC-1001', 'Naresh Vemparala', 'Founder & CEO', 'Leadership', 'L9', 'Hyderabad', 'no manager', 'Active', 'naresh@brightcone.ai', 'Jan 10, 2023'),
         ('BC-1004', 'Ananya Rao', 'People Ops Lead', 'People Ops', 'L6', 'Hyderabad', 'Naresh Vemparala', 'Active', 'ananya.rao@brightcone.ai', 'Jan 10, 2023'),
         ('BC-1012', 'Karthik Reddy', 'Principal AI Engineer', 'AI Research', 'L8', 'Hyderabad', 'Naresh Vemparala', 'Active', 'karthik.reddy@brightcone.ai', 'Feb 01, 2023'),
         ('BC-1019', 'Priya Menon', 'Engineering Manager', 'Engineering', 'L7', 'Bengaluru', 'Karthik Reddy', 'Active', 'priya.menon@brightcone.ai', 'Mar 15, 2023'),
         ('BC-1038', 'Rahul Varma', 'Backend Engineer', 'Engineering', 'L4', 'Hyderabad', 'Priya Menon', 'Active', 'rahul.varma@brightcone.ai', 'Feb 15, 2024'),
         ('BC-1042', 'Meera Joshi', 'Finance Manager', 'Finance', 'L6', 'Hyderabad', 'Naresh Vemparala', 'Active', 'meera.joshi@brightcone.ai', 'Apr 01, 2024')
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 6. Initial Employee Profiles
    sqlx::query(
        "INSERT INTO employee_profiles (employee_id, full_name, work_email, role, department, status, phone, dob, blood_group, address, emergency_contact, manager, grade, location, doj, bank_name, bank_account, ifsc, pan, uan, nominee_name, nominee_relation) VALUES 
         ('BC-1038', 'Rahul Varma', 'rahul.varma@brightcone.ai', 'Backend Engineer', 'Engineering', 'Active', '+91 98480 11038', 'Oct 7, 1996', 'B+', 'Flat 402, Aparna Towers, Kondapur, Hyderabad 500084', 'Lakshmi Varma (Mother), +91 98490 22110', 'Priya Menon', 'L4', 'Hyderabad', 'Feb 15, 2024', 'HDFC Bank', '50100492817291', 'HDFC0001234', 'ABCDE1234F', '100984920194', 'Lakshmi Varma', 'Mother')
         ON CONFLICT (employee_id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 7. Initial Manpower Requirements
    sqlx::query(
        "INSERT INTO manpower_requirements (id, designation, department, positions, employment_type, location, experience, budget_ctc, reason, replacing, hiring_manager, required_by, status, job_description, created_at, updated_at, archived) VALUES 
         ('MR-0001', 'asa', 'ss', 2, 'Full-time', 'sed', '2', '120000', 'New position', '—', '—', 'Feb 20, 2020', 'Overdue', 'sswefg', '15 Oct 2026, 4:17 pm', '19 Oct 2026, 12:21 pm', FALSE),
         ('MR-0002', 'Senior ML Engineer', 'AI Research', 3, 'Full-time', 'Hyderabad', '3-5', '3500000', 'Expansion', '—', 'Karthik Reddy', 'Nov 30, 2026', 'Approved', 'Core ML platform development in Rust & Python', '16 Oct 2026, 10:00 am', '19 Oct 2026, 11:30 am', FALSE)
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 8. Initial Candidate Trackers
    sqlx::query(
        "INSERT INTO candidate_trackers (id, full_name, applied_for, source, total_experience, expected_ctc, stage, email, phone, status, archived) VALUES 
         ('CAN-001', 'Rahul Varma', 'Senior ML Engineer', 'LinkedIn', 4, 2800000, 'Interview', 'rahul.varma@brightcone.ai', '+91 98765 43210', 'Active', FALSE),
         ('CAN-002', 'Siddharth Rao', 'Full-Stack Developer', 'Careers Page', 3, 2200000, 'Screening', 'siddharth.rao@example.com', '+91 98450 12345', 'Active', FALSE),
         ('CAN-003', 'Neha Bansal', 'QA Automation Lead', 'Referral', 5, 2500000, 'Interview', 'neha.bansal@example.com', '+91 91234 56789', 'Active', FALSE),
         ('CAN-004', 'Aarav Sharma', 'AI Systems Architect', 'Direct', 7, 4500000, 'Applied', 'aarav.sharma@example.com', '+91 99887 76655', 'Active', FALSE)
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 9. Initial Offer Letters (1 archived matching Screenshot 5)
    sqlx::query(
        "INSERT INTO offer_letters (id, candidate_id, candidate_name, designation, annual_ctc, joining_date, valid_till, status, archived) VALUES 
         ('OFF-001', 'CAN-002', 'Siddharth Rao', 'Full-Stack Developer', '₹22,00,000', 'Nov 15, 2026', 'Oct 25, 2026', 'Draft', TRUE)
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 10. Initial Goals (if empty)
    sqlx::query(
        "INSERT INTO goals (id, employee_id, employee_name, title, category, target_date, progress, status) VALUES 
         (1, 'BC-1012', 'Karthik Reddy', 'Deploy Distributed Vector LLM Serving Engine in Rust', 'Innovation', '2026-11-15', 85, 'In Progress'),
         (2, 'BC-1038', 'Rahul Varma', 'Migrate Core Database Schema to PostgreSQL 18 & Optimize Connection Pool', 'Engineering', '2026-10-31', 90, 'In Progress'),
         (3, 'BC-1019', 'Priya Menon', 'Establish Engineering Mentorship Framework for L4-L6 Engineers', 'Leadership', '2026-12-15', 60, 'In Progress'),
         (4, 'BC-1031', 'Sneha Iyer', 'Design System Unification for Brightcone People Web & Mobile UI', 'Product', '2026-11-30', 75, 'In Progress')
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 11. Initial Reviews (if empty)
    sqlx::query(
        "INSERT INTO reviews (id, employee_id, employee_name, reviewer_name, cycle, rating, feedback, status, updated_at) VALUES 
         (1, 'BC-1038', 'Rahul Varma', 'Priya Menon (Engineering Manager)', 'Q3 2026 Mid-Year', 4.8, 'Rahul has demonstrated extraordinary ownership of backend Rust services, reducing latency by 45% and delivering candidate pipeline synchronization.', 'Completed', 'Oct 10, 2026'),
         (2, 'BC-1012', 'Karthik Reddy', 'Naresh Vemparala (Founder & CEO)', 'Q3 2026 Mid-Year', 4.9, 'Pioneering technical visionary. Karthik led the architecture of the AI vector matching engine while upholding top-tier code maintainability.', 'Completed', 'Oct 08, 2026')
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 12. Initial Service Requests (if empty)
    sqlx::query(
        "INSERT INTO service_requests (id, employee_id, employee_name, category, subject, priority, details, status, created_at, updated_at) VALUES 
         ('REQ-0001', 'BC-1038', 'Rahul Varma', 'Benefits & insurance', 'Health Insurance Dependent Addition Request', 'Normal', 'Ticket #HD-892: Please add spouse and newborn to medical insurance cover.', 'Resolved', '15 Oct 2026, 10:00 am', '16 Oct 2026, 04:30 pm'),
         ('REQ-0002', 'BC-1038', 'Rahul Varma', 'Payroll & tax', 'Tax Exemption Declaration Clarification', 'Normal', 'Ticket #HD-941: Seeking clarification on HRA rent receipt submissions for Q3.', 'In Review', '18 Oct 2026, 02:15 pm', '18 Oct 2026, 02:15 pm')
         ON CONFLICT (id) DO NOTHING;"
    )
    .execute(pool)
    .await?;

    // 13. Initial Joining Formalities (if empty)
    let joining_count_row = sqlx::query("SELECT count(*) as c FROM joining_formalities;")
        .fetch_one(pool)
        .await?;
    let j_count: i64 = joining_count_row.get("c");
    if j_count == 0 {
        sqlx::query(
            "INSERT INTO joining_formalities (
                id, candidate_id, offer_letter_id, employee_id, candidate_name, employee_name,
                joining_date, department, designation, reporting_manager, documents_submitted,
                documents_verified, verification_status, joining_status, remarks,
                documents_collected, id_card_issued, work_email_created, laptop_issued,
                pf_nomination_form, appointment_letter_issued, archived, history,
                created_at, updated_at
            ) VALUES 
             ('JN-001', 'CAN-002', 'OFF-001', 'BC-1049', 'Tanvi Desai', 'Tanvi Desai', 'Nov 01, 2026', 'Product', 'Product Specialist', 'Ananya Rao', 'Aadhar, PAN, Relieving Letter, Degree Certificate', TRUE, 'Verified', 'In progress', 'Standard onboarding package dispatched. Physical address verification completed.', 'Yes', 'Yes', 'Yes', 'No', 'Yes', 'Yes', FALSE, '[{\"status\":\"In progress\",\"timestamp\":\"19 Oct 2026, 3:51 pm\",\"user\":\"ananya.rao@brightcone.ai\"},{\"status\":\"Pending\",\"timestamp\":\"19 Oct 2026, 12:43 pm\",\"user\":\"ananya.rao@brightcone.ai\"}]', '19 Oct 2026, 12:43 pm', '19 Oct 2026, 3:51 pm'),
             ('JN-002', 'CAN-003', 'OFF-005', 'BC-1061', 'Neha Bansal', 'Neha Bansal', 'Nov 15, 2026', 'Engineering', 'QA Automation Lead', 'Priya Menon', 'PAN Card, Previous 3 Months Payslips', FALSE, 'In progress', 'Pending', 'Awaiting previous employer relieving certificate and Form 16.', 'Yes', 'No', 'No', 'No', 'No', 'No', FALSE, '[{\"status\":\"Pending\",\"timestamp\":\"20 Oct 2026, 10:15 am\",\"user\":\"ananya.rao@brightcone.ai\"}]', '20 Oct 2026, 10:15 am', '20 Oct 2026, 10:15 am')
             ON CONFLICT (id) DO NOTHING;"
        )
        .execute(pool)
        .await?;
    }

    // 14. Initial Onboarding Checklists & Items (if empty)
    let chk_count_row = sqlx::query("SELECT count(*) as c FROM onboarding_checklists;")
        .fetch_one(pool)
        .await?;
    let c_count: i64 = chk_count_row.get("c");
    if c_count == 0 {
        sqlx::query(
            "INSERT INTO onboarding_checklists (
                id, employee_id, employee_name, title, description, assigned_by,
                start_date, due_date, status, completion_percentage, created_at, updated_at
            ) VALUES 
             ('CHK-001', 'BC-1038', 'Rahul Varma', 'Engineering New Joiner Ramp-up Checklist', 'End-to-end IT, security, and team ramp-up tasks for backend platform engineering.', 'Ananya Rao', 'Nov 01, 2026', 'Nov 15, 2026', 'In Progress', 50, '19 Oct 2026, 10:00 am', '19 Oct 2026, 04:00 pm'),
             ('CHK-002', 'BC-1049', 'Tanvi Desai', 'Product Specialist Day-1 Orientation', 'Account credentials, product demo access, and customer journey walkthrough.', 'Ananya Rao', 'Nov 05, 2026', 'Nov 20, 2026', 'Pending', 0, '20 Oct 2026, 11:30 am', '20 Oct 2026, 11:30 am')
             ON CONFLICT (id) DO NOTHING;"
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "INSERT INTO onboarding_checklist_items (
                id, checklist_id, title, description, assigned_to, due_date, status,
                completed_at, created_at, updated_at
            ) VALUES 
             ('CHK-001-ITEM-01', 'CHK-001', 'Set up G-Suite & Slack credentials (@brightcone.ai)', 'Provision corporate Google Workspace, 1Password vault, and GitHub access.', 'IT Ops', 'Nov 02, 2026', 'Completed', '19 Oct 2026, 02:00 pm', '19 Oct 2026, 10:00 am', '19 Oct 2026, 02:00 pm'),
             ('CHK-001-ITEM-02', 'CHK-001', 'Ship pre-configured MacBook Pro and Security Token', 'Hardware tag BC-HW-0842 with YubiKey 5C NFC key.', 'Workplace Ops', 'Nov 03, 2026', 'Completed', '19 Oct 2026, 04:00 pm', '19 Oct 2026, 10:00 am', '19 Oct 2026, 04:00 pm'),
             ('CHK-001-ITEM-03', 'CHK-001', 'Schedule 1:1 Introduction with Engineering Manager', 'Overview of team goals, sprint cadence, and architecture review.', 'Priya Menon', 'Nov 05, 2026', 'Pending', '', '19 Oct 2026, 10:00 am', '19 Oct 2026, 10:00 am'),
             ('CHK-001-ITEM-04', 'CHK-001', 'Complete Security Awareness and POSH compliance course', 'Mandatory 45-minute information security training.', 'People Ops', 'Nov 10, 2026', 'Pending', '', '19 Oct 2026, 10:00 am', '19 Oct 2026, 10:00 am'),
             ('CHK-002-ITEM-01', 'CHK-002', 'Create Brightcone Product Sandbox Account', 'Set up enterprise tenant credentials on staging environment.', 'IT Ops', 'Nov 06, 2026', 'Pending', '', '20 Oct 2026, 11:30 am', '20 Oct 2026, 11:30 am'),
             ('CHK-002-ITEM-02', 'CHK-002', 'Assign Onboarding Buddy from Product Team', 'Pair with a senior product manager for first 30 days.', 'Ananya Rao', 'Nov 07, 2026', 'Pending', '', '20 Oct 2026, 11:30 am', '20 Oct 2026, 11:30 am')
             ON CONFLICT (id) DO NOTHING;"
        )
        .execute(pool)
        .await?;
    }

    // 15. Initial Internships (if empty)
    let int_count_row = sqlx::query("SELECT count(*) as c FROM internships;")
        .fetch_one(pool)
        .await?;
    let i_count: i64 = int_count_row.get("c");
    if i_count == 0 {
        sqlx::query(
            "INSERT INTO internships (
                id, intern_name, email, phone, college, degree, department,
                internship_role, mentor_employee_id, mentor_name, start_date, end_date,
                stipend, status, work_mode, project_title, performance_rating,
                feedback, created_at, updated_at
            ) VALUES 
             ('INT-001', 'Arjun Mehta', 'arjun.mehta@intern.brightcone.ai', '+91 98450 11223', 'IIT Hyderabad', 'B.Tech Computer Science', 'AI Research', 'AI Research Intern', 'BC-1012', 'Karthik Reddy', 'Jan 15, 2026', 'Jul 15, 2026', '₹45,000/month', 'Active', 'In-office', 'Distributed Tensor Quantization for Low-Latency Serving', 4.8, 'Outstanding ownership on GPU kernel profiling; benchmarked 30% speedup on KV cache quantization.', '15 Jan 2026, 09:30 am', '15 Oct 2026, 05:00 pm'),
             ('INT-002', 'Sneha Kulkarni', 'sneha.kulkarni@intern.brightcone.ai', '+91 99120 44556', 'BITS Pilani', 'M.Sc (Tech) Information Systems', 'Engineering', 'Systems Engineering Intern', 'BC-1019', 'Priya Menon', 'Feb 01, 2026', 'Jul 31, 2026', '₹40,000/month', 'PPO Offered', 'Hybrid', 'Multi-tenant PostgreSQL Connection Multiplexer in Rust', 4.9, 'Demonstrated production-grade engineering excellence; candidate recommended for full-time L4 Backend Engineer PPO.', '01 Feb 2026, 10:00 am', '18 Oct 2026, 04:30 pm'),
             ('INT-003', 'Rohit Nair', 'rohit.nair@intern.brightcone.ai', '+91 97230 77889', 'NIT Trichy', 'B.Tech Electronics & Comm.', 'Product', 'Product Analytics Intern', 'BC-1004', 'Ananya Rao', 'Mar 01, 2026', 'Aug 31, 2026', '₹35,000/month', 'Completed', 'Remote', 'Funnel Conversion Analytics & Dropoff Telemetry Dashboard', 4.5, 'Successfully delivered interactive funnel dashboard used weekly by leadership.', '01 Mar 2026, 11:00 am', '01 Sep 2026, 06:00 pm')
             ON CONFLICT (id) DO NOTHING;"
        )
        .execute(pool)
        .await?;
    }

    info!("[Database Seed] Initial seed records verified successfully.");
    Ok(())
}

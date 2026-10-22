use loco_rs::testing::prelude::*;
use serial_test::serial;
use brightcone_hr_portal::app::App;

#[tokio::test]
#[serial]
async fn test_api_health() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/health").await;
        assert_eq!(res.status_code(), 200);
        assert!(res.text().contains("ok"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_employees_and_seed_data() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/employees").await;
        assert_eq!(res.status_code(), 200);
        let text = res.text();
        // Check demonstration records
        assert!(text.contains("BC-1001"));
        assert!(text.contains("Naresh Vemparala"));
        assert!(text.contains("BC-1004"));
        assert!(text.contains("Ananya Rao"));
        assert!(text.contains("BC-1042"));
        assert!(text.contains("Meera Joshi"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_dashboard_stats() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/dashboard/stats").await;
        assert_eq!(res.status_code(), 200);
        let text = res.text();
        assert!(text.contains("total_employees"));
        assert!(text.contains("total_jobs"));
        assert!(text.contains("total_candidates"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_recruitment_jobs_and_candidates() {
    request::<App, _, _>(|request, _ctx| async move {
        let res_jobs = request.get("/api/jobs").await;
        assert_eq!(res_jobs.status_code(), 200);

        let res_cand = request.get("/api/candidates").await;
        assert_eq!(res_cand.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_performance_goals_and_reviews() {
    request::<App, _, _>(|request, _ctx| async move {
        let res_goals = request.get("/api/performance/goals").await;
        assert_eq!(res_goals.status_code(), 200);

        let res_reviews = request.get("/api/performance/reviews").await;
        assert_eq!(res_reviews.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_auth_flow() {
    request::<App, _, _>(|request, _ctx| async move {
        let login_body = serde_json::json!({
            "email": "admin@brightcone.ai",
            "password": "admin123"
        });

        let res = request.post("/api/auth/login").json(&login_body).await;
        assert_eq!(res.status_code(), 200);
        let text = res.text();
        assert!(text.contains("token"));
        assert!(text.contains("admin@brightcone.ai"));
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_joining_formalities() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. List
        let res = request.get("/api/joining-formalities").await;
        assert_eq!(res.status_code(), 200);
        let list_text = res.text();
        assert!(list_text.contains("JN-") || list_text.contains("["));

        // 2. Create
        let create_body = serde_json::json!({
            "candidate_name": "Siddharth Verma",
            "joining_date": "2026-11-01",
            "department": "Engineering",
            "designation": "Staff Rust Engineer",
            "documents_collected": "Yes",
            "id_card_issued": "No",
            "work_email_created": "Yes",
            "status": "In progress"
        });
        let res_create = request
            .post("/api/joining-formalities")
            .json(&create_body)
            .await;
        assert_eq!(res_create.status_code(), 200);
        let created_json: serde_json::Value = serde_json::from_str(&res_create.text()).unwrap();
        let record_id = created_json["id"].as_str().unwrap();

        // 3. Update status
        let status_body = serde_json::json!({
            "status": "Completed",
            "remarks": "All day-one formalities verified."
        });
        let res_status = request
            .patch(&format!("/api/joining-formalities/{}/status", record_id))
            .json(&status_body)
            .await;
        assert_eq!(res_status.status_code(), 200);

        // 4. Complete and link employee
        let complete_body = serde_json::json!({
            "employee_id": "BC-1099"
        });
        let res_complete = request
            .post(&format!("/api/joining-formalities/{}/complete", record_id))
            .json(&complete_body)
            .await;
        assert_eq!(res_complete.status_code(), 200);
        let complete_json: serde_json::Value = serde_json::from_str(&res_complete.text()).unwrap();
        assert_eq!(complete_json["success"], true);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_onboarding_checklists() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. List
        let res = request.get("/api/onboarding-checklists").await;
        assert_eq!(res.status_code(), 200);

        // 2. Create with initial items
        let create_body = serde_json::json!({
            "employee_id": "BC-1004",
            "title": "Full-Stack Engineer Onboarding",
            "description": "Onboarding sprint for team integration",
            "start_date": "2026-11-01",
            "due_date": "2026-11-15",
            "items": [
                { "title": "Setup local development environment", "status": "Pending" },
                { "title": "Security compliance training", "status": "Pending" }
            ]
        });
        let res_create = request
            .post("/api/onboarding-checklists")
            .json(&create_body)
            .await;
        assert_eq!(res_create.status_code(), 200);
        let chk_json: serde_json::Value = serde_json::from_str(&res_create.text()).unwrap();
        let chk_id = chk_json["id"].as_str().unwrap();
        assert_eq!(chk_json["completion_percentage"], 0);

        // 3. Add an additional item
        let add_item_body = serde_json::json!({
            "title": "First code review and deployment",
            "assigned_to": "Manager",
            "status": "Pending"
        });
        let res_item = request
            .post(&format!("/api/onboarding-checklists/{}/items", chk_id))
            .json(&add_item_body)
            .await;
        assert_eq!(res_item.status_code(), 200);
        let item_json: serde_json::Value = serde_json::from_str(&res_item.text()).unwrap();
        let item_id = item_json["id"].as_str().unwrap();

        // 4. Update item status to Completed -> Verify backend recalculates progress percentage
        let status_body = serde_json::json!({
            "status": "Completed"
        });
        let res_status = request
            .patch(&format!(
                "/api/onboarding-checklists/{}/items/{}/status",
                chk_id, item_id
            ))
            .json(&status_body)
            .await;
        assert_eq!(res_status.status_code(), 200);
        let updated_chk: serde_json::Value = serde_json::from_str(&res_status.text()).unwrap();
        // 1 of 3 items completed = 33%
        assert_eq!(updated_chk["completion_percentage"], 33);
        assert_eq!(updated_chk["status"], "In Progress");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_onboarding_tasks() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. List onboarding tasks
        let res = request.get("/api/onboarding-tasks").await;
        assert_eq!(res.status_code(), 200);

        // 2. Create single onboarding task
        let create_task = serde_json::json!({
            "employee_id": "BC-1001",
            "task": "Issue Security Access Card & VPN Key",
            "owner": "IT Support",
            "due_date": "2026-11-05",
            "status": "Pending",
            "notes": "Verify biometric access for 3rd floor labs"
        });
        let res_create = request
            .post("/api/onboarding-tasks")
            .json(&create_task)
            .await;
        assert_eq!(res_create.status_code(), 200);
        let created_task: serde_json::Value = serde_json::from_str(&res_create.text()).unwrap();
        let task_id = created_task["id"].as_str().unwrap();
        assert_eq!(
            created_task["title"],
            "Issue Security Access Card & VPN Key"
        );
        assert_eq!(created_task["assigned_to"], "IT Support");

        // 3. Update status
        let update_status = serde_json::json!({
            "status": "Completed",
            "completed_on": "2026-11-04"
        });
        let res_status = request
            .patch(&format!("/api/onboarding-tasks/{}/status", task_id))
            .json(&update_status)
            .await;
        assert_eq!(res_status.status_code(), 200);
        let updated_task: serde_json::Value = serde_json::from_str(&res_status.text()).unwrap();
        assert_eq!(updated_task["status"], "Completed");

        // 4. Archive task
        let res_archive = request
            .patch(&format!("/api/onboarding-tasks/{}/archive", task_id))
            .json(&serde_json::json!({ "archived": true }))
            .await;
        assert_eq!(res_archive.status_code(), 200);

        // 5. Create Standard Checklist (8 standard tasks)
        let res_std = request
            .post("/api/onboarding-tasks/standard")
            .json(&serde_json::json!({ "employee_id": "BC-1002" }))
            .await;
        assert_eq!(res_std.status_code(), 200);
        let std_json: serde_json::Value = serde_json::from_str(&res_std.text()).unwrap();
        assert!(std_json["success"].as_bool().unwrap());
        assert!(std_json["created_count"].as_u64().unwrap() > 0);

        // 6. Delete task
        let res_del = request
            .delete(&format!("/api/onboarding-tasks/{}", task_id))
            .await;
        assert_eq!(res_del.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn test_api_internships() {
    request::<App, _, _>(|request, _ctx| async move {
        // 1. List
        let res = request.get("/api/internships").await;
        assert_eq!(res.status_code(), 200);

        // 2. Create
        let create_body = serde_json::json!({
            "intern_name": "Aarav Sharma",
            "email": "aarav.sharma@brightcone.ai",
            "phone": "+91 9988776655",
            "college": "IIT Bombay",
            "degree": "B.Tech Computer Science",
            "department": "Engineering",
            "internship_role": "Backend Rust Intern",
            "start_date": "2026-06-01",
            "end_date": "2026-12-01",
            "stipend": "₹35,000 / month",
            "status": "Active",
            "work_mode": "Hybrid",
            "project_title": "PostgreSQL Query Optimizer"
        });
        let res_create = request.post("/api/internships").json(&create_body).await;
        assert_eq!(res_create.status_code(), 200);
        let intern_json: serde_json::Value = serde_json::from_str(&res_create.text()).unwrap();
        let intern_id = intern_json["id"].as_str().unwrap();
        assert_eq!(intern_json["intern_name"], "Aarav Sharma");

        // 3. Update status to PPO Offered with 5-star rating
        let status_body = serde_json::json!({
            "status": "PPO Offered",
            "performance_rating": 5,
            "feedback": "Outstanding architectural contributions and high productivity."
        });
        let res_status = request
            .patch(&format!("/api/internships/{}/status", intern_id))
            .json(&status_body)
            .await;
        assert_eq!(res_status.status_code(), 200);
        let updated_json: serde_json::Value = serde_json::from_str(&res_status.text()).unwrap();
        assert_eq!(updated_json["status"], "PPO Offered");
        assert_eq!(updated_json["performance_rating"].as_f64(), Some(5.0));
    })
    .await;
}

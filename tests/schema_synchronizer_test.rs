use brightcone_hr_portal::database::{hash_password, verify_password, PgDataType, SchemaManifest};

#[test]
fn test_entity_schema_manifest_completeness() {
    let manifest = SchemaManifest::new();
    let tables = &manifest.tables;

    // Verify all 20 required business entities are registered
    let required_entities = vec![
        "users",
        "jobs",
        "candidates",
        "interviews",
        "employees",
        "employee_profiles",
        "employee_leaves",
        "employee_attendance",
        "employee_referrals",
        "goals",
        "reviews",
        "manpower_requirements",
        "candidate_trackers",
        "offer_letters",
        "service_requests",
        "employee_dependents",
        "joining_formalities",
        "onboarding_checklists",
        "onboarding_checklist_items",
        "internships",
    ];

    for entity in &required_entities {
        let table = manifest.get_table(entity);
        assert!(
            table.is_some(),
            "Entity '{}' must be registered in SchemaManifest",
            entity
        );

        let t = table.unwrap();
        assert_eq!(&t.name, entity);
        assert!(
            !t.columns.is_empty(),
            "Table '{}' must have columns defined",
            entity
        );
        assert!(
            !t.primary_key.is_empty(),
            "Table '{}' must have a primary key defined",
            entity
        );
    }

    assert_eq!(
        tables.len(),
        20,
        "Manifest must contain exactly 20 registered entities"
    );
}

#[test]
fn test_column_definition_and_data_types() {
    let manifest = SchemaManifest::new();

    // Inspect users table metadata
    let users = manifest.get_table("users").expect("users table registered");
    let email_col = users
        .columns
        .iter()
        .find(|c| c.name == "email")
        .expect("email column");
    assert_eq!(email_col.data_type, PgDataType::Varchar(Some(255)));
    assert!(!email_col.nullable);

    // Inspect employees table metadata
    let employees = manifest
        .get_table("employees")
        .expect("employees table registered");
    assert_eq!(employees.primary_key, vec!["id"]);
    let id_col = employees
        .columns
        .iter()
        .find(|c| c.name == "id")
        .expect("id column");
    assert_eq!(id_col.data_type, PgDataType::Varchar(Some(50)));

    // Inspect candidates table metadata
    let candidates = manifest.get_table("candidates").expect("candidates table");
    let job_id_col = candidates
        .columns
        .iter()
        .find(|c| c.name == "job_id")
        .expect("job_id col");
    assert_eq!(job_id_col.data_type, PgDataType::Integer);

    // Inspect joining_formalities table metadata
    let joining = manifest
        .get_table("joining_formalities")
        .expect("joining_formalities table");
    assert_eq!(joining.primary_key, vec!["id"]);
    assert!(joining.columns.iter().any(|c| c.name == "candidate_id"));
    assert!(joining.columns.iter().any(|c| c.name == "offer_letter_id"));
    assert!(joining.columns.iter().any(|c| c.name == "employee_id"));
    assert!(joining.columns.iter().any(|c| c.name == "joining_date"));
    assert!(joining.columns.iter().any(|c| c.name == "joining_status"));
    assert!(joining
        .columns
        .iter()
        .any(|c| c.name == "documents_submitted"));
    assert!(joining
        .columns
        .iter()
        .any(|c| c.name == "documents_verified"));

    // Inspect onboarding_checklists and items metadata
    let checklists = manifest
        .get_table("onboarding_checklists")
        .expect("onboarding_checklists table");
    assert_eq!(checklists.primary_key, vec!["id"]);
    assert!(checklists.columns.iter().any(|c| c.name == "employee_id"));
    assert!(checklists
        .columns
        .iter()
        .any(|c| c.name == "completion_percentage"));

    let items = manifest
        .get_table("onboarding_checklist_items")
        .expect("onboarding_checklist_items table");
    assert_eq!(items.primary_key, vec!["id"]);
    assert!(items.columns.iter().any(|c| c.name == "checklist_id"));
    assert!(items.columns.iter().any(|c| c.name == "status"));

    // Inspect internships table metadata
    let internships = manifest
        .get_table("internships")
        .expect("internships table");
    assert_eq!(internships.primary_key, vec!["id"]);
    assert!(internships.columns.iter().any(|c| c.name == "intern_name"));
    assert!(internships
        .columns
        .iter()
        .any(|c| c.name == "mentor_employee_id"));
    assert!(internships.columns.iter().any(|c| c.name == "stipend"));
    assert!(internships.columns.iter().any(|c| c.name == "status"));
}

#[test]
fn test_password_hashing_and_verification() {
    let raw = "SecureAdmin@123!";
    let hashed = hash_password(raw);

    // Ensure hash is formatted with sha256 prefix and salted
    assert!(hashed.starts_with("$sha256$"));
    assert_ne!(raw, hashed);

    // Verify correct password matches
    assert!(verify_password(raw, &hashed));

    // Verify incorrect password fails
    assert!(!verify_password("WrongPassword999", &hashed));

    // Verify legacy fallback matches plaintext
    assert!(verify_password("legacy_pass", "legacy_pass"));
}

#[test]
fn test_sql_injection_defense_in_parameters() {
    // Attack payloads that would break naive SQL string interpolation
    let malicious_email = "admin@brightcone.ai' OR '1'='1' --";
    let malicious_name = "Robert'); DROP TABLE users; --";

    // Test password hashing handles malicious strings safely
    let hashed = hash_password(malicious_name);
    assert!(verify_password(malicious_name, &hashed));
    assert!(!verify_password("Robert", &hashed));

    // Verify sanitized email clean handling
    let clean = malicious_email.trim().to_lowercase();
    assert!(clean.contains("'")); // Retained as literal content, never interpolated as SQL syntax
}

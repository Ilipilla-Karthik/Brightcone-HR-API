use super::schema::{ColumnDefinition, IndexDefinition, PgDataType, TableDefinition};

/// Centralized Entity Registry holding the authoritative schema definition
/// for all persistent business entities in the Brightcone HR Portal.
pub struct SchemaManifest {
    pub tables: Vec<TableDefinition>,
}

impl Default for SchemaManifest {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaManifest {
    /// Build the complete entity registry
    #[allow(clippy::vec_init_then_push)]
    pub fn new() -> Self {
        let mut tables = Vec::new();

        // 1. Users table
        tables.push(
            TableDefinition::new("users")
                .column(ColumnDefinition::new("id", PgDataType::Integer).primary_key())
                .column(ColumnDefinition::new("name", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("email", PgDataType::Varchar(Some(255)))
                        .not_null()
                        .unique(),
                )
                .column(
                    ColumnDefinition::new("password", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(ColumnDefinition::new("role", PgDataType::Varchar(Some(100))).not_null())
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100)))
                        .default_val("'Today'"),
                )
                .index(IndexDefinition::new("idx_users_email", &["email"]).unique()),
        );

        // 2. Jobs table
        tables.push(
            TableDefinition::new("jobs")
                .column(ColumnDefinition::new("id", PgDataType::Integer).primary_key())
                .column(ColumnDefinition::new("title", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("department", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("location", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(ColumnDefinition::new("skills", PgDataType::Text).not_null())
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .not_null()
                        .default_val("'Active'"),
                )
                .column(
                    ColumnDefinition::new("posted_date", PgDataType::Varchar(Some(100))).not_null(),
                )
                .index(IndexDefinition::new("idx_jobs_status", &["status"])),
        );

        // 3. Candidates table
        tables.push(
            TableDefinition::new("candidates")
                .column(ColumnDefinition::new("id", PgDataType::Integer).primary_key())
                .column(ColumnDefinition::new("name", PgDataType::Varchar(Some(255))).not_null())
                .column(ColumnDefinition::new("email", PgDataType::Varchar(Some(255))).not_null())
                .column(ColumnDefinition::new("job_id", PgDataType::Integer).not_null())
                .column(
                    ColumnDefinition::new("job_title", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(ColumnDefinition::new("skills", PgDataType::Text).not_null())
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .not_null()
                        .default_val("'Applied'"),
                )
                .column(
                    ColumnDefinition::new("applied_date", PgDataType::Varchar(Some(100)))
                        .not_null(),
                )
                .index(IndexDefinition::new("idx_candidates_job_id", &["job_id"]))
                .index(IndexDefinition::new("idx_candidates_email", &["email"])),
        );

        // 4. Interviews table
        tables.push(
            TableDefinition::new("interviews")
                .column(ColumnDefinition::new("id", PgDataType::Integer).primary_key())
                .column(ColumnDefinition::new("candidate_id", PgDataType::Integer).not_null())
                .column(
                    ColumnDefinition::new("candidate_name", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("candidate_email", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("job_title", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("interview_date", PgDataType::Varchar(Some(100)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("formatted_date", PgDataType::Varchar(Some(100)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("interviewer", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("interview_type", PgDataType::Varchar(Some(100)))
                        .not_null(),
                )
                .column(ColumnDefinition::new("notes", PgDataType::Text).default_val("''"))
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .not_null()
                        .default_val("'Scheduled'"),
                )
                .column(ColumnDefinition::new(
                    "round",
                    PgDataType::Varchar(Some(100)),
                ))
                .column(ColumnDefinition::new("mode", PgDataType::Varchar(Some(50))))
                .column(ColumnDefinition::new("time", PgDataType::Varchar(Some(50))))
                .column(ColumnDefinition::new(
                    "venue_link",
                    PgDataType::Varchar(Some(255)),
                ))
                .column(ColumnDefinition::new("rating", PgDataType::Integer).default_val("0"))
                .column(
                    ColumnDefinition::new("result", PgDataType::Varchar(Some(50)))
                        .default_val("'Pending'"),
                )
                .column(ColumnDefinition::new("feedback", PgDataType::Text).default_val("''"))
                .column(ColumnDefinition::new("archived", PgDataType::Boolean).default_val("FALSE"))
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("updated_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(ColumnDefinition::new("history", PgDataType::Text).default_val("'[]'"))
                .index(IndexDefinition::new(
                    "idx_interviews_candidate_id",
                    &["candidate_id"],
                ))
                .index(IndexDefinition::new("idx_interviews_status", &["status"])),
        );

        // 5. Employees table
        tables.push(
            TableDefinition::new("employees")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(ColumnDefinition::new("name", PgDataType::Varchar(Some(255))).not_null())
                .column(ColumnDefinition::new("role", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("department", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("grade", PgDataType::Varchar(Some(50)))
                        .default_val("'L3'"),
                )
                .column(
                    ColumnDefinition::new("location", PgDataType::Varchar(Some(255)))
                        .default_val("'Hyderabad'"),
                )
                .column(
                    ColumnDefinition::new("manager", PgDataType::Varchar(Some(255)))
                        .default_val("'Ananya Rao'"),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Active'"),
                )
                .column(ColumnDefinition::new("email", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("joined_date", PgDataType::Varchar(Some(100)))
                        .default_val("'Oct 01, 2026'"),
                )
                .index(IndexDefinition::new("idx_employees_email", &["email"]))
                .index(IndexDefinition::new("idx_employees_dept", &["department"])),
        );

        // 6. Employee Profiles table
        tables.push(
            TableDefinition::new("employee_profiles")
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50)))
                        .primary_key(),
                )
                .column(
                    ColumnDefinition::new("full_name", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("work_email", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(ColumnDefinition::new("role", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("department", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Active'"),
                )
                .column(
                    ColumnDefinition::new("phone", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("dob", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("blood_group", PgDataType::Varchar(Some(20)))
                        .default_val("''"),
                )
                .column(ColumnDefinition::new("address", PgDataType::Text).default_val("''"))
                .column(
                    ColumnDefinition::new("emergency_contact", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("manager", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("grade", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("location", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("doj", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("bank_name", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("bank_account", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("ifsc", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("pan", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("uan", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("nominee_name", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("nominee_relation", PgDataType::Varchar(Some(50)))
                        .default_val("''"),
                ),
        );

        // 7. Employee Leaves table
        tables.push(
            TableDefinition::new("employee_leaves")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(
                    ColumnDefinition::new("leave_type", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(
                    ColumnDefinition::new("from_date", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(ColumnDefinition::new("to_date", PgDataType::Varchar(Some(50))).not_null())
                .column(ColumnDefinition::new("days", PgDataType::Integer).not_null())
                .column(ColumnDefinition::new("reason", PgDataType::Text))
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Pending'"),
                )
                .index(IndexDefinition::new("idx_leaves_emp_id", &["employee_id"])),
        );

        // 8. Employee Attendance table
        tables.push(
            TableDefinition::new("employee_attendance")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(ColumnDefinition::new("date", PgDataType::Varchar(Some(50))).not_null())
                .column(ColumnDefinition::new("punch_in", PgDataType::Varchar(Some(50))).not_null())
                .column(ColumnDefinition::new(
                    "punch_out",
                    PgDataType::Varchar(Some(50)),
                ))
                .column(
                    ColumnDefinition::new("work_hours", PgDataType::Varchar(Some(50)))
                        .default_val("'8h 00m'"),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Present'"),
                )
                .index(IndexDefinition::new(
                    "idx_att_emp_date",
                    &["employee_id", "date"],
                )),
        );

        // 9. Employee Referrals table
        tables.push(
            TableDefinition::new("employee_referrals")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("candidate_name", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(ColumnDefinition::new("role", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("referred_by", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(ColumnDefinition::new("bonus", PgDataType::Integer).default_val("30000"))
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Applied'"),
                )
                .column(
                    ColumnDefinition::new("date", PgDataType::Varchar(Some(50)))
                        .default_val("'Today'"),
                )
                .column(
                    ColumnDefinition::new("email", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("phone", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(ColumnDefinition::new("notes", PgDataType::Text).default_val("''"))
                .index(IndexDefinition::new(
                    "idx_referrals_referred_by",
                    &["referred_by"],
                )),
        );

        // 10. Goals table
        tables.push(
            TableDefinition::new("goals")
                .column(ColumnDefinition::new("id", PgDataType::Integer).primary_key())
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(
                    ColumnDefinition::new("employee_name", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(ColumnDefinition::new("title", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("category", PgDataType::Varchar(Some(100))).not_null(),
                )
                .column(
                    ColumnDefinition::new("target_date", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(ColumnDefinition::new("progress", PgDataType::Integer).default_val("0"))
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'In Progress'"),
                )
                .index(IndexDefinition::new("idx_goals_emp_id", &["employee_id"])),
        );

        // 11. Reviews table
        tables.push(
            TableDefinition::new("reviews")
                .column(ColumnDefinition::new("id", PgDataType::Integer).primary_key())
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(
                    ColumnDefinition::new("employee_name", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("reviewer_name", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(ColumnDefinition::new("cycle", PgDataType::Varchar(Some(50))).not_null())
                .column(ColumnDefinition::new("rating", PgDataType::Float).default_val("4.0"))
                .column(ColumnDefinition::new("feedback", PgDataType::Text).not_null())
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Completed'"),
                )
                .column(
                    ColumnDefinition::new("updated_at", PgDataType::Varchar(Some(50)))
                        .default_val("'Today'"),
                )
                .index(IndexDefinition::new("idx_reviews_emp_id", &["employee_id"])),
        );

        // 12. Manpower Requirements table
        tables.push(
            TableDefinition::new("manpower_requirements")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("designation", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("department", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("positions", PgDataType::Integer)
                        .not_null()
                        .default_val("1"),
                )
                .column(
                    ColumnDefinition::new("employment_type", PgDataType::Varchar(Some(100)))
                        .default_val("'Full-time'"),
                )
                .column(
                    ColumnDefinition::new("location", PgDataType::Varchar(Some(255)))
                        .default_val("'Hyderabad'"),
                )
                .column(
                    ColumnDefinition::new("experience", PgDataType::Varchar(Some(100)))
                        .default_val("'2-5'"),
                )
                .column(
                    ColumnDefinition::new("budget_ctc", PgDataType::Varchar(Some(100)))
                        .default_val("'120000'"),
                )
                .column(
                    ColumnDefinition::new("reason", PgDataType::Varchar(Some(255)))
                        .default_val("'New position'"),
                )
                .column(
                    ColumnDefinition::new("replacing", PgDataType::Varchar(Some(255)))
                        .default_val("'—'"),
                )
                .column(
                    ColumnDefinition::new("hiring_manager", PgDataType::Varchar(Some(255)))
                        .default_val("'—'"),
                )
                .column(
                    ColumnDefinition::new("required_by", PgDataType::Varchar(Some(100)))
                        .default_val("'Feb 20, 2020'"),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Approved'"),
                )
                .column(
                    ColumnDefinition::new("job_description", PgDataType::Text).default_val("''"),
                )
                .column(ColumnDefinition::new(
                    "created_at",
                    PgDataType::Varchar(Some(100)),
                ))
                .column(ColumnDefinition::new(
                    "updated_at",
                    PgDataType::Varchar(Some(100)),
                ))
                .column(ColumnDefinition::new("archived", PgDataType::Boolean).default_val("FALSE"))
                .index(IndexDefinition::new("idx_manpower_dept", &["department"])),
        );

        // 13. Candidate Trackers table
        tables.push(
            TableDefinition::new("candidate_trackers")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("full_name", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("applied_for", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("source", PgDataType::Varchar(Some(100)))
                        .default_val("'Careers Page'"),
                )
                .column(
                    ColumnDefinition::new("total_experience", PgDataType::Integer).default_val("3"),
                )
                .column(ColumnDefinition::new("current_ctc", PgDataType::BigInt).default_val("0"))
                .column(
                    ColumnDefinition::new("expected_ctc", PgDataType::BigInt)
                        .default_val("1500000"),
                )
                .column(
                    ColumnDefinition::new("notice_period", PgDataType::Integer).default_val("0"),
                )
                .column(
                    ColumnDefinition::new("current_location", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("resume", PgDataType::Varchar(Some(255)))
                        .default_val("'—'"),
                )
                .column(
                    ColumnDefinition::new("consent_to_store_data", PgDataType::Varchar(Some(50)))
                        .default_val("'Yes'"),
                )
                .column(
                    ColumnDefinition::new("stage", PgDataType::Varchar(Some(50)))
                        .default_val("'Applied'"),
                )
                .column(ColumnDefinition::new("email", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("phone", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50)))
                        .default_val("'—'"),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Active'"),
                )
                .column(ColumnDefinition::new("archived", PgDataType::Boolean).default_val("FALSE"))
                .column(
                    ColumnDefinition::new("created_at", PgDataType::TimestampWithTimeZone)
                        .default_val("CURRENT_TIMESTAMP"),
                )
                .index(IndexDefinition::new("idx_cand_track_email", &["email"]))
                .index(IndexDefinition::new("idx_cand_track_stage", &["stage"])),
        );

        // 14. Offer Letters table
        tables.push(
            TableDefinition::new("offer_letters")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(ColumnDefinition::new(
                    "candidate_id",
                    PgDataType::Varchar(Some(50)),
                ))
                .column(
                    ColumnDefinition::new("candidate_name", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("designation", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(
                    ColumnDefinition::new("department", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("annual_ctc", PgDataType::Varchar(Some(100))).not_null(),
                )
                .column(
                    ColumnDefinition::new("joining_date", PgDataType::Varchar(Some(100)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("offer_date", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("valid_till", PgDataType::Varchar(Some(100))).not_null(),
                )
                .column(
                    ColumnDefinition::new("approved_by", PgDataType::Varchar(Some(100)))
                        .default_val("'—'"),
                )
                .column(
                    ColumnDefinition::new("offer_letter_file", PgDataType::Varchar(Some(255)))
                        .default_val("'—'"),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .default_val("'Pending approval'"),
                )
                .column(ColumnDefinition::new("archived", PgDataType::Boolean).default_val("FALSE"))
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("updated_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(ColumnDefinition::new("history", PgDataType::Text).default_val("'[]'"))
                .index(IndexDefinition::new(
                    "idx_offers_cand_id",
                    &["candidate_id"],
                ))
                .index(IndexDefinition::new("idx_offers_status", &["status"])),
        );

        // 17. Service Requests table
        tables.push(
            TableDefinition::new("service_requests")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(
                    ColumnDefinition::new("employee_name", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("category", PgDataType::Varchar(Some(100))).not_null(),
                )
                .column(ColumnDefinition::new("subject", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("priority", PgDataType::Varchar(Some(50)))
                        .not_null()
                        .default_val("'Normal'"),
                )
                .column(ColumnDefinition::new("details", PgDataType::Text).default_val("''"))
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .not_null()
                        .default_val("'Submitted'"),
                )
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100))).not_null(),
                )
                .column(
                    ColumnDefinition::new("updated_at", PgDataType::Varchar(Some(100))).not_null(),
                )
                .index(IndexDefinition::new(
                    "idx_service_requests_emp_id",
                    &["employee_id"],
                ))
                .index(IndexDefinition::new(
                    "idx_service_requests_category",
                    &["category"],
                )),
        );

        // 18. Employee Dependents table
        tables.push(
            TableDefinition::new("employee_dependents")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(ColumnDefinition::new("name", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("relationship", PgDataType::Varchar(Some(100)))
                        .not_null(),
                )
                .column(ColumnDefinition::new("dob", PgDataType::Varchar(Some(100))).not_null())
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .index(IndexDefinition::new(
                    "idx_employee_dependents_emp_id",
                    &["employee_id"],
                )),
        );

        // 19. Joining Formalities table
        tables.push(
            TableDefinition::new("joining_formalities")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("candidate_id", PgDataType::Varchar(Some(50)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("offer_letter_id", PgDataType::Varchar(Some(50)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("candidate_name", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("employee_name", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("joining_date", PgDataType::Varchar(Some(100)))
                        .not_null()
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("department", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("designation", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("reporting_manager", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("documents_submitted", PgDataType::Text)
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("documents_verified", PgDataType::Boolean)
                        .default_val("FALSE"),
                )
                .column(
                    ColumnDefinition::new("verification_status", PgDataType::Varchar(Some(50)))
                        .default_val("'Pending'"),
                )
                .column(
                    ColumnDefinition::new("joining_status", PgDataType::Varchar(Some(50)))
                        .default_val("'Pending'"),
                )
                .column(ColumnDefinition::new("remarks", PgDataType::Text).default_val("''"))
                .column(
                    ColumnDefinition::new("documents_collected", PgDataType::Varchar(Some(10)))
                        .default_val("'No'"),
                )
                .column(
                    ColumnDefinition::new("id_card_issued", PgDataType::Varchar(Some(10)))
                        .default_val("'No'"),
                )
                .column(
                    ColumnDefinition::new("work_email_created", PgDataType::Varchar(Some(10)))
                        .default_val("'No'"),
                )
                .column(
                    ColumnDefinition::new("laptop_issued", PgDataType::Varchar(Some(10)))
                        .default_val("'No'"),
                )
                .column(
                    ColumnDefinition::new("pf_nomination_form", PgDataType::Varchar(Some(10)))
                        .default_val("'No'"),
                )
                .column(
                    ColumnDefinition::new(
                        "appointment_letter_issued",
                        PgDataType::Varchar(Some(10)),
                    )
                    .default_val("'No'"),
                )
                .column(ColumnDefinition::new("archived", PgDataType::Boolean).default_val("FALSE"))
                .column(ColumnDefinition::new("history", PgDataType::Text).default_val("'[]'"))
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("updated_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .index(IndexDefinition::new("idx_joining_emp_id", &["employee_id"]))
                .index(IndexDefinition::new(
                    "idx_joining_cand_id",
                    &["candidate_id"],
                ))
                .index(IndexDefinition::new(
                    "idx_joining_offer_id",
                    &["offer_letter_id"],
                ))
                .index(IndexDefinition::new(
                    "idx_joining_status",
                    &["joining_status"],
                )),
        );

        // 20. Onboarding Checklists table
        tables.push(
            TableDefinition::new("onboarding_checklists")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(
                    ColumnDefinition::new("employee_name", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(ColumnDefinition::new("title", PgDataType::Varchar(Some(255))).not_null())
                .column(ColumnDefinition::new("description", PgDataType::Text).default_val("''"))
                .column(
                    ColumnDefinition::new("assigned_by", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("start_date", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("due_date", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .not_null()
                        .default_val("'Pending'"),
                )
                .column(
                    ColumnDefinition::new("completion_percentage", PgDataType::Integer)
                        .not_null()
                        .default_val("0"),
                )
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("updated_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .index(IndexDefinition::new(
                    "idx_checklists_emp_id",
                    &["employee_id"],
                ))
                .index(IndexDefinition::new("idx_checklists_status", &["status"])),
        );

        // 21. Onboarding Checklist Items table
        tables.push(
            TableDefinition::new("onboarding_checklist_items")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("checklist_id", PgDataType::Varchar(Some(50))).not_null(),
                )
                .column(ColumnDefinition::new("title", PgDataType::Varchar(Some(255))).not_null())
                .column(ColumnDefinition::new("description", PgDataType::Text).default_val("''"))
                .column(
                    ColumnDefinition::new("assigned_to", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("due_date", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .not_null()
                        .default_val("'Pending'"),
                )
                .column(
                    ColumnDefinition::new("completed_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("employee_id", PgDataType::Varchar(Some(50)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("employee_name", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(ColumnDefinition::new("archived", PgDataType::Boolean).default_val("FALSE"))
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("updated_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(ColumnDefinition::new("history", PgDataType::Text).default_val("'[]'"))
                .index(IndexDefinition::new(
                    "idx_checklist_items_chk_id",
                    &["checklist_id"],
                ))
                .index(IndexDefinition::new(
                    "idx_checklist_items_emp_id",
                    &["employee_id"],
                ))
                .index(IndexDefinition::new(
                    "idx_checklist_items_status",
                    &["status"],
                )),
        );

        // 22. Internships table
        tables.push(
            TableDefinition::new("internships")
                .column(ColumnDefinition::new("id", PgDataType::Varchar(Some(50))).primary_key())
                .column(
                    ColumnDefinition::new("intern_name", PgDataType::Varchar(Some(255))).not_null(),
                )
                .column(ColumnDefinition::new("email", PgDataType::Varchar(Some(255))).not_null())
                .column(
                    ColumnDefinition::new("phone", PgDataType::Varchar(Some(50))).default_val("''"),
                )
                .column(
                    ColumnDefinition::new("college", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("degree", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("department", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("internship_role", PgDataType::Varchar(Some(255)))
                        .not_null(),
                )
                .column(
                    ColumnDefinition::new("mentor_employee_id", PgDataType::Varchar(Some(50)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("mentor_name", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("start_date", PgDataType::Varchar(Some(100))).not_null(),
                )
                .column(
                    ColumnDefinition::new("end_date", PgDataType::Varchar(Some(100))).not_null(),
                )
                .column(
                    ColumnDefinition::new("stipend", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("status", PgDataType::Varchar(Some(50)))
                        .not_null()
                        .default_val("'Active'"),
                )
                .column(
                    ColumnDefinition::new("work_mode", PgDataType::Varchar(Some(50)))
                        .default_val("'Hybrid'"),
                )
                .column(
                    ColumnDefinition::new("project_title", PgDataType::Varchar(Some(255)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("performance_rating", PgDataType::Float)
                        .default_val("0.0"),
                )
                .column(ColumnDefinition::new("feedback", PgDataType::Text).default_val("''"))
                .column(
                    ColumnDefinition::new("created_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .column(
                    ColumnDefinition::new("updated_at", PgDataType::Varchar(Some(100)))
                        .default_val("''"),
                )
                .index(IndexDefinition::new("idx_internships_status", &["status"]))
                .index(IndexDefinition::new(
                    "idx_internships_mentor",
                    &["mentor_employee_id"],
                ))
                .index(IndexDefinition::new("idx_internships_email", &["email"])),
        );

        Self { tables }
    }

    /// Retrieve a table definition by name
    pub fn get_table(&self, name: &str) -> Option<&TableDefinition> {
        self.tables.iter().find(|t| t.name == name)
    }
}

use crate::database::repository::DatabaseRepository;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: usize,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: usize,
    pub title: String,
    pub department: String,
    pub location: String,
    pub skills: String,
    pub status: String, // "Active", "Closed", "Draft"
    pub posted_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candidate {
    pub id: usize,
    pub name: String,
    pub email: String,
    pub job_id: usize,
    pub job_title: String,
    pub skills: String,
    pub status: String, // "Applied", "Shortlisted", "Interview", "Selected", "Rejected"
    pub applied_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interview {
    pub id: usize,
    pub candidate_id: usize,
    pub candidate_name: String,
    pub candidate_email: String,
    pub job_title: String,
    pub interview_date: String,
    pub formatted_date: String,
    pub interviewer: String,
    pub interview_type: String, // "Technical Screening", "System Design", "HR Cultural Fit", "Final Executive Round"
    pub notes: String,
    pub status: String, // "Scheduled", "Completed", "Cancelled"
    #[serde(default)]
    pub round: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub venue_link: Option<String>,
    #[serde(default)]
    pub rating: Option<u32>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub feedback: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub history: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditHistoryItem {
    pub status: String,
    pub timestamp: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfferLetter {
    pub id: String,
    pub candidate_id: String,
    pub candidate_name: String,
    pub designation: String,
    #[serde(default)]
    pub department: Option<String>,
    pub annual_ctc: String,
    pub joining_date: String,
    #[serde(default)]
    pub offer_date: Option<String>,
    pub valid_till: String,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub offer_letter_file: Option<String>,
    pub status: String,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub history: Option<Vec<AuditHistoryItem>>,
}

pub fn default_offer_letters() -> Vec<OfferLetter> {
    vec![OfferLetter {
        id: "OF-0002".to_string(),
        candidate_id: "CAN-0001".to_string(),
        candidate_name: "rohith (CAN-0001)".to_string(),
        designation: "me".to_string(),
        department: Some("emg".to_string()),
        annual_ctc: "₹20,32,020".to_string(),
        joining_date: "Feb 20, 2020".to_string(),
        offer_date: Some("Feb 20, 2020".to_string()),
        valid_till: "Feb 20, 3020".to_string(),
        approved_by: Some("—".to_string()),
        offer_letter_file: Some("—".to_string()),
        status: "Pending approval".to_string(),
        archived: Some(false),
        created_at: Some("21 Oct 2026, 12:59 pm".to_string()),
        updated_at: Some("21 Oct 2026, 12:59 pm".to_string()),
        history: Some(vec![AuditHistoryItem {
            status: "Pending approval".to_string(),
            timestamp: "21 Oct 2026, 12:59 pm".to_string(),
            user: "ananya.rao@brightcone.ai".to_string(),
        }]),
    }]
}

// ===========================================================================
// JOINING FORMALITIES MODELS
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoiningRecord {
    pub id: String,
    #[serde(default)]
    pub candidate_id: Option<String>,
    #[serde(default)]
    pub offer_letter_id: Option<String>,
    #[serde(default)]
    pub employee_id: Option<String>,
    #[serde(default)]
    pub candidate_name: Option<String>,
    #[serde(default)]
    pub employee_name: Option<String>,
    pub joining_date: String,
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub designation: Option<String>,
    #[serde(default)]
    pub reporting_manager: Option<String>,
    #[serde(default)]
    pub documents_submitted: Option<String>,
    #[serde(default)]
    pub documents_verified: Option<bool>,
    #[serde(default)]
    pub verification_status: Option<String>,
    #[serde(default)]
    pub joining_status: Option<String>,
    #[serde(default)]
    pub remarks: Option<String>,
    // Specific checklist items
    #[serde(default)]
    pub documents_collected: Option<String>,
    #[serde(default)]
    pub id_card_issued: Option<String>,
    #[serde(default)]
    pub work_email_created: Option<String>,
    #[serde(default)]
    pub laptop_issued: Option<String>,
    #[serde(default)]
    pub pf_nomination_form: Option<String>,
    #[serde(default)]
    pub appointment_letter_issued: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub history: Option<Vec<AuditHistoryItem>>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateJoiningRecordPayload {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub candidate_id: Option<String>,
    #[serde(default)]
    pub offer_letter_id: Option<String>,
    #[serde(default)]
    pub employee_id: Option<String>,
    #[serde(default)]
    pub candidate_name: Option<String>,
    #[serde(default)]
    pub employee_name: Option<String>,
    pub joining_date: String,
    #[serde(default)]
    pub department: Option<String>,
    #[serde(default)]
    pub designation: Option<String>,
    #[serde(default)]
    pub reporting_manager: Option<String>,
    #[serde(default)]
    pub documents_submitted: Option<String>,
    #[serde(default)]
    pub documents_verified: Option<bool>,
    #[serde(default)]
    pub verification_status: Option<String>,
    #[serde(default)]
    pub joining_status: Option<String>,
    #[serde(default)]
    pub remarks: Option<String>,
    #[serde(default)]
    pub documents_collected: Option<String>,
    #[serde(default)]
    pub id_card_issued: Option<String>,
    #[serde(default)]
    pub work_email_created: Option<String>,
    #[serde(default)]
    pub laptop_issued: Option<String>,
    #[serde(default)]
    pub pf_nomination_form: Option<String>,
    #[serde(default)]
    pub appointment_letter_issued: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub history: Option<Vec<AuditHistoryItem>>,
    #[serde(default)]
    pub create_employee: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateJoiningStatusPayload {
    pub status: String,
    #[serde(default)]
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateJoiningDocumentsPayload {
    #[serde(default)]
    pub documents_submitted: Option<String>,
    #[serde(default)]
    pub documents_verified: Option<bool>,
    #[serde(default)]
    pub verification_status: Option<String>,
    #[serde(default)]
    pub documents_collected: Option<String>,
    #[serde(default)]
    pub id_card_issued: Option<String>,
    #[serde(default)]
    pub work_email_created: Option<String>,
    #[serde(default)]
    pub laptop_issued: Option<String>,
    #[serde(default)]
    pub pf_nomination_form: Option<String>,
    #[serde(default)]
    pub appointment_letter_issued: Option<String>,
    #[serde(default)]
    pub remarks: Option<String>,
}

// ===========================================================================
// ONBOARDING CHECKLIST MODELS
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingChecklistItem {
    pub id: String,
    pub checklist_id: String,
    #[serde(default)]
    pub employee_id: Option<String>,
    #[serde(default)]
    pub employee_name: Option<String>,
    pub title: String,
    #[serde(default)]
    pub task: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub assigned_to: Option<String>,
    #[serde(default)]
    pub owner: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    pub status: String, // "Pending", "In progress", "Completed"
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub completed_on: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub history: Option<Vec<AuditHistoryItem>>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl Default for OnboardingChecklistItem {
    fn default() -> Self {
        Self {
            id: String::new(),
            checklist_id: String::new(),
            employee_id: None,
            employee_name: None,
            title: String::new(),
            task: None,
            description: None,
            notes: None,
            assigned_to: None,
            owner: None,
            due_date: None,
            status: "Pending".to_string(),
            completed_at: None,
            completed_on: None,
            archived: Some(false),
            history: None,
            created_at: None,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOnboardingTaskPayload {
    #[serde(default)]
    pub id: Option<String>,
    pub employee_id: String,
    #[serde(default)]
    pub employee_name: Option<String>,
    #[serde(alias = "title")]
    pub task: String,
    #[serde(default, alias = "assigned_to")]
    pub owner: Option<String>,
    pub due_date: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, alias = "completed_at")]
    pub completed_on: Option<String>,
    #[serde(default, alias = "description")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateStandardChecklistPayload {
    pub employee_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingChecklist {
    pub id: String,
    pub employee_id: String,
    #[serde(default)]
    pub employee_name: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub assigned_by: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    pub status: String, // "Pending", "In Progress", "Completed"
    pub completion_percentage: u32,
    #[serde(default)]
    pub items: Vec<OnboardingChecklistItem>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateChecklistPayload {
    pub employee_id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub assigned_by: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub items: Option<Vec<CreateChecklistItemPayload>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateChecklistItemPayload {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub assigned_to: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateChecklistItemPayload {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub assigned_to: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateItemStatusPayload {
    pub status: String,
}

// ===========================================================================
// INTERNSHIP MANAGEMENT MODELS
// ===========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Internship {
    pub id: String,
    pub intern_name: String,
    pub email: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub college: Option<String>,
    #[serde(default)]
    pub degree: Option<String>,
    #[serde(default)]
    pub department: Option<String>,
    pub internship_role: String,
    #[serde(default)]
    pub mentor_employee_id: Option<String>,
    #[serde(default)]
    pub mentor_name: Option<String>,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub stipend: Option<String>,
    pub status: String, // "Active", "Completed", "PPO Offered", "Terminated"
    #[serde(default)]
    pub work_mode: Option<String>, // "In-office", "Hybrid", "Remote"
    #[serde(default)]
    pub project_title: Option<String>,
    #[serde(default)]
    pub performance_rating: Option<f32>,
    #[serde(default)]
    pub feedback: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateInternshipPayload {
    pub intern_name: String,
    pub email: String,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub college: Option<String>,
    #[serde(default)]
    pub degree: Option<String>,
    #[serde(default)]
    pub department: Option<String>,
    pub internship_role: String,
    #[serde(default)]
    pub mentor_employee_id: Option<String>,
    #[serde(default)]
    pub mentor_name: Option<String>,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub stipend: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub work_mode: Option<String>,
    #[serde(default)]
    pub project_title: Option<String>,
    #[serde(default)]
    pub performance_rating: Option<f32>,
    #[serde(default)]
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInternshipStatusPayload {
    pub status: String,
    #[serde(default)]
    pub feedback: Option<String>,
    #[serde(default)]
    pub performance_rating: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeDependent {
    pub id: String,
    pub employee_id: String,
    pub name: String,
    pub relationship: String,
    pub dob: String,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDependentPayload {
    #[serde(default)]
    pub employee_id: Option<String>,
    pub name: String,
    pub relationship: String,
    pub dob: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub grade: String,
    pub location: String,
    pub manager: String,
    pub status: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: usize,
    pub employee_id: String,
    pub employee_name: String,
    pub title: String,
    pub category: String,
    pub target_date: String,
    pub progress: u32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReview {
    pub id: usize,
    pub employee_id: String,
    pub employee_name: String,
    pub reviewer_name: String,
    pub cycle: String,
    pub rating: f32,
    pub feedback: String,
    pub status: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeProfile {
    pub employee_id: String,
    pub full_name: String,
    pub work_email: String,
    pub role: String,
    pub department: String,
    pub status: String,
    pub phone: String,
    pub dob: String,
    pub blood_group: String,
    pub address: String,
    pub emergency_contact: String,
    pub manager: String,
    pub grade: String,
    pub location: String,
    pub doj: String,
    pub bank_name: String,
    pub bank_account: String,
    pub ifsc: String,
    pub pan: String,
    pub uan: String,
    pub nominee_name: String,
    pub nominee_relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManpowerRequirement {
    pub id: String,
    pub designation: String,
    pub department: String,
    pub positions: u32,
    pub hiring_manager: String,
    pub required_by: String,
    pub status: String,
    #[serde(default)]
    pub employment_type: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub experience: Option<String>,
    #[serde(default)]
    pub budget_ctc: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub replacing: Option<String>,
    #[serde(default)]
    pub job_description: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTracker {
    pub id: String,
    pub full_name: String,
    pub applied_for: String,
    pub source: String,
    pub total_experience: u32,
    #[serde(default)]
    pub current_ctc: Option<u64>,
    pub expected_ctc: u64,
    #[serde(default)]
    pub notice_period: Option<u32>,
    #[serde(default)]
    pub current_location: Option<String>,
    #[serde(default)]
    pub resume: Option<String>,
    #[serde(default)]
    pub consent_to_store_data: Option<String>,
    pub stage: String,
    pub email: String,
    pub phone: String,
    #[serde(default)]
    pub employee_id: Option<String>,
    #[serde(default)]
    pub archived: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeReferral {
    pub id: String,
    pub candidate_name: String,
    pub role: String,
    pub referred_by: String,
    pub bonus: u64,
    pub status: String,
    pub date: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeLeave {
    pub id: String,
    pub employee_id: String,
    pub leave_type: String,
    pub from_date: String,
    pub to_date: String,
    pub days: u32,
    pub reason: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeAttendance {
    pub id: String,
    pub employee_id: String,
    pub date: String,
    pub punch_in: String,
    pub punch_out: Option<String>,
    pub work_hours: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    pub id: String,
    pub employee_id: String,
    pub employee_name: String,
    pub category: String,
    pub subject: String,
    #[serde(default = "default_service_priority")]
    pub priority: String,
    #[serde(default)]
    pub details: String,
    #[serde(default = "default_service_status")]
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

fn default_service_priority() -> String {
    "Normal".to_string()
}

fn default_service_status() -> String {
    "Submitted".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateServiceRequestPayload {
    #[serde(default)]
    pub employee_id: Option<String>,
    #[serde(default)]
    pub employee_name: Option<String>,
    pub category: String,
    pub subject: String,
    #[serde(default = "default_service_priority")]
    pub priority: String,
    #[serde(default)]
    pub details: String,
}

pub fn default_profiles() -> Vec<EmployeeProfile> {
    vec![
        EmployeeProfile {
            employee_id: "BC-1038".to_string(),
            full_name: "Rahul Varma".to_string(),
            work_email: "rahul.varma@brightcone.ai".to_string(),
            role: "Backend Engineer".to_string(),
            department: "Engineering".to_string(),
            status: "Active".to_string(),
            phone: "+91 98480 11038".to_string(),
            dob: "Oct 7, 1996".to_string(),
            blood_group: "B+".to_string(),
            address: "Flat 402, Aparna Towers, Kondapur, Hyderabad 500084".to_string(),
            emergency_contact: "Lakshmi Varma (Mother), +91 98490 22110".to_string(),
            manager: "Priya Menon".to_string(),
            grade: "L4".to_string(),
            location: "Hyderabad".to_string(),
            doj: "Feb 15, 2024".to_string(),
            bank_name: "HDFC Bank".to_string(),
            bank_account: "50100492817291".to_string(),
            ifsc: "HDFC0001234".to_string(),
            pan: "ABCDE1234F".to_string(),
            uan: "100984920194".to_string(),
            nominee_name: "Lakshmi Varma".to_string(),
            nominee_relation: "Mother".to_string(),
        },
        EmployeeProfile {
            employee_id: "BC-1004".to_string(),
            full_name: "Ananya Rao".to_string(),
            work_email: "ananya.rao@brightcone.ai".to_string(),
            role: "People Ops Lead".to_string(),
            department: "People Ops".to_string(),
            status: "Active".to_string(),
            phone: "+91 98765 43210".to_string(),
            dob: "May 12, 1993".to_string(),
            blood_group: "O+".to_string(),
            address: "Plot 88, Jubilee Hills, Hyderabad 500033".to_string(),
            emergency_contact: "Vikram Rao (Spouse), +91 98765 01234".to_string(),
            manager: "Naresh Vemparala".to_string(),
            grade: "L6".to_string(),
            location: "Hyderabad".to_string(),
            doj: "Jan 10, 2023".to_string(),
            bank_name: "ICICI Bank".to_string(),
            bank_account: "001201928374".to_string(),
            ifsc: "ICIC0000012".to_string(),
            pan: "XYZPA9876Q".to_string(),
            uan: "100482910283".to_string(),
            nominee_name: "Vikram Rao".to_string(),
            nominee_relation: "Spouse".to_string(),
        },
    ]
}

pub fn default_manpower() -> Vec<ManpowerRequirement> {
    vec![
        ManpowerRequirement {
            id: "MR-0001".to_string(),
            designation: "asa".to_string(),
            department: "ss".to_string(),
            positions: 2,
            hiring_manager: "—".to_string(),
            required_by: "Feb 20, 2020".to_string(),
            status: "Overdue".to_string(),
            employment_type: Some("Full-time".to_string()),
            location: Some("sed".to_string()),
            experience: Some("2".to_string()),
            budget_ctc: Some("120000".to_string()),
            reason: Some("New position".to_string()),
            replacing: Some("—".to_string()),
            job_description: Some("sswefg".to_string()),
            created_at: Some("15 Oct 2026, 4:17 pm".to_string()),
            updated_at: Some("19 Oct 2026, 12:21 pm".to_string()),
            archived: Some(false),
        },
        ManpowerRequirement {
            id: "MR-0002".to_string(),
            designation: "Senior ML Engineer".to_string(),
            department: "AI Research".to_string(),
            positions: 3,
            hiring_manager: "Karthik Reddy".to_string(),
            required_by: "Nov 30, 2026".to_string(),
            status: "Approved".to_string(),
            employment_type: Some("Full-time".to_string()),
            location: Some("Hyderabad".to_string()),
            experience: Some("3-5".to_string()),
            budget_ctc: Some("3500000".to_string()),
            reason: Some("Expansion".to_string()),
            replacing: Some("—".to_string()),
            job_description: Some("Core ML platform development in Rust & Python".to_string()),
            created_at: Some("16 Oct 2026, 10:00 am".to_string()),
            updated_at: Some("19 Oct 2026, 11:30 am".to_string()),
            archived: Some(false),
        },
        ManpowerRequirement {
            id: "MR-0003".to_string(),
            designation: "Solutions Architect".to_string(),
            department: "Sales".to_string(),
            positions: 1,
            hiring_manager: "Daniel Brooks".to_string(),
            required_by: "Dec 15, 2026".to_string(),
            status: "Pending".to_string(),
            employment_type: Some("Full-time".to_string()),
            location: Some("San Francisco".to_string()),
            experience: Some("5-8".to_string()),
            budget_ctc: Some("4200000".to_string()),
            reason: Some("Client Success".to_string()),
            replacing: Some("—".to_string()),
            job_description: Some("Enterprise architectural design".to_string()),
            created_at: Some("17 Oct 2026, 02:00 pm".to_string()),
            updated_at: Some("19 Oct 2026, 01:00 pm".to_string()),
            archived: Some(false),
        },
    ]
}

pub fn default_candidate_trackers() -> Vec<CandidateTracker> {
    vec![
        CandidateTracker {
            id: "CAN-0001".to_string(),
            full_name: "Siddharth Rao".to_string(),
            applied_for: "QA Automation Engineer".to_string(),
            source: "Referral".to_string(),
            total_experience: 4,
            current_ctc: Some(1800000),
            expected_ctc: 2400000,
            notice_period: Some(15),
            current_location: Some("Hyderabad".to_string()),
            resume: Some("—".to_string()),
            consent_to_store_data: Some("Yes".to_string()),
            stage: "Interview".to_string(),
            email: "siddharth.rao@example.com".to_string(),
            phone: "+91 99887 66554".to_string(),
            employee_id: Some("—".to_string()),
            archived: Some(false),
            created_at: Some("15 Oct 2026, 4:41 pm".to_string()),
            updated_at: Some("19 Oct 2026, 3:51 pm".to_string()),
        },
        CandidateTracker {
            id: "CAN-0002".to_string(),
            full_name: "wffs".to_string(),
            applied_for: "Senior ML Engineer".to_string(),
            source: "Referral".to_string(),
            total_experience: 5,
            current_ctc: Some(3500000),
            expected_ctc: 4500000,
            notice_period: Some(30),
            current_location: Some("Bengaluru".to_string()),
            resume: Some("—".to_string()),
            consent_to_store_data: Some("Yes".to_string()),
            stage: "Applied".to_string(),
            email: "wffs@example.com".to_string(),
            phone: "+91 98112 33445".to_string(),
            employee_id: Some("—".to_string()),
            archived: Some(false),
            created_at: Some("16 Oct 2026, 11:20 am".to_string()),
            updated_at: Some("19 Oct 2026, 12:43 pm".to_string()),
        },
        CandidateTracker {
            id: "CAN-0003".to_string(),
            full_name: "Neha Bansal".to_string(),
            applied_for: "Product Designer".to_string(),
            source: "LinkedIn".to_string(),
            total_experience: 6,
            current_ctc: Some(2500000),
            expected_ctc: 3200000,
            notice_period: Some(15),
            current_location: Some("Mumbai".to_string()),
            resume: Some("—".to_string()),
            consent_to_store_data: Some("Yes".to_string()),
            stage: "Interview".to_string(),
            email: "neha.bansal@example.com".to_string(),
            phone: "+91 97654 32109".to_string(),
            employee_id: Some("—".to_string()),
            archived: Some(false),
            created_at: Some("17 Oct 2026, 02:15 pm".to_string()),
            updated_at: Some("19 Oct 2026, 3:49 pm".to_string()),
        },
    ]
}

pub fn default_referrals() -> Vec<EmployeeReferral> {
    vec![
        EmployeeReferral {
            id: "RF-001".to_string(),
            candidate_name: "wffs".to_string(),
            role: "Senior ML Engineer".to_string(),
            referred_by: "rahul.varma@brightcone.ai".to_string(),
            bonus: 50000,
            status: "Applied".to_string(),
            date: "Oct 15, 2026".to_string(),
            email: Some("wffs@example.com".to_string()),
            phone: Some("+91 98765 43210".to_string()),
            notes: Some("Excellent expertise in high performance systems".to_string()),
        },
        EmployeeReferral {
            id: "RF-002".to_string(),
            candidate_name: "Siddharth Rao".to_string(),
            role: "QA Automation Engineer".to_string(),
            referred_by: "rahul.varma@brightcone.ai".to_string(),
            bonus: 30000,
            status: "Interview".to_string(),
            date: "Oct 12, 2026".to_string(),
            email: Some("siddharth.rao@example.com".to_string()),
            phone: Some("+91 98450 12345".to_string()),
            notes: Some("Strong background in Playwright and Rust test harnesses".to_string()),
        },
    ]
}

pub fn default_leaves() -> Vec<EmployeeLeave> {
    vec![
        EmployeeLeave {
            id: "LV-101".to_string(),
            employee_id: "BC-1038".to_string(),
            leave_type: "Casual leave".to_string(),
            from_date: "2026-10-09".to_string(),
            to_date: "2026-10-09".to_string(),
            days: 1,
            reason: "Personal family event".to_string(),
            status: "Approved".to_string(),
        },
        EmployeeLeave {
            id: "LV-102".to_string(),
            employee_id: "BC-1038".to_string(),
            leave_type: "Sick leave".to_string(),
            from_date: "2026-09-14".to_string(),
            to_date: "2026-09-15".to_string(),
            days: 2,
            reason: "Viral fever recovery".to_string(),
            status: "Approved".to_string(),
        },
    ]
}

pub fn default_attendance() -> Vec<EmployeeAttendance> {
    vec![
        EmployeeAttendance {
            id: "ATT-101".to_string(),
            employee_id: "BC-1038".to_string(),
            date: "2026-10-15".to_string(),
            punch_in: "09:15 AM".to_string(),
            punch_out: Some("06:30 PM".to_string()),
            work_hours: "8h 45m".to_string(),
            status: "Present".to_string(),
        },
        EmployeeAttendance {
            id: "ATT-102".to_string(),
            employee_id: "BC-1038".to_string(),
            date: "2026-10-16".to_string(),
            punch_in: "09:05 AM".to_string(),
            punch_out: None,
            work_hours: "In progress".to_string(),
            status: "Present".to_string(),
        },
    ]
}

pub fn default_employees() -> Vec<Employee> {
    vec![
        Employee {
            id: "BC-1001".to_string(),
            name: "Naresh Vemparala".to_string(),
            role: "Founder & CEO".to_string(),
            department: "Leadership".to_string(),
            grade: "L9".to_string(),
            location: "Hyderabad".to_string(),
            manager: "no manager".to_string(),
            status: "Active".to_string(),
            email: "naresh@brightcone.ai".to_string(),
        },
        Employee {
            id: "BC-1004".to_string(),
            name: "Ananya Rao".to_string(),
            role: "People Ops Lead".to_string(),
            department: "People Ops".to_string(),
            grade: "L6".to_string(),
            location: "Hyderabad".to_string(),
            manager: "Naresh Vemparala".to_string(),
            status: "Active".to_string(),
            email: "ananya.rao@brightcone.ai".to_string(),
        },
        Employee {
            id: "BC-1012".to_string(),
            name: "Karthik Reddy".to_string(),
            role: "Principal AI Engineer".to_string(),
            department: "AI Research".to_string(),
            grade: "L8".to_string(),
            location: "Hyderabad".to_string(),
            manager: "Naresh Vemparala".to_string(),
            status: "Active".to_string(),
            email: "karthik.reddy@brightcone.ai".to_string(),
        },
        Employee {
            id: "BC-1019".to_string(),
            name: "Priya Menon".to_string(),
            role: "Engineering Manager".to_string(),
            department: "Engineering".to_string(),
            grade: "L7".to_string(),
            location: "Bengaluru".to_string(),
            manager: "Karthik Reddy".to_string(),
            status: "Active".to_string(),
            email: "priya.menon@brightcone.ai".to_string(),
        },
        Employee {
            id: "BC-1023".to_string(),
            name: "Daniel Brooks".to_string(),
            role: "Account Executive".to_string(),
            department: "Sales".to_string(),
            grade: "L6".to_string(),
            location: "San Francisco".to_string(),
            manager: "Naresh Vemparala".to_string(),
            status: "Active".to_string(),
            email: "daniel.brooks@brightcone.ai".to_string(),
        },
        Employee {
            id: "BC-1031".to_string(),
            name: "Sneha Iyer".to_string(),
            role: "Product Designer".to_string(),
            department: "Design".to_string(),
            grade: "L5".to_string(),
            location: "Hyderabad".to_string(),
            manager: "Priya Menon".to_string(),
            status: "On leave".to_string(),
            email: "sneha.iyer@brightcone.ai".to_string(),
        },
        Employee {
            id: "BC-1038".to_string(),
            name: "Rahul Varma".to_string(),
            role: "Backend Engineer".to_string(),
            department: "Engineering".to_string(),
            grade: "L4".to_string(),
            location: "Hyderabad".to_string(),
            manager: "Priya Menon".to_string(),
            status: "Active".to_string(),
            email: "rahul.varma@brightcone.ai".to_string(),
        },
        Employee {
            id: "BC-1042".to_string(),
            name: "Meera Joshi".to_string(),
            role: "Finance Manager".to_string(),
            department: "Finance".to_string(),
            grade: "L6".to_string(),
            location: "Hyderabad".to_string(),
            manager: "Naresh Vemparala".to_string(),
            status: "Active".to_string(),
            email: "meera.joshi@brightcone.ai".to_string(),
        },
    ]
}

pub fn default_goals() -> Vec<Goal> {
    vec![
        Goal {
            id: 1,
            employee_id: "BC-1012".to_string(),
            employee_name: "Karthik Reddy".to_string(),
            title: "Deploy Distributed Vector LLM Serving Engine in Rust".to_string(),
            category: "Innovation".to_string(),
            target_date: "2026-11-15".to_string(),
            progress: 85,
            status: "In Progress".to_string(),
        },
        Goal {
            id: 2,
            employee_id: "BC-1038".to_string(),
            employee_name: "Rahul Varma".to_string(),
            title: "Migrate Core Database Schema to PostgreSQL 18 & Optimize Connection Pool"
                .to_string(),
            category: "Engineering".to_string(),
            target_date: "2026-10-31".to_string(),
            progress: 90,
            status: "In Progress".to_string(),
        },
        Goal {
            id: 3,
            employee_id: "BC-1019".to_string(),
            employee_name: "Priya Menon".to_string(),
            title: "Establish Engineering Mentorship Framework for L4-L6 Engineers".to_string(),
            category: "Leadership".to_string(),
            target_date: "2026-12-15".to_string(),
            progress: 60,
            status: "In Progress".to_string(),
        },
        Goal {
            id: 4,
            employee_id: "BC-1031".to_string(),
            employee_name: "Sneha Iyer".to_string(),
            title: "Design System Unification for Brightcone People Web & Mobile UI".to_string(),
            category: "Product".to_string(),
            target_date: "2026-11-30".to_string(),
            progress: 75,
            status: "In Progress".to_string(),
        },
    ]
}

pub fn default_reviews() -> Vec<PerformanceReview> {
    vec![
        PerformanceReview {
            id: 1,
            employee_id: "BC-1038".to_string(),
            employee_name: "Rahul Varma".to_string(),
            reviewer_name: "Priya Menon (Engineering Manager)".to_string(),
            cycle: "Q3 2026 Mid-Year".to_string(),
            rating: 4.8,
            feedback: "Rahul has demonstrated extraordinary ownership of backend Rust services, reducing latency by 45% and delivering candidate pipeline synchronization.".to_string(),
            status: "Completed".to_string(),
            updated_at: "Oct 10, 2026".to_string(),
        },
        PerformanceReview {
            id: 2,
            employee_id: "BC-1012".to_string(),
            employee_name: "Karthik Reddy".to_string(),
            reviewer_name: "Naresh Vemparala (Founder & CEO)".to_string(),
            cycle: "Q3 2026 Mid-Year".to_string(),
            rating: 4.9,
            feedback: "Pioneering technical visionary. Karthik led the architecture of the AI vector matching engine while upholding top-tier code maintainability.".to_string(),
            status: "Completed".to_string(),
            updated_at: "Oct 08, 2026".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_jobs: usize,
    pub total_candidates: usize,
    pub shortlisted_candidates: usize,
    pub scheduled_interviews: usize,
    pub selected_candidates: usize,
    #[serde(default)]
    pub total_employees: usize,
    #[serde(default)]
    pub active_employees: usize,
    #[serde(default)]
    pub employees_on_leave: usize,
    #[serde(default)]
    pub goal_completion_rate: usize,
    #[serde(default)]
    pub pending_reviews: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransientState {
    pub wakeup_count: usize,
    pub last_wakeup_time: String,
    pub last_db_latency_ms: u64,
}

impl Default for TransientState {
    fn default() -> Self {
        Self {
            wakeup_count: 0,
            last_wakeup_time: "Never".to_string(),
            last_db_latency_ms: 0,
        }
    }
}

/// Thread-safe, database-backed application state for Brightcone HR Portal.
/// PostgreSQL is the single source of truth for all business entities.
#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<DatabaseRepository>,
    pub transient: Arc<RwLock<TransientState>>,
}

pub type SharedState = Arc<AppState>;

impl AppState {
    pub fn new(repo: DatabaseRepository) -> Self {
        Self {
            repo: Arc::new(repo),
            transient: Arc::new(RwLock::new(TransientState::default())),
        }
    }

    pub fn record_wakeup_ping(&self) {
        if let Ok(mut t) = self.transient.write() {
            t.wakeup_count += 1;
            t.last_wakeup_time = "Just now".to_string();
        }
    }

    pub fn get_wakeup_info(&self) -> serde_json::Value {
        let (count, time, lat) = if let Ok(t) = self.transient.read() {
            (
                t.wakeup_count,
                t.last_wakeup_time.clone(),
                t.last_db_latency_ms,
            )
        } else {
            (0, "Unknown".to_string(), 0)
        };
        serde_json::json!({
            "wakeup_count": count,
            "last_wakeup_time": time,
            "database_latency_ms": lat,
            "status": "active"
        })
    }
}

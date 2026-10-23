use serde::{Deserialize, Serialize};

// =========================================================================
// Shift Models
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Shift {
    pub id: i64,
    pub pid: String,
    pub name: String,
    pub code: String,
    pub start_time: String,
    pub end_time: String,
    pub break_duration_minutes: i32,
    pub grace_period_minutes: i32,
    pub is_overnight: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateShiftPayload {
    pub name: String,
    pub code: String,
    pub start_time: String,
    pub end_time: String,
    pub break_duration_minutes: Option<i32>,
    pub grace_period_minutes: Option<i32>,
    pub is_overnight: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateShiftPayload {
    pub name: Option<String>,
    pub code: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub break_duration_minutes: Option<i32>,
    pub grace_period_minutes: Option<i32>,
    pub is_overnight: Option<bool>,
    pub is_active: Option<bool>,
}

// =========================================================================
// Holiday Models
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Holiday {
    pub id: i64,
    pub pid: String,
    pub name: String,
    pub holiday_date: String,
    pub description: Option<String>,
    pub applicable_location: Option<String>,
    pub is_recurring: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateHolidayPayload {
    pub name: String,
    pub holiday_date: String,
    pub description: Option<String>,
    pub applicable_location: Option<String>,
    pub is_recurring: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateHolidayPayload {
    pub name: Option<String>,
    pub holiday_date: Option<String>,
    pub description: Option<String>,
    pub applicable_location: Option<String>,
    pub is_recurring: Option<bool>,
}

// =========================================================================
// Leave Type Models
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaveType {
    pub id: i64,
    pub pid: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_paid: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaveTypePayload {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_paid: Option<bool>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeaveTypePayload {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub is_paid: Option<bool>,
    pub is_active: Option<bool>,
}

// =========================================================================
// Leave Quota Models
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct LeaveQuota {
    pub id: i64,
    pub pid: String,
    pub leave_type_id: i64,
    pub leave_type_name: Option<String>,
    pub leave_type_code: Option<String>,
    pub year: i32,
    pub entitled_days: f64,
    pub applicable_group: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeaveQuotaPayload {
    pub leave_type_id: i64,
    pub year: i32,
    pub entitled_days: f64,
    pub applicable_group: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeaveQuotaPayload {
    pub leave_type_id: Option<i64>,
    pub year: Option<i32>,
    pub entitled_days: Option<f64>,
    pub applicable_group: Option<String>,
}

// =========================================================================
// Common Error Response
// =========================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// =========================================================================
// Employees & Weekly Roster Models
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Employee {
    pub id: i64,
    pub pid: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub department: String,
    pub default_shift_id: Option<i64>,
    pub default_shift_name: Option<String>,
    pub default_shift_code: Option<String>,
    pub avatar_color: String,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RosterAssignment {
    pub id: i64,
    pub pid: String,
    pub employee_id: i64,
    pub roster_date: String,
    pub shift_id: Option<i64>,
    pub shift_code: Option<String>,
    pub shift_name: Option<String>,
    pub is_off: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct WeeklyRosterQuery {
    pub week_start: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignShiftPayload {
    pub employee_id: i64,
    pub date: String,
    pub shift_id: Option<i64>,
    pub is_off: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PublishRosterPayload {
    pub week_start_date: String,
    pub published_by: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayRosterEntry {
    pub date: String,
    pub day_name: String,
    pub day_number: u32,
    pub shift_id: Option<i64>,
    pub shift_code: Option<String>,
    pub shift_name: Option<String>,
    pub is_off: bool,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeRosterRow {
    pub id: i64,
    pub pid: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub department: String,
    pub default_shift_id: Option<i64>,
    pub default_shift_name: Option<String>,
    pub default_shift_code: Option<String>,
    pub avatar_color: String,
    pub days: Vec<DayRosterEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterSummary {
    pub scheduled_today: i64,
    pub total_employees: i64,
    pub shifts_defined: i64,
    pub shift_codes: Vec<String>,
    pub swap_requests_count: i64,
    pub on_leave_this_week_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyRosterResponse {
    pub week_start: String,
    pub week_end: String,
    pub is_published: bool,
    pub published_at: Option<String>,
    pub published_by: Option<String>,
    pub summary: RosterSummary,
    pub employees: Vec<EmployeeRosterRow>,
}

// =========================================================================
// Daily Attendance Models
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyAttendanceRecord {
    pub id: i64,
    pub pid: String,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub employee_email: Option<String>,
    pub employee_department: Option<String>,
    pub avatar_color: Option<String>,
    pub attendance_date: String,
    pub attendance_status: String,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub hours_worked: f64,
    pub entry_status: String,
    pub is_archived: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceSummary {
    pub total_records: i64,
    pub recorded_count: i64,
    pub pending_count: i64,
    pub regularised_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct AttendanceQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub entry_status: Option<String>,
    pub archived: Option<bool>,
    pub date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAttendancePayload {
    pub employee_id: i64,
    pub date: String,
    pub attendance_status: String,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub entry_status: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAttendancePayload {
    pub attendance_status: Option<String>,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub entry_status: Option<String>,
    pub is_archived: Option<bool>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceListResponse {
    pub summary: AttendanceSummary,
    pub records: Vec<DailyAttendanceRecord>,
    pub archived_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ClockInPayload {
    pub employee_id: i64,
    pub date: Option<String>,
    pub time: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClockOutPayload {
    pub employee_id: i64,
    pub date: Option<String>,
    pub time: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TodayAttendanceQuery {
    pub employee_id: i64,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayAttendanceStatus {
    pub date: String,
    pub employee_id: i64,
    pub record: Option<DailyAttendanceRecord>,
    pub can_clock_in: bool,
    pub can_clock_out: bool,
    pub status: String,
}

// =========================================================================
// Attendance Correction Request Models
// =========================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttendanceCorrectionRequest {
    pub id: i64,
    pub pid: String,
    pub employee_id: i64,
    pub employee_name: Option<String>,
    pub employee_email: Option<String>,
    pub employee_department: Option<String>,
    pub avatar_color: Option<String>,
    pub attendance_date: String,
    pub correction_type: String,
    pub requested_time: String,
    pub reason: String,
    pub status: String,
    pub previous_in_time: Option<String>,
    pub previous_out_time: Option<String>,
    pub previous_hours_worked: Option<f64>,
    pub previous_status: Option<String>,
    pub reviewer_name: Option<String>,
    pub rejection_reason: Option<String>,
    pub reviewed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateCorrectionRequestPayload {
    pub employee_id: i64,
    pub attendance_date: String,
    pub correction_type: String,
    pub requested_time: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ApproveCorrectionPayload {
    pub reviewer_name: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RejectCorrectionPayload {
    pub reviewer_name: Option<String>,
    pub rejection_reason: String,
}

#[derive(Debug, Deserialize)]
pub struct CorrectionQuery {
    pub employee_id: Option<i64>,
    pub status: Option<String>,
    pub date: Option<String>,
}


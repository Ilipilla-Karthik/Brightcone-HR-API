export interface Shift {
  id: number;
  pid: string;
  name: string;
  code: string;
  start_time: string;
  end_time: string;
  break_duration_minutes: number;
  grace_period_minutes: number;
  is_overnight: boolean;
  is_active: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface Holiday {
  id: number;
  pid: string;
  name: string;
  holiday_date: string;
  description?: string | null;
  applicable_location?: string | null;
  is_recurring: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface LeaveType {
  id: number;
  pid: string;
  name: string;
  code: string;
  description?: string | null;
  is_paid: boolean;
  is_active: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface LeaveQuota {
  id: number;
  pid: string;
  leave_type_id: number;
  leave_type_name?: string;
  leave_type_code?: string;
  year: number;
  entitled_days: number;
  applicable_group: string;
  created_at?: string;
  updated_at?: string;
}

export interface Employee {
  id: number;
  pid: string;
  name: string;
  email: string;
  role: string;
  department: string;
  default_shift_id?: number | null;
  default_shift_name?: string | null;
  default_shift_code?: string | null;
  avatar_color: string;
  is_active: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface DayRosterEntry {
  date: string;
  day_name: string;
  day_number: number;
  shift_id: number | null;
  shift_code: string | null;
  shift_name: string | null;
  is_off: boolean;
  notes?: string | null;
}

export interface EmployeeRosterRow {
  id: number;
  pid: string;
  name: string;
  email: string;
  role: string;
  department: string;
  default_shift_id?: number | null;
  default_shift_name?: string | null;
  default_shift_code?: string | null;
  avatar_color: string;
  days: DayRosterEntry[];
}

export interface RosterSummary {
  scheduled_today: number;
  total_employees: number;
  shifts_defined: number;
  shift_codes: string[];
  swap_requests_count: number;
  on_leave_this_week_count: number;
}

export interface WeeklyRosterResponse {
  week_start: string;
  week_end: string;
  is_published: boolean;
  published_at?: string | null;
  published_by?: string | null;
  summary: RosterSummary;
  employees: EmployeeRosterRow[];
}

export interface AssignShiftPayload {
  employee_id: number;
  date: string;
  shift_id?: number | null;
  is_off?: boolean;
  notes?: string;
}

export interface PublishRosterPayload {
  week_start_date: string;
  published_by?: string;
  notes?: string;
}

export type ActiveTab = 'weekly-roster' | 'daily-attendance' | 'shifts' | 'holidays' | 'leave-types' | 'leave-quotas' | 'leave-calculator';

export interface DailyAttendanceRecord {
  id: number;
  pid: string;
  employee_id: number;
  employee_name: string;
  employee_email: string;
  employee_department: string;
  employee_avatar_color: string;
  attendance_date: string;
  attendance_status: 'PRESENT' | 'ABSENT' | 'HALF_DAY' | 'ON_LEAVE' | string;
  in_time: string | null;
  out_time: string | null;
  hours_worked: number;
  entry_status: 'RECORDED' | 'PENDING' | 'REGULARISED' | string;
  is_archived: boolean;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface AttendanceSummary {
  total_records: number;
  recorded_count: number;
  pending_count: number;
  regularised_count: number;
}

export interface AttendanceListResponse {
  summary: AttendanceSummary;
  records: DailyAttendanceRecord[];
  archived_count: number;
}

export interface CreateAttendancePayload {
  employee_id: number;
  date: string;
  attendance_status: string;
  in_time?: string | null;
  out_time?: string | null;
  entry_status?: string | null;
  notes?: string | null;
}

export interface UpdateAttendancePayload {
  attendance_status?: string;
  in_time?: string | null;
  out_time?: string | null;
  entry_status?: string;
  is_archived?: boolean;
  notes?: string | null;
}

export interface ClockInPayload {
  employee_id: number;
  date?: string;
  time?: string;
  notes?: string;
}

export interface ClockOutPayload {
  employee_id: number;
  date?: string;
  time?: string;
  notes?: string;
}

export interface TodayAttendanceStatus {
  date: string;
  employee_id: number;
  record: DailyAttendanceRecord | null;
  can_clock_in: boolean;
  can_clock_out: boolean;
  status: 'NOT_CLOCKED_IN' | 'CLOCKED_IN' | 'CLOCKED_OUT' | string;
}


export interface DateDayDetail {
  date: string;
  day_name: string;
  is_weekend: boolean;
  is_scheduled_off: boolean;
  is_holiday: boolean;
  holiday_name: string | null;
  is_chargeable: boolean;
  chargeable_value: number;
}

export interface LeaveCalculationResult {
  start_date: string;
  end_date: string;
  total_calendar_days: number;
  chargeable_days: number;
  weekend_days: number;
  holiday_days: number;
  working_days: number;
  policy_applied: string;
  leave_type_code: string | null;
  breakdown: DateDayDetail[];
}

export interface CountLeaveDaysRequest {
  start_date: string;
  end_date: string;
  employee_id?: number;
  leave_type_id?: number;
  leave_type_code?: string;
  is_half_day?: boolean;
  count_calendar_days?: boolean;
  exclude_weekends?: boolean;
  exclude_holidays?: boolean;
  applicable_location?: string;
}

export type CorrectionType = 'MISSED_CLOCK_IN' | 'MISSED_CLOCK_OUT';
export type CorrectionStatus = 'PENDING' | 'APPROVED' | 'REJECTED';

export interface AttendanceCorrectionRequest {
  id: number;
  pid: string;
  employee_id: number;
  employee_name?: string | null;
  employee_email?: string | null;
  employee_department?: string | null;
  avatar_color?: string | null;
  attendance_date: string;
  correction_type: CorrectionType | string;
  requested_time: string;
  reason: string;
  status: CorrectionStatus | string;
  previous_in_time?: string | null;
  previous_out_time?: string | null;
  previous_hours_worked?: number | null;
  previous_status?: string | null;
  reviewer_name?: string | null;
  rejection_reason?: string | null;
  reviewed_at?: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateCorrectionPayload {
  employee_id: number;
  attendance_date: string;
  correction_type: CorrectionType | string;
  requested_time: string;
  reason: string;
}

export interface ApproveCorrectionPayload {
  reviewer_name?: string;
  notes?: string;
}

export interface RejectCorrectionPayload {
  reviewer_name?: string;
  rejection_reason: string;
}

export interface CorrectionQuery {
  employee_id?: number;
  status?: string;
  date?: string;
}


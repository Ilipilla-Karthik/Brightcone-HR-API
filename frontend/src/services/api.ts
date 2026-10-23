import type {
  Shift,
  Holiday,
  LeaveType,
  LeaveQuota,
  Employee,
  WeeklyRosterResponse,
  AssignShiftPayload,
  PublishRosterPayload,
  CountLeaveDaysRequest,
  LeaveCalculationResult,
} from '../types';


export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api';

// Initial reference data matching the design specification
export const INITIAL_SHIFTS: Shift[] = [
  {
    id: 1,
    pid: '550e8400-e29b-41d4-a716-446655440001',
    name: 'General Day Shift',
    code: 'GEN-01',
    start_time: '09:00:00',
    end_time: '18:00:00',
    break_duration_minutes: 60,
    grace_period_minutes: 15,
    is_overnight: false,
    is_active: true,
    created_at: '2026-10-15T09:00:00Z',
    updated_at: '2026-10-15T09:00:00Z',
  },
  {
    id: 2,
    pid: '550e8400-e29b-41d4-a716-446655440002',
    name: 'Early Morning Shift',
    code: 'MOR-01',
    start_time: '06:00:00',
    end_time: '14:30:00',
    break_duration_minutes: 45,
    grace_period_minutes: 10,
    is_overnight: false,
    is_active: true,
    created_at: '2026-10-15T09:00:00Z',
    updated_at: '2026-10-15T09:00:00Z',
  },
  {
    id: 3,
    pid: '550e8400-e29b-41d4-a716-446655440003',
    name: 'Night Support Shift',
    code: 'NGT-01',
    start_time: '21:00:00',
    end_time: '06:00:00',
    break_duration_minutes: 60,
    grace_period_minutes: 15,
    is_overnight: true,
    is_active: true,
    created_at: '2026-10-15T09:00:00Z',
    updated_at: '2026-10-15T09:00:00Z',
  },
  {
    id: 4,
    pid: '550e8400-e29b-41d4-a716-446655440004',
    name: 'Weekend Standby Shift',
    code: 'WKD-01',
    start_time: '10:00:00',
    end_time: '16:00:00',
    break_duration_minutes: 30,
    grace_period_minutes: 20,
    is_overnight: false,
    is_active: false,
    created_at: '2026-10-15T09:00:00Z',
    updated_at: '2026-10-15T09:00:00Z',
  },
];

export const INITIAL_HOLIDAYS: Holiday[] = [
  {
    id: 1,
    pid: '650e8400-e29b-41d4-a716-446655440001',
    name: 'Republic Day',
    holiday_date: '2026-01-26',
    description: 'National holiday commemorating the Constitution',
    applicable_location: null, // Global
    is_recurring: true,
    created_at: '2026-10-15T09:00:00Z',
  },
  {
    id: 2,
    pid: '650e8400-e29b-41d4-a716-446655440002',
    name: 'Independence Day',
    holiday_date: '2026-08-15',
    description: 'National Independence celebration',
    applicable_location: null,
    is_recurring: true,
    created_at: '2026-10-15T09:00:00Z',
  },
  {
    id: 3,
    pid: '650e8400-e29b-41d4-a716-446655440003',
    name: 'Diwali',
    holiday_date: '2026-11-08',
    description: 'Festival of Lights holiday',
    applicable_location: null,
    is_recurring: false,
    created_at: '2026-10-15T09:00:00Z',
  },
  {
    id: 4,
    pid: '650e8400-e29b-41d4-a716-446655440004',
    name: 'Telangana Formation Day',
    holiday_date: '2026-06-02',
    description: 'State holiday for Hyderabad branch office',
    applicable_location: 'Hyderabad',
    is_recurring: true,
    created_at: '2026-10-15T09:00:00Z',
  },
  {
    id: 5,
    pid: '650e8400-e29b-41d4-a716-446655440005',
    name: 'Karnataka Rajyotsava',
    holiday_date: '2026-11-01',
    description: 'State holiday for Bengaluru technology centre',
    applicable_location: 'Bengaluru',
    is_recurring: true,
    created_at: '2026-10-15T09:00:00Z',
  },
];

export const INITIAL_LEAVE_TYPES: LeaveType[] = [
  {
    id: 1,
    pid: '750e8400-e29b-41d4-a716-446655440001',
    name: 'Casual Leave',
    code: 'CL',
    description: 'For personal errands, family obligations, and unplanned short absences.',
    is_paid: true,
    is_active: true,
  },
  {
    id: 2,
    pid: '750e8400-e29b-41d4-a716-446655440002',
    name: 'Sick Leave',
    code: 'SL',
    description: 'For recovery from personal illnesses, medical appointments, and healthcare needs.',
    is_paid: true,
    is_active: true,
  },
  {
    id: 3,
    pid: '750e8400-e29b-41d4-a716-446655440003',
    name: 'Earned Leave / Annual Vacation',
    code: 'EL',
    description: 'Planned vacation days accumulated monthly throughout the active service year.',
    is_paid: true,
    is_active: true,
  },
  {
    id: 4,
    pid: '750e8400-e29b-41d4-a716-446655440004',
    name: 'Maternity Leave',
    code: 'ML',
    description: 'Statutory paid leave for expectant mothers and post-delivery care.',
    is_paid: true,
    is_active: true,
  },
  {
    id: 5,
    pid: '750e8400-e29b-41d4-a716-446655440005',
    name: 'Loss of Pay',
    code: 'LOP',
    description: 'Unpaid leaves taken after exhaust of eligible paid quotas.',
    is_paid: false,
    is_active: true,
  },
];

export const INITIAL_LEAVE_QUOTAS: LeaveQuota[] = [
  {
    id: 1,
    pid: '850e8400-e29b-41d4-a716-446655440001',
    leave_type_id: 1,
    leave_type_name: 'Casual Leave',
    leave_type_code: 'CL',
    year: 2026,
    entitled_days: 12.0,
    applicable_group: 'ALL',
  },
  {
    id: 2,
    pid: '850e8400-e29b-41d4-a716-446655440002',
    leave_type_id: 2,
    leave_type_name: 'Sick Leave',
    leave_type_code: 'SL',
    year: 2026,
    entitled_days: 10.5,
    applicable_group: 'ALL',
  },
  {
    id: 3,
    pid: '850e8400-e29b-41d4-a716-446655440003',
    leave_type_id: 3,
    leave_type_name: 'Earned Leave / Annual Vacation',
    leave_type_code: 'EL',
    year: 2026,
    entitled_days: 18.0,
    applicable_group: 'ALL',
  },
  {
    id: 4,
    pid: '850e8400-e29b-41d4-a716-446655440004',
    leave_type_id: 4,
    leave_type_name: 'Maternity Leave',
    leave_type_code: 'ML',
    year: 2026,
    entitled_days: 90.0,
    applicable_group: 'FEMALE_EMPLOYEES',
  },
];

export interface ApiResponse<T> {
  data: T | null;
  error: string | null;
  isLiveApi: boolean;
}

export async function request<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE_URL}${endpoint}`, {
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
    ...options,
  });

  if (!res.ok) {
    let errorMsg = `API Error (${res.status}): ${res.statusText}`;
    try {
      const errJson = await res.json();
      if (errJson && errJson.error) {
        errorMsg = errJson.error;
      }
    } catch {
      try {
        const errorText = await res.text();
        if (errorText) errorMsg = errorText;
      } catch {
        // Fallback to initial errorMsg
      }
    }
    throw new Error(errorMsg);
  }

  // Handle HTTP 204 No Content or empty response body (e.g. from DELETE)
  if (res.status === 204 || res.headers.get('content-length') === '0') {
    return null as T;
  }

  return res.json();
}

export const api = {
  async checkHealth(): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE_URL}/health`, { method: 'GET' });
      return res.ok;
    } catch {
      return false;
    }
  },

  // Shifts
  async fetchShifts(): Promise<ApiResponse<Shift[]>> {
    try {
      const data = await request<Shift[]>('/shifts');
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to reach Rust backend',
        isLiveApi: false,
      };
    }
  },

  async createShift(shift: Omit<Shift, 'id' | 'pid' | 'created_at' | 'updated_at'>): Promise<Shift> {
    return request<Shift>('/shifts', {
      method: 'POST',
      body: JSON.stringify(shift),
    });
  },

  async updateShift(id: number, shift: Partial<Shift>): Promise<Shift> {
    return request<Shift>(`/shifts/${id}`, {
      method: 'PUT',
      body: JSON.stringify(shift),
    });
  },

  async deleteShift(id: number): Promise<void> {
    await request<void>(`/shifts/${id}`, { method: 'DELETE' });
  },

  // Holidays
  async fetchHolidays(): Promise<ApiResponse<Holiday[]>> {
    try {
      const data = await request<Holiday[]>('/holidays');
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to reach Rust backend',
        isLiveApi: false,
      };
    }
  },

  async createHoliday(holiday: Omit<Holiday, 'id' | 'pid' | 'created_at' | 'updated_at'>): Promise<Holiday> {
    return request<Holiday>('/holidays', {
      method: 'POST',
      body: JSON.stringify(holiday),
    });
  },

  async updateHoliday(id: number, holiday: Partial<Holiday>): Promise<Holiday> {
    return request<Holiday>(`/holidays/${id}`, {
      method: 'PUT',
      body: JSON.stringify(holiday),
    });
  },

  async deleteHoliday(id: number): Promise<void> {
    await request<void>(`/holidays/${id}`, { method: 'DELETE' });
  },

  // Leave Types
  async fetchLeaveTypes(): Promise<ApiResponse<LeaveType[]>> {
    try {
      const data = await request<LeaveType[]>('/leave-types');
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to reach Rust backend',
        isLiveApi: false,
      };
    }
  },

  async createLeaveType(lt: Omit<LeaveType, 'id' | 'pid' | 'created_at' | 'updated_at'>): Promise<LeaveType> {
    return request<LeaveType>('/leave-types', {
      method: 'POST',
      body: JSON.stringify(lt),
    });
  },

  async updateLeaveType(id: number, lt: Partial<LeaveType>): Promise<LeaveType> {
    return request<LeaveType>(`/leave-types/${id}`, {
      method: 'PUT',
      body: JSON.stringify(lt),
    });
  },

  async deleteLeaveType(id: number): Promise<void> {
    await request<void>(`/leave-types/${id}`, { method: 'DELETE' });
  },

  // Leave Quotas
  async fetchLeaveQuotas(): Promise<ApiResponse<LeaveQuota[]>> {
    try {
      const data = await request<LeaveQuota[]>('/leave-quotas');
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to reach Rust backend',
        isLiveApi: false,
      };
    }
  },

  async createLeaveQuota(lq: Omit<LeaveQuota, 'id' | 'pid' | 'created_at' | 'updated_at'>): Promise<LeaveQuota> {
    return request<LeaveQuota>('/leave-quotas', {
      method: 'POST',
      body: JSON.stringify(lq),
    });
  },

  async updateLeaveQuota(id: number, lq: Partial<LeaveQuota>): Promise<LeaveQuota> {
    return request<LeaveQuota>(`/leave-quotas/${id}`, {
      method: 'PUT',
      body: JSON.stringify(lq),
    });
  },

  async deleteLeaveQuota(id: number): Promise<void> {
    await request<void>(`/leave-quotas/${id}`, { method: 'DELETE' });
  },

  // Weekly Roster & Employees
  async fetchWeeklyRoster(weekStart?: string): Promise<ApiResponse<WeeklyRosterResponse>> {
    try {
      const endpoint = weekStart ? `/roster?week_start=${encodeURIComponent(weekStart)}` : '/roster';
      const data = await request<WeeklyRosterResponse>(endpoint);
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to fetch weekly roster',
        isLiveApi: false,
      };
    }
  },

  async assignShift(payload: AssignShiftPayload): Promise<any> {
    return request<any>('/roster/assign', {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  },

  async publishRoster(payload: PublishRosterPayload): Promise<any> {
    return request<any>('/roster/publish', {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  },

  async fetchEmployees(): Promise<ApiResponse<Employee[]>> {
    try {
      const data = await request<Employee[]>('/employees');
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to fetch employees',
        isLiveApi: false,
      };
    }
  },

  async countLeaveDays(params: CountLeaveDaysRequest): Promise<ApiResponse<LeaveCalculationResult>> {
    try {
      const data = await request<LeaveCalculationResult>('/leave/count-days', {
        method: 'POST',
        body: JSON.stringify(params),
      });
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to calculate leave days',
        isLiveApi: false,
      };
    }
  },
};


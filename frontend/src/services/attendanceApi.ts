import { API_BASE_URL, request, type ApiResponse } from './api';
import type {
  DailyAttendanceRecord,
  AttendanceListResponse,
  CreateAttendancePayload,
  UpdateAttendancePayload,
  ClockInPayload,
  ClockOutPayload,
  TodayAttendanceStatus,
  AttendanceCorrectionRequest,
  CreateCorrectionPayload,
  ApproveCorrectionPayload,
  RejectCorrectionPayload,
  CorrectionQuery,
} from '../types';

export interface AttendanceFilterParams {
  search?: string;
  status?: string;
  entry_status?: string;
  archived?: boolean;
  date?: string;
}

function buildQueryString(params?: AttendanceFilterParams): string {
  if (!params) return '';
  const searchParams = new URLSearchParams();
  if (params.search && params.search.trim()) {
    searchParams.set('search', params.search.trim());
  }
  if (params.status && params.status !== 'ALL') {
    searchParams.set('status', params.status);
  }
  if (params.entry_status && params.entry_status !== 'ALL') {
    searchParams.set('entry_status', params.entry_status);
  }
  if (params.archived !== undefined) {
    searchParams.set('archived', String(params.archived));
  }
  if (params.date) {
    searchParams.set('date', params.date);
  }
  const qs = searchParams.toString();
  return qs ? `?${qs}` : '';
}

export const attendanceApi = {
  async fetchAttendance(params?: AttendanceFilterParams): Promise<ApiResponse<AttendanceListResponse>> {
    try {
      const qs = buildQueryString(params);
      const data = await request<AttendanceListResponse>(`/attendance${qs}`);
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to fetch attendance records',
        isLiveApi: false,
      };
    }
  },

  async createAttendance(payload: CreateAttendancePayload): Promise<DailyAttendanceRecord> {
    return request<DailyAttendanceRecord>('/attendance', {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  },

  async updateAttendance(id: number, payload: UpdateAttendancePayload): Promise<DailyAttendanceRecord> {
    return request<DailyAttendanceRecord>(`/attendance/${id}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    });
  },

  async toggleArchive(id: number): Promise<DailyAttendanceRecord> {
    return request<DailyAttendanceRecord>(`/attendance/${id}/archive`, {
      method: 'POST',
    });
  },

  async deleteAttendance(id: number): Promise<void> {
    await request<void>(`/attendance/${id}`, {
      method: 'DELETE',
    });
  },

  async exportCsv(params?: AttendanceFilterParams): Promise<void> {
    const qs = buildQueryString(params);
    const res = await fetch(`${API_BASE_URL}/attendance/export${qs}`);
    if (!res.ok) {
      throw new Error(`Failed to export CSV: ${res.statusText}`);
    }
    const blob = await res.blob();
    const url = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `attendance_export_${new Date().toISOString().split('T')[0]}.csv`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(url);
  },

  async getTodayStatus(employeeId: number, date?: string): Promise<ApiResponse<TodayAttendanceStatus>> {
    try {
      const qs = new URLSearchParams({ employee_id: String(employeeId) });
      if (date) qs.set('date', date);
      const data = await request<TodayAttendanceStatus>(`/attendance/today?${qs.toString()}`);
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to fetch today attendance status',
        isLiveApi: false,
      };
    }
  },

  async clockIn(payload: ClockInPayload): Promise<DailyAttendanceRecord> {
    return request<DailyAttendanceRecord>('/attendance/clock-in', {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  },

  async clockOut(payload: ClockOutPayload): Promise<DailyAttendanceRecord> {
    return request<DailyAttendanceRecord>('/attendance/clock-out', {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  },

  // Attendance Correction Requests Workflow
  async fetchCorrections(params?: CorrectionQuery): Promise<ApiResponse<AttendanceCorrectionRequest[]>> {
    try {
      const searchParams = new URLSearchParams();
      if (params?.status && params.status !== 'ALL') {
        searchParams.set('status', params.status);
      }
      if (params?.employee_id) {
        searchParams.set('employee_id', String(params.employee_id));
      }
      if (params?.date) {
        searchParams.set('date', params.date);
      }
      const qs = searchParams.toString();
      const endpoint = qs ? `/attendance/corrections?${qs}` : '/attendance/corrections';
      const data = await request<AttendanceCorrectionRequest[]>(endpoint);
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : 'Failed to fetch correction requests',
        isLiveApi: false,
      };
    }
  },

  async getCorrection(id: number): Promise<ApiResponse<AttendanceCorrectionRequest>> {
    try {
      const data = await request<AttendanceCorrectionRequest>(`/attendance/corrections/${id}`);
      return { data, error: null, isLiveApi: true };
    } catch (err: unknown) {
      return {
        data: null,
        error: err instanceof Error ? err.message : `Failed to fetch correction request #${id}`,
        isLiveApi: false,
      };
    }
  },

  async createCorrection(payload: CreateCorrectionPayload): Promise<AttendanceCorrectionRequest> {
    return request<AttendanceCorrectionRequest>('/attendance/corrections', {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  },

  async approveCorrection(
    id: number,
    payload: ApproveCorrectionPayload = {}
  ): Promise<AttendanceCorrectionRequest> {
    return request<AttendanceCorrectionRequest>(`/attendance/corrections/${id}/approve`, {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  },

  async rejectCorrection(
    id: number,
    payload: RejectCorrectionPayload
  ): Promise<AttendanceCorrectionRequest> {
    return request<AttendanceCorrectionRequest>(`/attendance/corrections/${id}/reject`, {
      method: 'POST',
      body: JSON.stringify(payload),
    });
  },
};

import React, { useState, useEffect, useCallback, useMemo } from 'react';
import {
  Clock,
  Calendar,
  LogIn,
  LogOut,
  CheckCircle2,
  AlertCircle,
  User,
  Timer,
  ChevronDown,
  Sparkles,
  Loader2,
  ShieldAlert,
} from 'lucide-react';
import type { Employee, DailyAttendanceRecord, TodayAttendanceStatus } from '../../types';
import { attendanceApi } from '../../services/attendanceApi';

interface AttendanceClockCardProps {
  employees: Employee[];
  onAttendanceChange?: () => void;
  showSuccessToast?: (msg: string) => void;
  onRequestCorrection?: (employeeId?: number) => void;
}

export const AttendanceClockCard: React.FC<AttendanceClockCardProps> = ({
  employees,
  onAttendanceChange,
  showSuccessToast,
  onRequestCorrection,
}) => {
  const [selectedEmpId, setSelectedEmpId] = useState<number | null>(null);
  const [todayStatus, setTodayStatus] = useState<TodayAttendanceStatus | null>(null);
  const [currentTime, setCurrentTime] = useState<Date>(new Date());
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
  const [isLoadingStatus, setIsLoadingStatus] = useState<boolean>(false);
  const [cardError, setCardError] = useState<string | null>(null);
  const [clockNote, setClockNote] = useState<string>('');
  const [showNoteInput, setShowNoteInput] = useState<boolean>(false);

  // Initialize selected employee to first available employee
  useEffect(() => {
    if (selectedEmpId === null && employees.length > 0) {
      setSelectedEmpId(employees[0].id);
    }
  }, [employees, selectedEmpId]);

  // Real-time ticking clock
  useEffect(() => {
    const timer = setInterval(() => {
      setCurrentTime(new Date());
    }, 1000);
    return () => clearInterval(timer);
  }, []);

  // Fetch today's attendance status for selected employee with stale response cancellation
  useEffect(() => {
    if (!selectedEmpId) return;

    let isCurrent = true;
    setIsLoadingStatus(true);
    setCardError(null);

    attendanceApi
      .getTodayStatus(selectedEmpId)
      .then((res) => {
        if (!isCurrent) return; // Discard stale response if a newer employee was selected
        setIsLoadingStatus(false);
        if (res.data) {
          setTodayStatus(res.data);
        } else if (res.error) {
          setCardError(res.error);
        }
      })
      .catch((err: unknown) => {
        if (!isCurrent) return;
        setIsLoadingStatus(false);
        setCardError(err instanceof Error ? err.message : 'Failed to query attendance status');
      });

    return () => {
      isCurrent = false; // Invalidate previous request on employee change
    };
  }, [selectedEmpId]);

  // Refresh status helper
  const reloadCurrentStatus = useCallback(async () => {
    if (!selectedEmpId) return;
    setIsLoadingStatus(true);
    try {
      const res = await attendanceApi.getTodayStatus(selectedEmpId);
      if (res.data) {
        setTodayStatus(res.data);
      }
    } catch {
      // Ignored for background reload
    } finally {
      setIsLoadingStatus(false);
    }
  }, [selectedEmpId]);

  // Selected employee object
  const currentEmployee = useMemo(() => {
    return employees.find((e) => e.id === selectedEmpId) || employees[0] || null;
  }, [employees, selectedEmpId]);

  // Formatted date string
  const formattedToday = useMemo(() => {
    return new Intl.DateTimeFormat('en-US', {
      weekday: 'long',
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    }).format(currentTime);
  }, [currentTime]);

  // Formatted current time string
  const formattedClock = useMemo(() => {
    return currentTime.toLocaleTimeString('en-US', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: true,
    });
  }, [currentTime]);

  // Helper to format stored TIME string (e.g. "09:00:00" -> "09:00 AM")
  const formatTimeDisplay = (timeStr?: string | null): string => {
    if (!timeStr) return '--:--';
    const parts = timeStr.trim().split(':');
    if (parts.length >= 2) {
      const hour = parseInt(parts[0], 10);
      const min = parts[1];
      const ampm = hour >= 12 ? 'PM' : 'AM';
      const hour12 = hour % 12 === 0 ? 12 : hour % 12;
      return `${hour12 < 10 ? '0' + hour12 : hour12}:${min} ${ampm}`;
    }
    return timeStr;
  };

  // Live elapsed time since clock-in if shift is active
  const liveElapsedTime = useMemo(() => {
    if (!todayStatus?.record?.in_time || todayStatus?.record?.out_time) {
      return null;
    }
    const [h, m, s] = todayStatus.record.in_time.split(':').map((v) => parseInt(v, 10));
    const clockInDate = new Date(currentTime);
    clockInDate.setHours(h || 0, m || 0, s || 0, 0);

    const diffMs = currentTime.getTime() - clockInDate.getTime();
    if (diffMs <= 0) return '0h 0m';

    const diffMins = Math.floor(diffMs / 60000);
    const hours = Math.floor(diffMins / 60);
    const mins = diffMins % 60;
    return `${hours}h ${mins}m`;
  }, [todayStatus, currentTime]);

  // Handle Clock-In Action
  const handleClockIn = async () => {
    if (!selectedEmpId) return;
    setIsSubmitting(true);
    setCardError(null);

    try {
      const record: DailyAttendanceRecord = await attendanceApi.clockIn({
        employee_id: selectedEmpId,
        notes: clockNote.trim() || undefined,
      });

      setTodayStatus({
        date: record.attendance_date,
        employee_id: selectedEmpId,
        record,
        can_clock_in: false,
        can_clock_out: true,
        status: 'CLOCKED_IN',
      });

      setClockNote('');
      setShowNoteInput(false);

      if (showSuccessToast) {
        showSuccessToast(
          `Clock-in recorded for ${record.employee_name || currentEmployee?.name || 'Employee'} at ${formatTimeDisplay(record.in_time)}.`
        );
      }
      if (onAttendanceChange) {
        onAttendanceChange();
      }
    } catch (err: unknown) {
      setCardError(err instanceof Error ? err.message : 'Clock-in failed');
    } finally {
      setIsSubmitting(false);
    }
  };

  // Handle Clock-Out Action
  const handleClockOut = async () => {
    if (!selectedEmpId) return;
    setIsSubmitting(true);
    setCardError(null);

    try {
      const record: DailyAttendanceRecord = await attendanceApi.clockOut({
        employee_id: selectedEmpId,
        notes: clockNote.trim() || undefined,
      });

      setTodayStatus({
        date: record.attendance_date,
        employee_id: selectedEmpId,
        record,
        can_clock_in: false,
        can_clock_out: false,
        status: 'CLOCKED_OUT',
      });

      setClockNote('');
      setShowNoteInput(false);

      if (showSuccessToast) {
        showSuccessToast(
          `Clock-out recorded for ${record.employee_name || currentEmployee?.name || 'Employee'}! Total worked: ${record.hours_worked} hrs.`
        );
      }
      if (onAttendanceChange) {
        onAttendanceChange();
      }
    } catch (err: unknown) {
      setCardError(err instanceof Error ? err.message : 'Clock-out failed');
    } finally {
      setIsSubmitting(false);
    }
  };

  const status = todayStatus?.status || 'NOT_CLOCKED_IN';
  const record = todayStatus?.record;

  return (
    <div className="relative overflow-hidden rounded-2xl bg-gradient-to-br from-[#0A1326] via-[#0E1B38] to-[#0A1326] border border-[#1C3262] shadow-xl p-5 sm:p-6 transition-all">
      {/* Decorative background glow elements */}
      <div className="absolute -top-16 -right-16 w-56 h-56 rounded-full bg-[#0B6FDB]/10 blur-3xl pointer-events-none" />
      <div className="absolute -bottom-16 -left-16 w-56 h-56 rounded-full bg-[#38BDF8]/10 blur-3xl pointer-events-none" />

      {/* Card Header: Title & Employee Switcher */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-[#16274E]">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-tr from-[#0B6FDB] to-[#38BDF8] flex items-center justify-center text-white shadow-md shadow-[#0B6FDB]/20">
            <Timer className="w-5 h-5" />
          </div>
          <div>
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="text-base font-bold text-white tracking-tight">
                Attendance Terminal
              </h2>
              <span className="inline-flex items-center gap-1 text-[10px] font-semibold tracking-wider uppercase px-2 py-0.5 rounded-full bg-[#0B6FDB]/20 text-[#38BDF8] border border-[#0B6FDB]/30">
                <Sparkles className="w-2.5 h-2.5" /> TIM-02
              </span>
              <span className="inline-flex items-center gap-1 text-[10px] font-medium text-[#F59E0B] bg-[#78350F]/30 border border-[#B45309]/40 px-2 py-0.5 rounded-md" title="Simulated terminal for prototype testing. In production, employee identity is derived from authenticated session tokens.">
                <ShieldAlert className="w-2.5 h-2.5" /> Demo: Unauthenticated
              </span>
            </div>
            <p className="text-xs text-[#889DC7]">
              Real-time PostgreSQL clock-in and clock-out tracking (Client-simulated identity)
            </p>
          </div>
        </div>

        {/* Employee Switcher */}
        {employees.length > 0 && (
          <div className="flex items-center gap-2 shrink-0">
            {isLoadingStatus && (
              <span className="flex items-center gap-1 text-[10px] text-[#38BDF8] animate-pulse">
                <Loader2 className="w-3 h-3 animate-spin" />
                <span>Syncing...</span>
              </span>
            )}
            <div className="relative inline-block text-left">
              <label htmlFor="employee-select" className="sr-only">
                Select Employee
              </label>
              <div className="relative">
                <select
                  id="employee-select"
                  value={selectedEmpId || ''}
                  onChange={(e) => setSelectedEmpId(Number(e.target.value))}
                  disabled={isLoadingStatus || isSubmitting}
                  className="appearance-none bg-[#0D1833] hover:bg-[#132244] border border-[#1E3466] text-white text-xs font-medium rounded-xl pl-8 pr-8 py-2 focus:outline-hidden focus:border-[#38BDF8] cursor-pointer transition-colors shadow-xs disabled:opacity-50"
                >
                  {employees.map((emp) => (
                    <option key={emp.id} value={emp.id} className="bg-[#0A1326] text-white">
                      {emp.name} ({emp.department} · {emp.role})
                    </option>
                  ))}
                </select>
                <User className="w-3.5 h-3.5 text-[#889DC7] absolute left-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
                <ChevronDown className="w-3.5 h-3.5 text-[#889DC7] absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Error Banner */}
      {cardError && (
        <div className="mt-4 p-3 rounded-xl bg-[#3B1219] border border-[#771D2D] text-xs text-[#F87171] flex items-center justify-between gap-3 animate-fadeIn">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{cardError}</span>
          </div>
          <button
            type="button"
            onClick={() => setCardError(null)}
            className="text-[#F87171] hover:text-white underline cursor-pointer text-[11px]"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Main Grid: Date/Clock Display, Status Metrics, and Action Button */}
      <div className="mt-5 grid grid-cols-1 md:grid-cols-12 gap-5 items-center">
        {/* Left Column: Live Clock & Date */}
        <div className="md:col-span-4 flex flex-col justify-center space-y-1 bg-[#070D1A]/80 border border-[#16274E] rounded-xl p-4">
          <div className="flex items-center gap-2 text-xs font-semibold text-[#889DC7]">
            <Calendar className="w-3.5 h-3.5 text-[#38BDF8]" />
            <span>{formattedToday}</span>
          </div>
          <div className="text-2xl sm:text-3xl font-black font-mono tracking-tight text-white flex items-center gap-2">
            <Clock className="w-5 h-5 text-[#0B6FDB]" />
            <span>{formattedClock}</span>
          </div>
          <div className="pt-2 flex items-center gap-2 text-[11px] text-[#5B6787]">
            <span>Active Employee:</span>
            <span className="font-semibold text-[#889DC7] truncate max-w-[150px]">
              {currentEmployee?.name || 'Default Employee'}
            </span>
          </div>
        </div>

        {/* Middle Column: 3 Attendance Metrics (Clock In, Clock Out, Hours Worked) */}
        <div className="md:col-span-5 grid grid-cols-3 gap-2.5">
          {/* Metric 1: Clock In Time */}
          <div className="bg-[#070D1A]/80 border border-[#16274E] rounded-xl p-3 text-center flex flex-col justify-center min-h-[72px]">
            <span className="text-[10px] font-bold text-[#889DC7] uppercase tracking-wider">
              Clock In
            </span>
            {isLoadingStatus ? (
              <div className="flex items-center justify-center py-1">
                <Loader2 className="w-3.5 h-3.5 animate-spin text-[#38BDF8]" />
              </div>
            ) : (
              <span className="text-sm sm:text-base font-bold font-mono text-white mt-0.5 truncate">
                {formatTimeDisplay(record?.in_time)}
              </span>
            )}
            <span className="text-[10px] text-[#5B6787] mt-0.5">
              {record?.in_time ? 'Recorded' : 'Pending'}
            </span>
          </div>

          {/* Metric 2: Clock Out Time */}
          <div className="bg-[#070D1A]/80 border border-[#16274E] rounded-xl p-3 text-center flex flex-col justify-center min-h-[72px]">
            <span className="text-[10px] font-bold text-[#889DC7] uppercase tracking-wider">
              Clock Out
            </span>
            {isLoadingStatus ? (
              <div className="flex items-center justify-center py-1">
                <Loader2 className="w-3.5 h-3.5 animate-spin text-[#38BDF8]" />
              </div>
            ) : (
              <span className="text-sm sm:text-base font-bold font-mono text-white mt-0.5 truncate">
                {formatTimeDisplay(record?.out_time)}
              </span>
            )}
            <span className="text-[10px] text-[#5B6787] mt-0.5">
              {record?.out_time ? 'Recorded' : 'Pending'}
            </span>
          </div>

          {/* Metric 3: Hours Worked */}
          <div className="bg-[#070D1A]/80 border border-[#16274E] rounded-xl p-3 text-center flex flex-col justify-center min-h-[72px]">
            <span className="text-[10px] font-bold text-[#889DC7] uppercase tracking-wider">
              Hours Worked
            </span>
            {isLoadingStatus ? (
              <div className="flex items-center justify-center py-1">
                <Loader2 className="w-3.5 h-3.5 animate-spin text-[#38BDF8]" />
              </div>
            ) : (
              <span className="text-sm sm:text-base font-bold font-mono text-[#38BDF8] mt-0.5">
                {record?.hours_worked ? `${record.hours_worked}h` : liveElapsedTime || '0.00h'}
              </span>
            )}
            <span className="text-[10px] text-[#5B6787] mt-0.5">
              {status === 'CLOCKED_IN' ? 'Running' : status === 'CLOCKED_OUT' ? 'Total' : 'Zero'}
            </span>
          </div>
        </div>

        {/* Right Column: Status Badge & Dynamic Action Button */}
        <div className="md:col-span-3 flex flex-col items-center sm:items-end justify-center space-y-2.5">
          {/* Current Status Pill */}
          <div>
            {isLoadingStatus ? (
              <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-[#16274E] text-[#889DC7] border border-[#1C3262]">
                <Loader2 className="w-3 h-3 animate-spin text-[#38BDF8]" />
                Checking status...
              </span>
            ) : status === 'NOT_CLOCKED_IN' ? (
              <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-[#1C2844] text-[#889DC7] border border-[#2B3F6C]">
                <span className="w-2 h-2 rounded-full bg-[#889DC7]" />
                Not Clocked In
              </span>
            ) : status === 'CLOCKED_IN' ? (
              <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-[#064E3B]/70 text-[#34D399] border border-[#059669]/50 shadow-xs shadow-[#059669]/20">
                <span className="relative flex h-2 w-2">
                  <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#34D399] opacity-75" />
                  <span className="relative inline-flex rounded-full h-2 w-2 bg-[#10B981]" />
                </span>
                Active Shift
              </span>
            ) : (
              <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold bg-[#1E1B4B]/80 text-[#818CF8] border border-[#4338CA]/50">
                <CheckCircle2 className="w-3.5 h-3.5 text-[#818CF8]" />
                Shift Completed
              </span>
            )}
          </div>

          {/* Action Button: Clock In / Clock Out / Completed */}
          {status === 'NOT_CLOCKED_IN' && (
            <button
              type="button"
              onClick={handleClockIn}
              disabled={isSubmitting || isLoadingStatus}
              className="w-full sm:w-auto flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl text-xs font-bold text-white bg-gradient-to-r from-[#0B6FDB] to-[#0284C7] hover:from-[#0957B0] hover:to-[#0369A1] shadow-lg shadow-[#0B6FDB]/25 transition-all transform active:scale-95 disabled:opacity-50 cursor-pointer"
            >
              {isSubmitting ? (
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
              ) : (
                <LogIn className="w-4 h-4" />
              )}
              <span>Clock In Now</span>
            </button>
          )}

          {status === 'CLOCKED_IN' && (
            <button
              type="button"
              onClick={handleClockOut}
              disabled={isSubmitting || isLoadingStatus}
              className="w-full sm:w-auto flex items-center justify-center gap-2 px-5 py-2.5 rounded-xl text-xs font-bold text-white bg-gradient-to-r from-[#DC2626] to-[#EA580C] hover:from-[#B91C1C] hover:to-[#C2410C] shadow-lg shadow-[#DC2626]/25 transition-all transform active:scale-95 disabled:opacity-50 cursor-pointer"
            >
              {isSubmitting ? (
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
              ) : (
                <LogOut className="w-4 h-4" />
              )}
              <span>Clock Out Now</span>
            </button>
          )}

          {status === 'CLOCKED_OUT' && (
            <div className="flex items-center gap-1.5 px-4 py-2 rounded-xl text-xs font-semibold text-[#889DC7] bg-[#0D1833] border border-[#1E3466]">
              <CheckCircle2 className="w-4 h-4 text-[#10B981]" />
              <span>Attendance Recorded</span>
            </div>
          )}

          {/* Optional Note Toggle */}
          {status !== 'CLOCKED_OUT' && (
            <button
              type="button"
              onClick={() => setShowNoteInput(!showNoteInput)}
              className="text-[11px] text-[#889DC7] hover:text-white underline cursor-pointer"
            >
              {showNoteInput ? 'Cancel note' : '+ Add optional shift note'}
            </button>
          )}

          {/* Missed Punch Correction Trigger */}
          {onRequestCorrection && (
            <button
              type="button"
              onClick={() => onRequestCorrection(selectedEmpId || undefined)}
              className="text-[11px] text-[#38BDF8] hover:text-white underline cursor-pointer flex items-center gap-1"
            >
              <Clock className="w-3 h-3" />
              <span>Missed a punch? Request correction</span>
            </button>
          )}
        </div>
      </div>


      {/* Collapsible Optional Note Input */}
      {showNoteInput && status !== 'CLOCKED_OUT' && (
        <div className="mt-4 pt-3 border-t border-[#16274E] flex items-center gap-3">
          <input
            type="text"
            value={clockNote}
            onChange={(e) => setClockNote(e.target.value)}
            placeholder="Add optional note (e.g. Work from home, client meeting, late check-in reason)..."
            className="flex-1 bg-[#070D1A] border border-[#16274E] focus:border-[#38BDF8] rounded-xl px-3 py-1.5 text-xs text-white placeholder-[#5B6787] focus:outline-hidden"
          />
          <span className="text-[10px] text-[#5B6787]">Persisted in database</span>
        </div>
      )}
    </div>
  );
};

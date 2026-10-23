import React, { useState, useEffect, useCallback } from 'react';
import {
  Users,
  Calendar as CalendarIcon,
  ArrowLeftRight,
  Clock,
  ChevronLeft,
  ChevronRight,
  Send,
  CheckCircle2,
} from 'lucide-react';
import type {
  WeeklyRosterResponse,
  Shift,
  EmployeeRosterRow,
  DayRosterEntry,
} from '../types';
import { api } from '../services/api';
import { ShiftAssignModal } from '../components/roster/ShiftAssignModal';
import { PublishConfirmModal } from '../components/roster/PublishConfirmModal';

interface WeeklyRosterPageProps {
  shifts: Shift[];
  searchQuery?: string;
  showSuccessToast?: (msg: string) => void;
}

export const WeeklyRosterPage: React.FC<WeeklyRosterPageProps> = ({
  shifts,
  searchQuery = '',
  showSuccessToast,
}) => {
  // Target week starts at reference Monday Oct 12, 2026
  const [currentWeekStart, setCurrentWeekStart] = useState<string>('2026-10-12');
  const [rosterData, setRosterData] = useState<WeeklyRosterResponse | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  // Modal states
  const [selectedCell, setSelectedCell] = useState<{
    employee: EmployeeRosterRow;
    dayEntry: DayRosterEntry;
  } | null>(null);
  const [isPublishModalOpen, setIsPublishModalOpen] = useState<boolean>(false);

  // Fetch roster data from backend
  const loadRoster = useCallback(async (weekStart: string) => {
    setIsLoading(true);
    setError(null);
    try {
      const res = await api.fetchWeeklyRoster(weekStart);
      if (res.data) {
        setRosterData(res.data);
      } else {
        setError(res.error || 'Failed to load weekly roster');
      }
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Error fetching roster');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadRoster(currentWeekStart);
  }, [currentWeekStart, loadRoster]);

  // Navigation handlers
  const handlePrevWeek = () => {
    const d = new Date(currentWeekStart);
    d.setDate(d.getDate() - 7);
    setCurrentWeekStart(d.toISOString().split('T')[0]);
  };

  const handleNextWeek = () => {
    const d = new Date(currentWeekStart);
    d.setDate(d.getDate() + 7);
    setCurrentWeekStart(d.toISOString().split('T')[0]);
  };

  const handleThisWeek = () => {
    setCurrentWeekStart('2026-10-12');
  };

  // Format date range string for header
  const formatDateRange = () => {
    if (!rosterData) return 'Oct 12, 2026 to Oct 18, 2026';
    const s = new Date(rosterData.week_start + 'T00:00:00');
    const e = new Date(rosterData.week_end + 'T00:00:00');
    const opts: Intl.DateTimeFormatOptions = { month: 'short', day: 'numeric', year: 'numeric' };
    return `${s.toLocaleDateString('en-US', opts)} to ${e.toLocaleDateString('en-US', opts)}`;
  };

  // Cell click handler
  const handleCellClick = (employee: EmployeeRosterRow, dayEntry: DayRosterEntry) => {
    setSelectedCell({ employee, dayEntry });
  };

  // Save assignment handler
  const handleSaveAssignment = async (
    shiftId: number | null,
    isOff: boolean,
    notes?: string
  ) => {
    if (!selectedCell) return;
    await api.assignShift({
      employee_id: selectedCell.employee.id,
      date: selectedCell.dayEntry.date,
      shift_id: shiftId,
      is_off: isOff,
      notes,
    });
    if (showSuccessToast) {
      showSuccessToast(
        `Updated ${selectedCell.employee.name}'s schedule for ${selectedCell.dayEntry.day_name}.`
      );
    }
    await loadRoster(currentWeekStart);
  };

  // Publish roster handler
  const handlePublishRoster = async () => {
    if (!rosterData) return;
    await api.publishRoster({
      week_start_date: rosterData.week_start,
      published_by: 'Ananya Rao',
    });
    if (showSuccessToast) {
      showSuccessToast(`Weekly roster starting ${rosterData.week_start} published successfully.`);
    }
    await loadRoster(currentWeekStart);
  };

  // Filter employees by search query
  const filteredEmployees = (rosterData?.employees || []).filter((emp) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      emp.name.toLowerCase().includes(q) ||
      emp.role.toLowerCase().includes(q) ||
      emp.department.toLowerCase().includes(q) ||
      (emp.default_shift_name && emp.default_shift_name.toLowerCase().includes(q))
    );
  });

  const getShiftBadgeStyle = (code: string | null, isOff: boolean) => {
    if (isOff || !code) {
      return 'bg-[#121E38] text-[#556994] border border-[#1D2B52] hover:border-[#2B3F73]';
    }
    const cleanCode = code.split('-')[0].toUpperCase();
    if (cleanCode === 'GEN') return 'bg-[#0B6FDB] text-white hover:brightness-110 shadow-sm';
    if (cleanCode === 'USP') return 'bg-[#722ED1] text-white hover:brightness-110 shadow-sm';
    if (cleanCode === 'USC') return 'bg-[#4F46E5] text-white hover:brightness-110 shadow-sm';
    if (cleanCode === 'NGT') return 'bg-[#0891B2] text-white hover:brightness-110 shadow-sm';
    if (cleanCode === 'MOR') return 'bg-[#059669] text-white hover:brightness-110 shadow-sm';
    return 'bg-[#2563EB] text-white hover:brightness-110';
  };

  const getAvatarBg = (color: string) => {
    switch (color) {
      case 'purple':
        return 'bg-gradient-to-tr from-[#6B21A8] to-[#9333EA]';
      case 'teal':
        return 'bg-gradient-to-tr from-[#0D9488] to-[#14B8A6]';
      case 'pink':
      case 'rose':
        return 'bg-gradient-to-tr from-[#BE123C] to-[#F43F5E]';
      case 'blue':
        return 'bg-gradient-to-tr from-[#1D4ED8] to-[#3B82F6]';
      case 'emerald':
        return 'bg-gradient-to-tr from-[#047857] to-[#10B981]';
      default:
        return 'bg-gradient-to-tr from-[#6366F1] to-[#8B5CF6]';
    }
  };

  const isThisWeekSelected = currentWeekStart === '2026-10-12';

  return (
    <div className="space-y-6 animate-in fade-in duration-200">
      {error && (
        <div className="p-3 text-xs bg-red-500/10 border border-red-500/30 text-red-400 rounded-xl">
          {error}
        </div>
      )}
      {/* Page Title & Controls Bar */}
      <div className="flex flex-col md:flex-row md:items-end justify-between gap-4">
        <div>
          <div className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-[11px] font-bold tracking-wider uppercase text-[#0BA6CC] bg-[#0BA6CC]/10 border border-[#0BA6CC]/20 mb-2">
            <span className="w-1.5 h-1.5 rounded-full bg-[#0BA6CC]" />
            SHIFT
          </div>
          <h1 className="text-2xl lg:text-3xl font-bold tracking-tight text-white font-['Sora']">
            Weekly roster
          </h1>
          <p className="text-sm text-[#8D9BB8] mt-1">
            {formatDateRange()}. Click any cell to change that person’s shift.
          </p>
        </div>

        {/* Action Controls */}
        <div className="flex items-center gap-3">
          {/* Week Navigation Group */}
          <div className="inline-flex items-center rounded-xl bg-[#091226] border border-[#18274A] p-1 text-xs font-semibold text-[#C4D0EC]">
            <button
              onClick={handlePrevWeek}
              className="flex items-center gap-1 px-3 py-1.5 rounded-lg hover:bg-white/5 hover:text-white transition-colors cursor-pointer"
            >
              <ChevronLeft className="w-3.5 h-3.5" />
              <span>Prev</span>
            </button>
            <button
              onClick={handleThisWeek}
              className={`px-3 py-1.5 rounded-lg transition-all cursor-pointer ${
                isThisWeekSelected
                  ? 'bg-[#142347] text-white shadow-xs font-bold border border-[#1E3363]'
                  : 'hover:bg-white/5 hover:text-white'
              }`}
            >
              This week
            </button>
            <button
              onClick={handleNextWeek}
              className="flex items-center gap-1 px-3 py-1.5 rounded-lg hover:bg-white/5 hover:text-white transition-colors cursor-pointer"
            >
              <span>Next</span>
              <ChevronRight className="w-3.5 h-3.5" />
            </button>
          </div>

          {/* Publish Roster Button */}
          <button
            onClick={() => setIsPublishModalOpen(true)}
            className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold text-white bg-gradient-to-r from-[#0B6FDB] to-[#0BA6CC] hover:brightness-110 shadow-lg shadow-[#0B6FDB]/30 transition-all cursor-pointer"
          >
            {rosterData?.is_published ? (
              <>
                <CheckCircle2 className="w-4 h-4 text-white" />
                <span>Published</span>
              </>
            ) : (
              <>
                <Send className="w-3.5 h-3.5" />
                <span>Publish roster</span>
              </>
            )}
          </button>
        </div>
      </div>

      {/* Summary Metric Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Card 1: Scheduled Today */}
        <div className="p-4 rounded-2xl bg-[#0B152E] border border-[#17264D] shadow-sm flex flex-col justify-between">
          <div className="flex items-center gap-2 text-xs font-medium text-[#8D9BB8]">
            <div className="w-7 h-7 rounded-lg bg-[#0B6FDB]/15 border border-[#0B6FDB]/30 flex items-center justify-center text-[#0B6FDB]">
              <Users className="w-4 h-4" />
            </div>
            <span>Scheduled today</span>
          </div>
          <div className="mt-3">
            <div className="text-3xl font-extrabold text-white font-['Sora']">
              {rosterData?.summary.scheduled_today ?? 13}
            </div>
            <div className="text-xs text-[#6979A0] mt-0.5">
              of {rosterData?.summary.total_employees ?? 14} people
            </div>
          </div>
        </div>

        {/* Card 2: Shifts Defined */}
        <div className="p-4 rounded-2xl bg-[#0B152E] border border-[#17264D] shadow-sm flex flex-col justify-between">
          <div className="flex items-center gap-2 text-xs font-medium text-[#8D9BB8]">
            <div className="w-7 h-7 rounded-lg bg-[#0BA6CC]/15 border border-[#0BA6CC]/30 flex items-center justify-center text-[#0BA6CC]">
              <CalendarIcon className="w-4 h-4" />
            </div>
            <span>Shifts defined</span>
          </div>
          <div className="mt-3">
            <div className="text-3xl font-extrabold text-white font-['Sora']">
              4
            </div>
            <div className="text-xs text-[#6979A0] mt-0.5 truncate">
              GEN · USP · USC · NGT
            </div>
          </div>
        </div>

        {/* Card 3: Swap Requests */}
        <div className="p-4 rounded-2xl bg-[#0B152E] border border-[#17264D] shadow-sm flex flex-col justify-between">
          <div className="flex items-center gap-2 text-xs font-medium text-[#8D9BB8]">
            <div className="w-7 h-7 rounded-lg bg-[#722ED1]/15 border border-[#722ED1]/30 flex items-center justify-center text-[#8540EB]">
              <ArrowLeftRight className="w-4 h-4" />
            </div>
            <span>Swap requests</span>
          </div>
          <div className="mt-3">
            <div className="text-3xl font-extrabold text-white font-['Sora']">
              {rosterData?.summary.swap_requests_count ?? 2}
            </div>
            <div className="text-xs text-[#6979A0] mt-0.5">
              Waiting for approval
            </div>
          </div>
        </div>

        {/* Card 4: On Leave This Week */}
        <div className="p-4 rounded-2xl bg-[#0B152E] border border-[#17264D] shadow-sm flex flex-col justify-between">
          <div className="flex items-center gap-2 text-xs font-medium text-[#8D9BB8]">
            <div className="w-7 h-7 rounded-lg bg-[#10B981]/15 border border-[#10B981]/30 flex items-center justify-center text-[#10B981]">
              <Clock className="w-4 h-4" />
            </div>
            <span>On leave this week</span>
          </div>
          <div className="mt-3">
            <div className="text-3xl font-extrabold text-white font-['Sora']">
              {rosterData?.summary.on_leave_this_week_count ?? 1}
            </div>
            <div className="text-xs text-[#6979A0] mt-0.5">
              From approved leave
            </div>
          </div>
        </div>
      </div>

      {/* Weekly Roster Table Container */}
      <div className="rounded-2xl bg-[#0B152E] border border-[#17264D] overflow-hidden shadow-xl">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse min-w-[760px]">
            {/* Table Header */}
            <thead>
              <tr className="border-b border-[#18274A] bg-[#0E1A38]/70 text-[11px] font-bold uppercase tracking-wider text-[#7E8EA8]">
                <th className="py-4 px-6 w-72">Employee</th>
                {rosterData?.employees[0]?.days.map((day) => (
                  <th key={day.date} className="py-4 px-3 text-center">
                    <div className="text-[11px] font-semibold text-[#8D9BB8]">
                      {day.day_name}
                    </div>
                    <div className="text-sm font-extrabold text-white mt-0.5">
                      {day.day_number}
                    </div>
                  </th>
                ))}
              </tr>
            </thead>

            {/* Table Body */}
            <tbody className="divide-y divide-[#152345] text-sm">
              {filteredEmployees.map((emp) => (
                <tr
                  key={emp.id}
                  className="hover:bg-white/[0.02] transition-colors group"
                >
                  {/* Employee Info Column */}
                  <td className="py-3.5 px-6">
                    <div className="flex items-center gap-3">
                      <div
                        className={`w-9 h-9 rounded-full flex items-center justify-center font-bold text-xs text-white shadow-xs shrink-0 ${getAvatarBg(
                          emp.avatar_color
                        )}`}
                      >
                        {emp.name
                          .split(' ')
                          .map((n) => n[0])
                          .join('')
                          .substring(0, 2)
                          .toUpperCase()}
                      </div>
                      <div className="min-w-0">
                        <div className="font-semibold text-white truncate text-sm">
                          {emp.name}
                        </div>
                        <div className="text-xs text-[#8D9BB8] truncate">
                          {emp.default_shift_name || 'General shift'}
                        </div>
                      </div>
                    </div>
                  </td>

                  {/* 7 Day Schedule Columns */}
                  {emp.days.map((day) => {
                    const shiftDisplay = day.is_off
                      ? 'OFF'
                      : day.shift_code
                      ? day.shift_code.split('-')[0]
                      : 'OFF';

                    return (
                      <td
                        key={day.date}
                        onClick={() => handleCellClick(emp, day)}
                        className="py-3 px-2.5 text-center cursor-pointer group/cell"
                      >
                        <div className="flex items-center justify-center">
                          <span
                            className={`w-14 py-1.5 rounded-lg text-xs font-bold tracking-wide transition-all transform group-hover/cell:scale-105 ${getShiftBadgeStyle(
                              day.shift_code,
                              day.is_off
                            )}`}
                          >
                            {shiftDisplay}
                          </span>
                        </div>
                      </td>
                    );
                  })}
                </tr>
              ))}

              {filteredEmployees.length === 0 && !isLoading && (
                <tr>
                  <td
                    colSpan={8}
                    className="py-12 text-center text-sm text-[#8D9BB8]"
                  >
                    No employees matching &ldquo;{searchQuery}&rdquo;
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Shift Assignment Modal */}
      <ShiftAssignModal
        isOpen={Boolean(selectedCell)}
        onClose={() => setSelectedCell(null)}
        employee={selectedCell?.employee || null}
        dayEntry={selectedCell?.dayEntry || null}
        availableShifts={shifts}
        onSave={handleSaveAssignment}
      />

      {/* Publish Confirmation Modal */}
      <PublishConfirmModal
        isOpen={isPublishModalOpen}
        onClose={() => setIsPublishModalOpen(false)}
        weekStart={rosterData?.week_start || currentWeekStart}
        weekEnd={rosterData?.week_end || ''}
        onConfirm={handlePublishRoster}
      />
    </div>
  );
};

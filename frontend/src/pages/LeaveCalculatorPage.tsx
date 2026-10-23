import React, { useState } from 'react';
import {
  Calculator,
  Calendar,
  AlertCircle,
  CheckCircle2,
  CalendarDays,
  Briefcase,
  Sun,
  ShieldCheck,
  Sparkles,
  Info,
} from 'lucide-react';
import type { LeaveCalculationResult } from '../types';
import { api } from '../services/api';

const LEAVE_TYPE_OPTIONS = [
  { code: 'CL', label: 'Casual Leave (CL)', desc: 'Excludes weekends & public holidays' },
  { code: 'SL', label: 'Sick Leave (SL)', desc: 'Excludes weekends & public holidays' },
  { code: 'EL', label: 'Earned Leave (EL)', desc: 'Privilege annual leave, working days counted' },
  { code: 'ML', label: 'Maternity Leave (ML)', desc: 'Calendar-day policy (consecutive days inclusive)' },
  { code: 'LOP', label: 'Loss of Pay (LOP)', desc: 'Unpaid leave, working days counted' },
];

export const LeaveCalculatorPage: React.FC = () => {
  // Form input states
  const [startDate, setStartDate] = useState<string>('2026-10-12');
  const [endDate, setEndDate] = useState<string>('2026-10-16');
  const [leaveTypeCode, setLeaveTypeCode] = useState<string>('CL');
  const [isHalfDay, setIsHalfDay] = useState<boolean>(false);

  // Response & submission states
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [result, setResult] = useState<LeaveCalculationResult | null>(null);

  const handleCalculate = async (e?: React.FormEvent) => {
    if (e) e.preventDefault();

    const start = startDate.trim();
    const end = endDate.trim();

    if (!start || !end) {
      setErrorMsg('Please select both a start date and an end date.');
      return;
    }

    if (end < start) {
      setErrorMsg(`End date (${end}) cannot be earlier than start date (${start}).`);
      return;
    }

    setIsLoading(true);
    setErrorMsg(null);

    try {
      const res = await api.countLeaveDays({
        start_date: start,
        end_date: end,
        leave_type_code: leaveTypeCode,
        is_half_day: isHalfDay,
      });

      if (res.data) {
        setResult(res.data);
      } else {
        setErrorMsg(res.error || 'Failed to calculate leave days from backend.');
      }
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : 'Network error reaching backend.');
    } finally {
      setIsLoading(false);
    }
  };

  const applyPreset = (start: string, end: string, code: string, halfDay: boolean) => {
    setStartDate(start);
    setEndDate(end);
    setLeaveTypeCode(code);
    setIsHalfDay(halfDay);
    setErrorMsg(null);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-1.5 text-[11px] font-bold text-[#0BA6CC] tracking-wider uppercase mb-1">
            <span className="w-1.5 h-1.5 rounded-full bg-[#0BA6CC]" />
            Attendance &amp; Leave · TIM-04
          </div>
          <h1 className="text-2xl font-bold text-white tracking-tight flex items-center gap-2.5">
            <Calculator className="w-6 h-6 text-[#0BA6CC]" />
            Leave Days Calculator
          </h1>
          <p className="text-xs text-[#889DC7] mt-1">
            Policy-aware calculation engine verifying calendar days, working days, weekends, holidays, and half-day deductions.
          </p>
        </div>

        {/* Quick Presets */}
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-[11px] font-medium text-[#647496]">Quick Test Presets:</span>
          <button
            type="button"
            onClick={() => applyPreset('2026-10-12', '2026-10-16', 'CL', false)}
            className="text-[11px] px-2.5 py-1 rounded-lg bg-[#0D1836] hover:bg-[#142347] border border-[#1A2C54] text-[#C9D3EE] font-medium transition-colors cursor-pointer"
          >
            Oct 12-16 (Mon-Fri)
          </button>
          <button
            type="button"
            onClick={() => applyPreset('2026-10-09', '2026-10-13', 'SL', false)}
            className="text-[11px] px-2.5 py-1 rounded-lg bg-[#0D1836] hover:bg-[#142347] border border-[#1A2C54] text-[#C9D3EE] font-medium transition-colors cursor-pointer"
          >
            Spanning Weekend
          </button>
          <button
            type="button"
            onClick={() => applyPreset('2026-10-14', '2026-10-14', 'CL', true)}
            className="text-[11px] px-2.5 py-1 rounded-lg bg-[#0D1836] hover:bg-[#142347] border border-[#1A2C54] text-[#C9D3EE] font-medium transition-colors cursor-pointer"
          >
            Half-Day Single Day
          </button>
          <button
            type="button"
            onClick={() => applyPreset('2026-10-01', '2026-10-14', 'ML', false)}
            className="text-[11px] px-2.5 py-1 rounded-lg bg-[#0D1836] hover:bg-[#142347] border border-[#1A2C54] text-[#C9D3EE] font-medium transition-colors cursor-pointer"
          >
            Maternity Leave (ML)
          </button>
        </div>
      </div>

      {/* Main Grid: Form Left, Instructions / Context Right */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Form Card */}
        <div className="lg:col-span-2 bg-[#070F22] border border-[#152347] rounded-2xl p-6 shadow-xl">
          <h2 className="text-sm font-semibold text-white mb-4 flex items-center gap-2">
            <CalendarDays className="w-4 h-4 text-[#0BA6CC]" />
            Request Parameters
          </h2>

          <form onSubmit={handleCalculate} className="space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              {/* Start Date */}
              <div>
                <label className="block text-xs font-semibold text-[#8D9BB8] mb-1.5">
                  Start Date <span className="text-red-400">*</span>
                </label>
                <input
                  type="date"
                  required
                  value={startDate}
                  onChange={(e) => setStartDate(e.target.value)}
                  className="w-full bg-[#040A15] border border-[#1B2A4A] rounded-xl px-3.5 py-2.5 text-xs text-white focus:outline-hidden focus:border-[#0BA6CC] transition-colors"
                />
              </div>

              {/* End Date */}
              <div>
                <label className="block text-xs font-semibold text-[#8D9BB8] mb-1.5">
                  End Date <span className="text-red-400">*</span>
                </label>
                <input
                  type="date"
                  required
                  value={endDate}
                  onChange={(e) => setEndDate(e.target.value)}
                  className="w-full bg-[#040A15] border border-[#1B2A4A] rounded-xl px-3.5 py-2.5 text-xs text-white focus:outline-hidden focus:border-[#0BA6CC] transition-colors"
                />
              </div>
            </div>

            {/* Leave Type Code Selection */}
            <div>
              <label className="block text-xs font-semibold text-[#8D9BB8] mb-1.5">
                Leave Type Code
              </label>
              <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
                {LEAVE_TYPE_OPTIONS.map((lt) => {
                  const isSelected = leaveTypeCode === lt.code;
                  return (
                    <button
                      key={lt.code}
                      type="button"
                      onClick={() => setLeaveTypeCode(lt.code)}
                      className={`text-left p-3 rounded-xl border transition-all cursor-pointer ${
                        isSelected
                          ? 'bg-[#0B6FDB]/20 border-[#0BA6CC] text-white shadow-xs'
                          : 'bg-[#040A15] border-[#1B2A4A] text-[#8D9BB8] hover:border-[#2D4373] hover:text-[#C9D3EE]'
                      }`}
                    >
                      <div className="flex items-center justify-between font-bold text-xs">
                        <span>{lt.code}</span>
                        {isSelected && <CheckCircle2 className="w-3.5 h-3.5 text-[#0BA6CC]" />}
                      </div>
                      <div className="text-[10px] text-[#647496] mt-0.5 truncate">{lt.label}</div>
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Half-Day Option Checkbox */}
            <div className="pt-2 border-t border-[#152347]">
              <label className="flex items-start gap-3 p-3 rounded-xl bg-[#040A15] border border-[#1B2A4A] cursor-pointer hover:border-[#2D4373] transition-colors">
                <input
                  type="checkbox"
                  checked={isHalfDay}
                  onChange={(e) => setIsHalfDay(e.target.checked)}
                  className="mt-0.5 w-4 h-4 rounded border-[#1B2A4A] text-[#0B6FDB] focus:ring-0 focus:ring-offset-0 cursor-pointer"
                />
                <div className="text-xs">
                  <span className="font-semibold text-white block">Half-Day Leave Option</span>
                  <span className="text-[11px] text-[#647496] block mt-0.5">
                    Applies 0.5 chargeable days on single-day leave requests (when start date equals end date).
                  </span>
                </div>
              </label>
            </div>

            {/* Error Message Alert */}
            {errorMsg && (
              <div className="p-3.5 rounded-xl bg-red-500/10 border border-red-500/30 text-red-400 text-xs flex items-center gap-2.5">
                <AlertCircle className="w-4 h-4 shrink-0 text-red-400" />
                <span>{errorMsg}</span>
              </div>
            )}

            {/* Submit Action */}
            <div className="pt-2 flex items-center justify-end">
              <button
                type="submit"
                disabled={isLoading}
                className="w-full sm:w-auto flex items-center justify-center gap-2 px-6 py-2.5 rounded-xl bg-gradient-to-r from-[#0B6FDB] to-[#0BA6CC] text-white font-semibold text-xs shadow-md shadow-[#0B6FDB]/30 hover:opacity-95 transition-opacity disabled:opacity-50 cursor-pointer"
              >
                <Calculator className="w-4 h-4" />
                {isLoading ? 'Counting Leave Days...' : 'Count Leave Days'}
              </button>
            </div>
          </form>
        </div>

        {/* Policy & Info Card */}
        <div className="bg-[#070F22] border border-[#152347] rounded-2xl p-6 shadow-xl flex flex-col justify-between">
          <div>
            <h2 className="text-sm font-semibold text-white mb-3 flex items-center gap-2">
              <Info className="w-4 h-4 text-[#0BA6CC]" />
              Policy Calculation Rules
            </h2>
            <div className="space-y-3 text-xs text-[#8D9BB8]">
              <div className="p-3 rounded-xl bg-[#040A15] border border-[#152347]">
                <b className="text-white block mb-0.5">Working Days (CL, SL, EL, LOP)</b>
                <span>Weekends (Saturday &amp; Sunday) and configured public holidays are excluded from chargeable count.</span>
              </div>
              <div className="p-3 rounded-xl bg-[#040A15] border border-[#152347]">
                <b className="text-white block mb-0.5">Calendar Days (ML)</b>
                <span>Maternity Leave counts all consecutive calendar days regardless of weekends or holidays.</span>
              </div>
              <div className="p-3 rounded-xl bg-[#040A15] border border-[#152347]">
                <b className="text-white block mb-0.5">Database Verified</b>
                <span>Public holidays are dynamically resolved from the PostgreSQL database table.</span>
              </div>
            </div>
          </div>

          <div className="pt-4 border-t border-[#152347] mt-4 flex items-center gap-2 text-[11px] text-[#647496]">
            <ShieldCheck className="w-4 h-4 text-emerald-400 shrink-0" />
            <span>Endpoint: <code>POST /api/leave/count-days</code></span>
          </div>
        </div>
      </div>

      {/* Results Section */}
      {result && (
        <div className="space-y-6 animate-fadeIn">
          {/* Summary Stat Cards */}
          <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-3.5">
            <div className="bg-[#070F22] border border-[#152347] rounded-xl p-4">
              <div className="text-[11px] font-semibold text-[#8D9BB8] flex items-center gap-1.5">
                <CalendarDays className="w-3.5 h-3.5 text-[#0BA6CC]" />
                Calendar Days
              </div>
              <div className="text-2xl font-bold text-white mt-1.5">{result.total_calendar_days}</div>
              <div className="text-[10px] text-[#647496] mt-0.5">Total span</div>
            </div>

            <div className="bg-[#0B6FDB]/10 border border-[#0B6FDB]/40 rounded-xl p-4 shadow-sm">
              <div className="text-[11px] font-semibold text-[#0BA6CC] flex items-center gap-1.5">
                <Sparkles className="w-3.5 h-3.5 text-[#0BA6CC]" />
                Chargeable Days
              </div>
              <div className="text-2xl font-bold text-[#0BA6CC] mt-1.5">{result.chargeable_days}</div>
              <div className="text-[10px] text-[#0BA6CC]/70 mt-0.5">Deducted from quota</div>
            </div>

            <div className="bg-[#070F22] border border-[#152347] rounded-xl p-4">
              <div className="text-[11px] font-semibold text-[#8D9BB8] flex items-center gap-1.5">
                <Briefcase className="w-3.5 h-3.5 text-blue-400" />
                Working Days
              </div>
              <div className="text-2xl font-bold text-white mt-1.5">{result.working_days}</div>
              <div className="text-[10px] text-[#647496] mt-0.5">Scheduled on-duty</div>
            </div>

            <div className="bg-[#070F22] border border-[#152347] rounded-xl p-4">
              <div className="text-[11px] font-semibold text-[#8D9BB8] flex items-center gap-1.5">
                <Sun className="w-3.5 h-3.5 text-amber-400" />
                Weekend Days
              </div>
              <div className="text-2xl font-bold text-white mt-1.5">{result.weekend_days}</div>
              <div className="text-[10px] text-[#647496] mt-0.5">Sat / Sun</div>
            </div>

            <div className="bg-[#070F22] border border-[#152347] rounded-xl p-4">
              <div className="text-[11px] font-semibold text-[#8D9BB8] flex items-center gap-1.5">
                <Calendar className="w-3.5 h-3.5 text-purple-400" />
                Holiday Days
              </div>
              <div className="text-2xl font-bold text-white mt-1.5">{result.holiday_days}</div>
              <div className="text-[10px] text-[#647496] mt-0.5">Public holidays</div>
            </div>
          </div>

          {/* Breakdown Table Card */}
          <div className="bg-[#070F22] border border-[#152347] rounded-2xl overflow-hidden shadow-xl">
            <div className="p-4 border-b border-[#152347] flex flex-col sm:flex-row sm:items-center justify-between gap-2">
              <div>
                <h3 className="text-sm font-bold text-white">Daily Calculation Breakdown</h3>
                <p className="text-[11px] text-[#8D9BB8] mt-0.5">
                  Applied Policy: <span className="text-[#0BA6CC] font-semibold">{result.policy_applied}</span>
                  {result.leave_type_code && ` · Type: ${result.leave_type_code}`}
                </p>
              </div>
              <span className="text-xs px-2.5 py-1 rounded-full bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 font-semibold self-start sm:self-auto">
                Total Chargeable: {result.chargeable_days} Day(s)
              </span>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs">
                <thead>
                  <tr className="border-b border-[#152347] bg-[#040A15]/80 text-[#8D9BB8] text-[11px] font-bold uppercase tracking-wider">
                    <th className="py-3 px-4">Date</th>
                    <th className="py-3 px-4">Day</th>
                    <th className="py-3 px-4">Day Classification</th>
                    <th className="py-3 px-4">Chargeable Impact</th>
                    <th className="py-3 px-4 text-right">Status</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[#152347]/60">
                  {result.breakdown.map((day) => {
                    let classification = 'Working Day';
                    let classBadgeColor = 'bg-blue-500/10 text-blue-400 border-blue-500/30';

                    if (day.is_holiday) {
                      classification = day.holiday_name ? `Public Holiday: ${day.holiday_name}` : 'Public Holiday';
                      classBadgeColor = 'bg-purple-500/10 text-purple-400 border-purple-500/30';
                    } else if (day.is_weekend) {
                      classification = 'Weekend';
                      classBadgeColor = 'bg-amber-500/10 text-amber-400 border-amber-500/30';
                    } else if (day.is_scheduled_off) {
                      classification = 'Scheduled Off';
                      classBadgeColor = 'bg-slate-500/10 text-slate-400 border-slate-500/30';
                    }

                    return (
                      <tr key={day.date} className="hover:bg-white/[0.02] transition-colors">
                        <td className="py-3 px-4 font-mono font-medium text-white">{day.date}</td>
                        <td className="py-3 px-4 font-semibold text-[#8D9BB8]">{day.day_name}</td>
                        <td className="py-3 px-4">
                          <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-[10px] font-semibold border ${classBadgeColor}`}>
                            {classification}
                          </span>
                        </td>
                        <td className="py-3 px-4">
                          {day.is_chargeable ? (
                            <span className="font-semibold text-emerald-400">
                              +{day.chargeable_value} day
                            </span>
                          ) : (
                            <span className="text-[#647496]">0.0 (Excluded)</span>
                          )}
                        </td>
                        <td className="py-3 px-4 text-right">
                          {day.is_chargeable ? (
                            <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/30">
                              <CheckCircle2 className="w-3 h-3" />
                              Chargeable
                            </span>
                          ) : (
                            <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-[10px] font-medium bg-[#142347] text-[#8D9BB8]">
                              Non-chargeable
                            </span>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

import React, { useState, useEffect } from 'react';
import { X, Clock, AlertCircle, Calendar, User, ShieldAlert, Sparkles, CheckCircle2 } from 'lucide-react';
import type { Employee, DailyAttendanceRecord, CreateCorrectionPayload, CorrectionType } from '../../types';
import { attendanceApi } from '../../services/attendanceApi';

interface AttendanceCorrectionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  employees: Employee[];
  initialEmployeeId?: number | null;
  initialDate?: string;
  showSuccessToast?: (msg: string) => void;
}

export const AttendanceCorrectionModal: React.FC<AttendanceCorrectionModalProps> = ({
  isOpen,
  onClose,
  onSuccess,
  employees,
  initialEmployeeId,
  initialDate,
  showSuccessToast,
}) => {
  const [employeeId, setEmployeeId] = useState<number>(0);
  const [date, setDate] = useState<string>('');
  const [correctionType, setCorrectionType] = useState<CorrectionType>('MISSED_CLOCK_IN');
  const [requestedTime, setRequestedTime] = useState<string>('09:00');
  const [reason, setReason] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
  const [formError, setFormError] = useState<string | null>(null);

  // Existing record check for context
  const [existingRecord, setExistingRecord] = useState<DailyAttendanceRecord | null>(null);
  const [isCheckingRecord, setIsCheckingRecord] = useState<boolean>(false);

  useEffect(() => {
    if (isOpen) {
      const defaultEmpId = initialEmployeeId || (employees.length > 0 ? employees[0].id : 0);
      setEmployeeId(defaultEmpId);
      const defaultDate = initialDate || new Date().toISOString().split('T')[0];
      setDate(defaultDate);
      setCorrectionType('MISSED_CLOCK_IN');
      setRequestedTime('09:00');
      setReason('');
      setFormError(null);
    }
  }, [isOpen, initialEmployeeId, initialDate, employees]);

  // Check existing attendance record whenever employee or date changes
  useEffect(() => {
    if (!isOpen || !employeeId || !date) return;
    let isMounted = true;
    setIsCheckingRecord(true);

    attendanceApi
      .fetchAttendance({ date })
      .then((res) => {
        if (!isMounted) return;
        setIsCheckingRecord(false);
        if (res.data) {
          const match = res.data.records.find((r) => r.employee_id === employeeId);
          setExistingRecord(match || null);
        }
      })
      .catch(() => {
        if (isMounted) setIsCheckingRecord(false);
      });

    return () => {
      isMounted = false;
    };
  }, [isOpen, employeeId, date]);

  // Adjust suggested default time when type changes
  const handleTypeChange = (newType: CorrectionType) => {
    setCorrectionType(newType);
    setFormError(null);
    if (newType === 'MISSED_CLOCK_IN' && requestedTime === '18:00') {
      setRequestedTime('09:00');
    } else if (newType === 'MISSED_CLOCK_OUT' && requestedTime === '09:00') {
      setRequestedTime('18:00');
    }
  };

  if (!isOpen) return null;

  const selectedEmployee = employees.find((e) => e.id === employeeId);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);

    // Validation
    if (!employeeId || employeeId === 0) {
      setFormError('Please select an employee.');
      return;
    }

    if (!date) {
      setFormError('Please select the attendance date for correction.');
      return;
    }

    if (!requestedTime) {
      setFormError('Please enter the requested clock-in or clock-out time.');
      return;
    }

    const cleanReason = reason.trim();
    if (!cleanReason) {
      setFormError('Please provide a reason for the attendance correction request.');
      return;
    }

    if (cleanReason.length < 5) {
      setFormError('Reason must be at least 5 characters long.');
      return;
    }

    // Logical consistency checks against existing attendance record
    if (correctionType === 'MISSED_CLOCK_IN' && existingRecord?.in_time) {
      setFormError(
        `A clock-in time is already recorded for this date (${existingRecord.in_time.slice(0, 5)}). Missed clock-in request is not applicable.`
      );
      return;
    }

    if (correctionType === 'MISSED_CLOCK_OUT') {
      if (!existingRecord?.in_time) {
        setFormError(
          'Cannot submit a missed clock-out request without an active clock-in on record for this date.'
        );
        return;
      }
      if (existingRecord?.out_time) {
        setFormError(
          `A clock-out time is already recorded for this date (${existingRecord.out_time.slice(0, 5)}). Missed clock-out request is not applicable.`
        );
        return;
      }
    }

    // Format requested_time to HH:MM:SS
    const formattedTime = requestedTime.length === 5 ? `${requestedTime}:00` : requestedTime;

    const payload: CreateCorrectionPayload = {
      employee_id: employeeId,
      attendance_date: date,
      correction_type: correctionType,
      requested_time: formattedTime,
      reason: cleanReason,
    };

    setIsSubmitting(true);
    try {
      await attendanceApi.createCorrection(payload);
      if (showSuccessToast) {
        showSuccessToast('Attendance correction request submitted for manager review.');
      }
      onSuccess();
      onClose();
    } catch (err: unknown) {
      setFormError(err instanceof Error ? err.message : 'Failed to submit correction request.');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs animate-fadeIn">
      <div className="relative w-full max-w-lg bg-[#0A1326] border border-[#16274E] rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        {/* Modal Header */}
        <div className="flex items-center justify-between p-5 border-b border-[#16274E] bg-gradient-to-r from-[#0A1326] via-[#0E1B38] to-[#0A1326]">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-[#0B6FDB] to-[#38BDF8] flex items-center justify-center text-white shadow-md shadow-[#0B6FDB]/20">
              <Clock className="w-5 h-5" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h3 className="text-base font-bold text-white tracking-tight">
                  Request Attendance Correction
                </h3>
                <span className="inline-flex items-center gap-1 text-[10px] font-semibold tracking-wider uppercase px-2 py-0.5 rounded-full bg-[#0B6FDB]/20 text-[#38BDF8] border border-[#0B6FDB]/30">
                  <Sparkles className="w-2.5 h-2.5" /> Workflow
                </span>
              </div>
              <p className="text-xs text-[#889DC7]">
                Submit a request for a missed clock-in or clock-out
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1.5 text-[#889DC7] hover:text-white rounded-lg hover:bg-[#16274E] transition-colors cursor-pointer"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Demo Notice Banner */}
        <div className="bg-[#78350F]/20 border-b border-[#B45309]/30 px-5 py-2.5 flex items-center gap-2.5 text-[11px] text-[#FBBF24]">
          <ShieldAlert className="w-4 h-4 text-[#F59E0B] shrink-0" />
          <span>
            <strong>Demo / Prototype Workflow:</strong> Employee identity is selected manually. In production, this request would automatically bind to the authenticated employee session.
          </span>
        </div>

        {/* Form Body */}
        <form onSubmit={handleSubmit} className="p-5 space-y-4 overflow-y-auto flex-1">
          {/* Error Message */}
          {formError && (
            <div className="p-3 rounded-xl bg-[#3B1219] border border-[#771D2D] text-xs text-[#F87171] flex items-start gap-2.5 animate-shake">
              <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
              <span>{formError}</span>
            </div>
          )}

          {/* Employee Selector */}
          <div>
            <label htmlFor="corr-employee" className="block text-xs font-semibold text-[#889DC7] mb-1.5">
              Employee <span className="text-[#F87171]">*</span>
            </label>
            <div className="relative">
              <select
                id="corr-employee"
                value={employeeId}
                onChange={(e) => setEmployeeId(Number(e.target.value))}
                className="w-full bg-[#070D1A] border border-[#16274E] focus:border-[#38BDF8] text-white text-xs rounded-xl px-3 py-2.5 appearance-none focus:outline-hidden cursor-pointer"
              >
                {employees.map((emp) => (
                  <option key={emp.id} value={emp.id} className="bg-[#0A1326] text-white">
                    {emp.name} ({emp.department} · {emp.role})
                  </option>
                ))}
              </select>
              <User className="w-4 h-4 text-[#889DC7] absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none" />
            </div>
          </div>

          {/* Attendance Date */}
          <div>
            <label htmlFor="corr-date" className="block text-xs font-semibold text-[#889DC7] mb-1.5">
              Attendance Date <span className="text-[#F87171]">*</span>
            </label>
            <div className="relative">
              <input
                id="corr-date"
                type="date"
                value={date}
                max={new Date().toISOString().split('T')[0]}
                onChange={(e) => setDate(e.target.value)}
                required
                className="w-full bg-[#070D1A] border border-[#16274E] focus:border-[#38BDF8] text-white text-xs rounded-xl px-3 py-2.5 focus:outline-hidden"
              />
              <Calendar className="w-4 h-4 text-[#889DC7] absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none" />
            </div>
          </div>

          {/* Current Attendance Status Context Box */}
          <div className="p-3 rounded-xl bg-[#070D1A]/70 border border-[#16274E] text-xs">
            <div className="text-[11px] font-semibold text-[#889DC7] uppercase tracking-wider mb-1 flex items-center justify-between">
              <span>Existing Attendance Record on Date:</span>
              {isCheckingRecord && <span className="text-[#38BDF8] animate-pulse">Checking...</span>}
            </div>
            {existingRecord ? (
              <div className="grid grid-cols-3 gap-2 mt-2 pt-2 border-t border-[#16274E]/60 text-center">
                <div>
                  <span className="text-[10px] text-[#5B6787] block">Clock In</span>
                  <span className="font-mono font-semibold text-white">
                    {existingRecord.in_time ? existingRecord.in_time.slice(0, 5) : 'None'}
                  </span>
                </div>
                <div>
                  <span className="text-[10px] text-[#5B6787] block">Clock Out</span>
                  <span className="font-mono font-semibold text-white">
                    {existingRecord.out_time ? existingRecord.out_time.slice(0, 5) : 'None'}
                  </span>
                </div>
                <div>
                  <span className="text-[10px] text-[#5B6787] block">Status</span>
                  <span className="font-semibold text-[#38BDF8]">
                    {existingRecord.entry_status || existingRecord.attendance_status}
                  </span>
                </div>
              </div>
            ) : (
              <p className="text-[11px] text-[#5B6787] italic mt-1">
                No attendance entry currently exists for this date. Submitting a missed clock-in will create a new regularised entry upon approval.
              </p>
            )}
          </div>

          {/* Correction Type Selection */}
          <div>
            <label className="block text-xs font-semibold text-[#889DC7] mb-1.5">
              Correction Type <span className="text-[#F87171]">*</span>
            </label>
            <div className="grid grid-cols-2 gap-2">
              <button
                type="button"
                onClick={() => handleTypeChange('MISSED_CLOCK_IN')}
                className={`flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl text-xs font-semibold border transition-all cursor-pointer ${
                  correctionType === 'MISSED_CLOCK_IN'
                    ? 'bg-[#0B6FDB]/20 border-[#0B6FDB] text-white shadow-xs'
                    : 'bg-[#070D1A] border-[#16274E] text-[#889DC7] hover:border-[#1E3466]'
                }`}
              >
                <span className={`w-2 h-2 rounded-full ${correctionType === 'MISSED_CLOCK_IN' ? 'bg-[#38BDF8]' : 'bg-[#5B6787]'}`} />
                <span>Missed Clock-in</span>
              </button>

              <button
                type="button"
                onClick={() => handleTypeChange('MISSED_CLOCK_OUT')}
                className={`flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl text-xs font-semibold border transition-all cursor-pointer ${
                  correctionType === 'MISSED_CLOCK_OUT'
                    ? 'bg-[#0B6FDB]/20 border-[#0B6FDB] text-white shadow-xs'
                    : 'bg-[#070D1A] border-[#16274E] text-[#889DC7] hover:border-[#1E3466]'
                }`}
              >
                <span className={`w-2 h-2 rounded-full ${correctionType === 'MISSED_CLOCK_OUT' ? 'bg-[#38BDF8]' : 'bg-[#5B6787]'}`} />
                <span>Missed Clock-out</span>
              </button>
            </div>
          </div>

          {/* Requested Time */}
          <div>
            <label htmlFor="corr-time" className="block text-xs font-semibold text-[#889DC7] mb-1.5">
              Requested {correctionType === 'MISSED_CLOCK_IN' ? 'Clock-in' : 'Clock-out'} Time <span className="text-[#F87171]">*</span>
            </label>
            <div className="relative">
              <input
                id="corr-time"
                type="time"
                value={requestedTime}
                onChange={(e) => setRequestedTime(e.target.value)}
                required
                className="w-full bg-[#070D1A] border border-[#16274E] focus:border-[#38BDF8] text-white text-xs rounded-xl px-3 py-2.5 focus:outline-hidden font-mono"
              />
              <Clock className="w-4 h-4 text-[#889DC7] absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none" />
            </div>
            <p className="text-[10px] text-[#5B6787] mt-1">
              Enter the actual time you arrived or departed on this date.
            </p>
          </div>

          {/* Reason for Correction */}
          <div>
            <label htmlFor="corr-reason" className="block text-xs font-semibold text-[#889DC7] mb-1.5">
              Reason for Correction <span className="text-[#F87171]">*</span>
            </label>
            <textarea
              id="corr-reason"
              rows={3}
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="e.g. Card scanner was offline at main lobby entrance, forgot badge, biometric sync delay..."
              required
              className="w-full bg-[#070D1A] border border-[#16274E] focus:border-[#38BDF8] text-white text-xs rounded-xl p-3 focus:outline-hidden placeholder-[#5B6787] resize-none"
            />
            <div className="flex justify-between items-center text-[10px] text-[#5B6787] mt-1">
              <span>Must provide a clear, auditable explanation</span>
              <span>{reason.length} chars</span>
            </div>
          </div>

          {/* Notice about approval workflow */}
          <div className="p-3 rounded-xl bg-[#0D1833] border border-[#1E3466] text-[11px] text-[#889DC7] flex items-start gap-2">
            <CheckCircle2 className="w-4 h-4 text-[#38BDF8] shrink-0 mt-0.5" />
            <span>
              Submitting this form creates a <strong>PENDING</strong> correction request. The attendance record will only be updated after an HR manager or supervisor approves the request.
            </span>
          </div>

          {/* Form Actions */}
          <div className="flex items-center justify-end gap-3 pt-3 border-t border-[#16274E]">
            <button
              type="button"
              onClick={onClose}
              disabled={isSubmitting}
              className="px-4 py-2 rounded-xl text-xs font-semibold text-[#889DC7] hover:text-white hover:bg-[#16274E] transition-colors cursor-pointer disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="flex items-center gap-2 px-5 py-2 rounded-xl text-xs font-semibold bg-[#0B6FDB] hover:bg-[#0957B0] text-white transition-all shadow-md shadow-[#0B6FDB]/25 cursor-pointer disabled:opacity-50"
            >
              {isSubmitting ? (
                <>
                  <div className="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin" />
                  <span>Submitting...</span>
                </>
              ) : (
                <span>Submit Request</span>
              )}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

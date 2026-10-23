import React, { useState, useEffect } from 'react';
import { X, Clock, AlertCircle } from 'lucide-react';
import type {
  DailyAttendanceRecord,
  Employee,
  CreateAttendancePayload,
  UpdateAttendancePayload,
} from '../../types';

interface AttendanceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (
    payload: CreateAttendancePayload | UpdateAttendancePayload,
    isEdit: boolean,
    id?: number
  ) => Promise<void>;
  editingRecord: DailyAttendanceRecord | null;
  employees: Employee[];
}

export const AttendanceModal: React.FC<AttendanceModalProps> = ({
  isOpen,
  onClose,
  onSave,
  editingRecord,
  employees,
}) => {
  const [employeeId, setEmployeeId] = useState<number>(0);
  const [date, setDate] = useState<string>('');
  const [attendanceStatus, setAttendanceStatus] = useState<string>('PRESENT');
  const [inTime, setInTime] = useState<string>('09:00');
  const [outTime, setOutTime] = useState<string>('18:00');
  const [entryStatus, setEntryStatus] = useState<string>('RECORDED');
  const [notes, setNotes] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (editingRecord) {
      setEmployeeId(editingRecord.employee_id);
      setDate(editingRecord.attendance_date);
      setAttendanceStatus(editingRecord.attendance_status);
      setInTime(editingRecord.in_time ? editingRecord.in_time.slice(0, 5) : '09:00');
      setOutTime(editingRecord.out_time ? editingRecord.out_time.slice(0, 5) : '18:00');
      setEntryStatus(editingRecord.entry_status);
      setNotes(editingRecord.notes || '');
    } else {
      setEmployeeId(employees.length > 0 ? employees[0].id : 0);
      const today = new Date().toISOString().split('T')[0];
      setDate(today);
      setAttendanceStatus('PRESENT');
      setInTime('09:00');
      setOutTime('18:00');
      setEntryStatus('RECORDED');
      setNotes('');
    }
    setErrorMessage(null);
  }, [editingRecord, employees, isOpen]);

  if (!isOpen) return null;

  // Live calculation of preview worked hours
  const calculateWorkedHoursPreview = (): { hours: number; isOvernight: boolean } => {
    if (attendanceStatus === 'ABSENT' || attendanceStatus === 'ON_LEAVE') {
      return { hours: 0, isOvernight: false };
    }
    if (attendanceStatus === 'HALF_DAY' && (!inTime || !outTime)) {
      return { hours: 4.0, isOvernight: false };
    }
    if (!inTime || !outTime) {
      return { hours: 0, isOvernight: false };
    }

    const inParts = inTime.split(':').map(Number);
    const outParts = outTime.split(':').map(Number);
    if (inParts.length < 2 || outParts.length < 2 || isNaN(inParts[0]) || isNaN(outParts[0])) {
      return { hours: 0, isOvernight: false };
    }

    const inMins = inParts[0] * 60 + inParts[1];
    const outMins = outParts[0] * 60 + outParts[1];

    if (outMins >= inMins) {
      return { hours: (outMins - inMins) / 60, isOvernight: false };
    } else {
      // Overnight shift
      return { hours: (24 * 60 - inMins + outMins) / 60, isOvernight: true };
    }
  };

  const preview = calculateWorkedHoursPreview();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMessage(null);

    if (!editingRecord && (!employeeId || employeeId === 0)) {
      setErrorMessage('Please select an employee.');
      return;
    }

    if (!date) {
      setErrorMessage('Please select a valid date.');
      return;
    }

    setIsSubmitting(true);
    try {
      if (editingRecord) {
        const payload: UpdateAttendancePayload = {
          attendance_status: attendanceStatus,
          in_time:
            attendanceStatus === 'ABSENT' || attendanceStatus === 'ON_LEAVE'
              ? null
              : `${inTime}:00`,
          out_time:
            attendanceStatus === 'ABSENT' || attendanceStatus === 'ON_LEAVE'
              ? null
              : `${outTime}:00`,
          entry_status: entryStatus,
          notes: notes.trim() || null,
        };
        await onSave(payload, true, editingRecord.id);
      } else {
        const payload: CreateAttendancePayload = {
          employee_id: employeeId,
          date,
          attendance_status: attendanceStatus,
          in_time:
            attendanceStatus === 'ABSENT' || attendanceStatus === 'ON_LEAVE'
              ? null
              : `${inTime}:00`,
          out_time:
            attendanceStatus === 'ABSENT' || attendanceStatus === 'ON_LEAVE'
              ? null
              : `${outTime}:00`,
          entry_status: entryStatus,
          notes: notes.trim() || null,
        };
        await onSave(payload, false);
      }
      onClose();
    } catch (err: unknown) {
      setErrorMessage(err instanceof Error ? err.message : 'Failed to save attendance record');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/70 backdrop-blur-xs">
      <div className="bg-[#0B1528] border border-[#16274E] rounded-2xl w-full max-w-lg shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[#16274E] bg-[#081020]">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-xl bg-[#0D2044] border border-[#1A3870] flex items-center justify-center text-[#38BDF8]">
              <Clock className="w-4 h-4" />
            </div>
            <div>
              <h3 className="text-sm font-semibold text-white">
                {editingRecord ? 'Edit Attendance Entry' : 'Add Attendance Entry'}
              </h3>
              <p className="text-[11px] text-[#546894]">
                One entry per employee per day
              </p>
            </div>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="p-1.5 rounded-lg text-[#546894] hover:text-white hover:bg-[#16274E] transition-colors cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Form Body */}
        <form onSubmit={handleSubmit} className="p-6 space-y-4 overflow-y-auto">
          {errorMessage && (
            <div className="p-3 rounded-xl bg-[#3B1219] border border-[#771D2D] text-xs text-[#F87171] flex items-center gap-2">
              <AlertCircle className="w-4 h-4 shrink-0" />
              <span>{errorMessage}</span>
            </div>
          )}

          {/* Employee Selection */}
          <div>
            <label className="block text-xs font-medium text-[#889DC7] mb-1.5">
              Employee <span className="text-rose-400">*</span>
            </label>
            {editingRecord ? (
              <input
                type="text"
                disabled
                value={`${editingRecord.employee_name} (${editingRecord.employee_department})`}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#070E1C] border border-[#16274E] text-[#6D7D9F] cursor-not-allowed"
              />
            ) : (
              <select
                value={employeeId}
                onChange={(e) => setEmployeeId(Number(e.target.value))}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#0E1A35] border border-[#1C3262] text-white focus:outline-hidden focus:border-[#0B6FDB] cursor-pointer"
                required
              >
                {employees.map((emp) => (
                  <option key={emp.id} value={emp.id}>
                    {emp.name} — {emp.department} ({emp.email})
                  </option>
                ))}
              </select>
            )}
          </div>

          {/* Date */}
          <div>
            <label className="block text-xs font-medium text-[#889DC7] mb-1.5">
              Date <span className="text-rose-400">*</span>
            </label>
            <input
              type="date"
              disabled={!!editingRecord}
              value={date}
              onChange={(e) => setDate(e.target.value)}
              className={`w-full px-3 py-2 rounded-xl text-xs border text-white focus:outline-hidden ${
                editingRecord
                  ? 'bg-[#070E1C] border-[#16274E] text-[#6D7D9F] cursor-not-allowed'
                  : 'bg-[#0E1A35] border-[#1C3262] focus:border-[#0B6FDB]'
              }`}
              required
            />
          </div>

          {/* Attendance Status & Entry Status Grid */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-medium text-[#889DC7] mb-1.5">
                Attendance Status <span className="text-rose-400">*</span>
              </label>
              <select
                value={attendanceStatus}
                onChange={(e) => setAttendanceStatus(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#0E1A35] border border-[#1C3262] text-white focus:outline-hidden focus:border-[#0B6FDB] cursor-pointer"
              >
                <option value="PRESENT">Present</option>
                <option value="ABSENT">Absent</option>
                <option value="HALF_DAY">Half day</option>
                <option value="ON_LEAVE">On leave</option>
              </select>
            </div>

            <div>
              <label className="block text-xs font-medium text-[#889DC7] mb-1.5">
                Entry Status
              </label>
              <select
                value={entryStatus}
                onChange={(e) => setEntryStatus(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#0E1A35] border border-[#1C3262] text-white focus:outline-hidden focus:border-[#0B6FDB] cursor-pointer"
              >
                <option value="RECORDED">Recorded</option>
                <option value="PENDING">Pending</option>
                <option value="REGULARISED">Regularised</option>
              </select>
            </div>
          </div>

          {/* Clock In / Out Times (only if Present or Half Day) */}
          {attendanceStatus !== 'ABSENT' && attendanceStatus !== 'ON_LEAVE' && (
            <div className="p-3.5 rounded-xl bg-[#081022] border border-[#16274E] space-y-3">
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-medium text-[#889DC7] mb-1">
                    In Time
                  </label>
                  <input
                    type="time"
                    value={inTime}
                    onChange={(e) => setInTime(e.target.value)}
                    className="w-full px-3 py-2 rounded-xl text-xs bg-[#0E1A35] border border-[#1C3262] text-white focus:outline-hidden focus:border-[#0B6FDB]"
                  />
                </div>
                <div>
                  <label className="block text-xs font-medium text-[#889DC7] mb-1">
                    Out Time
                  </label>
                  <input
                    type="time"
                    value={outTime}
                    onChange={(e) => setOutTime(e.target.value)}
                    className="w-full px-3 py-2 rounded-xl text-xs bg-[#0E1A35] border border-[#1C3262] text-white focus:outline-hidden focus:border-[#0B6FDB]"
                  />
                </div>
              </div>

              {/* Calculated worked hours banner */}
              <div className="flex items-center justify-between text-xs px-3 py-2 rounded-lg bg-[#0E1A38] text-[#889DC7]">
                <span>Calculated Worked Hours:</span>
                <span className="font-bold text-white">
                  {preview.hours.toFixed(1)} hrs
                  {preview.isOvernight && (
                    <span className="ml-1 text-[10px] text-[#38BDF8] font-medium">
                      (Overnight shift)
                    </span>
                  )}
                </span>
              </div>
            </div>
          )}

          {/* Notes */}
          <div>
            <label className="block text-xs font-medium text-[#889DC7] mb-1.5">
              Notes / Remarks (Optional)
            </label>
            <textarea
              rows={2}
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="e.g. Regular shift, approved half day, regularised by manager..."
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#0E1A35] border border-[#1C3262] text-white placeholder-[#546894] focus:outline-hidden focus:border-[#0B6FDB] resize-none"
            />
          </div>

          {/* Action buttons */}
          <div className="flex items-center justify-end gap-3 pt-3 border-t border-[#16274E]">
            <button
              type="button"
              onClick={onClose}
              disabled={isSubmitting}
              className="px-4 py-2 rounded-xl text-xs font-medium text-[#889DC7] hover:text-white hover:bg-[#16274E] transition-colors cursor-pointer"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="px-5 py-2 rounded-xl text-xs font-semibold bg-[#0B6FDB] hover:bg-[#0957B0] text-white transition-all shadow-md shadow-[#0B6FDB]/20 cursor-pointer disabled:opacity-50"
            >
              {isSubmitting ? 'Saving...' : editingRecord ? 'Update Entry' : 'Save Entry'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

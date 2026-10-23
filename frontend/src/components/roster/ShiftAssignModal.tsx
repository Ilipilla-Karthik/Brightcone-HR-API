import React, { useState } from 'react';
import { X, Clock, Calendar, Check, AlertCircle } from 'lucide-react';
import type { Shift, EmployeeRosterRow, DayRosterEntry } from '../../types';

interface ShiftAssignModalProps {
  isOpen: boolean;
  onClose: () => void;
  employee: EmployeeRosterRow | null;
  dayEntry: DayRosterEntry | null;
  availableShifts: Shift[];
  onSave: (shiftId: number | null, isOff: boolean, notes?: string) => Promise<void>;
}

export const ShiftAssignModal: React.FC<ShiftAssignModalProps> = ({
  isOpen,
  onClose,
  employee,
  dayEntry,
  availableShifts,
  onSave,
}) => {
  if (!isOpen || !employee || !dayEntry) return null;

  const [selectedShiftId, setSelectedShiftId] = useState<number | null>(
    dayEntry.is_off ? null : dayEntry.shift_id
  );
  const [isOff, setIsOff] = useState<boolean>(dayEntry.is_off);
  const [notes, setNotes] = useState<string>(dayEntry.notes || '');
  const [isSaving, setIsSaving] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const handleSelectShift = (shiftId: number) => {
    setSelectedShiftId(shiftId);
    setIsOff(false);
  };

  const handleSelectOff = () => {
    setSelectedShiftId(null);
    setIsOff(true);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSaving(true);
    setError(null);
    try {
      await onSave(selectedShiftId, isOff, notes.trim() || undefined);
      onClose();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to save shift assignment');
    } finally {
      setIsSaving(false);
    }
  };

  const getShiftBadgeColor = (code: string) => {
    if (code.startsWith('GEN')) return 'bg-[#0B6FDB] text-white';
    if (code.startsWith('USP')) return 'bg-[#722ED1] text-white';
    if (code.startsWith('USC')) return 'bg-[#4F46E5] text-white';
    if (code.startsWith('NGT')) return 'bg-[#0891B2] text-white';
    if (code.startsWith('MOR')) return 'bg-[#059669] text-white';
    return 'bg-[#2563EB] text-white';
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs animate-in fade-in">
      <div className="relative w-full max-w-md bg-[#0B152E] border border-[#1D2B52] rounded-2xl shadow-2xl overflow-hidden text-[#E2E8F0]">
        {/* Modal Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[#18274A] bg-[#0E1A38]">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-full bg-gradient-to-tr from-[#722ED1] to-[#0B6FDB] flex items-center justify-center font-bold text-sm text-white shadow-inner">
              {employee.name
                .split(' ')
                .map((n) => n[0])
                .join('')
                .substring(0, 2)
                .toUpperCase()}
            </div>
            <div>
              <h3 className="font-semibold text-white text-base leading-tight">
                {employee.name}
              </h3>
              <p className="text-xs text-[#8D9BB8] flex items-center gap-1.5 mt-0.5">
                <Calendar className="w-3.5 h-3.5 text-[#0BA6CC]" />
                <span>
                  {dayEntry.day_name}, {dayEntry.date}
                </span>
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg hover:bg-white/10 text-[#8D9BB8] hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Modal Form */}
        <form onSubmit={handleSubmit} className="p-6 space-y-5">
          {error && (
            <div className="flex items-center gap-2 p-3 text-xs bg-red-500/10 border border-red-500/30 text-red-400 rounded-xl">
              <AlertCircle className="w-4 h-4 shrink-0" />
              <span>{error}</span>
            </div>
          )}

          <div>
            <label className="block text-xs font-semibold text-[#8D9BB8] uppercase tracking-wider mb-2.5">
              Select Shift or Day Off
            </label>
            <div className="grid grid-cols-1 gap-2 max-h-56 overflow-y-auto pr-1">
              {/* Day Off Option */}
              <div
                onClick={handleSelectOff}
                className={`flex items-center justify-between p-3 rounded-xl border cursor-pointer transition-all ${
                  isOff
                    ? 'bg-[#142347] border-[#0B6FDB] shadow-md shadow-[#0B6FDB]/20 text-white'
                    : 'bg-[#091226] border-[#18274A] text-[#8D9BB8] hover:border-[#223563] hover:text-white'
                }`}
              >
                <div className="flex items-center gap-3">
                  <span className="px-2.5 py-1 rounded-md text-xs font-bold bg-[#172545] text-[#8D9BB8] border border-[#223563]">
                    OFF
                  </span>
                  <div>
                    <div className="text-sm font-medium">Day Off (Rest Day)</div>
                    <div className="text-[11px] text-[#6979A0]">No shift scheduled</div>
                  </div>
                </div>
                {isOff && <Check className="w-4 h-4 text-[#0BA6CC]" />}
              </div>

              {/* Defined Shifts */}
              {availableShifts.map((shift) => {
                const isSelected = !isOff && selectedShiftId === shift.id;
                return (
                  <div
                    key={shift.id}
                    onClick={() => handleSelectShift(shift.id)}
                    className={`flex items-center justify-between p-3 rounded-xl border cursor-pointer transition-all ${
                      isSelected
                        ? 'bg-[#142347] border-[#0B6FDB] shadow-md shadow-[#0B6FDB]/20 text-white'
                        : 'bg-[#091226] border-[#18274A] text-[#8D9BB8] hover:border-[#223563] hover:text-white'
                    }`}
                  >
                    <div className="flex items-center gap-3">
                      <span
                        className={`px-2.5 py-1 rounded-md text-xs font-bold ${getShiftBadgeColor(
                          shift.code
                        )}`}
                      >
                        {shift.code.split('-')[0]}
                      </span>
                      <div>
                        <div className="text-sm font-medium text-white">{shift.name}</div>
                        <div className="text-[11px] text-[#8D9BB8] flex items-center gap-1.5">
                          <Clock className="w-3 h-3 text-[#0BA6CC]" />
                          <span>
                            {shift.start_time.substring(0, 5)} – {shift.end_time.substring(0, 5)}
                            {shift.is_overnight ? ' (+1 day)' : ''}
                          </span>
                        </div>
                      </div>
                    </div>
                    {isSelected && <Check className="w-4 h-4 text-[#0BA6CC]" />}
                  </div>
                );
              })}
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-[#8D9BB8] uppercase tracking-wider mb-1.5">
              Assignment Notes (Optional)
            </label>
            <input
              type="text"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="e.g. Swapped with Priya, Covering weekend support"
              className="w-full px-3.5 py-2.5 rounded-xl bg-[#091226] border border-[#18274A] text-white text-sm focus:outline-hidden focus:border-[#0B6FDB] transition-colors placeholder-[#4C5B82]"
            />
          </div>

          {/* Modal Actions */}
          <div className="flex items-center justify-end gap-3 pt-3 border-t border-[#18274A]">
            <button
              type="button"
              onClick={onClose}
              disabled={isSaving}
              className="px-4 py-2 rounded-xl text-xs font-semibold text-[#8D9BB8] hover:text-white hover:bg-white/5 transition-colors cursor-pointer"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSaving}
              className="px-5 py-2 rounded-xl text-xs font-semibold text-white bg-gradient-to-r from-[#0B6FDB] to-[#0BA6CC] hover:brightness-110 shadow-lg shadow-[#0B6FDB]/30 transition-all cursor-pointer flex items-center gap-2"
            >
              {isSaving ? 'Saving...' : 'Save Assignment'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

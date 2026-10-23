import React, { useState, useEffect } from 'react';
import type { Shift } from '../../types';
import { Modal } from '../common/Modal';
import { AlertCircle } from 'lucide-react';

export interface ShiftFormData {
  name: string;
  code: string;
  start_time: string;
  end_time: string;
  break_duration_minutes: number;
  grace_period_minutes: number;
  is_overnight: boolean;
  is_active: boolean;
}

interface ShiftModalProps {
  isOpen: boolean;
  onClose: () => void;
  editingShift: Shift | null;
  onSubmit: (data: ShiftFormData) => Promise<void>;
  errorMsg: string | null;
  isSubmitting: boolean;
}

export const ShiftModal: React.FC<ShiftModalProps> = ({
  isOpen,
  onClose,
  editingShift,
  onSubmit,
  errorMsg,
  isSubmitting,
}) => {
  const [name, setName] = useState('');
  const [code, setCode] = useState('');
  const [startTime, setStartTime] = useState('09:00');
  const [endTime, setEndTime] = useState('18:00');
  const [breakDuration, setBreakDuration] = useState('60');
  const [gracePeriod, setGracePeriod] = useState('15');
  const [isOvernight, setIsOvernight] = useState(false);
  const [isActive, setIsActive] = useState(true);

  useEffect(() => {
    if (editingShift) {
      setName(editingShift.name);
      setCode(editingShift.code);
      setStartTime(editingShift.start_time.slice(0, 5));
      setEndTime(editingShift.end_time.slice(0, 5));
      setBreakDuration(String(editingShift.break_duration_minutes));
      setGracePeriod(String(editingShift.grace_period_minutes));
      setIsOvernight(editingShift.is_overnight);
      setIsActive(editingShift.is_active);
    } else {
      setName('');
      setCode('');
      setStartTime('09:00');
      setEndTime('18:00');
      setBreakDuration('60');
      setGracePeriod('15');
      setIsOvernight(false);
      setIsActive(true);
    }
  }, [editingShift, isOpen]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await onSubmit({
      name: name.trim(),
      code: code.trim().toUpperCase(),
      start_time: startTime.length === 5 ? `${startTime}:00` : startTime,
      end_time: endTime.length === 5 ? `${endTime}:00` : endTime,
      break_duration_minutes: parseInt(breakDuration, 10) || 0,
      grace_period_minutes: parseInt(gracePeriod, 10) || 0,
      is_overnight: isOvernight,
      is_active: isActive,
    });
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={editingShift ? 'Edit Shift Definition' : 'Create New Shift Definition'}
      description={
        editingShift
          ? `Update shift timings and configuration for ${editingShift.code}.`
          : 'Add a master shift timing rule. Validated against PostgreSQL schema constraints.'
      }
    >
      <form onSubmit={handleSubmit} className="space-y-4">
        {errorMsg && (
          <div className="flex items-center gap-2 p-3 rounded-xl bg-[#FDE8EA] dark:bg-[#3D141B] text-[#D93D4A] text-xs">
            <AlertCircle className="w-4 h-4 flex-none" />
            <span>{errorMsg}</span>
          </div>
        )}

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Shift Name *
            </label>
            <input
              type="text"
              required
              placeholder="e.g. Morning Shift"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Shift Code * (Unique)
            </label>
            <input
              type="text"
              required
              placeholder="e.g. MOR-02"
              value={code}
              onChange={(e) => setCode(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs uppercase font-mono bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Start Time *
            </label>
            <input
              type="time"
              required
              value={startTime}
              onChange={(e) => setStartTime(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              End Time *
            </label>
            <input
              type="time"
              required
              value={endTime}
              onChange={(e) => setEndTime(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
          </div>
        </div>

        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Break Duration (Minutes)
            </label>
            <input
              type="number"
              min="0"
              value={breakDuration}
              onChange={(e) => setBreakDuration(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
          </div>
          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Grace Period (Minutes)
            </label>
            <input
              type="number"
              min="0"
              value={gracePeriod}
              onChange={(e) => setGracePeriod(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
          </div>
        </div>

        <div className="pt-2 border-t border-[#E3E8F2] dark:border-[#1D2B52] space-y-2">
          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={isOvernight}
              onChange={(e) => setIsOvernight(e.target.checked)}
              className="w-4 h-4 rounded text-[#0B6FDB] focus:ring-[#0B6FDB]"
            />
            <span className="text-xs font-medium text-[#0E1730] dark:text-[#E8EDFB]">
              Overnight Shift (Crosses midnight boundary into next calendar day)
            </span>
          </label>

          <label className="flex items-center gap-2 cursor-pointer select-none">
            <input
              type="checkbox"
              checked={isActive}
              onChange={(e) => setIsActive(e.target.checked)}
              className="w-4 h-4 rounded text-[#0B6FDB] focus:ring-[#0B6FDB]"
            />
            <span className="text-xs font-medium text-[#0E1730] dark:text-[#E8EDFB]">
              Active Shift (Available for employee rostering)
            </span>
          </label>
        </div>

        <div className="flex items-center justify-end gap-3 pt-4 border-t border-[#E3E8F2] dark:border-[#1D2B52]">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 rounded-xl text-xs font-semibold border border-[#E3E8F2] dark:border-[#1D2B52] hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSubmitting}
            className="px-4 py-2 rounded-xl text-xs font-semibold bg-[#0B6FDB] hover:bg-[#0B6FDB]/90 text-white shadow-sm transition-all disabled:opacity-50 cursor-pointer"
          >
            {isSubmitting ? 'Saving...' : editingShift ? 'Update Shift' : 'Create Shift'}
          </button>
        </div>
      </form>
    </Modal>
  );
};

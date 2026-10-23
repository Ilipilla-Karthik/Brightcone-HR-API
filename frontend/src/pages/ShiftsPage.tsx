import React, { useState } from 'react';
import type { Shift } from '../types';
import { StatCard } from '../components/common/StatCard';
import { ShiftModal, type ShiftFormData } from '../components/shifts/ShiftModal';
import { Clock, Moon, Plus, Sun, CheckCircle, XCircle, Pencil, Trash2 } from 'lucide-react';

interface ShiftsPageProps {
  shifts: Shift[];
  onAddShift: (shift: Omit<Shift, 'id' | 'pid' | 'created_at' | 'updated_at'>) => Promise<boolean>;
  onUpdateShift: (id: number, shift: Partial<Shift>) => Promise<boolean>;
  onDeleteShift: (id: number, name: string) => Promise<boolean>;
  searchQuery: string;
}

export const ShiftsPage: React.FC<ShiftsPageProps> = ({
  shifts,
  onAddShift,
  onUpdateShift,
  onDeleteShift,
  searchQuery,
}) => {
  const [filterActive, setFilterActive] = useState<'all' | 'active' | 'inactive'>('all');
  const [filterOvernight, setFilterOvernight] = useState<'all' | 'regular' | 'overnight'>('all');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingShift, setEditingShift] = useState<Shift | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Filter shifts
  const filteredShifts = shifts.filter((s) => {
    const matchesSearch =
      s.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      s.code.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesActive =
      filterActive === 'all'
        ? true
        : filterActive === 'active'
        ? s.is_active
        : !s.is_active;
    const matchesOvernight =
      filterOvernight === 'all'
        ? true
        : filterOvernight === 'regular'
        ? !s.is_overnight
        : s.is_overnight;
    return matchesSearch && matchesActive && matchesOvernight;
  });

  const activeCount = shifts.filter((s) => s.is_active).length;
  const overnightCount = shifts.filter((s) => s.is_overnight).length;
  const avgBreak = shifts.length
    ? Math.round(shifts.reduce((acc, s) => acc + s.break_duration_minutes, 0) / shifts.length)
    : 0;

  const handleOpenAddModal = () => {
    setEditingShift(null);
    setErrorMsg(null);
    setIsModalOpen(true);
  };

  const handleOpenEditModal = (shift: Shift) => {
    setEditingShift(shift);
    setErrorMsg(null);
    setIsModalOpen(true);
  };

  const handleDelete = async (shift: Shift) => {
    const confirmed = window.confirm(
      `Are you sure you want to delete shift "${shift.name}" (${shift.code})?\nThis action cannot be undone.`
    );
    if (!confirmed) return;

    try {
      await onDeleteShift(shift.id, shift.name);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete shift');
    }
  };

  const handleModalSubmit = async (data: ShiftFormData) => {
    setErrorMsg(null);
    setIsSubmitting(true);
    try {
      if (editingShift) {
        await onUpdateShift(editingShift.id, data);
      } else {
        await onAddShift(data);
      }
      setIsModalOpen(false);
      setEditingShift(null);
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : 'Operation failed');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h2 className="text-xl font-bold text-[#0E1730] dark:text-[#E8EDFB]">
            Shift Master Configuration
          </h2>
          <p className="text-xs text-[#5B6787] dark:text-[#9AA6C6] mt-0.5">
            Configure working hours, overnight rules, break durations, and grace periods for employee rosters.
          </p>
        </div>
        <button
          onClick={handleOpenAddModal}
          className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-[#0B6FDB] hover:bg-[#0B6FDB]/90 text-white font-semibold text-xs shadow-sm transition-all cursor-pointer flex-none"
        >
          <Plus className="w-4 h-4" />
          <span>Add New Shift</span>
        </button>
      </div>

      {/* Summary KPI Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Total Shifts"
          value={shifts.length}
          subtext="Defined schedule profiles"
          icon={Clock}
        />
        <StatCard
          label="Active Shifts"
          value={activeCount}
          subtext={`${Math.round((activeCount / (shifts.length || 1)) * 100)}% active in rosters`}
          icon={CheckCircle}
          badge={{ text: 'In Service', variant: 'ok' }}
        />
        <StatCard
          label="Overnight Shifts"
          value={overnightCount}
          subtext="Crossing midnight (00:00)"
          icon={Moon}
          badge={{ text: 'Overnight', variant: 'info' }}
        />
        <StatCard
          label="Avg Break Duration"
          value={`${avgBreak}m`}
          subtext="Allocated per work shift"
          icon={Sun}
        />
      </div>

      {/* Filter Bar */}
      <div className="flex flex-wrap items-center justify-between gap-3 p-4 rounded-2xl bg-white dark:bg-[#0A1224] border border-[#E3E8F2] dark:border-[#1D2B52]">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-xs font-semibold text-[#5B6787] dark:text-[#9AA6C6]">Status:</span>
          {(['all', 'active', 'inactive'] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => setFilterActive(mode)}
              className={`px-3 py-1.5 rounded-xl text-xs font-semibold capitalize transition-all cursor-pointer ${
                filterActive === mode
                  ? 'bg-[#0B6FDB] text-white shadow-xs'
                  : 'bg-[#F8FAFD] dark:bg-[#122042] text-[#5B6787] dark:text-[#9AA6C6] hover:bg-[#E3E8F2] dark:hover:bg-[#1D2B52]'
              }`}
            >
              {mode}
            </button>
          ))}
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <span className="text-xs font-semibold text-[#5B6787] dark:text-[#9AA6C6]">Timing:</span>
          {(['all', 'regular', 'overnight'] as const).map((mode) => (
            <button
              key={mode}
              onClick={() => setFilterOvernight(mode)}
              className={`px-3 py-1.5 rounded-xl text-xs font-semibold capitalize transition-all cursor-pointer ${
                filterOvernight === mode
                  ? 'bg-[#0B6FDB] text-white shadow-xs'
                  : 'bg-[#F8FAFD] dark:bg-[#122042] text-[#5B6787] dark:text-[#9AA6C6] hover:bg-[#E3E8F2] dark:hover:bg-[#1D2B52]'
              }`}
            >
              {mode}
            </button>
          ))}
        </div>
      </div>

      {/* Table Container */}
      <div className="rounded-2xl bg-white dark:bg-[#0A1224] border border-[#E3E8F2] dark:border-[#1D2B52] overflow-hidden shadow-xs">
        <div className="overflow-x-auto">
          <table className="w-full text-left border-collapse text-xs">
            <thead>
              <tr className="border-b border-[#E3E8F2] dark:border-[#1D2B52] bg-[#F8FAFD] dark:bg-[#122042]/70 text-[#5B6787] dark:text-[#9AA6C6]">
                <th className="py-3 px-4 font-semibold">Shift Name & Code</th>
                <th className="py-3 px-4 font-semibold">Hours (Start - End)</th>
                <th className="py-3 px-4 font-semibold">Classification</th>
                <th className="py-3 px-4 font-semibold">Break</th>
                <th className="py-3 px-4 font-semibold">Grace</th>
                <th className="py-3 px-4 font-semibold">Status</th>
                <th className="py-3 px-4 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#E3E8F2] dark:divide-[#1D2B52]">
              {filteredShifts.length === 0 ? (
                <tr>
                  <td colSpan={7} className="py-12 text-center text-[#5B6787] dark:text-[#9AA6C6]">
                    <div className="max-w-xs mx-auto space-y-2">
                      <Clock className="w-8 h-8 mx-auto text-[#9AA6C6] dark:text-[#5B6787]" />
                      <p className="font-semibold text-sm text-[#0E1730] dark:text-[#E8EDFB]">
                        No shifts match your filter
                      </p>
                      <p className="text-xs">
                        Try adjusting your search criteria or create a new shift configuration.
                      </p>
                    </div>
                  </td>
                </tr>
              ) : (
                filteredShifts.map((shift) => (
                  <tr
                    key={shift.id}
                    className="hover:bg-[#F8FAFD]/70 dark:hover:bg-[#122042]/50 transition-colors"
                  >
                    <td className="py-3.5 px-4">
                      <div className="font-semibold text-[#0E1730] dark:text-[#E8EDFB]">
                        {shift.name}
                      </div>
                      <span className="font-mono text-[11px] px-1.5 py-0.5 rounded bg-[#E3F0FF] dark:bg-[#16275A] text-[#0B6FDB]">
                        {shift.code}
                      </span>
                    </td>
                    <td className="py-3.5 px-4 font-mono text-xs">
                      {shift.start_time.slice(0, 5)} - {shift.end_time.slice(0, 5)}
                    </td>
                    <td className="py-3.5 px-4">
                      {shift.is_overnight ? (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-purple-50 dark:bg-purple-950/40 text-purple-600 dark:text-purple-300 border border-purple-200 dark:border-purple-800">
                          <Moon className="w-3 h-3" /> Overnight
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-50 dark:bg-amber-950/40 text-amber-600 dark:text-amber-300 border border-amber-200 dark:border-amber-800">
                          <Sun className="w-3 h-3" /> Day Shift
                        </span>
                      )}
                    </td>
                    <td className="py-3.5 px-4 text-[#5B6787] dark:text-[#9AA6C6]">
                      {shift.break_duration_minutes} mins
                    </td>
                    <td className="py-3.5 px-4 text-[#5B6787] dark:text-[#9AA6C6]">
                      {shift.grace_period_minutes} mins
                    </td>
                    <td className="py-3.5 px-4">
                      {shift.is_active ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-semibold bg-[#E3F6EE] dark:bg-[#0E3326] text-[#15A36B]">
                          <CheckCircle className="w-3 h-3" /> Active
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-semibold bg-[#FDE8EA] dark:bg-[#3D141B] text-[#D93D4A]">
                          <XCircle className="w-3 h-3" /> Inactive
                        </span>
                      )}
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <div className="inline-flex items-center justify-end gap-1.5">
                        <button
                          onClick={() => handleOpenEditModal(shift)}
                          title="Edit Shift"
                          className="p-1.5 rounded-lg text-[#5B6787] hover:text-[#0B6FDB] hover:bg-[#E3F0FF] dark:hover:bg-[#16275A] transition-colors cursor-pointer"
                        >
                          <Pencil className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleDelete(shift)}
                          title="Delete Shift"
                          className="p-1.5 rounded-lg text-[#5B6787] hover:text-[#D93D4A] hover:bg-[#FDE8EA] dark:hover:bg-[#3D141B] transition-colors cursor-pointer"
                        >
                          <Trash2 className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Add / Edit Shift Modal */}
      <ShiftModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        editingShift={editingShift}
        onSubmit={handleModalSubmit}
        errorMsg={errorMsg}
        isSubmitting={isSubmitting}
      />
    </div>
  );
};

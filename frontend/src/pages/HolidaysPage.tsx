import React, { useState } from 'react';
import type { Holiday } from '../types';

import { StatCard } from '../components/common/StatCard';
import { Modal } from '../components/common/Modal';
import { Calendar, Globe, MapPin, Plus, Repeat, AlertCircle, Pencil, Trash2 } from 'lucide-react';

interface HolidaysPageProps {
  holidays: Holiday[];
  onAddHoliday: (holiday: Omit<Holiday, 'id' | 'pid' | 'created_at' | 'updated_at'>) => Promise<boolean>;
  onUpdateHoliday: (id: number, holiday: Partial<Holiday>) => Promise<boolean>;
  onDeleteHoliday: (id: number, name: string) => Promise<boolean>;
  searchQuery: string;
}

export const HolidaysPage: React.FC<HolidaysPageProps> = ({
  holidays,
  onAddHoliday,
  onUpdateHoliday,
  onDeleteHoliday,
  searchQuery,
}) => {
  const [filterLocation, setFilterLocation] = useState<string>('all');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingHoliday, setEditingHoliday] = useState<Holiday | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Form State
  const [name, setName] = useState('');
  const [holidayDate, setHolidayDate] = useState('2026-05-01');
  const [locationType, setLocationType] = useState<'global' | 'branch'>('global');
  const [customLocation, setCustomLocation] = useState('Hyderabad');
  const [description, setDescription] = useState('');
  const [isRecurring, setIsRecurring] = useState(false);

  // Filter holidays
  const filteredHolidays = holidays.filter((h) => {
    const matchesSearch =
      h.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (h.description || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
      (h.applicable_location || 'global').toLowerCase().includes(searchQuery.toLowerCase());

    const matchesLocation =
      filterLocation === 'all'
        ? true
        : filterLocation === 'global'
        ? h.applicable_location === null || h.applicable_location === ''
        : h.applicable_location === filterLocation;

    return matchesSearch && matchesLocation;
  });

  const globalCount = holidays.filter((h) => !h.applicable_location).length;
  const recurringCount = holidays.filter((h) => h.is_recurring).length;
  const branchCount = holidays.filter((h) => !!h.applicable_location).length;

  const handleOpenAddModal = () => {
    setEditingHoliday(null);
    setName('');
    setHolidayDate(new Date().toISOString().split('T')[0]);
    setLocationType('global');
    setCustomLocation('Hyderabad');
    setDescription('');
    setIsRecurring(false);
    setErrorMsg(null);
    setIsModalOpen(true);
  };

  const handleOpenEditModal = (holiday: Holiday) => {
    setEditingHoliday(holiday);
    setName(holiday.name);
    setHolidayDate(holiday.holiday_date);
    setLocationType(holiday.applicable_location ? 'branch' : 'global');
    setCustomLocation(holiday.applicable_location || 'Hyderabad');
    setDescription(holiday.description || '');
    setIsRecurring(holiday.is_recurring);
    setErrorMsg(null);
    setIsModalOpen(true);
  };

  const handleDelete = async (holiday: Holiday) => {
    const locText = holiday.applicable_location ? `(${holiday.applicable_location})` : '(All Locations)';
    const confirmed = window.confirm(
      `Are you sure you want to delete holiday "${holiday.name}" on ${holiday.holiday_date} ${locText}?\nThis action cannot be undone.`
    );
    if (!confirmed) return;

    try {
      await onDeleteHoliday(holiday.id, holiday.name);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete holiday');
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMsg(null);

    if (!name.trim() || !holidayDate) {
      setErrorMsg('Holiday name and date are required.');
      return;
    }

    const loc = locationType === 'global' ? null : customLocation.trim() || null;

    // Check duplicate date + location
    const exists = holidays.some(
      (h) =>
        h.id !== editingHoliday?.id &&
        h.holiday_date === holidayDate &&
        (h.applicable_location || null) === (loc || null)
    );

    if (exists) {
      setErrorMsg(
        `A holiday on ${holidayDate} for location "${loc || 'All Locations'}" already exists (unique constraint).`
      );
      return;
    }

    setIsSubmitting(true);
    try {
      let ok = false;
      if (editingHoliday) {
        ok = await onUpdateHoliday(editingHoliday.id, {
          name: name.trim(),
          holiday_date: holidayDate,
          description: description.trim() || null,
          applicable_location: loc,
          is_recurring: isRecurring,
        });
      } else {
        ok = await onAddHoliday({
          name: name.trim(),
          holiday_date: holidayDate,
          description: description.trim() || null,
          applicable_location: loc,
          is_recurring: isRecurring,
        });
      }

      if (ok) {
        setIsModalOpen(false);
      }
    } catch (err) {
      setErrorMsg(err instanceof Error ? err.message : 'Operation failed');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header Action Bar */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-xl sm:text-2xl font-bold font-['Sora'] text-[#0E1730] dark:text-[#E8EDFB]">
            Holiday Master Calendar
          </h1>
          <p className="text-xs sm:text-sm text-[#5B6787] dark:text-[#8D97B0] mt-0.5">
            Maintain statutory, national, and state-specific holidays across all company work locations.
          </p>
        </div>
        <button
          onClick={handleOpenAddModal}
          className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-[#0B6FDB] hover:bg-[#0B6FDB]/90 text-white font-semibold text-xs sm:text-sm shadow-md shadow-[#0B6FDB]/20 transition-all cursor-pointer"
        >
          <Plus className="w-4 h-4" />
          <span>Add Holiday</span>
        </button>
      </div>

      {/* KPI Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Total Holidays"
          value={holidays.length}
          subtext="Configured on company calendar"
          icon={Calendar}
        />
        <StatCard
          label="Global / All Sites"
          value={globalCount}
          subtext="Mandatory nationwide holidays"
          icon={Globe}
          badge={{ text: 'Company-Wide', variant: 'ok' }}
        />
        <StatCard
          label="Regional State Holidays"
          value={branchCount}
          subtext="Location specific branches"
          icon={MapPin}
          badge={{ text: 'State Specific', variant: 'info' }}
        />
        <StatCard
          label="Annual Recurring"
          value={recurringCount}
          subtext="Carried over automatically"
          icon={Repeat}
        />
      </div>

      {/* Filter and Table Container */}
      <div className="bg-white dark:bg-[#0E1833] border border-[#E3E8F2] dark:border-[#1D2B52] rounded-2xl p-5 shadow-xs">
        {/* Filters */}
        <div className="flex flex-wrap items-center justify-between gap-3 pb-4 mb-4 border-b border-[#E3E8F2] dark:border-[#1D2B52]">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium text-[#5B6787] dark:text-[#8D97B0]">Filter Location:</span>
            <div className="inline-flex p-1 rounded-xl bg-[#F4F6FB] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52]">
              {(['all', 'global', 'Hyderabad', 'Bengaluru'] as const).map((loc) => (
                <button
                  key={loc}
                  onClick={() => setFilterLocation(loc)}
                  className={`px-3 py-1 rounded-lg text-xs font-semibold capitalize transition-all ${
                    filterLocation === loc
                      ? 'bg-white dark:bg-[#0E1833] text-[#0B6FDB] shadow-xs'
                      : 'text-[#5B6787] dark:text-[#8D97B0] hover:text-[#0E1730] dark:hover:text-[#E8EDFB]'
                  }`}
                >
                  {loc === 'global' ? 'All Locations' : loc}
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Table */}
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs sm:text-sm">
            <thead>
              <tr className="bg-[#F8FAFD] dark:bg-[#122042] text-[#5B6787] dark:text-[#8D97B0] border-b border-[#E3E8F2] dark:border-[#1D2B52]">
                <th className="py-3 px-4 font-semibold">Date</th>
                <th className="py-3 px-4 font-semibold">Holiday Name</th>
                <th className="py-3 px-4 font-semibold">Location Scope</th>
                <th className="py-3 px-4 font-semibold">Description</th>
                <th className="py-3 px-4 font-semibold">Frequency</th>
                <th className="py-3 px-4 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#E3E8F2] dark:divide-[#1D2B52]">
              {filteredHolidays.length === 0 ? (
                <tr>
                  <td colSpan={6} className="py-8 text-center text-[#8D97B0]">
                    No holidays found matching the selected filter.
                  </td>
                </tr>
              ) : (
                filteredHolidays.map((holiday) => (
                  <tr
                    key={holiday.id}
                    className="hover:bg-[#F8FAFD]/70 dark:hover:bg-[#122042]/50 transition-colors"
                  >
                    <td className="py-3.5 px-4 font-mono font-medium text-xs text-[#0B6FDB]">
                      {holiday.holiday_date}
                    </td>
                    <td className="py-3.5 px-4 font-semibold text-[#0E1730] dark:text-[#E8EDFB]">
                      {holiday.name}
                    </td>
                    <td className="py-3.5 px-4">
                      {holiday.applicable_location ? (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-blue-50 dark:bg-blue-950/40 text-blue-600 dark:text-blue-300 border border-blue-200 dark:border-blue-800">
                          <MapPin className="w-3 h-3" /> {holiday.applicable_location}
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-50 dark:bg-emerald-950/40 text-emerald-600 dark:text-emerald-300 border border-emerald-200 dark:border-emerald-800">
                          <Globe className="w-3 h-3" /> All Locations (Global)
                        </span>
                      )}
                    </td>
                    <td className="py-3.5 px-4 text-[#5B6787] dark:text-[#9AA6C6] max-w-xs truncate">
                      {holiday.description || '—'}
                    </td>
                    <td className="py-3.5 px-4">
                      {holiday.is_recurring ? (
                        <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-semibold bg-[#E3F0FF] dark:bg-[#16275A] text-[#0B6FDB]">
                          <Repeat className="w-3 h-3" /> Annual
                        </span>
                      ) : (
                        <span className="text-[#8D97B0] text-xs">One-off</span>
                      )}
                    </td>
                    <td className="py-3.5 px-4 text-right">
                      <div className="inline-flex items-center justify-end gap-1.5">
                        <button
                          onClick={() => handleOpenEditModal(holiday)}
                          title="Edit Holiday"
                          className="p-1.5 rounded-lg text-[#5B6787] hover:text-[#0B6FDB] hover:bg-[#E3F0FF] dark:hover:bg-[#16275A] transition-colors cursor-pointer"
                        >
                          <Pencil className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleDelete(holiday)}
                          title="Delete Holiday"
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

      {/* Add / Edit Holiday Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={editingHoliday ? 'Edit Master Holiday' : 'Add Master Holiday'}
        description={
          editingHoliday
            ? `Update holiday details for ${editingHoliday.name}.`
            : 'Creates a record in PostgreSQL table holidays with unique date and location constraints.'
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
                Holiday Name *
              </label>
              <input
                type="text"
                required
                placeholder="e.g. Telangana Formation Day"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
                Date *
              </label>
              <input
                type="date"
                required
                value={holidayDate}
                onChange={(e) => setHolidayDate(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Applicable Location
            </label>
            <div className="grid grid-cols-2 gap-3 mb-2">
              <label className="flex items-center gap-2 p-2.5 rounded-xl border border-[#E3E8F2] dark:border-[#1D2B52] cursor-pointer text-xs">
                <input
                  type="radio"
                  name="locType"
                  value="global"
                  checked={locationType === 'global'}
                  onChange={() => setLocationType('global')}
                  className="text-[#0B6FDB]"
                />
                <span>All Locations (Global)</span>
              </label>
              <label className="flex items-center gap-2 p-2.5 rounded-xl border border-[#E3E8F2] dark:border-[#1D2B52] cursor-pointer text-xs">
                <input
                  type="radio"
                  name="locType"
                  value="branch"
                  checked={locationType === 'branch'}
                  onChange={() => setLocationType('branch')}
                  className="text-[#0B6FDB]"
                />
                <span>Specific Branch</span>
              </label>
            </div>
            {locationType === 'branch' && (
              <input
                type="text"
                placeholder="e.g. Hyderabad, Bengaluru, Pune"
                value={customLocation}
                onChange={(e) => setCustomLocation(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
              />
            )}
          </div>

          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Description (Optional)
            </label>
            <textarea
              rows={2}
              placeholder="Context or statutory reference regarding this holiday..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
          </div>

          <div className="pt-2 border-t border-[#E3E8F2] dark:border-[#1D2B52]">
            <label className="flex items-center gap-2 cursor-pointer select-none">
              <input
                type="checkbox"
                checked={isRecurring}
                onChange={(e) => setIsRecurring(e.target.checked)}
                className="w-4 h-4 rounded text-[#0B6FDB] focus:ring-[#0B6FDB]"
              />
              <span className="text-xs font-medium text-[#0E1730] dark:text-[#E8EDFB]">
                Recurring Holiday (Occurs annually on same calendar day)
              </span>
            </label>
          </div>

          <div className="flex items-center justify-end gap-3 pt-4 border-t border-[#E3E8F2] dark:border-[#1D2B52]">
            <button
              type="button"
              onClick={() => setIsModalOpen(false)}
              className="px-4 py-2 rounded-xl text-xs font-semibold border border-[#E3E8F2] dark:border-[#1D2B52] hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSubmitting}
              className="px-4 py-2 rounded-xl text-xs font-semibold bg-[#0B6FDB] hover:bg-[#0B6FDB]/90 text-white shadow-sm transition-all disabled:opacity-50 cursor-pointer"
            >
              {isSubmitting ? 'Saving...' : editingHoliday ? 'Update Holiday' : 'Add Holiday'}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
};

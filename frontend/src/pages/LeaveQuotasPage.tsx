import React, { useState } from 'react';
import type { LeaveQuota, LeaveType } from '../types';
import { StatCard } from '../components/common/StatCard';
import { Modal } from '../components/common/Modal';
import { PieChart, Users, Calendar, Award, Plus, AlertCircle, Pencil, Trash2 } from 'lucide-react';

interface LeaveQuotasPageProps {
  leaveQuotas: LeaveQuota[];
  leaveTypes: LeaveType[];
  onAddQuota: (quota: Omit<LeaveQuota, 'id' | 'pid' | 'created_at' | 'updated_at'>) => Promise<boolean>;
  onUpdateQuota: (id: number, quota: Partial<LeaveQuota>) => Promise<boolean>;
  onDeleteQuota: (id: number, label: string) => Promise<boolean>;
  searchQuery: string;
}

export const LeaveQuotasPage: React.FC<LeaveQuotasPageProps> = ({
  leaveQuotas,
  leaveTypes,
  onAddQuota,
  onUpdateQuota,
  onDeleteQuota,
  searchQuery,
}) => {
  const [filterYear, setFilterYear] = useState<string>('all');
  const [filterType, setFilterType] = useState<string>('all');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingQuota, setEditingQuota] = useState<LeaveQuota | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Form State
  const [leaveTypeId, setLeaveTypeId] = useState<number>(leaveTypes[0]?.id || 1);
  const [year, setYear] = useState('2026');
  const [entitledDays, setEntitledDays] = useState('12.0');
  const [applicableGroup, setApplicableGroup] = useState('ALL');

  const filteredQuotas = leaveQuotas.filter((q) => {
    const typeObj = leaveTypes.find((lt) => lt.id === q.leave_type_id);
    const typeName = typeObj?.name || q.leave_type_name || '';

    const matchesSearch =
      typeName.toLowerCase().includes(searchQuery.toLowerCase()) ||
      q.applicable_group.toLowerCase().includes(searchQuery.toLowerCase()) ||
      q.year.toString().includes(searchQuery);

    const matchesYear = filterYear === 'all' ? true : q.year.toString() === filterYear;
    const matchesType = filterType === 'all' ? true : q.leave_type_id.toString() === filterType;

    return matchesSearch && matchesYear && matchesType;
  });

  const distinctGroups = new Set(leaveQuotas.map((q) => q.applicable_group)).size;
  const avgDays = leaveQuotas.length
    ? (leaveQuotas.reduce((acc, q) => acc + Number(q.entitled_days), 0) / leaveQuotas.length).toFixed(1)
    : '0';

  const handleOpenAddModal = () => {
    setEditingQuota(null);
    setLeaveTypeId(leaveTypes[0]?.id || 1);
    setYear('2026');
    setEntitledDays('12.0');
    setApplicableGroup('ALL');
    setErrorMsg(null);
    setIsModalOpen(true);
  };

  const handleOpenEditModal = (quota: LeaveQuota) => {
    setEditingQuota(quota);
    setLeaveTypeId(quota.leave_type_id);
    setYear(quota.year.toString());
    setEntitledDays(quota.entitled_days.toString());
    setApplicableGroup(quota.applicable_group);
    setErrorMsg(null);
    setIsModalOpen(true);
  };

  const handleDelete = async (quota: LeaveQuota) => {
    const typeObj = leaveTypes.find((lt) => lt.id === quota.leave_type_id);
    const typeName = typeObj?.name || quota.leave_type_name || `Type #${quota.leave_type_id}`;
    const confirmed = window.confirm(
      `Are you sure you want to delete quota for "${typeName}" (Year ${quota.year}, Group ${quota.applicable_group})?\nThis action cannot be undone.`
    );
    if (!confirmed) return;

    try {
      await onDeleteQuota(quota.id, `${typeName} (${quota.year})`);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete leave quota');
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMsg(null);

    const yr = parseInt(year, 10);
    const days = parseFloat(entitledDays);

    if (isNaN(yr) || yr < 2000 || yr > 2100) {
      setErrorMsg('Year must be between 2000 and 2100.');
      return;
    }
    if (isNaN(days) || days <= 0) {
      setErrorMsg('Entitled days must be greater than 0.');
      return;
    }
    if (!applicableGroup.trim()) {
      setErrorMsg('Applicable group is required (e.g. ALL).');
      return;
    }

    const cleanGroup = applicableGroup.trim().toUpperCase();

    // Check unique constraint: (leave_type_id, year, applicable_group)
    const exists = leaveQuotas.some(
      (q) =>
        q.id !== editingQuota?.id &&
        q.leave_type_id === leaveTypeId &&
        q.year === yr &&
        q.applicable_group.toUpperCase() === cleanGroup
    );

    if (exists) {
      setErrorMsg(
        `A quota policy already exists for this Leave Type in Year ${yr} for Group "${cleanGroup}". Unique constraint prevents duplicates.`
      );
      return;
    }

    const typeObj = leaveTypes.find((lt) => lt.id === leaveTypeId);

    setIsSubmitting(true);
    try {
      if (editingQuota) {
        await onUpdateQuota(editingQuota.id, {
          leave_type_id: leaveTypeId,
          year: yr,
          entitled_days: days,
          applicable_group: cleanGroup,
        });
      } else {
        await onAddQuota({
          leave_type_id: leaveTypeId,
          leave_type_name: typeObj?.name,
          leave_type_code: typeObj?.code,
          year: yr,
          entitled_days: days,
          applicable_group: cleanGroup,
        });
      }
      setIsModalOpen(false);
      setEditingQuota(null);
    } catch (err) {
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
            Leave Quota Allocation Master
          </h2>
          <p className="text-xs text-[#5B6787] dark:text-[#8D97B0] mt-0.5">
            Relational quota rules stored in PostgreSQL table <code className="px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-800 font-mono text-[11px]">leave_quotas</code> with foreign-key integrity to <code className="px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-800 font-mono text-[11px]">leave_types</code>.
          </p>
        </div>

        <button
          onClick={handleOpenAddModal}
          className="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl bg-[#0B6FDB] hover:bg-[#0B6FDB]/90 text-white font-semibold text-xs shadow-sm transition-all cursor-pointer flex-none"
        >
          <Plus className="w-4 h-4" />
          <span>New Quota Allocation</span>
        </button>
      </div>

      {/* KPI Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Total Quota Policies"
          value={leaveQuotas.length}
          subtext="Annual rule assignments"
          icon={PieChart}
        />
        <StatCard
          label="Covered Leave Types"
          value={new Set(leaveQuotas.map((q) => q.leave_type_id)).size}
          subtext={`Out of ${leaveTypes.length} configured types`}
          icon={Award}
          badge={{ text: 'Category Mapped', variant: 'ok' }}
        />
        <StatCard
          label="Average Entitlement"
          value={`${avgDays}d`}
          subtext="Per employee profile"
          icon={Calendar}
        />
        <StatCard
          label="Target Groups"
          value={distinctGroups}
          subtext="Cohort eligibility rules"
          icon={Users}
        />
      </div>

      {/* Filter and Table Container */}
      <div className="bg-white dark:bg-[#0E1833] border border-[#E3E8F2] dark:border-[#1D2B52] rounded-2xl p-5 shadow-xs">
        {/* Filters */}
        <div className="flex flex-wrap items-center justify-between gap-3 pb-4 mb-4 border-b border-[#E3E8F2] dark:border-[#1D2B52]">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex items-center gap-2">
              <span className="text-xs font-medium text-[#5B6787] dark:text-[#8D97B0]">Year:</span>
              <select
                value={filterYear}
                onChange={(e) => setFilterYear(e.target.value)}
                className="text-xs px-3 py-1.5 rounded-xl bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] text-[#0E1730] dark:text-[#E8EDFB] focus:outline-none focus:ring-2 focus:ring-[#0B6FDB]"
              >
                <option value="all">All Years</option>
                <option value="2026">2026</option>
                <option value="2027">2027</option>
              </select>
            </div>

            <div className="flex items-center gap-2">
              <span className="text-xs font-medium text-[#5B6787] dark:text-[#8D97B0]">Leave Type:</span>
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value)}
                className="text-xs px-3 py-1.5 rounded-xl bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] text-[#0E1730] dark:text-[#E8EDFB] focus:outline-none focus:ring-2 focus:ring-[#0B6FDB]"
              >
                <option value="all">All Categories</option>
                {leaveTypes.map((lt) => (
                  <option key={lt.id} value={lt.id.toString()}>
                    {lt.name} ({lt.code})
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {/* Table */}
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs sm:text-sm">
            <thead>
              <tr className="bg-[#F8FAFD] dark:bg-[#122042] text-[#5B6787] dark:text-[#8D97B0] border-b border-[#E3E8F2] dark:border-[#1D2B52]">
                <th className="py-3 px-4 font-semibold">Associated Leave Type</th>
                <th className="py-3 px-4 font-semibold">Policy Year</th>
                <th className="py-3 px-4 font-semibold">Entitled Days</th>
                <th className="py-3 px-4 font-semibold">Eligible Group</th>
                <th className="py-3 px-4 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#E3E8F2] dark:divide-[#1D2B52]">
              {filteredQuotas.length === 0 ? (
                <tr>
                  <td colSpan={5} className="py-8 text-center text-[#8D97B0]">
                    No quota allocations found matching the active filters.
                  </td>
                </tr>
              ) : (
                filteredQuotas.map((quota) => {
                  const typeObj = leaveTypes.find((lt) => lt.id === quota.leave_type_id);
                  const name = typeObj?.name || quota.leave_type_name || `Type #${quota.leave_type_id}`;
                  const code = typeObj?.code || quota.leave_type_code || '';

                  return (
                    <tr
                      key={quota.id}
                      className="hover:bg-[#F8FAFD]/70 dark:hover:bg-[#122042]/50 transition-colors"
                    >
                      <td className="py-3.5 px-4">
                        <div className="font-semibold text-[#0E1730] dark:text-[#E8EDFB]">
                          {name}
                        </div>
                        {code && (
                          <span className="font-mono text-[11px] px-1.5 py-0.5 rounded bg-[#E3F0FF] dark:bg-[#16275A] text-[#0B6FDB]">
                            {code}
                          </span>
                        )}
                      </td>
                      <td className="py-3.5 px-4 font-mono font-semibold text-[#0E1730] dark:text-[#E8EDFB]">
                        {quota.year}
                      </td>
                      <td className="py-3.5 px-4 font-semibold text-base text-[#0B6FDB] dark:text-[#0BA6CC]">
                        {Number(quota.entitled_days).toFixed(1)} <span className="text-xs font-normal text-[#5B6787] dark:text-[#8D97B0]">days</span>
                      </td>
                      <td className="py-3.5 px-4">
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-gray-100 dark:bg-gray-800 text-[#0E1730] dark:text-[#E8EDFB] border border-gray-200 dark:border-gray-700">
                          <Users className="w-3 h-3 text-[#5B6787]" /> {quota.applicable_group}
                        </span>
                      </td>
                      <td className="py-3.5 px-4 text-right">
                        <div className="inline-flex items-center justify-end gap-1.5">
                          <button
                            onClick={() => handleOpenEditModal(quota)}
                            title="Edit Quota"
                            className="p-1.5 rounded-lg text-[#5B6787] hover:text-[#0B6FDB] hover:bg-[#E3F0FF] dark:hover:bg-[#16275A] transition-colors cursor-pointer"
                          >
                            <Pencil className="w-4 h-4" />
                          </button>
                          <button
                            onClick={() => handleDelete(quota)}
                            title="Delete Quota"
                            className="p-1.5 rounded-lg text-[#5B6787] hover:text-[#D93D4A] hover:bg-[#FDE8EA] dark:hover:bg-[#3D141B] transition-colors cursor-pointer"
                          >
                            <Trash2 className="w-4 h-4" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Add / Edit Quota Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={editingQuota ? 'Edit Leave Quota Policy' : 'Create Leave Quota Policy'}
        description="Creates foreign-key linked quota in PostgreSQL table leave_quotas."
      >
        <form onSubmit={handleSubmit} className="space-y-4">
          {errorMsg && (
            <div className="flex items-center gap-2 p-3 rounded-xl bg-[#FDE8EA] dark:bg-[#3D141B] text-[#D93D4A] text-xs">
              <AlertCircle className="w-4 h-4 flex-none" />
              <span>{errorMsg}</span>
            </div>
          )}

          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Select Leave Type Category * (Foreign Key)
            </label>
            <select
              required
              value={leaveTypeId}
              onChange={(e) => setLeaveTypeId(parseInt(e.target.value, 10))}
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            >
              {leaveTypes.map((lt) => (
                <option key={lt.id} value={lt.id}>
                  {lt.name} ({lt.code}) — {lt.is_paid ? 'Paid' : 'Unpaid'}
                </option>
              ))}
            </select>
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
                Policy Year * (2000-2100)
              </label>
              <input
                type="number"
                required
                min="2000"
                max="2100"
                value={year}
                onChange={(e) => setYear(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
              />
            </div>

            <div>
              <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
                Entitled Days * (Supports Half-Days)
              </label>
              <input
                type="number"
                step="0.5"
                min="0.5"
                required
                value={entitledDays}
                onChange={(e) => setEntitledDays(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs font-semibold bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Applicable Employee Group *
            </label>
            <input
              type="text"
              required
              placeholder="e.g. ALL, FEMALE_EMPLOYEES, PROBATION, CONTRACTORS"
              value={applicableGroup}
              onChange={(e) => setApplicableGroup(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs uppercase font-mono bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
            <span className="text-[11px] text-[#8D97B0] mt-1 block">
              Default is &apos;ALL&apos;. Group-specific rules override or supplement default policy.
            </span>
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
              {isSubmitting ? 'Saving...' : editingQuota ? 'Update Quota Policy' : 'Save Quota Allocation'}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
};

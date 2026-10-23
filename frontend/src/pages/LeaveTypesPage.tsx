import React, { useState } from 'react';
import type { LeaveType } from '../types';

import { StatCard } from '../components/common/StatCard';
import { Modal } from '../components/common/Modal';
import { FileText, CheckCircle, AlertCircle, Plus, DollarSign, XCircle, Pencil, Trash2 } from 'lucide-react';

interface LeaveTypesPageProps {
  leaveTypes: LeaveType[];
  onAddLeaveType: (lt: Omit<LeaveType, 'id' | 'pid' | 'created_at' | 'updated_at'>) => Promise<boolean>;
  onUpdateLeaveType: (id: number, lt: Partial<LeaveType>) => Promise<boolean>;
  onDeleteLeaveType: (id: number, name: string) => Promise<boolean>;
  searchQuery: string;
}

export const LeaveTypesPage: React.FC<LeaveTypesPageProps> = ({
  leaveTypes,
  onAddLeaveType,
  onUpdateLeaveType,
  onDeleteLeaveType,
  searchQuery,
}) => {
  const [filterPaid, setFilterPaid] = useState<'all' | 'paid' | 'unpaid'>('all');
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [editingLeaveType, setEditingLeaveType] = useState<LeaveType | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  // Form State
  const [name, setName] = useState('');
  const [code, setCode] = useState('');
  const [description, setDescription] = useState('');
  const [isPaid, setIsPaid] = useState(true);
  const [isActive, setIsActive] = useState(true);

  const filteredLeaveTypes = leaveTypes.filter((lt) => {
    const matchesSearch =
      lt.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      lt.code.toLowerCase().includes(searchQuery.toLowerCase()) ||
      (lt.description || '').toLowerCase().includes(searchQuery.toLowerCase());

    const matchesPaid =
      filterPaid === 'all'
        ? true
        : filterPaid === 'paid'
        ? lt.is_paid
        : !lt.is_paid;

    return matchesSearch && matchesPaid;
  });

  const paidCount = leaveTypes.filter((lt) => lt.is_paid).length;
  const unpaidCount = leaveTypes.filter((lt) => !lt.is_paid).length;
  const activeCount = leaveTypes.filter((lt) => lt.is_active).length;

  const handleOpenAddModal = () => {
    setEditingLeaveType(null);
    setName('');
    setCode('');
    setDescription('');
    setIsPaid(true);
    setIsActive(true);
    setErrorMsg(null);
    setIsModalOpen(true);
  };

  const handleOpenEditModal = (lt: LeaveType) => {
    setEditingLeaveType(lt);
    setName(lt.name);
    setCode(lt.code);
    setDescription(lt.description || '');
    setIsPaid(lt.is_paid);
    setIsActive(lt.is_active);
    setErrorMsg(null);
    setIsModalOpen(true);
  };

  const handleDelete = async (lt: LeaveType) => {
    const confirmed = window.confirm(
      `Are you sure you want to delete leave category "${lt.name}" (${lt.code})?\nThis action cannot be undone.`
    );
    if (!confirmed) return;

    try {
      await onDeleteLeaveType(lt.id, lt.name);
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to delete leave type');
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErrorMsg(null);

    if (!name.trim() || !code.trim()) {
      setErrorMsg('Leave category name and code are required.');
      return;
    }

    const cleanCode = code.trim().toUpperCase();
    const exists = leaveTypes.some(
      (lt) => lt.id !== editingLeaveType?.id && lt.code.toUpperCase() === cleanCode
    );
    if (exists) {
      setErrorMsg(`Leave type code "${cleanCode}" already exists. Codes must be unique.`);
      return;
    }

    setIsSubmitting(true);
    try {
      let ok = false;
      if (editingLeaveType) {
        ok = await onUpdateLeaveType(editingLeaveType.id, {
          name: name.trim(),
          code: cleanCode,
          description: description.trim() || null,
          is_paid: isPaid,
          is_active: isActive,
        });
      } else {
        ok = await onAddLeaveType({
          name: name.trim(),
          code: cleanCode,
          description: description.trim() || null,
          is_paid: isPaid,
          is_active: isActive,
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
            Leave Categories & Policies
          </h1>
          <p className="text-xs sm:text-sm text-[#5B6787] dark:text-[#8D97B0] mt-0.5">
            Master leave classification for casual, sick, annual, and statutory maternity benefits.
          </p>
        </div>
        <button
          onClick={handleOpenAddModal}
          className="inline-flex items-center gap-2 px-4 py-2 rounded-xl bg-[#0B6FDB] hover:bg-[#0B6FDB]/90 text-white font-semibold text-xs sm:text-sm shadow-md shadow-[#0B6FDB]/20 transition-all cursor-pointer"
        >
          <Plus className="w-4 h-4" />
          <span>Add Leave Type</span>
        </button>
      </div>

      {/* KPI Stats */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Leave Types"
          value={leaveTypes.length}
          subtext="Configured policies"
          icon={FileText}
        />
        <StatCard
          label="Paid Leave Types"
          value={paidCount}
          subtext="Salary credited as normal"
          icon={DollarSign}
          badge={{ text: 'Compensated', variant: 'ok' }}
        />
        <StatCard
          label="Unpaid / LOP"
          value={unpaidCount}
          subtext="Loss of Pay leaves"
          icon={XCircle}
          badge={{ text: 'Deduction', variant: 'warn' }}
        />
        <StatCard
          label="Active Policies"
          value={activeCount}
          subtext="Available for employee application"
          icon={CheckCircle}
        />
      </div>

      {/* Filter and Table Container */}
      <div className="bg-white dark:bg-[#0E1833] border border-[#E3E8F2] dark:border-[#1D2B52] rounded-2xl p-5 shadow-xs">
        {/* Filters */}
        <div className="flex flex-wrap items-center justify-between gap-3 pb-4 mb-4 border-b border-[#E3E8F2] dark:border-[#1D2B52]">
          <div className="flex items-center gap-2">
            <span className="text-xs font-medium text-[#5B6787] dark:text-[#8D97B0]">Filter Compensation:</span>
            <div className="inline-flex p-1 rounded-xl bg-[#F4F6FB] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52]">
              {(['all', 'paid', 'unpaid'] as const).map((mode) => (
                <button
                  key={mode}
                  onClick={() => setFilterPaid(mode)}
                  className={`px-3 py-1 rounded-lg text-xs font-semibold capitalize transition-all ${
                    filterPaid === mode
                      ? 'bg-white dark:bg-[#0E1833] text-[#0B6FDB] shadow-xs'
                      : 'text-[#5B6787] dark:text-[#8D97B0] hover:text-[#0E1730] dark:hover:text-[#E8EDFB]'
                  }`}
                >
                  {mode === 'all' ? 'All Types' : mode === 'paid' ? 'Paid Only' : 'Unpaid (LOP)'}
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
                <th className="py-3 px-4 font-semibold">Category Name</th>
                <th className="py-3 px-4 font-semibold">Code</th>
                <th className="py-3 px-4 font-semibold">Remuneration</th>
                <th className="py-3 px-4 font-semibold">Description</th>
                <th className="py-3 px-4 font-semibold">Status</th>
                <th className="py-3 px-4 font-semibold text-right">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[#E3E8F2] dark:divide-[#1D2B52]">
              {filteredLeaveTypes.length === 0 ? (
                <tr>
                  <td colSpan={6} className="py-8 text-center text-[#8D97B0]">
                    No leave categories found.
                  </td>
                </tr>
              ) : (
                filteredLeaveTypes.map((lt) => (
                  <tr
                    key={lt.id}
                    className="hover:bg-[#F8FAFD]/70 dark:hover:bg-[#122042]/50 transition-colors"
                  >
                    <td className="py-3.5 px-4 font-semibold text-[#0E1730] dark:text-[#E8EDFB]">
                      {lt.name}
                    </td>
                    <td className="py-3.5 px-4">
                      <span className="font-mono text-xs px-2 py-0.5 rounded bg-[#E3F0FF] dark:bg-[#16275A] text-[#0B6FDB] font-bold">
                        {lt.code}
                      </span>
                    </td>
                    <td className="py-3.5 px-4">
                      {lt.is_paid ? (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-[#E3F6EE] dark:bg-[#0E3326] text-[#15A36B]">
                          <DollarSign className="w-3 h-3" /> Fully Paid
                        </span>
                      ) : (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-[#FFF4DB] dark:bg-[#3A2C08] text-[#C98200]">
                          Loss of Pay (Unpaid)
                        </span>
                      )}
                    </td>
                    <td className="py-3.5 px-4 text-[#5B6787] dark:text-[#9AA6C6] max-w-sm">
                      {lt.description || '—'}
                    </td>
                    <td className="py-3.5 px-4">
                      {lt.is_active ? (
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
                          onClick={() => handleOpenEditModal(lt)}
                          title="Edit Leave Category"
                          className="p-1.5 rounded-lg text-[#5B6787] hover:text-[#0B6FDB] hover:bg-[#E3F0FF] dark:hover:bg-[#16275A] transition-colors cursor-pointer"
                        >
                          <Pencil className="w-4 h-4" />
                        </button>
                        <button
                          onClick={() => handleDelete(lt)}
                          title="Delete Leave Category"
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

      {/* Add / Edit Leave Type Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title={editingLeaveType ? 'Edit Leave Category' : 'Create Leave Category'}
        description={
          editingLeaveType
            ? `Update classification rules for leave type ${editingLeaveType.code}.`
            : 'Defines a master leave category in PostgreSQL table leave_types.'
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
                Leave Category Name *
              </label>
              <input
                type="text"
                required
                placeholder="e.g. Bereavement Leave"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
                Code * (Unique)
              </label>
              <input
                type="text"
                required
                placeholder="e.g. BL"
                value={code}
                onChange={(e) => setCode(e.target.value)}
                className="w-full px-3 py-2 rounded-xl text-xs uppercase font-mono bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
              />
            </div>
          </div>

          <div>
            <label className="block text-xs font-semibold text-[#0E1730] dark:text-[#E8EDFB] mb-1">
              Description (Policy guidance)
            </label>
            <textarea
              rows={2}
              placeholder="Eligibility guidelines and entitlement rules for this category..."
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full px-3 py-2 rounded-xl text-xs bg-[#F8FAFD] dark:bg-[#122042] border border-[#E3E8F2] dark:border-[#1D2B52] focus:ring-2 focus:ring-[#0B6FDB] focus:outline-none"
            />
          </div>

          <div className="pt-2 border-t border-[#E3E8F2] dark:border-[#1D2B52] space-y-2">
            <label className="flex items-center gap-2 cursor-pointer select-none">
              <input
                type="checkbox"
                checked={isPaid}
                onChange={(e) => setIsPaid(e.target.checked)}
                className="w-4 h-4 rounded text-[#0B6FDB] focus:ring-[#0B6FDB]"
              />
              <span className="text-xs font-medium text-[#0E1730] dark:text-[#E8EDFB]">
                Paid Leave (Salary paid as normal during absence)
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
                Active Policy (Visible to employees in leave application portal)
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
              {isSubmitting ? 'Saving...' : editingLeaveType ? 'Update Policy' : 'Create Category'}
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
};

import React, { useState, useEffect, useCallback } from 'react';
import { Download, Plus, AlertCircle, Clock, Calendar } from 'lucide-react';
import type {
  DailyAttendanceRecord,
  AttendanceSummary,
  Employee,
  CreateAttendancePayload,
  UpdateAttendancePayload,
} from '../types';
import { api } from '../services/api';
import { attendanceApi } from '../services/attendanceApi';
import { AttendanceSummaryCards } from '../components/attendance/AttendanceSummaryCards';
import { NoEmployeesBanner } from '../components/attendance/NoEmployeesBanner';
import { AttendanceFilterBar } from '../components/attendance/AttendanceFilterBar';
import { AttendanceTable } from '../components/attendance/AttendanceTable';
import { AttendanceModal } from '../components/attendance/AttendanceModal';
import { AttendanceClockCard } from '../components/attendance/AttendanceClockCard';
import { AttendanceCorrectionModal } from '../components/attendance/AttendanceCorrectionModal';
import { AttendanceCorrectionsList } from '../components/attendance/AttendanceCorrectionsList';

interface DailyAttendancePageProps {
  showSuccessToast?: (msg: string) => void;
  onNavigateToEmployees?: () => void;
}

export const DailyAttendancePage: React.FC<DailyAttendancePageProps> = ({
  showSuccessToast,
  onNavigateToEmployees,
}) => {
  const [records, setRecords] = useState<DailyAttendanceRecord[]>([]);
  const [summary, setSummary] = useState<AttendanceSummary>({
    total_records: 0,
    recorded_count: 0,
    pending_count: 0,
    regularised_count: 0,
  });
  const [archivedCount, setArchivedCount] = useState<number>(0);
  const [employees, setEmployees] = useState<Employee[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [pageError, setPageError] = useState<string | null>(null);

  // Tab State: 'RECORDS' | 'CORRECTIONS'
  const [activeTab, setActiveTab] = useState<'RECORDS' | 'CORRECTIONS'>('RECORDS');
  const [pendingCorrectionCount, setPendingCorrectionCount] = useState<number>(0);

  // Filters
  const [search, setSearch] = useState<string>('');
  const [status, setStatus] = useState<string>('ALL');
  const [showArchived, setShowArchived] = useState<boolean>(false);

  // Attendance Entry Modal State
  const [isModalOpen, setIsModalOpen] = useState<boolean>(false);
  const [editingRecord, setEditingRecord] = useState<DailyAttendanceRecord | null>(null);

  // Correction Request Modal State
  const [isCorrectionModalOpen, setIsCorrectionModalOpen] = useState<boolean>(false);
  const [correctionEmployeeId, setCorrectionEmployeeId] = useState<number | null>(null);

  // Load employees
  useEffect(() => {
    let isMounted = true;
    api.fetchEmployees().then((res) => {
      if (isMounted && res.data) {
        setEmployees(res.data);
      }
    });
    return () => {
      isMounted = false;
    };
  }, []);

  // Refresh pending correction count
  const refreshPendingCount = useCallback(async () => {
    try {
      const res = await attendanceApi.fetchCorrections({ status: 'PENDING' });
      if (res.data) {
        setPendingCorrectionCount(res.data.length);
      }
    } catch {
      // background count failover
    }
  }, []);

  // Load attendance data
  const loadAttendance = useCallback(async () => {
    setIsLoading(true);
    setPageError(null);
    try {
      const res = await attendanceApi.fetchAttendance({
        search: search.trim() || undefined,
        status: status !== 'ALL' ? status : undefined,
        archived: showArchived,
      });

      if (res.data) {
        setRecords(res.data.records);
        setSummary(res.data.summary);
        setArchivedCount(res.data.archived_count);
      } else {
        setPageError(res.error || 'Failed to load attendance records');
      }
    } catch (err: unknown) {
      setPageError(err instanceof Error ? err.message : 'Error fetching attendance records');
    } finally {
      setIsLoading(false);
    }
  }, [search, status, showArchived]);

  useEffect(() => {
    loadAttendance();
    refreshPendingCount();
  }, [loadAttendance, refreshPendingCount]);

  // Actions
  const handleOpenAddModal = () => {
    setEditingRecord(null);
    setIsModalOpen(true);
  };

  const handleOpenEditModal = (rec: DailyAttendanceRecord) => {
    setEditingRecord(rec);
    setIsModalOpen(true);
  };

  const handleSaveAttendance = async (
    payload: CreateAttendancePayload | UpdateAttendancePayload,
    isEdit: boolean,
    id?: number
  ) => {
    if (isEdit && id) {
      await attendanceApi.updateAttendance(id, payload as UpdateAttendancePayload);
      if (showSuccessToast) showSuccessToast('Attendance record updated successfully.');
    } else {
      await attendanceApi.createAttendance(payload as CreateAttendancePayload);
      if (showSuccessToast) showSuccessToast('Attendance entry created successfully.');
    }
    await loadAttendance();
  };

  const handleToggleArchive = async (rec: DailyAttendanceRecord) => {
    try {
      await attendanceApi.toggleArchive(rec.id);
      if (showSuccessToast) {
        showSuccessToast(
          rec.is_archived
            ? `Restored attendance record for ${rec.employee_name}.`
            : `Archived attendance record for ${rec.employee_name}.`
        );
      }
      await loadAttendance();
    } catch (err: unknown) {
      setPageError(err instanceof Error ? err.message : 'Failed to toggle archive');
    }
  };

  const handleDelete = async (rec: DailyAttendanceRecord) => {
    const confirmed = window.confirm(
      `Are you sure you want to delete the attendance entry for ${rec.employee_name} on ${rec.attendance_date}?`
    );
    if (!confirmed) return;

    try {
      await attendanceApi.deleteAttendance(rec.id);
      if (showSuccessToast) {
        showSuccessToast(`Deleted attendance entry for ${rec.employee_name}.`);
      }
      await loadAttendance();
    } catch (err: unknown) {
      setPageError(err instanceof Error ? err.message : 'Failed to delete attendance record');
    }
  };

  const handleExportCsv = async () => {
    try {
      await attendanceApi.exportCsv({
        search: search.trim() || undefined,
        status: status !== 'ALL' ? status : undefined,
        archived: showArchived,
      });
      if (showSuccessToast) {
        showSuccessToast('Attendance CSV downloaded successfully.');
      }
    } catch (err: unknown) {
      setPageError(err instanceof Error ? err.message : 'Failed to export CSV');
    }
  };

  return (
    <div className="space-y-6">
      {/* Top Header Section matching screenshot */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-1.5 text-[11px] font-bold text-[#38BDF8] tracking-wider uppercase mb-1">
            <span className="w-1.5 h-1.5 rounded-full bg-[#38BDF8]" />
            Shift
          </div>
          <h1 className="text-2xl font-bold text-white tracking-tight">
            Daily attendance
          </h1>
          <p className="text-xs text-[#889DC7] mt-1">
            One entry per employee per day. Hours are worked out from in and out times.
          </p>
        </div>

        {/* Action Buttons */}
        <div className="flex flex-wrap items-center gap-3 shrink-0">
          <button
            type="button"
            onClick={handleExportCsv}
            className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold bg-[#0A1326] hover:bg-[#132244] border border-[#1C3262] text-white transition-all shadow-xs cursor-pointer"
          >
            <Download className="w-4 h-4 text-[#889DC7]" />
            <span>Export CSV</span>
          </button>

          <button
            type="button"
            onClick={() => {
              setCorrectionEmployeeId(null);
              setIsCorrectionModalOpen(true);
            }}
            className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold bg-[#0D1833] hover:bg-[#132244] border border-[#1E3466] text-[#38BDF8] transition-all shadow-xs cursor-pointer"
          >
            <Clock className="w-4 h-4 text-[#38BDF8]" />
            <span>Request correction</span>
          </button>

          <button
            type="button"
            onClick={handleOpenAddModal}
            className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold bg-[#0B6FDB] hover:bg-[#0957B0] text-white transition-all shadow-md shadow-[#0B6FDB]/25 cursor-pointer"
          >
            <Plus className="w-4 h-4" />
            <span>Add attendance entry</span>
          </button>
        </div>
      </div>

      {/* Page Error Banner */}
      {pageError && (
        <div className="p-4 rounded-2xl bg-[#3B1219] border border-[#771D2D] text-xs text-[#F87171] flex items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{pageError}</span>
          </div>
          <button
            type="button"
            onClick={() => setPageError(null)}
            className="text-[#F87171] hover:text-white underline cursor-pointer"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Empty Employee Master Data Warning Banner (matches screenshot) */}
      {employees.length === 0 && !isLoading && (
        <NoEmployeesBanner onOpenEmployeeMaster={onNavigateToEmployees} />
      )}

      {/* Employee-facing Attendance Clock-in / Clock-out Card (TIM-02) */}
      <AttendanceClockCard
        employees={employees}
        onAttendanceChange={() => {
          loadAttendance();
          refreshPendingCount();
        }}
        showSuccessToast={showSuccessToast}
        onRequestCorrection={(empId) => {
          setCorrectionEmployeeId(empId || null);
          setIsCorrectionModalOpen(true);
        }}
      />

      {/* Navigation Tabs between Attendance Records and Correction Requests */}
      <div className="flex items-center gap-2 border-b border-[#16274E] pb-2">
        <button
          type="button"
          onClick={() => setActiveTab('RECORDS')}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer ${
            activeTab === 'RECORDS'
              ? 'bg-[#0B6FDB] text-white shadow-md shadow-[#0B6FDB]/20'
              : 'text-[#889DC7] hover:text-white hover:bg-[#0E1B38]'
          }`}
        >
          <Calendar className="w-4 h-4" />
          <span>Daily Attendance Records</span>
        </button>

        <button
          type="button"
          onClick={() => setActiveTab('CORRECTIONS')}
          className={`flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-bold transition-all cursor-pointer ${
            activeTab === 'CORRECTIONS'
              ? 'bg-[#0B6FDB] text-white shadow-md shadow-[#0B6FDB]/20'
              : 'text-[#889DC7] hover:text-white hover:bg-[#0E1B38]'
          }`}
        >
          <Clock className="w-4 h-4" />
          <span>Correction Requests</span>
          {pendingCorrectionCount > 0 && (
            <span className="px-1.5 py-0.2 rounded-full text-[10px] font-extrabold bg-[#F59E0B] text-black">
              {pendingCorrectionCount}
            </span>
          )}
        </button>
      </div>

      {/* Tab 1: Daily Attendance Records View */}
      {activeTab === 'RECORDS' && (
        <div className="space-y-6 animate-fadeIn">
          {/* 4 Summary Cards (Real database counts) */}
          <AttendanceSummaryCards summary={summary} />

          {/* Attendance Table Card Container */}
          <div className="bg-[#0A1326] border border-[#16274E] rounded-2xl overflow-hidden shadow-xs">
            <AttendanceFilterBar
              search={search}
              onSearchChange={setSearch}
              status={status}
              onStatusChange={setStatus}
              showArchived={showArchived}
              onToggleArchived={setShowArchived}
              archivedCount={archivedCount}
            />
            <AttendanceTable
              records={records}
              isLoading={isLoading}
              onEdit={handleOpenEditModal}
              onToggleArchive={handleToggleArchive}
              onDelete={handleDelete}
              onAddClick={handleOpenAddModal}
            />
          </div>
        </div>
      )}

      {/* Tab 2: Attendance Correction Requests Workflow View */}
      {activeTab === 'CORRECTIONS' && (
        <div className="animate-fadeIn">
          <AttendanceCorrectionsList
            employees={employees}
            showSuccessToast={showSuccessToast}
            onCorrectionApproved={() => {
              loadAttendance();
              refreshPendingCount();
            }}
          />
        </div>
      )}

      {/* Modal for Create / Edit Attendance Entry */}
      <AttendanceModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onSave={handleSaveAttendance}
        editingRecord={editingRecord}
        employees={employees}
      />

      {/* Modal for Attendance Correction Request */}
      <AttendanceCorrectionModal
        isOpen={isCorrectionModalOpen}
        onClose={() => setIsCorrectionModalOpen(false)}
        onSuccess={() => {
          loadAttendance();
          refreshPendingCount();
        }}
        employees={employees}
        initialEmployeeId={correctionEmployeeId}
        showSuccessToast={showSuccessToast}
      />
    </div>
  );
};


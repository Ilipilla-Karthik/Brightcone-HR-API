import React, { useState, useEffect, useCallback } from 'react';
import {
  Clock,
  Calendar,
  User,
  CheckCircle2,
  XCircle,
  AlertCircle,
  Plus,
  RefreshCw,
  Search,
  Eye,
  ShieldCheck,
} from 'lucide-react';
import type { AttendanceCorrectionRequest, Employee } from '../../types';
import { attendanceApi } from '../../services/attendanceApi';
import { CorrectionReviewModal } from './CorrectionReviewModal';
import { AttendanceCorrectionModal } from './AttendanceCorrectionModal';

interface AttendanceCorrectionsListProps {
  employees: Employee[];
  showSuccessToast?: (msg: string) => void;
  onCorrectionApproved?: () => void;
}

export const AttendanceCorrectionsList: React.FC<AttendanceCorrectionsListProps> = ({
  employees,
  showSuccessToast,
  onCorrectionApproved,
}) => {
  const [requests, setRequests] = useState<AttendanceCorrectionRequest[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  // Filters
  const [statusFilter, setStatusFilter] = useState<string>('ALL');
  const [selectedEmpId, setSelectedEmpId] = useState<number | 'ALL'>('ALL');
  const [searchTerm, setSearchTerm] = useState<string>('');

  // Modals
  const [reviewRequest, setReviewRequest] = useState<AttendanceCorrectionRequest | null>(null);
  const [isReviewModalOpen, setIsReviewModalOpen] = useState<boolean>(false);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState<boolean>(false);

  const loadRequests = useCallback(async () => {
    setIsLoading(true);
    setErrorMsg(null);
    try {
      const res = await attendanceApi.fetchCorrections({
        status: statusFilter !== 'ALL' ? statusFilter : undefined,
        employee_id: selectedEmpId !== 'ALL' ? selectedEmpId : undefined,
      });

      if (res.data) {
        setRequests(res.data);
      } else {
        setErrorMsg(res.error || 'Failed to load correction requests.');
      }
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : 'Error fetching correction requests.');
    } finally {
      setIsLoading(false);
    }
  }, [statusFilter, selectedEmpId]);

  useEffect(() => {
    loadRequests();
  }, [loadRequests]);

  // Handle successful review
  const handleReviewSuccess = () => {
    loadRequests();
    if (onCorrectionApproved) {
      onCorrectionApproved();
    }
  };

  const handleCreateSuccess = () => {
    loadRequests();
  };

  const handleOpenReview = (req: AttendanceCorrectionRequest) => {
    setReviewRequest(req);
    setIsReviewModalOpen(true);
  };

  // Filter by search term on client side (employee name, department, or reason)
  const filteredRequests = requests.filter((req) => {
    if (!searchTerm.trim()) return true;
    const term = searchTerm.toLowerCase();
    const nameMatch = req.employee_name?.toLowerCase().includes(term);
    const reasonMatch = req.reason?.toLowerCase().includes(term);
    const dateMatch = req.attendance_date.includes(term);
    const deptMatch = req.employee_department?.toLowerCase().includes(term);
    return nameMatch || reasonMatch || dateMatch || deptMatch;
  });

  const pendingCount = requests.filter((r) => r.status === 'PENDING').length;
  const approvedCount = requests.filter((r) => r.status === 'APPROVED').length;
  const rejectedCount = requests.filter((r) => r.status === 'REJECTED').length;

  const formatTimeDisplay = (timeStr?: string | null): string => {
    if (!timeStr) return '--:--';
    const parts = timeStr.trim().split(':');
    if (parts.length >= 2) {
      const hour = parseInt(parts[0], 10);
      const min = parts[1];
      const ampm = hour >= 12 ? 'PM' : 'AM';
      const hour12 = hour % 12 === 0 ? 12 : hour % 12;
      return `${hour12 < 10 ? '0' + hour12 : hour12}:${min} ${ampm}`;
    }
    return timeStr;
  };

  return (
    <div className="space-y-4">
      {/* Top Controls Bar */}
      <div className="bg-[#0A1326] border border-[#16274E] rounded-2xl p-4 sm:p-5 shadow-xs">
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
          {/* Status Tabs */}
          <div className="flex flex-wrap items-center gap-1.5 p-1 bg-[#070D1A] border border-[#16274E] rounded-xl w-fit">
            <button
              type="button"
              onClick={() => setStatusFilter('ALL')}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                statusFilter === 'ALL'
                  ? 'bg-[#0B6FDB] text-white shadow-xs'
                  : 'text-[#889DC7] hover:text-white'
              }`}
            >
              All Requests ({requests.length})
            </button>
            <button
              type="button"
              onClick={() => setStatusFilter('PENDING')}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                statusFilter === 'PENDING'
                  ? 'bg-[#D97706] text-white shadow-xs'
                  : 'text-[#889DC7] hover:text-white'
              }`}
            >
              <span>Pending</span>
              {pendingCount > 0 && (
                <span className="px-1.5 py-0.2 rounded-full text-[10px] bg-white/20 font-bold">
                  {pendingCount}
                </span>
              )}
            </button>
            <button
              type="button"
              onClick={() => setStatusFilter('APPROVED')}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                statusFilter === 'APPROVED'
                  ? 'bg-[#059669] text-white shadow-xs'
                  : 'text-[#889DC7] hover:text-white'
              }`}
            >
              Approved ({approvedCount})
            </button>
            <button
              type="button"
              onClick={() => setStatusFilter('REJECTED')}
              className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
                statusFilter === 'REJECTED'
                  ? 'bg-[#DC2626] text-white shadow-xs'
                  : 'text-[#889DC7] hover:text-white'
              }`}
            >
              Rejected ({rejectedCount})
            </button>
          </div>

          {/* Action: New Request & Refresh */}
          <div className="flex items-center gap-3">
            <button
              type="button"
              onClick={loadRequests}
              disabled={isLoading}
              title="Refresh requests"
              className="p-2 rounded-xl text-xs font-semibold bg-[#070D1A] hover:bg-[#132244] border border-[#1C3262] text-[#889DC7] hover:text-white transition-colors cursor-pointer disabled:opacity-50"
            >
              <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin text-[#38BDF8]' : ''}`} />
            </button>

            <button
              type="button"
              onClick={() => setIsCreateModalOpen(true)}
              className="flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold bg-[#0B6FDB] hover:bg-[#0957B0] text-white transition-all shadow-md shadow-[#0B6FDB]/25 cursor-pointer"
            >
              <Plus className="w-4 h-4" />
              <span>Request Correction</span>
            </button>
          </div>
        </div>

        {/* Secondary Filter Bar: Search & Employee Select */}
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mt-4 pt-4 border-t border-[#16274E]">
          {/* Search box */}
          <div className="relative">
            <input
              type="text"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              placeholder="Search by employee, department, date, or reason..."
              className="w-full bg-[#070D1A] border border-[#16274E] focus:border-[#38BDF8] text-white text-xs rounded-xl pl-9 pr-3 py-2 focus:outline-hidden placeholder-[#5B6787]"
            />
            <Search className="w-4 h-4 text-[#889DC7] absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
          </div>

          {/* Employee dropdown filter */}
          <div className="relative">
            <select
              value={selectedEmpId}
              onChange={(e) =>
                setSelectedEmpId(e.target.value === 'ALL' ? 'ALL' : Number(e.target.value))
              }
              className="w-full bg-[#070D1A] border border-[#16274E] focus:border-[#38BDF8] text-white text-xs rounded-xl pl-9 pr-3 py-2 appearance-none focus:outline-hidden cursor-pointer"
            >
              <option value="ALL" className="bg-[#0A1326] text-white">
                All Employees
              </option>
              {employees.map((emp) => (
                <option key={emp.id} value={emp.id} className="bg-[#0A1326] text-white">
                  {emp.name} ({emp.department})
                </option>
              ))}
            </select>
            <User className="w-4 h-4 text-[#889DC7] absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
          </div>
        </div>
      </div>

      {/* Error Banner */}
      {errorMsg && (
        <div className="p-4 rounded-2xl bg-[#3B1219] border border-[#771D2D] text-xs text-[#F87171] flex items-center justify-between gap-3">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-4 h-4 shrink-0" />
            <span>{errorMsg}</span>
          </div>
          <button
            type="button"
            onClick={() => setErrorMsg(null)}
            className="text-[#F87171] hover:text-white underline cursor-pointer"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Requests Table / Cards Container */}
      <div className="bg-[#0A1326] border border-[#16274E] rounded-2xl overflow-hidden shadow-xs">
        {isLoading ? (
          <div className="py-16 flex flex-col items-center justify-center space-y-3">
            <div className="w-7 h-7 border-2 border-[#38BDF8] border-t-transparent rounded-full animate-spin" />
            <span className="text-xs text-[#889DC7]">Loading correction requests...</span>
          </div>
        ) : filteredRequests.length === 0 ? (
          <div className="py-16 px-4 text-center space-y-3">
            <div className="w-12 h-12 rounded-2xl bg-[#0D1833] border border-[#16274E] flex items-center justify-center mx-auto text-[#889DC7]">
              <Clock className="w-6 h-6 text-[#38BDF8]" />
            </div>
            <h4 className="text-sm font-bold text-white">No correction requests found</h4>
            <p className="text-xs text-[#889DC7] max-w-sm mx-auto">
              {statusFilter !== 'ALL'
                ? `There are currently no ${statusFilter.toLowerCase()} requests matching the selected filters.`
                : 'No employee has submitted a missed clock-in or clock-out correction request.'}
            </p>
            <button
              type="button"
              onClick={() => setIsCreateModalOpen(true)}
              className="inline-flex items-center gap-2 px-4 py-2 rounded-xl text-xs font-semibold bg-[#0B6FDB] hover:bg-[#0957B0] text-white transition-all shadow-md shadow-[#0B6FDB]/25 cursor-pointer mt-2"
            >
              <Plus className="w-4 h-4" />
              <span>Submit First Correction Request</span>
            </button>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full text-left text-xs border-collapse">
              <thead>
                <tr className="border-b border-[#16274E] bg-[#070D1A]/90 text-[10px] font-bold text-[#889DC7] uppercase tracking-wider">
                  <th className="py-3.5 px-4">Employee</th>
                  <th className="py-3.5 px-4">Attendance Date</th>
                  <th className="py-3.5 px-4">Correction Type</th>
                  <th className="py-3.5 px-4">Requested Time</th>
                  <th className="py-3.5 px-4">Reason</th>
                  <th className="py-3.5 px-4">Status</th>
                  <th className="py-3.5 px-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[#16274E]/60">
                {filteredRequests.map((req) => (
                  <tr
                    key={req.id}
                    className="hover:bg-[#0E1B38]/50 transition-colors group"
                  >
                    {/* Employee */}
                    <td className="py-3.5 px-4">
                      <div className="flex items-center gap-2.5">
                        <div
                          className="w-7 h-7 rounded-lg flex items-center justify-center font-bold text-white text-[11px] shrink-0"
                          style={{ backgroundColor: req.avatar_color || '#0B6FDB' }}
                        >
                          {req.employee_name ? req.employee_name.charAt(0) : 'E'}
                        </div>
                        <div>
                          <span className="font-semibold text-white block">
                            {req.employee_name || `Employee #${req.employee_id}`}
                          </span>
                          <span className="text-[10px] text-[#5B6787]">
                            {req.employee_department || 'General'}
                          </span>
                        </div>
                      </div>
                    </td>

                    {/* Date */}
                    <td className="py-3.5 px-4 whitespace-nowrap">
                      <div className="flex items-center gap-1.5 font-medium text-white">
                        <Calendar className="w-3.5 h-3.5 text-[#38BDF8]" />
                        <span>{req.attendance_date}</span>
                      </div>
                    </td>

                    {/* Correction Type */}
                    <td className="py-3.5 px-4 whitespace-nowrap">
                      <span
                        className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-semibold border ${
                          req.correction_type === 'MISSED_CLOCK_IN'
                            ? 'bg-[#0B6FDB]/15 text-[#38BDF8] border-[#0B6FDB]/30'
                            : 'bg-[#9333EA]/15 text-[#C084FC] border-[#9333EA]/30'
                        }`}
                      >
                        <Clock className="w-3 h-3" />
                        {req.correction_type === 'MISSED_CLOCK_IN'
                          ? 'Missed Clock-in'
                          : 'Missed Clock-out'}
                      </span>
                    </td>

                    {/* Requested Time */}
                    <td className="py-3.5 px-4 whitespace-nowrap">
                      <span className="font-mono font-bold text-white text-xs bg-[#070D1A] px-2 py-1 rounded-md border border-[#16274E]">
                        {formatTimeDisplay(req.requested_time)}
                      </span>
                    </td>

                    {/* Reason */}
                    <td className="py-3.5 px-4 max-w-xs truncate" title={req.reason}>
                      <span className="text-[#889DC7] italic">"{req.reason}"</span>
                    </td>

                    {/* Status Badge */}
                    <td className="py-3.5 px-4 whitespace-nowrap">
                      {req.status === 'PENDING' && (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold tracking-wider uppercase bg-[#78350F]/40 text-[#F59E0B] border border-[#B45309]/50 animate-pulse">
                          <Clock className="w-2.5 h-2.5" /> Pending Review
                        </span>
                      )}
                      {req.status === 'APPROVED' && (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold tracking-wider uppercase bg-[#064E3B]/40 text-[#10B981] border border-[#059669]/40">
                          <CheckCircle2 className="w-2.5 h-2.5" /> Approved
                        </span>
                      )}
                      {req.status === 'REJECTED' && (
                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-bold tracking-wider uppercase bg-[#3B1219] text-[#F87171] border border-[#771D2D]">
                          <XCircle className="w-2.5 h-2.5" /> Rejected
                        </span>
                      )}
                    </td>

                    {/* Actions */}
                    <td className="py-3.5 px-4 text-right whitespace-nowrap">
                      {req.status === 'PENDING' ? (
                        <button
                          type="button"
                          onClick={() => handleOpenReview(req)}
                          className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl text-[11px] font-bold text-white bg-gradient-to-r from-[#0B6FDB] to-[#0284C7] hover:from-[#0957B0] hover:to-[#0369A1] shadow-sm shadow-[#0B6FDB]/20 transition-all cursor-pointer"
                        >
                          <ShieldCheck className="w-3.5 h-3.5" />
                          <span>Review</span>
                        </button>
                      ) : (
                        <button
                          type="button"
                          onClick={() => handleOpenReview(req)}
                          className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded-lg text-[11px] font-semibold text-[#889DC7] hover:text-white hover:bg-[#16274E] transition-colors cursor-pointer"
                        >
                          <Eye className="w-3.5 h-3.5" />
                          <span>View Details</span>
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Review Modal */}
      <CorrectionReviewModal
        request={reviewRequest}
        isOpen={isReviewModalOpen}
        onClose={() => setIsReviewModalOpen(false)}
        onSuccess={handleReviewSuccess}
        showSuccessToast={showSuccessToast}
      />

      {/* Create Modal */}
      <AttendanceCorrectionModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onSuccess={handleCreateSuccess}
        employees={employees}
        showSuccessToast={showSuccessToast}
      />
    </div>
  );
};

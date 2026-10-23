import React, { useState } from 'react';
import {
  X,
  CheckCircle2,
  XCircle,
  AlertCircle,
  Clock,
  Calendar,
  User,
  ShieldCheck,
  ShieldAlert,
  History,
} from 'lucide-react';
import type { AttendanceCorrectionRequest, ApproveCorrectionPayload, RejectCorrectionPayload } from '../../types';
import { attendanceApi } from '../../services/attendanceApi';

interface CorrectionReviewModalProps {
  request: AttendanceCorrectionRequest | null;
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  showSuccessToast?: (msg: string) => void;
}

export const CorrectionReviewModal: React.FC<CorrectionReviewModalProps> = ({
  request,
  isOpen,
  onClose,
  onSuccess,
  showSuccessToast,
}) => {
  const [activeAction, setActiveAction] = useState<'IDLE' | 'APPROVE' | 'REJECT'>('IDLE');
  const [reviewerName, setReviewerName] = useState<string>('Ananya Rao (HR Ops)');
  const [rejectionReason, setRejectionReason] = useState<string>('');
  const [approvalNotes, setApprovalNotes] = useState<string>('');
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);

  if (!isOpen || !request) return null;

  const isPending = request.status === 'PENDING';

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

  const handleApprove = async () => {
    setErrorMsg(null);
    setIsSubmitting(true);
    try {
      const payload: ApproveCorrectionPayload = {
        reviewer_name: reviewerName.trim() || undefined,
        notes: approvalNotes.trim() || undefined,
      };
      await attendanceApi.approveCorrection(request.id, payload);
      if (showSuccessToast) {
        showSuccessToast(
          `Correction request #${request.id} for ${request.employee_name || 'employee'} approved and attendance regularised.`
        );
      }
      onSuccess();
      onClose();
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : 'Failed to approve correction request.');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleReject = async () => {
    setErrorMsg(null);
    const cleanReason = rejectionReason.trim();
    if (!cleanReason) {
      setErrorMsg('Rejection reason is required when rejecting a correction request.');
      return;
    }

    setIsSubmitting(true);
    try {
      const payload: RejectCorrectionPayload = {
        reviewer_name: reviewerName.trim() || undefined,
        rejection_reason: cleanReason,
      };
      await attendanceApi.rejectCorrection(request.id, payload);
      if (showSuccessToast) {
        showSuccessToast(`Correction request #${request.id} rejected.`);
      }
      onSuccess();
      onClose();
    } catch (err: unknown) {
      setErrorMsg(err instanceof Error ? err.message : 'Failed to reject correction request.');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/75 backdrop-blur-xs animate-fadeIn">
      <div className="relative w-full max-w-xl bg-[#0A1326] border border-[#16274E] rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]">
        {/* Header */}
        <div className="flex items-center justify-between p-5 border-b border-[#16274E] bg-gradient-to-r from-[#0A1326] via-[#0E1B38] to-[#0A1326]">
          <div className="flex items-center gap-3">
            <div
              className={`w-9 h-9 rounded-xl flex items-center justify-center text-white shadow-md ${
                request.status === 'APPROVED'
                  ? 'bg-gradient-to-tr from-[#059669] to-[#10B981]'
                  : request.status === 'REJECTED'
                  ? 'bg-gradient-to-tr from-[#B91C1C] to-[#EF4444]'
                  : 'bg-gradient-to-tr from-[#D97706] to-[#F59E0B]'
              }`}
            >
              {request.status === 'APPROVED' ? (
                <CheckCircle2 className="w-5 h-5" />
              ) : request.status === 'REJECTED' ? (
                <XCircle className="w-5 h-5" />
              ) : (
                <Clock className="w-5 h-5" />
              )}
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h3 className="text-base font-bold text-white tracking-tight">
                  Correction Request #{request.id}
                </h3>
                <span
                  className={`inline-flex items-center gap-1 text-[10px] font-bold tracking-wider uppercase px-2 py-0.5 rounded-full border ${
                    request.status === 'APPROVED'
                      ? 'bg-[#064E3B]/40 text-[#10B981] border-[#059669]/40'
                      : request.status === 'REJECTED'
                      ? 'bg-[#3B1219] text-[#F87171] border-[#771D2D]'
                      : 'bg-[#78350F]/40 text-[#F59E0B] border-[#B45309]/50 animate-pulse'
                  }`}
                >
                  {request.status}
                </span>
              </div>
              <p className="text-xs text-[#889DC7]">
                {request.correction_type === 'MISSED_CLOCK_IN' ? 'Missed Clock-in' : 'Missed Clock-out'} for {request.attendance_date}
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

        {/* Security / Reviewer Demo Banner */}
        <div className="bg-[#78350F]/20 border-b border-[#B45309]/30 px-5 py-2 text-[11px] text-[#FBBF24] flex items-center gap-2">
          <ShieldAlert className="w-4 h-4 text-[#F59E0B] shrink-0" />
          <span>
            <strong>Simulated HR Review:</strong> In this demo environment, manager/HR authorization is client-simulated. Production systems require OAuth2/JWT role-based access control.
          </span>
        </div>

        {/* Content Body */}
        <div className="p-5 space-y-4 overflow-y-auto flex-1 text-xs">
          {errorMsg && (
            <div className="p-3 rounded-xl bg-[#3B1219] border border-[#771D2D] text-xs text-[#F87171] flex items-start gap-2.5 animate-shake">
              <AlertCircle className="w-4 h-4 shrink-0 mt-0.5" />
              <span>{errorMsg}</span>
            </div>
          )}

          {/* Employee & Date Summary */}
          <div className="grid grid-cols-2 gap-3 p-3.5 rounded-xl bg-[#070D1A]/80 border border-[#16274E]">
            <div className="flex items-center gap-2.5">
              <div
                className="w-8 h-8 rounded-lg flex items-center justify-center font-bold text-white text-xs shrink-0"
                style={{ backgroundColor: request.avatar_color || '#0B6FDB' }}
              >
                {request.employee_name ? request.employee_name.charAt(0) : 'E'}
              </div>
              <div>
                <span className="text-[10px] text-[#5B6787] block uppercase font-bold tracking-wider">
                  Employee
                </span>
                <span className="font-semibold text-white truncate block">
                  {request.employee_name || `Employee #${request.employee_id}`}
                </span>
                <span className="text-[10px] text-[#889DC7]">
                  {request.employee_department || 'General'}
                </span>
              </div>
            </div>

            <div>
              <span className="text-[10px] text-[#5B6787] block uppercase font-bold tracking-wider">
                Attendance Date
              </span>
              <span className="font-semibold text-white flex items-center gap-1.5 mt-0.5">
                <Calendar className="w-3.5 h-3.5 text-[#38BDF8]" />
                {request.attendance_date}
              </span>
              <span className="text-[10px] text-[#5B6787]">
                Requested on {new Date(request.created_at).toLocaleDateString()}
              </span>
            </div>
          </div>

          {/* Requested Correction Details */}
          <div className="p-3.5 rounded-xl bg-[#0D1833] border border-[#1E3466] space-y-2.5">
            <div className="flex items-center justify-between">
              <span className="text-[11px] font-bold text-[#38BDF8] uppercase tracking-wider flex items-center gap-1.5">
                <Clock className="w-3.5 h-3.5" /> Requested Correction
              </span>
              <span className="text-[10px] font-medium px-2 py-0.5 rounded-md bg-[#0A1326] text-white border border-[#16274E]">
                {request.correction_type === 'MISSED_CLOCK_IN' ? 'Clock In Time' : 'Clock Out Time'}
              </span>
            </div>

            <div className="flex items-center justify-between pt-1">
              <span className="text-[#889DC7]">Correction Type:</span>
              <span className="font-bold text-white">
                {request.correction_type === 'MISSED_CLOCK_IN' ? 'Missed Clock-in' : 'Missed Clock-out'}
              </span>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-[#889DC7]">Requested Time:</span>
              <span className="text-base font-black font-mono text-[#38BDF8]">
                {formatTimeDisplay(request.requested_time)}
              </span>
            </div>

            <div className="pt-2 border-t border-[#16274E]">
              <span className="text-[10px] text-[#5B6787] uppercase font-bold tracking-wider block mb-1">
                Reason Provided by Employee
              </span>
              <p className="text-white italic bg-[#070D1A] p-2.5 rounded-lg border border-[#16274E]">
                "{request.reason}"
              </p>
            </div>
          </div>

          {/* Historical Audit Trail Snapshot */}
          <div className="p-3.5 rounded-xl bg-[#070D1A]/60 border border-[#16274E] space-y-2">
            <span className="text-[10px] text-[#5B6787] uppercase font-bold tracking-wider flex items-center gap-1.5">
              <History className="w-3 h-3 text-[#889DC7]" /> Original Attendance Snapshot (Audit Trail)
            </span>
            <div className="grid grid-cols-3 gap-2 text-center pt-1">
              <div className="bg-[#0A1326] p-2 rounded-lg border border-[#16274E]">
                <span className="text-[9px] text-[#5B6787] block">Prior Clock-In</span>
                <span className="font-mono font-semibold text-white">
                  {request.previous_in_time ? formatTimeDisplay(request.previous_in_time) : 'None'}
                </span>
              </div>
              <div className="bg-[#0A1326] p-2 rounded-lg border border-[#16274E]">
                <span className="text-[9px] text-[#5B6787] block">Prior Clock-Out</span>
                <span className="font-mono font-semibold text-white">
                  {request.previous_out_time ? formatTimeDisplay(request.previous_out_time) : 'None'}
                </span>
              </div>
              <div className="bg-[#0A1326] p-2 rounded-lg border border-[#16274E]">
                <span className="text-[9px] text-[#5B6787] block">Prior Hours</span>
                <span className="font-mono font-semibold text-[#889DC7]">
                  {request.previous_hours_worked !== null && request.previous_hours_worked !== undefined
                    ? `${request.previous_hours_worked}h`
                    : '0.00h'}
                </span>
              </div>
            </div>
          </div>

          {/* Review Details if already reviewed */}
          {!isPending && (
            <div
              className={`p-3.5 rounded-xl border ${
                request.status === 'APPROVED'
                  ? 'bg-[#064E3B]/20 border-[#059669]/40'
                  : 'bg-[#3B1219]/40 border-[#771D2D]'
              }`}
            >
              <div className="flex items-center gap-2 mb-2 font-bold">
                {request.status === 'APPROVED' ? (
                  <>
                    <ShieldCheck className="w-4 h-4 text-[#10B981]" />
                    <span className="text-[#10B981]">Approval Audit Details</span>
                  </>
                ) : (
                  <>
                    <XCircle className="w-4 h-4 text-[#F87171]" />
                    <span className="text-[#F87171]">Rejection Audit Details</span>
                  </>
                )}
              </div>
              <div className="space-y-1.5 text-[11px] text-[#889DC7]">
                <div>
                  <span className="text-[#5B6787]">Reviewed By: </span>
                  <span className="text-white font-semibold">{request.reviewer_name || 'HR Manager'}</span>
                </div>
                {request.reviewed_at && (
                  <div>
                    <span className="text-[#5B6787]">Reviewed On: </span>
                    <span className="text-white">{new Date(request.reviewed_at).toLocaleString()}</span>
                  </div>
                )}
                {request.rejection_reason && (
                  <div className="mt-2 pt-2 border-t border-[#771D2D]/50">
                    <span className="text-[#F87171] font-semibold block mb-0.5">Rejection Reason:</span>
                    <p className="text-white italic">{request.rejection_reason}</p>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Interactive Reviewer Form for Pending Requests */}
          {isPending && (
            <div className="p-3.5 rounded-xl bg-[#0A1326] border border-[#16274E] space-y-3">
              <div>
                <label className="block text-xs font-semibold text-[#889DC7] mb-1">
                  Reviewer Persona (Demo Mode)
                </label>
                <div className="relative">
                  <input
                    type="text"
                    value={reviewerName}
                    onChange={(e) => setReviewerName(e.target.value)}
                    className="w-full bg-[#070D1A] border border-[#16274E] focus:border-[#38BDF8] text-white text-xs rounded-xl px-3 py-2 focus:outline-hidden"
                  />
                  <User className="w-3.5 h-3.5 text-[#889DC7] absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none" />
                </div>
              </div>

              {activeAction === 'REJECT' && (
                <div className="animate-fadeIn space-y-1">
                  <label className="block text-xs font-semibold text-[#F87171] mb-1">
                    Rejection Reason <span className="text-[#F87171]">*</span>
                  </label>
                  <textarea
                    rows={2}
                    value={rejectionReason}
                    onChange={(e) => setRejectionReason(e.target.value)}
                    placeholder="Explain why this request is rejected (e.g. CCTV shows arrival at 10:15, shift roster mismatch)..."
                    className="w-full bg-[#070D1A] border border-[#771D2D] focus:border-[#F87171] text-white text-xs rounded-xl p-2.5 focus:outline-hidden placeholder-[#5B6787] resize-none"
                    required
                  />
                  <p className="text-[10px] text-[#5B6787]">
                    A clear rejection reason is mandatory and will be visible in the employee audit history.
                  </p>
                </div>
              )}

              {activeAction === 'APPROVE' && (
                <div className="animate-fadeIn space-y-1">
                  <label className="block text-xs font-semibold text-[#10B981] mb-1">
                    Approval Notes (Optional)
                  </label>
                  <input
                    type="text"
                    value={approvalNotes}
                    onChange={(e) => setApprovalNotes(e.target.value)}
                    placeholder="e.g. Verified with security gate log"
                    className="w-full bg-[#070D1A] border border-[#059669]/40 focus:border-[#10B981] text-white text-xs rounded-xl px-3 py-2 focus:outline-hidden placeholder-[#5B6787]"
                  />
                  <p className="text-[10px] text-[#889DC7]">
                    Approving will immediately update <code className="text-[#38BDF8]">daily_attendance</code> with <code className="text-[#38BDF8]">entry_status = 'REGULARISED'</code> and recalculate hours.
                  </p>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Footer Actions */}
        <div className="flex items-center justify-between p-4 border-t border-[#16274E] bg-[#0A1326]">
          <button
            type="button"
            onClick={onClose}
            disabled={isSubmitting}
            className="px-4 py-2 rounded-xl text-xs font-semibold text-[#889DC7] hover:text-white hover:bg-[#16274E] transition-colors cursor-pointer disabled:opacity-50"
          >
            Close
          </button>

          {isPending && (
            <div className="flex items-center gap-2">
              {activeAction === 'IDLE' && (
                <>
                  <button
                    type="button"
                    onClick={() => setActiveAction('REJECT')}
                    className="flex items-center gap-1.5 px-4 py-2 rounded-xl text-xs font-semibold text-[#F87171] bg-[#3B1219] hover:bg-[#521922] border border-[#771D2D] transition-colors cursor-pointer"
                  >
                    <XCircle className="w-3.5 h-3.5" />
                    <span>Reject...</span>
                  </button>

                  <button
                    type="button"
                    onClick={() => setActiveAction('APPROVE')}
                    className="flex items-center gap-1.5 px-4 py-2 rounded-xl text-xs font-semibold text-white bg-[#059669] hover:bg-[#047857] shadow-md shadow-[#059669]/25 transition-all cursor-pointer"
                  >
                    <CheckCircle2 className="w-3.5 h-3.5" />
                    <span>Approve...</span>
                  </button>
                </>
              )}

              {activeAction === 'APPROVE' && (
                <>
                  <button
                    type="button"
                    onClick={() => setActiveAction('IDLE')}
                    disabled={isSubmitting}
                    className="px-3 py-1.5 rounded-xl text-xs text-[#889DC7] hover:text-white"
                  >
                    Back
                  </button>
                  <button
                    type="button"
                    onClick={handleApprove}
                    disabled={isSubmitting}
                    className="flex items-center gap-2 px-5 py-2 rounded-xl text-xs font-bold text-white bg-gradient-to-r from-[#059669] to-[#10B981] hover:from-[#047857] hover:to-[#059669] shadow-lg shadow-[#059669]/25 transition-all cursor-pointer disabled:opacity-50"
                  >
                    {isSubmitting ? (
                      <div className="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    ) : (
                      <CheckCircle2 className="w-4 h-4" />
                    )}
                    <span>Confirm Approval</span>
                  </button>
                </>
              )}

              {activeAction === 'REJECT' && (
                <>
                  <button
                    type="button"
                    onClick={() => setActiveAction('IDLE')}
                    disabled={isSubmitting}
                    className="px-3 py-1.5 rounded-xl text-xs text-[#889DC7] hover:text-white"
                  >
                    Back
                  </button>
                  <button
                    type="button"
                    onClick={handleReject}
                    disabled={isSubmitting || !rejectionReason.trim()}
                    className="flex items-center gap-2 px-5 py-2 rounded-xl text-xs font-bold text-white bg-gradient-to-r from-[#B91C1C] to-[#EF4444] hover:from-[#991B1B] hover:to-[#DC2626] shadow-lg shadow-[#B91C1C]/25 transition-all cursor-pointer disabled:opacity-50"
                  >
                    {isSubmitting ? (
                      <div className="w-3.5 h-3.5 border-2 border-white border-t-transparent rounded-full animate-spin" />
                    ) : (
                      <XCircle className="w-4 h-4" />
                    )}
                    <span>Confirm Rejection</span>
                  </button>
                </>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

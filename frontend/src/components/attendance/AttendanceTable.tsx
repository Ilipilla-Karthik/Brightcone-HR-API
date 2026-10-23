import React from 'react';
import { Clock, Edit2, Archive, ArchiveRestore, Trash2 } from 'lucide-react';
import type { DailyAttendanceRecord } from '../../types';

interface AttendanceTableProps {
  records: DailyAttendanceRecord[];
  isLoading: boolean;
  onEdit: (record: DailyAttendanceRecord) => void;
  onToggleArchive: (record: DailyAttendanceRecord) => void;
  onDelete: (record: DailyAttendanceRecord) => void;
  onAddClick: () => void;
}

export const AttendanceTable: React.FC<AttendanceTableProps> = ({
  records,
  isLoading,
  onEdit,
  onToggleArchive,
  onDelete,
  onAddClick,
}) => {
  const formatTime = (timeStr: string | null) => {
    if (!timeStr) return '--';
    const parts = timeStr.split(':');
    if (parts.length < 2) return timeStr;
    const hours = parseInt(parts[0], 10);
    const minutes = parts[1];
    const ampm = hours >= 12 ? 'PM' : 'AM';
    const formattedHours = hours % 12 || 12;
    return `${String(formattedHours).padStart(2, '0')}:${minutes} ${ampm}`;
  };

  const getAttendanceBadge = (status: string) => {
    switch (status.toUpperCase()) {
      case 'PRESENT':
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-[#063321] text-[#34D399] border border-[#0B6440]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#34D399]" />
            Present
          </span>
        );
      case 'ABSENT':
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-[#3B1219] text-[#F87171] border border-[#771D2D]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#F87171]" />
            Absent
          </span>
        );
      case 'HALF_DAY':
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-[#36210A] text-[#FBBF24] border border-[#6D4212]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#FBBF24]" />
            Half day
          </span>
        );
      case 'ON_LEAVE':
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-[#211843] text-[#A78BFA] border border-[#442E86]">
            <span className="w-1.5 h-1.5 rounded-full bg-[#A78BFA]" />
            On leave
          </span>
        );
      default:
        return (
          <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-[#111C33] text-[#94A3B8] border border-[#1E2D4A]">
            {status}
          </span>
        );
    }
  };

  const getEntryStatusBadge = (status: string) => {
    switch (status.toUpperCase()) {
      case 'RECORDED':
        return (
          <span className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-[#0D2447] text-[#38BDF8] border border-[#173E73]">
            Recorded
          </span>
        );
      case 'PENDING':
        return (
          <span className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-[#36210A] text-[#FBBF24] border border-[#6D4212]">
            Pending
          </span>
        );
      case 'REGULARISED':
        return (
          <span className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-[#063321] text-[#34D399] border border-[#0B6440]">
            Regularised
          </span>
        );
      default:
        return (
          <span className="inline-flex items-center px-2 py-0.5 rounded-md text-[11px] font-medium bg-[#111C33] text-[#94A3B8]">
            {status}
          </span>
        );
    }
  };

  return (
    <div className="w-full overflow-x-auto">
      <table className="w-full text-left text-xs border-collapse">
        <thead>
          <tr className="border-b border-[#16274E] text-[10px] uppercase font-bold tracking-wider text-[#546894] bg-[#070E1C]">
            <th className="py-3 px-4">Employee</th>
            <th className="py-3 px-4">Date</th>
            <th className="py-3 px-4">Attendance</th>
            <th className="py-3 px-4">In Time</th>
            <th className="py-3 px-4">Out Time</th>
            <th className="py-3 px-4">Hours Worked</th>
            <th className="py-3 px-4">Entry Status</th>
            <th className="py-3 px-4 text-right">Actions</th>
          </tr>
        </thead>
        <tbody className="divide-y divide-[#13203E] bg-[#081022]">
          {isLoading ? (
            <tr>
              <td colSpan={8} className="py-16 text-center text-[#546894]">
                <div className="inline-flex items-center gap-2">
                  <div className="w-4 h-4 border-2 border-[#0B6FDB] border-t-transparent rounded-full animate-spin" />
                  <span>Loading attendance records...</span>
                </div>
              </td>
            </tr>
          ) : records.length === 0 ? (
            <tr>
              <td colSpan={8} className="py-20 text-center">
                <div className="flex flex-col items-center justify-center">
                  <div className="w-12 h-12 rounded-full bg-[#0D1C38] border border-[#1B3568] flex items-center justify-center text-[#38BDF8] mb-4">
                    <Clock className="w-6 h-6" />
                  </div>
                  <p className="text-xs text-[#889DC7]">
                    No daily attendance yet. Use{' '}
                    <button
                      type="button"
                      onClick={onAddClick}
                      className="font-semibold text-white hover:text-[#38BDF8] underline underline-offset-2 transition-colors cursor-pointer"
                    >
                      Add attendance entry
                    </button>{' '}
                    to create the first one.
                  </p>
                </div>
              </td>
            </tr>
          ) : (
            records.map((rec) => (
              <tr
                key={rec.id}
                className={`hover:bg-[#0D1935] transition-colors ${
                  rec.is_archived ? 'opacity-60 bg-[#060D1E]/60' : ''
                }`}
              >
                {/* Employee info */}
                <td className="py-3.5 px-4">
                  <div className="flex items-center gap-3">
                    <div
                      className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold text-white shrink-0 shadow-xs"
                      style={{ backgroundColor: rec.employee_avatar_color || '#3B82F6' }}
                    >
                      {rec.employee_name
                        .split(' ')
                        .map((n) => n[0])
                        .slice(0, 2)
                        .join('')
                        .toUpperCase()}
                    </div>
                    <div>
                      <div className="font-semibold text-white">{rec.employee_name}</div>
                      <div className="text-[11px] text-[#546894]">
                        {rec.employee_department} · {rec.employee_email}
                      </div>
                    </div>
                  </div>
                </td>

                {/* Date */}
                <td className="py-3.5 px-4 font-medium text-[#C9D3EE]">
                  {rec.attendance_date}
                </td>

                {/* Attendance Status Badge */}
                <td className="py-3.5 px-4">
                  {getAttendanceBadge(rec.attendance_status)}
                </td>

                {/* In Time */}
                <td className="py-3.5 px-4 font-mono text-[#A8B7DC]">
                  {formatTime(rec.in_time)}
                </td>

                {/* Out Time */}
                <td className="py-3.5 px-4 font-mono text-[#A8B7DC]">
                  {formatTime(rec.out_time)}
                </td>

                {/* Hours Worked */}
                <td className="py-3.5 px-4 font-medium text-white">
                  {rec.hours_worked > 0 ? `${rec.hours_worked.toFixed(1)} hrs` : '0 hrs'}
                </td>

                {/* Entry Status */}
                <td className="py-3.5 px-4">
                  {getEntryStatusBadge(rec.entry_status)}
                </td>

                {/* Actions */}
                <td className="py-3.5 px-4 text-right">
                  <div className="flex items-center justify-end gap-1.5">
                    <button
                      type="button"
                      onClick={() => onEdit(rec)}
                      className="p-1.5 rounded-lg text-[#889DC7] hover:text-white hover:bg-[#16274E] transition-colors cursor-pointer"
                      title="Edit attendance"
                    >
                      <Edit2 className="w-3.5 h-3.5" />
                    </button>
                    <button
                      type="button"
                      onClick={() => onToggleArchive(rec)}
                      className="p-1.5 rounded-lg text-[#889DC7] hover:text-amber-400 hover:bg-[#16274E] transition-colors cursor-pointer"
                      title={rec.is_archived ? 'Unarchive record' : 'Archive record'}
                    >
                      {rec.is_archived ? (
                        <ArchiveRestore className="w-3.5 h-3.5" />
                      ) : (
                        <Archive className="w-3.5 h-3.5" />
                      )}
                    </button>
                    <button
                      type="button"
                      onClick={() => onDelete(rec)}
                      className="p-1.5 rounded-lg text-[#889DC7] hover:text-rose-400 hover:bg-[#16274E] transition-colors cursor-pointer"
                      title="Delete record"
                    >
                      <Trash2 className="w-3.5 h-3.5" />
                    </button>
                  </div>
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
};

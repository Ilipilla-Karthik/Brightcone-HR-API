import React from 'react';
import { Search, ChevronDown } from 'lucide-react';

interface AttendanceFilterBarProps {
  search: string;
  onSearchChange: (val: string) => void;
  status: string;
  onStatusChange: (val: string) => void;
  showArchived: boolean;
  onToggleArchived: (val: boolean) => void;
  archivedCount: number;
}

export const AttendanceFilterBar: React.FC<AttendanceFilterBarProps> = ({
  search,
  onSearchChange,
  status,
  onStatusChange,
  showArchived,
  onToggleArchived,
  archivedCount,
}) => {
  return (
    <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between gap-3 px-4 py-3 bg-[#0A1326] border-b border-[#16274E]">
      <div className="flex flex-1 flex-col sm:flex-row items-stretch sm:items-center gap-3">
        {/* Search Input */}
        <div className="relative flex-1 max-w-md">
          <Search className="w-4 h-4 text-[#546894] absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
          <input
            type="text"
            placeholder="Search daily attendance"
            value={search}
            onChange={(e) => onSearchChange(e.target.value)}
            className="w-full pl-9 pr-3 py-1.5 rounded-xl text-xs bg-[#0E1A35] border border-[#1C3262] text-white placeholder-[#546894] focus:outline-hidden focus:border-[#0B6FDB] transition-all"
          />
        </div>

        {/* Status Dropdown */}
        <div className="relative inline-block w-40">
          <select
            value={status}
            onChange={(e) => onStatusChange(e.target.value)}
            className="w-full appearance-none pl-3 pr-8 py-1.5 rounded-xl text-xs font-medium bg-[#0E1A35] border border-[#1C3262] text-white focus:outline-hidden focus:border-[#0B6FDB] cursor-pointer"
          >
            <option value="ALL">All statuses</option>
            <option value="PRESENT">Present</option>
            <option value="ABSENT">Absent</option>
            <option value="HALF_DAY">Half day</option>
            <option value="ON_LEAVE">On leave</option>
          </select>
          <ChevronDown className="w-3.5 h-3.5 text-[#546894] absolute right-2.5 top-1/2 -translate-y-1/2 pointer-events-none" />
        </div>
      </div>

      {/* Archived Checkbox */}
      <label className="flex items-center gap-2 cursor-pointer text-xs font-medium text-[#889DC7] select-none hover:text-white transition-colors self-end sm:self-center">
        <input
          type="checkbox"
          checked={showArchived}
          onChange={(e) => onToggleArchived(e.target.checked)}
          className="w-4 h-4 rounded-md border-[#1C3262] bg-[#0E1A35] text-[#0B6FDB] focus:ring-0 focus:ring-offset-0 cursor-pointer accent-[#0B6FDB]"
        />
        <span>Archived ({archivedCount})</span>
      </label>
    </div>
  );
};

import React from 'react';
import { Users } from 'lucide-react';

interface NoEmployeesBannerProps {
  onOpenEmployeeMaster?: () => void;
}

export const NoEmployeesBanner: React.FC<NoEmployeesBannerProps> = ({ onOpenEmployeeMaster }) => {
  return (
    <div className="mb-6 rounded-2xl bg-[#231A0B] border border-[#5E400B] px-5 py-3.5 flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-amber-200 shadow-xs">
      <div className="flex items-center gap-3">
        <div className="w-8 h-8 rounded-full bg-[#3B290B] border border-[#78530C] flex items-center justify-center text-amber-400 shrink-0">
          <Users className="w-4 h-4" />
        </div>
        <p className="text-xs text-amber-200">
          <strong className="font-semibold text-amber-300">No employees yet.</strong>{' '}
          These records link to an Employee ID, so add your people in Employee master data first.
        </p>
      </div>
      <button
        type="button"
        onClick={onOpenEmployeeMaster}
        className="self-start sm:self-auto px-4 py-2 rounded-xl bg-[#1A1E29] hover:bg-[#252C3D] border border-[#3C4A66] text-white text-xs font-semibold transition-all shadow-xs shrink-0 cursor-pointer"
      >
        Open Employee master data
      </button>
    </div>
  );
};

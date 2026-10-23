import React from 'react';
import { Clock } from 'lucide-react';
import type { AttendanceSummary } from '../../types';

interface AttendanceSummaryCardsProps {
  summary: AttendanceSummary;
}

export const AttendanceSummaryCards: React.FC<AttendanceSummaryCardsProps> = ({ summary }) => {
  const cards = [
    {
      title: 'Total records',
      value: summary.total_records,
      subtext: 'Live records',
    },
    {
      title: 'Recorded',
      value: summary.recorded_count,
      subtext: 'Records',
    },
    {
      title: 'Pending',
      value: summary.pending_count,
      subtext: 'Records',
    },
    {
      title: 'Regularised',
      value: summary.regularised_count,
      subtext: 'Records',
    },
  ];

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
      {cards.map((card, idx) => (
        <div
          key={idx}
          className="bg-[#0A1326] border border-[#16274E] rounded-2xl p-5 flex flex-col justify-between shadow-xs transition-colors hover:border-[#1F376E]"
        >
          <div className="flex items-center gap-2.5 mb-3">
            <div className="w-8 h-8 rounded-xl bg-[#0D2044] border border-[#1A3870] flex items-center justify-center text-[#38BDF8] shrink-0">
              <Clock className="w-4 h-4" />
            </div>
            <span className="text-xs font-medium text-[#889DC7]">{card.title}</span>
          </div>
          <div>
            <div className="text-3xl font-extrabold text-white tracking-tight">
              {card.value}
            </div>
            <div className="text-[11px] font-medium text-[#526692] mt-0.5">
              {card.subtext}
            </div>
          </div>
        </div>
      ))}
    </div>
  );
};

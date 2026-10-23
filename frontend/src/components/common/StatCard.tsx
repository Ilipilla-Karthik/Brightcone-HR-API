import React from 'react';
import type { LucideIcon } from 'lucide-react';


interface StatCardProps {
  label: string;
  value: string | number;
  subtext?: string;
  icon: LucideIcon;
  badge?: {
    text: string;
    variant: 'ok' | 'warn' | 'bad' | 'info';
  };
}

export const StatCard: React.FC<StatCardProps> = ({
  label,
  value,
  subtext,
  icon: Icon,
  badge,
}) => {
  const getBadgeStyle = () => {
    switch (badge?.variant) {
      case 'ok':
        return 'bg-[#E3F6EE] dark:bg-[#0E3326] text-[#15A36B]';
      case 'warn':
        return 'bg-[#FFF4DB] dark:bg-[#3A2C08] text-[#C98200]';
      case 'bad':
        return 'bg-[#FDE8EA] dark:bg-[#3D141B] text-[#D93D4A]';
      case 'info':
      default:
        return 'bg-[#E3F0FF] dark:bg-[#16275A] text-[#0B6FDB]';
    }
  };

  return (
    <div className="bg-white dark:bg-[#0E1833] border border-[#E3E8F2] dark:border-[#1D2B52] rounded-2xl p-5 shadow-xs transition-all hover:shadow-md">
      <div className="flex items-center justify-between">
        <span className="text-xs font-medium text-[#5B6787] dark:text-[#8D97B0]">
          {label}
        </span>
        <div className="w-8 h-8 rounded-lg bg-[#E3F0FF] dark:bg-[#16275A] text-[#0B6FDB] flex items-center justify-center">
          <Icon className="w-4 h-4" />
        </div>
      </div>

      <div className="mt-3 flex items-baseline justify-between gap-2">
        <span className="font-['Sora'] text-2xl font-semibold text-[#0E1730] dark:text-[#E8EDFB]">
          {value}
        </span>
        {badge && (
          <span className={`text-[11px] font-semibold px-2 py-0.5 rounded-full ${getBadgeStyle()}`}>
            {badge.text}
          </span>
        )}
      </div>

      {subtext && (
        <p className="mt-1 text-xs text-[#8D97B0] dark:text-[#6A779A] truncate">
          {subtext}
        </p>
      )}
    </div>
  );
};

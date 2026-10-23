import React from 'react';
import { AlertCircle, CheckCircle2, RefreshCw } from 'lucide-react';

interface ApiStatusBannerProps {
  isBackendConnected: boolean | null;
  onRefresh: () => void;
  error?: string | null;
}

export const ApiStatusBanner: React.FC<ApiStatusBannerProps> = ({
  isBackendConnected,
  onRefresh,
  error,
}) => {
  if (isBackendConnected === true) {
    return (
      <div className="mb-6 flex items-center justify-between gap-3 p-3.5 rounded-xl bg-[#E3F6EE] dark:bg-[#0E3326] border border-[#15A36B]/30 text-[#15A36B] text-xs">
        <div className="flex items-center gap-2.5">
          <CheckCircle2 className="w-4 h-4 flex-none" />
          <span>
            <strong>Connected to Rust + Loco Backend:</strong> Real-time PostgreSQL database persistence active.
          </span>
        </div>
        <button
          onClick={onRefresh}
          className="flex items-center gap-1 font-semibold text-xs underline hover:opacity-80"
        >
          <RefreshCw className="w-3 h-3" /> Re-check
        </button>
      </div>
    );
  }

  return (
    <div className="mb-6 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 p-3.5 rounded-xl bg-[#FFF4DB] dark:bg-[#3A2C08] border border-[#C98200]/30 text-[#C98200] text-xs">
      <div className="flex items-center gap-2.5">
        <AlertCircle className="w-4 h-4 flex-none text-amber-500" />
        <span>
          <strong>Backend Standby:</strong> Live backend connection at <code className="px-1.5 py-0.5 rounded bg-black/5 dark:bg-white/10 font-mono text-[11px]">http://localhost:5150</code> is pending database migration approval. Displaying design reference state. {error ? `(${error})` : ''}
        </span>
      </div>
      <button
        onClick={onRefresh}
        className="flex items-center gap-1.5 px-3 py-1 rounded-lg bg-[#C98200] text-white font-semibold text-xs hover:bg-[#C98200]/90 transition-colors self-end sm:self-auto cursor-pointer"
      >
        <RefreshCw className="w-3 h-3" /> Check Backend
      </button>
    </div>
  );
};

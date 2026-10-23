import React, { useState } from 'react';
import { Send, AlertTriangle, X } from 'lucide-react';

interface PublishConfirmModalProps {
  isOpen: boolean;
  onClose: () => void;
  weekStart: string;
  weekEnd: string;
  onConfirm: () => Promise<void>;
}

export const PublishConfirmModal: React.FC<PublishConfirmModalProps> = ({
  isOpen,
  onClose,
  weekStart,
  weekEnd,
  onConfirm,
}) => {
  if (!isOpen) return null;

  const [isPublishing, setIsPublishing] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const handlePublish = async () => {
    setIsPublishing(true);
    setError(null);
    try {
      await onConfirm();
      onClose();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : 'Failed to publish roster');
    } finally {
      setIsPublishing(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs animate-in fade-in">
      <div className="relative w-full max-w-md bg-[#0B152E] border border-[#1D2B52] rounded-2xl shadow-2xl overflow-hidden text-[#E2E8F0]">
        <div className="flex items-center justify-between px-6 py-4 border-b border-[#18274A] bg-[#0E1A38]">
          <div className="flex items-center gap-2.5 text-white font-semibold text-base">
            <div className="w-8 h-8 rounded-xl bg-[#0B6FDB]/20 border border-[#0B6FDB]/40 flex items-center justify-center text-[#0BA6CC]">
              <Send className="w-4 h-4" />
            </div>
            <span>Publish Weekly Roster</span>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg hover:bg-white/10 text-[#8D9BB8] hover:text-white transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        <div className="p-6 space-y-4">
          <p className="text-sm text-[#C4D0EC] leading-relaxed">
            Are you sure you want to publish the weekly roster for{' '}
            <strong className="text-white font-semibold">
              {weekStart} to {weekEnd}
            </strong>
            ?
          </p>

          <div className="flex items-start gap-3 p-3.5 rounded-xl bg-[#142347] border border-[#1E3363] text-xs text-[#8D9BB8]">
            <AlertTriangle className="w-4 h-4 text-[#FFB020] shrink-0 mt-0.5" />
            <p className="leading-relaxed">
              Once published, all 14 employees will be notified of their scheduled shifts. You can still modify individual assignments if shift changes or swaps occur.
            </p>
          </div>

          {error && (
            <div className="p-3 text-xs bg-red-500/10 border border-red-500/30 text-red-400 rounded-xl">
              {error}
            </div>
          )}

          <div className="flex items-center justify-end gap-3 pt-3 border-t border-[#18274A]">
            <button
              type="button"
              onClick={onClose}
              disabled={isPublishing}
              className="px-4 py-2 rounded-xl text-xs font-semibold text-[#8D9BB8] hover:text-white hover:bg-white/5 transition-colors cursor-pointer"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handlePublish}
              disabled={isPublishing}
              className="px-5 py-2 rounded-xl text-xs font-semibold text-white bg-gradient-to-r from-[#0B6FDB] to-[#0BA6CC] hover:brightness-110 shadow-lg shadow-[#0B6FDB]/30 transition-all cursor-pointer flex items-center gap-2"
            >
              <Send className="w-3.5 h-3.5" />
              <span>{isPublishing ? 'Publishing...' : 'Confirm & Publish'}</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

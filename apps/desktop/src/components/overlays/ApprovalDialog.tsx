import { useEffect, useCallback, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import { sendApprovalDecision } from '@/lib/approvalRespond';
import type { CommandExecutionApprovalDecision } from '@/generated/app-server/v2/CommandExecutionApprovalDecision';
import type { FileChangeApprovalDecision } from '@/generated/app-server/v2/FileChangeApprovalDecision';
import type { ReviewDecision } from '@/generated/app-server/ReviewDecision';
import { cn } from '@/lib/utils';
import { AlertTriangle, Loader2 } from 'lucide-react';

const easePanel = [0.4, 0, 0.2, 1] as [number, number, number, number];

export default function ApprovalDialog() {
  const { state, dispatch } = useAppStore();
  const approvalDialog = state.approvalDialog;
  const [processing, setProcessing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isOpen = approvalDialog !== null;
  const isDangerous = approvalDialog?.isDangerous ?? false;
  const title = approvalDialog?.title ?? (isDangerous ? '允许执行危险操作？' : '允许执行命令？');

  const handleClose = useCallback(() => {
    dispatch({ type: 'SET_APPROVAL_DIALOG', payload: null });
    setProcessing(false);
    setError(null);
  }, [dispatch]);

  const submitDecision = useCallback(
    async (
      decision:
        | CommandExecutionApprovalDecision
        | FileChangeApprovalDecision
        | ReviewDecision,
    ) => {
      if (!approvalDialog) {
        handleClose();
        return;
      }
      setProcessing(true);
      setError(null);
      try {
        await sendApprovalDecision(approvalDialog, decision);
        handleClose();
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
        setProcessing(false);
      }
    },
    [approvalDialog, handleClose],
  );

  const handleDeny = useCallback(() => void submitDecision('decline'), [submitDecision]);
  const handleAllowOnce = useCallback(() => void submitDecision('accept'), [submitDecision]);
  const handleAlwaysAllow = useCallback(() => void submitDecision('acceptForSession'), [submitDecision]);

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleDeny();
        return;
      }
      if (e.key === 'Enter' && e.metaKey && !e.shiftKey) {
        e.preventDefault();
        handleAllowOnce();
        return;
      }
      if (e.key === 'a' && e.metaKey && e.shiftKey) {
        e.preventDefault();
        handleAlwaysAllow();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, handleDeny, handleAllowOnce, handleAlwaysAllow]);

  if (!approvalDialog) {
    return null;
  }

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <motion.div
            className="absolute inset-0 bg-[var(--bg-overlay)]"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2, ease: 'easeOut' }}
            onClick={handleDeny}
          />

          <motion.div
            role="dialog"
            aria-modal="true"
            aria-labelledby="approval-dialog-title"
            className="relative z-10 w-full max-w-[520px] mx-4 bg-[var(--bg-elevated)] rounded-xl shadow-[var(--glass-shadow)] overflow-hidden border border-[var(--border-default)]"
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.98 }}
            transition={{ duration: 0.25, ease: easePanel }}
          >
            {isDangerous && <div className="h-[3px] bg-[var(--status-error)]" />}

            <div className="p-6">
              <div className="flex items-center gap-2 mb-4">
                {isDangerous ? (
                  <AlertTriangle size={20} className="text-[var(--status-error)]" />
                ) : (
                  <div className="w-5 h-5 rounded-full bg-[var(--accent-primary-muted)] flex items-center justify-center">
                    <div className="w-2 h-2 rounded-full bg-[var(--accent-primary)]" />
                  </div>
                )}
                <h2
                  id="approval-dialog-title"
                  className="text-lg font-semibold text-[var(--text-primary)]"
                >
                  {title}
                </h2>
              </div>

              <p className="text-sm text-[var(--text-secondary)] mb-3">
                {approvalDialog.description}
              </p>

              <div
                className={cn(
                  'bg-[var(--bg-base)] rounded-lg p-3 mb-3 font-mono text-sm border-l-[3px]',
                  isDangerous ? 'border-l-[var(--status-error)]' : 'border-l-[var(--accent-primary)]',
                )}
              >
                <span className="text-[var(--text-tertiary)]">$ </span>
                <span className="text-[var(--text-primary)]">{approvalDialog.command}</span>
                {approvalDialog.cwd && (
                  <p className="text-[11px] text-[var(--text-tertiary)] mt-1.5">
                    工作目录: {approvalDialog.cwd}
                  </p>
                )}
              </div>

              {error && (
                <p className="text-xs text-[var(--status-error)] mb-3">{error}</p>
              )}

              {isDangerous && (
                <div className="flex items-start gap-2 p-3 rounded-md bg-[rgba(184,92,80,0.08)] mb-4">
                  <AlertTriangle size={14} className="text-[var(--status-error)] shrink-0 mt-0.5" />
                  <p className="text-xs text-[var(--status-error)]">此操作具有破坏性且可能无法撤销。</p>
                </div>
              )}

              <div className="flex items-center justify-end gap-2 mt-5">
                <button
                  type="button"
                  onClick={handleDeny}
                  disabled={processing}
                  className="h-9 px-4 text-[13px] font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-hover)] rounded-lg transition-all duration-150 disabled:opacity-50"
                >
                  拒绝
                </button>
                <button
                  type="button"
                  onClick={handleAllowOnce}
                  disabled={processing}
                  className="h-9 px-4 text-[13px] font-medium text-[var(--text-primary)] bg-[var(--bg-hover)] hover:bg-[var(--bg-active)] rounded-lg transition-all duration-150 disabled:opacity-50 flex items-center gap-2"
                >
                  {processing && <Loader2 size={14} className="animate-spin" />}
                  允许一次
                </button>
                <button
                  type="button"
                  onClick={handleAlwaysAllow}
                  disabled={processing}
                  className={cn(
                    'h-9 px-4 text-[13px] font-medium rounded-lg transition-all duration-150 disabled:opacity-50 text-[var(--text-inverse)]',
                    isDangerous
                      ? 'bg-[var(--status-error)] hover:opacity-90'
                      : 'bg-[var(--accent-primary)] hover:bg-[var(--accent-primary-hover)]',
                  )}
                >
                  始终允许
                </button>
              </div>
            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
}

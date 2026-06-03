/**
 * 无工作区时的审批回退条（文档区不可用时）
 */
import { useCallback, useState } from 'react';
import { AlertTriangle, Loader2 } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import { sendApprovalDecision } from '@/lib/approvalRespond';
import { cn } from '@/lib/utils';

export default function ApprovalPendingBanner() {
  const { state, dispatch } = useAppStore();
  const approval = state.approvalDialog;
  const [processing, setProcessing] = useState(false);

  const close = useCallback(() => {
    dispatch({ type: 'SET_APPROVAL_DIALOG', payload: null });
    setProcessing(false);
  }, [dispatch]);

  const submit = useCallback(
    async (decision: 'accept' | 'acceptForSession' | 'decline') => {
      if (!approval) {
        return;
      }
      setProcessing(true);
      try {
        await sendApprovalDecision(approval, decision);
        close();
      } catch {
        setProcessing(false);
      }
    },
    [approval, close],
  );

  if (!approval || approval.documentPath) {
    return null;
  }

  return (
    <div
      className={cn(
        'chat-column shrink-0 border-t border-[var(--border-default)]',
        'bg-[var(--bg-elevated)] py-2',
      )}
    >
      <div className="mb-2 flex items-center gap-2">
        {approval.isDangerous ? (
          <AlertTriangle size={14} className="text-[var(--status-error)]" />
        ) : null}
        <span className="text-xs font-medium text-[var(--text-primary)]">
          {approval.title}
        </span>
      </div>
      <p className="mb-2 truncate font-mono text-2xs text-[var(--text-secondary)]">
        {approval.command}
      </p>
      <div className="flex justify-end gap-2">
        <button
          type="button"
          disabled={processing}
          onClick={() => void submit('decline')}
          className="h-7 rounded px-2 text-2xs text-[var(--text-secondary)] hover:bg-[var(--bg-hover)]"
        >
          拒绝
        </button>
        <button
          type="button"
          disabled={processing}
          onClick={() => void submit('accept')}
          className="flex h-7 items-center gap-1 rounded bg-[var(--bg-hover)] px-2 text-2xs"
        >
          {processing && <Loader2 size={12} className="animate-spin" />}
          允许
        </button>
      </div>
    </div>
  );
}

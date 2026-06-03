/**
 * 文档区审批视图：展示摘要 Markdown，底部操作栏确认（替代全屏弹窗）
 */
import { useCallback, useEffect, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { AlertTriangle, Loader2 } from 'lucide-react';
import { api } from '@/api';
import { useAppStore } from '@/hooks/useAppStore';
import { sendApprovalDecision } from '@/lib/approvalRespond';
import { isApprovalDocumentPath, buildApprovalMarkdown } from '@/lib/approvalDocument';
import type { CommandExecutionApprovalDecision } from '@/generated/app-server/v2/CommandExecutionApprovalDecision';
import type { FileChangeApprovalDecision } from '@/generated/app-server/v2/FileChangeApprovalDecision';
import type { ReviewDecision } from '@/generated/app-server/ReviewDecision';
import type { ApprovalRequest } from '@/types';
import { cn } from '@/lib/utils';

interface ApprovalReviewViewProps {
  filePath: string;
}

function isActiveApprovalForPath(
  approval: ApprovalRequest,
  filePath: string,
): boolean {
  return (
    approval.documentPath === filePath ||
    (isApprovalDocumentPath(filePath) && approval.id !== '')
  );
}

export default function ApprovalReviewView({ filePath }: ApprovalReviewViewProps) {
  const { state, dispatch } = useAppStore();
  const approval = state.approvalDialog;
  const [markdown, setMarkdown] = useState('');
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [processing, setProcessing] = useState(false);

  const isActiveApproval =
    approval !== null && isActiveApprovalForPath(approval, filePath);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      setLoading(true);
      setLoadError(null);
      try {
        const text = await api.readFile(filePath);
        if (!cancelled) {
          setMarkdown(text);
        }
      } catch {
        if (!cancelled) {
          if (approval && isActiveApprovalForPath(approval, filePath)) {
            setMarkdown(buildApprovalMarkdown({ approval }));
          } else {
            setLoadError('无法加载审批摘要');
          }
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };
    void load();
    return () => {
      cancelled = true;
    };
  }, [filePath, approval]);

  const closeApproval = useCallback(() => {
    dispatch({ type: 'SET_APPROVAL_DIALOG', payload: null });
    setProcessing(false);
    setActionError(null);
  }, [dispatch]);

  const submitDecision = useCallback(
    async (
      decision:
        | CommandExecutionApprovalDecision
        | FileChangeApprovalDecision
        | ReviewDecision,
    ) => {
      if (!approval) {
        closeApproval();
        return;
      }
      setProcessing(true);
      setActionError(null);
      try {
        await sendApprovalDecision(approval, decision);
        closeApproval();
      } catch (err) {
        setActionError(err instanceof Error ? err.message : String(err));
        setProcessing(false);
      }
    },
    [approval, closeApproval],
  );

  useEffect(() => {
    if (!isActiveApproval) {
      return;
    }
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        void submitDecision('decline');
        return;
      }
      if (e.key === 'Enter' && e.metaKey && !e.shiftKey) {
        e.preventDefault();
        void submitDecision('accept');
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [isActiveApproval, submitDecision]);

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <Loader2 className="h-6 w-6 animate-spin text-[var(--text-tertiary)]" />
      </div>
    );
  }

  if (loadError) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-[var(--status-error)]">
        {loadError}
      </div>
    );
  }

  const isDangerous = approval?.isDangerous ?? false;
  const showActions = isActiveApproval && approval !== null;

  return (
    <div className="flex h-full min-h-0 flex-col bg-[var(--bg-base)]">
      {showActions && isDangerous && (
        <div className="h-[3px] shrink-0 bg-[var(--status-error)]" />
      )}

      {showActions && (
        <div
          className={cn(
            'flex shrink-0 items-center gap-2 border-b px-4 py-2',
            'border-[var(--border-default)] bg-[var(--bg-elevated)]',
          )}
        >
          {isDangerous ? (
            <AlertTriangle size={16} className="text-[var(--status-error)]" />
          ) : (
            <div className="flex h-4 w-4 items-center justify-center rounded-full bg-[var(--accent-primary-muted)]">
              <div className="h-1.5 w-1.5 rounded-full bg-[var(--accent-primary)]" />
            </div>
          )}
          <h1 className="text-sm font-semibold text-[var(--text-primary)]">
            {approval.title}
          </h1>
          <span className="text-2xs text-[var(--text-tertiary)]">
            文档区审批 · 可编辑上方 Markdown 备注
          </span>
        </div>
      )}

      <div className="min-h-0 flex-1 overflow-auto p-6 custom-scrollbar">
        <div className="prose prose-sm max-w-3xl dark:prose-invert">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{markdown}</ReactMarkdown>
        </div>
      </div>

      {showActions && (
        <div
          className={cn(
            'shrink-0 border-t border-[var(--border-default)]',
            'bg-[var(--bg-elevated)] px-4 py-3',
          )}
        >
          {actionError && (
            <p className="mb-2 text-xs text-[var(--status-error)]">{actionError}</p>
          )}
          {isDangerous && (
            <p className="mb-2 flex items-center gap-1.5 text-xs text-[var(--status-error)]">
              <AlertTriangle size={12} />
              此操作具有破坏性且可能无法撤销
            </p>
          )}
          <div className="flex items-center justify-end gap-2">
            <button
              type="button"
              onClick={() => void submitDecision('decline')}
              disabled={processing}
              className="h-9 rounded-lg px-4 text-[13px] font-medium text-[var(--text-secondary)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] disabled:opacity-50"
            >
              拒绝
            </button>
            <button
              type="button"
              onClick={() => void submitDecision('accept')}
              disabled={processing}
              className="flex h-9 items-center gap-2 rounded-lg bg-[var(--bg-hover)] px-4 text-[13px] font-medium text-[var(--text-primary)] transition-colors hover:bg-[var(--bg-active)] disabled:opacity-50"
            >
              {processing && <Loader2 size={14} className="animate-spin" />}
              允许一次
            </button>
            <button
              type="button"
              onClick={() => void submitDecision('acceptForSession')}
              disabled={processing}
              className={cn(
                'h-9 rounded-lg px-4 text-[13px] font-medium text-[var(--text-inverse)] disabled:opacity-50',
                isDangerous
                  ? 'bg-[var(--status-error)] hover:opacity-90'
                  : 'bg-[var(--accent-primary)] hover:bg-[var(--accent-primary-hover)]',
              )}
            >
              始终允许
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

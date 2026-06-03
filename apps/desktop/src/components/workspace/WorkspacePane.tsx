import { useCallback, useMemo } from 'react';
import { Merge } from 'lucide-react';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/hooks/useAppStore';
import type { WorkspaceLeafNode } from '@/types';
import {
  canMergePane,
  countWorkspacePanes,
  getPaneDepth,
  getPaneOrdinal,
} from '@/lib/workspaceLayout';
import FilePreviewRouter from '@/components/preview/FilePreviewRouter';
import ApprovalReviewView from '@/components/preview/ApprovalReviewView';
import { isApprovalDocumentPath } from '@/lib/approvalDocument';
import WelcomeScreen from './WelcomeScreen';
import WorkspaceTabs from './WorkspaceTabs';
import SplitDropOverlay from './SplitDropOverlay';

interface WorkspacePaneProps {
  node: WorkspaceLeafNode;
}

export default function WorkspacePane({ node }: WorkspacePaneProps) {
  const { state, dispatch } = useAppStore();
  const isFocused = state.focusedPaneId === node.id;
  const activeTab =
    node.tabs.find((tab) => tab.id === node.activeTabId) ?? null;

  const paneLabel = useMemo(() => {
    const total = countWorkspacePanes(state.workspaceRoot);
    if (total <= 1) {
      return null;
    }
    const ordinal = getPaneOrdinal(state.workspaceRoot, node.id);
    const depth = getPaneDepth(state.workspaceRoot, node.id);
    return `编辑区 ${ordinal}/${total} · 层级 ${depth}`;
  }, [node.id, state.workspaceRoot]);

  const handleFocusPane = useCallback(() => {
    dispatch({ type: 'SET_FOCUSED_PANE', payload: node.id });
  }, [dispatch, node.id]);

  const handleSelectTab = useCallback(
    (tabId: string) => {
      dispatch({
        type: 'SET_ACTIVE_TAB',
        payload: { paneId: node.id, tabId },
      });
    },
    [dispatch, node.id],
  );

  const handleCloseTab = useCallback(
    (tabId: string) => {
      dispatch({ type: 'CLOSE_TAB', payload: tabId });
    },
    [dispatch],
  );

  const mergeAllowed =
    !!state.workspaceRoot && canMergePane(state.workspaceRoot, node.id);

  const handleCollapseSplits = useCallback(() => {
    dispatch({ type: 'COLLAPSE_WORKSPACE_SPLITS' });
  }, [dispatch]);

  return (
    <div
      className={cn(
        'flex h-full min-h-0 min-w-0 flex-col overflow-hidden',
        isFocused && 'ring-1 ring-inset ring-[var(--border-focus)]',
      )}
      onMouseDown={handleFocusPane}
    >
      {paneLabel && (
        <div
          className="flex shrink-0 items-center justify-between gap-2 px-2 py-0.5 text-[10px] text-[var(--text-tertiary)]"
          style={{ backgroundColor: 'var(--bg-elevated)' }}
        >
          <span>{paneLabel}</span>
          {mergeAllowed && (
            <button
              type="button"
              onClick={handleCollapseSplits}
              className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] text-[var(--text-secondary)] transition-colors hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]"
              title="关闭全部工作区分屏（⌘⌥M）"
            >
              <Merge size={12} />
              关闭分屏
            </button>
          )}
        </div>
      )}

      <WorkspaceTabs
        paneId={node.id}
        tabs={node.tabs}
        activeTabId={node.activeTabId}
        onSelect={handleSelectTab}
        onClose={handleCloseTab}
      />

      <div className="relative min-h-0 flex-1 overflow-hidden">
        {activeTab ? (
          isApprovalDocumentPath(activeTab.filePath) ? (
            <ApprovalReviewView
              key={activeTab.filePath}
              filePath={activeTab.filePath}
            />
          ) : (
            <FilePreviewRouter
              key={activeTab.filePath}
              filePath={activeTab.filePath}
            />
          )
        ) : (
          <WelcomeScreen />
        )}
        <SplitDropOverlay paneId={node.id} />
      </div>
    </div>
  );
}

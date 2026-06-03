import type { FC } from 'react';
import { useMemo } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { FileText, X, Terminal, FolderOpen } from 'lucide-react';
import CompareView from './CompareView';
import ReviewView from './ReviewView';
import SearchView from './SearchView';
import DraftView from './DraftView';
import PatentWorkflowPlaceholder from './PatentWorkflowPlaceholder';
import AgentWorkPane from './AgentWorkPane';
import { shouldShowPatentMockViews } from '@/lib/devMock';
import FilePreviewRouter from '@/components/preview/FilePreviewRouter';
import TerminalPanel from '@/components/terminal/TerminalPanel';
import StageIndicator from '@/components/stage/StageIndicator';
import TodoDock from '@/components/todo/TodoDock';
import { useAppStore } from '@/hooks/useAppStore';
import { isDesktopRpcReady } from '@/lib/configAccess';
import type { WorkStage } from '@/types';

interface CenterPanelProps {
  selectedFile: string | null;
  previewReloadKey?: number;
  onCloseFile: () => void;
}

const CenterPanel: FC<CenterPanelProps> = ({
  selectedFile,
  previewReloadKey = 0,
  onCloseFile,
}) => {
  const { state, dispatch } = useAppStore();
  const showTerminal = state.terminalOverlayOpen;
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const showPatentMock = shouldShowPatentMockViews(rpcReady);
  const hasWorkspace = !!state.workspaceCwd;

  const activeStage = useMemo(
    () => state.stages.find((s) => s.status === 'active')?.id ?? null,
    [state.stages],
  );

  const handleStageClick = (stage: WorkStage) => {
    dispatch({ type: 'UPDATE_STAGE', payload: { id: stage, status: 'active' } });
  };

  const renderView = () => {
    if (showTerminal) {
      return <TerminalPanel onClose={() => dispatch({ type: 'SET_TERMINAL_OVERLAY_OPEN', payload: false })} />;
    }

    if (!showPatentMock) {
      return (
        <AgentWorkPane
          messages={state.messages}
          activeStage={activeStage}
          isStreaming={state.isStreaming}
        />
      );
    }

    switch (activeStage) {
      case 'search':
        return <SearchView />;
      case 'compare':
        return <CompareView />;
      case 'review':
        return <ReviewView />;
      case 'draft':
        return <DraftView />;
      default:
        return <PatentWorkflowPlaceholder activeStage={null} />;
    }
  };

  const showEmptyState = !selectedFile && !hasWorkspace && !showTerminal;

  return (
    <div
      className="flex h-full flex-col overflow-hidden"
      style={{ backgroundColor: 'var(--bg-surface)' }}
    >
      <div
        className="flex items-center justify-between"
        style={{
          height: 40,
          padding: '0 8px',
          borderBottom: '1px solid var(--border-primary)',
          backgroundColor: 'var(--bg-elevated)',
        }}
      >
        {selectedFile ? (
          <div className="flex items-center gap-2">
            <FileText size={14} style={{ color: 'var(--accent-primary)' }} />
            <span
              className="max-w-[300px] truncate text-sm"
              style={{ color: 'var(--text-primary)' }}
            >
              {selectedFile.split('/').pop()}
            </span>
            <button
              onClick={onCloseFile}
              className="rounded p-1 transition-colors"
              style={{ color: 'var(--text-tertiary)' }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
                e.currentTarget.style.color = 'var(--text-secondary)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
                e.currentTarget.style.color = 'var(--text-tertiary)';
              }}
              type="button"
            >
              <X size={14} />
            </button>
          </div>
        ) : (
          <StageIndicator
            activeStage={activeStage}
            onStageClick={handleStageClick}
          />
        )}

        <button
          onClick={() => dispatch({ type: 'TOGGLE_TERMINAL_OVERLAY' })}
          className="relative flex items-center transition-colors duration-150"
          style={{
            height: 32,
            padding: '0 12px',
            borderRadius: 6,
            gap: 6,
            backgroundColor: showTerminal
              ? 'var(--bg-sidebar-active)'
              : 'transparent',
            color: showTerminal
              ? 'var(--accent-primary)'
              : 'var(--text-secondary)',
            fontSize: 12,
            fontWeight: showTerminal ? 500 : 400,
          }}
          type="button"
        >
          <Terminal size={14} />
          终端
        </button>
      </div>

      <div className="relative min-h-0 flex-1 overflow-hidden">
        <AnimatePresence mode="wait">
          {selectedFile ? (
            <motion.div
              key="file-preview"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="h-full"
            >
              <FilePreviewRouter
                key={`${selectedFile}-${previewReloadKey}`}
                filePath={selectedFile}
              />
            </motion.div>
          ) : showEmptyState ? (
            <motion.div
              key="empty-state"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="h-full flex flex-col items-center justify-center gap-3 px-8"
            >
              <FolderOpen size={36} style={{ color: 'var(--text-tertiary)', opacity: 0.4 }} />
              <p className="text-sm text-[var(--text-secondary)]">在左侧打开一个工作区目录开始工作</p>
              <p className="text-2xs text-[var(--text-tertiary)]">
                选择目录后可浏览文件、检索专利、对比分析
              </p>
            </motion.div>
          ) : (
            <motion.div
              key={activeStage ?? 'default'}
              initial={{ opacity: 0, x: 16 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0 }}
              transition={{
                duration: 0.25,
                ease: [0.4, 0, 0.2, 1] as [number, number, number, number],
              }}
              className="h-full flex flex-col"
            >
              <div className="min-h-0 flex-1">{renderView()}</div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      {!selectedFile && hasWorkspace && <TodoDock />}
    </div>
  );
};

export default CenterPanel;

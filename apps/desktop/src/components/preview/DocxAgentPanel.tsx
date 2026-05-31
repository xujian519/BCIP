import { useCallback, useRef, useState } from 'react';
import { cn } from '@/lib/utils';
import { useDocxAgentBridge } from '@/hooks/useDocxAgentBridge';
import DocxEditorView, { type DocxEditorViewRef } from '@/components/preview/DocxEditorView';

interface DocxAgentPanelProps {
  filePath: string;
  connectionReady: boolean;
  className?: string;
}

const AGENT_PRESETS = [
  { key: 'claims', label: '权利要求', task: '根据说明书内容生成或优化权利要求书' },
  { key: 'description', label: '说明书', task: '根据技术方案撰写详细说明书' },
  { key: 'abstract', label: '摘要', task: '根据权利要求和说明书生成专利摘要' },
  { key: 'oa-reply', label: 'OA答复', task: '根据审查意见撰写答复意见陈述书' },
] as const;

export default function DocxAgentPanel({
  filePath,
  connectionReady,
  className,
}: DocxAgentPanelProps) {
  const editorRef = useRef<DocxEditorViewRef>(null);
  const [agentTask, setAgentTask] = useState<string | null>(null);
  const [customTask, setCustomTask] = useState('');

  const bridge = useDocxAgentBridge({
    connectionReady,
    filePath,
    editorRef,
  });

  const handlePreset = useCallback(
    (preset: (typeof AGENT_PRESETS)[number]) => {
      setAgentTask(preset.task);
      void bridge.startDraft(preset.task);
    },
    [bridge],
  );

  const handleCustomTask = useCallback(() => {
    if (!customTask.trim()) return;
    setAgentTask(customTask);
    void bridge.startDraft(customTask);
    setCustomTask('');
  }, [customTask, bridge]);

  const handleCancel = useCallback(() => {
    void bridge.cancelCurrentAgent();
    setAgentTask(null);
  }, [bridge]);

  const handleRefresh = useCallback(() => {
    void bridge.checkAgentResult();
    void bridge.saveAndReload();
  }, [bridge]);

  return (
    <div className={cn('flex flex-col h-full', className)}>
      <div
        className="flex items-center gap-2 px-3 py-2 shrink-0"
        style={{
          backgroundColor: 'var(--bg-elevated)',
          borderBottom: '1px solid var(--border-subtle)',
        }}
      >
        <span className="text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
          专利助手
        </span>
        <div className="flex gap-1 ml-auto">
          {AGENT_PRESETS.map((preset) => (
            <button
              key={preset.key}
              onClick={() => handlePreset(preset)}
              disabled={bridge.drafting || !connectionReady}
              className={cn(
                'px-2 py-1 rounded text-xs transition-colors',
                'disabled:opacity-40 disabled:cursor-not-allowed',
              )}
              style={{
                backgroundColor: 'var(--bg-surface)',
                color: 'var(--text-primary)',
                border: '1px solid var(--border-default)',
              }}
              title={preset.task}
            >
              {preset.label}
            </button>
          ))}
        </div>
      </div>

      <div className="flex-1 min-h-0">
        <DocxEditorView ref={editorRef} filePath={filePath} />
      </div>

      {bridge.drafting && (
        <div
          className="flex items-center gap-2 px-3 py-2 shrink-0"
          style={{
            backgroundColor: 'var(--bg-elevated)',
            borderTop: '1px solid var(--border-subtle)',
          }}
        >
          <div
            className="animate-spin rounded-full h-4 w-4 border-b-2"
            style={{ borderColor: 'var(--accent-primary)' }}
          />
          <span className="text-xs" style={{ color: 'var(--text-secondary)' }}>
            {agentTask ? `正在处理：${agentTask.slice(0, 30)}…` : 'Agent 处理中…'}
          </span>
          <button
            onClick={handleCancel}
            className="ml-auto text-xs px-2 py-1 rounded"
            style={{ color: 'var(--status-error)' }}
          >
            取消
          </button>
        </div>
      )}

      {!bridge.drafting && connectionReady && (
        <div
          className="flex items-center gap-2 px-3 py-2 shrink-0"
          style={{
            backgroundColor: 'var(--bg-elevated)',
            borderTop: '1px solid var(--border-subtle)',
          }}
        >
          <input
            type="text"
            value={customTask}
            onChange={(e) => setCustomTask(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleCustomTask();
            }}
            placeholder="输入自定义任务…"
            className="flex-1 text-xs px-2 py-1 rounded outline-none"
            style={{
              backgroundColor: 'var(--bg-surface)',
              color: 'var(--text-primary)',
              border: '1px solid var(--border-default)',
            }}
          />
          <button
            onClick={handleRefresh}
            className="text-xs px-2 py-1 rounded"
            style={{ color: 'var(--accent-primary)' }}
          >
            刷新
          </button>
        </div>
      )}

      {bridge.agentError && (
        <div
          className="px-3 py-1 text-xs shrink-0"
          style={{ color: 'var(--status-error)', backgroundColor: 'var(--bg-elevated)' }}
        >
          {bridge.agentError}
        </div>
      )}
    </div>
  );
}

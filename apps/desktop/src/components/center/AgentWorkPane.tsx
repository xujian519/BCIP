import type { FC } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import type { WorkStage } from '@/types';
import { agentOutputForStage } from '@/lib/patentCenterContent';
import type { Message } from '@/types';

interface AgentWorkPaneProps {
  messages: Message[];
  activeStage: WorkStage | null;
  isStreaming: boolean;
}

/** 已连接时中心区：展示与阶段相关的 Agent 输出（非 mock 专利页） */
const AgentWorkPane: FC<AgentWorkPaneProps> = ({
  messages,
  activeStage,
  isStreaming,
}) => {
  const block = agentOutputForStage(messages, activeStage);

  if (!block) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 px-8 text-center">
        <p className="text-sm text-[var(--text-primary)]">等待 Agent 输出</p>
        <p className="max-w-md text-2xs text-[var(--text-secondary)]">
          在右侧输入专利任务（检索、对比、审查、起草等），生成结果会显示在此工作区。
        </p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="shrink-0 border-b border-[var(--border-default)] px-4 py-2">
        <h2 className="text-sm font-semibold text-[var(--text-primary)]">
          {block.title}
        </h2>
        {isStreaming && (
          <p className="text-2xs text-[var(--accent-primary)] mt-0.5">
            正在生成…
          </p>
        )}
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-4 py-3">
        <article className="prose prose-sm max-w-none text-[var(--text-primary)] dark:prose-invert">
          <ReactMarkdown remarkPlugins={[remarkGfm]}>{block.body}</ReactMarkdown>
        </article>
      </div>
    </div>
  );
};

export default AgentWorkPane;

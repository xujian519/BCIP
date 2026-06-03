import type { FC } from 'react';
import type { WorkStage } from '@/types';

const STAGE_HINT: Record<WorkStage, string> = {
  search: '专利检索',
  compare: '权利要求 / 技术对比',
  review: '审查意见分析',
  draft: '申请文件起草',
};

interface PatentWorkflowPlaceholderProps {
  activeStage: WorkStage | null;
}

/** 已连接 app-server 时替代 mock 专利视图 */
const PatentWorkflowPlaceholder: FC<PatentWorkflowPlaceholderProps> = ({
  activeStage,
}) => {
  const label = activeStage ? STAGE_HINT[activeStage] : '专利工作流';

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 px-8 text-center">
      <p className="text-sm font-medium text-[var(--text-primary)]">{label}</p>
      <p className="max-w-md text-2xs leading-relaxed text-[var(--text-secondary)]">
        该视图将从 Agent 对话与工具输出接入。请使用右侧助手检索、对比、审查或起草；
        在左侧打开文件可预览与标注，工作区变更会自动刷新。
      </p>
    </div>
  );
};

export default PatentWorkflowPlaceholder;

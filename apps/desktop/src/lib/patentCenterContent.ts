import type { Message } from '@/types';
import type { WorkStage } from '@/types';

const STAGE_KEYWORDS: Record<WorkStage, RegExp> = {
  search: /检索|现有技术|prior\s*art|搜索专利/i,
  compare: /对比|比较|相似|区别特征|D\d+/i,
  review: /审查|新颖性|创造性|答复|OA/i,
  draft: /撰写|起草|权利要求|说明书|申请文件/i,
};

/** 取与当前阶段最相关的一条 Agent 输出，供中心区展示 */
export function agentOutputForStage(
  messages: Message[],
  stage: WorkStage | null,
): { title: string; body: string } | null {
  const agents = messages.filter(
    (m) => m.role === 'agent' && m.content.trim().length > 0,
  );
  if (agents.length === 0) {
    return null;
  }

  if (stage) {
    const pattern = STAGE_KEYWORDS[stage];
    for (let i = agents.length - 1; i >= 0; i--) {
      const m = agents[i];
      if (pattern.test(m.content)) {
        return {
          title: stageTitle(stage),
          body: m.content,
        };
      }
    }
  }

  const last = agents[agents.length - 1];
  return {
    title: stage ? stageTitle(stage) : 'Agent 输出',
    body: last.content,
  };
}

function stageTitle(stage: WorkStage): string {
  switch (stage) {
    case 'search':
      return '检索分析';
    case 'compare':
      return '对比分析';
    case 'review':
      return '审查分析';
    case 'draft':
      return '撰写要点';
    default:
      return '工作区';
  }
}

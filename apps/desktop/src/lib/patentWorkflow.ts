/**
 * 专利工作流阶段：从用户消息或 Agent plan 文本推断并同步到 store。
 */
import type { Dispatch } from 'react';
import type { WorkStage, AppAction } from '@/types';

const STAGE_PATTERNS: { id: WorkStage; patterns: RegExp[] }[] = [
  {
    id: 'search',
    patterns: [/检索/i, /现有技术/i, /prior\s*art/i, /搜索专利/i],
  },
  {
    id: 'compare',
    patterns: [/对比/i, /比较/i, /相似度/i, /compare/i, /D\d+/i],
  },
  {
    id: 'review',
    patterns: [/审查/i, /新颖性/i, /创造性/i, /答复/i, /OA/i, /review/i],
  },
  {
    id: 'draft',
    patterns: [/撰写/i, /起草/i, /权利要求/i, /说明书/i, /draft/i, /申请文件/i],
  },
];

/** 从文本推断工作阶段；无匹配返回 null */
export function inferWorkStageFromText(text: string): WorkStage | null {
  const normalized = text.trim();
  if (!normalized) {
    return null;
  }
  for (const { id, patterns } of STAGE_PATTERNS) {
    if (patterns.some((p) => p.test(normalized))) {
      return id;
    }
  }
  return null;
}

/** 激活某一阶段（其余 active 回退为 pending） */
export function dispatchActivateWorkStage(
  dispatch: Dispatch<AppAction>,
  stage: WorkStage,
): void {
  dispatch({ type: 'UPDATE_STAGE', payload: { id: stage, status: 'active' } });
}

/** 根据 plan 步骤文本批量推断：取第一个匹配阶段 */
export function inferWorkStageFromPlanSteps(steps: string[]): WorkStage | null {
  for (const step of steps) {
    const stage = inferWorkStageFromText(step);
    if (stage) {
      return stage;
    }
  }
  return null;
}

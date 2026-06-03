/**
 * Slash 命令与专利工作流阶段联动（设计 spec §8.3）
 */
import type { WorkStage } from '@/types';

export interface PatentSlashAction {
  stage: WorkStage;
  prompt: string;
}

const SLASH_PATENT: Record<string, PatentSlashAction> = {
  search: {
    stage: 'search',
    prompt: '请检索与当前工作区技术方案相关的现有技术专利，并给出检索式建议。',
  },
  analyze: {
    stage: 'compare',
    prompt: '请对比当前打开的文件/项目与相近现有技术，列出区别技术特征。',
  },
  draft: {
    stage: 'draft',
    prompt: '请根据当前项目材料起草独立权利要求与说明书要点提纲。',
  },
};

export function patentSlashAction(commandId: string): PatentSlashAction | null {
  return SLASH_PATENT[commandId] ?? null;
}

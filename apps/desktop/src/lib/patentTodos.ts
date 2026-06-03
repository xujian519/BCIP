/**
 * Agent turn/plan 与 plan item → 中心 TodoDock
 */
import type { TurnPlanStep } from '@/generated/app-server/v2/TurnPlanStep';
import type { TodoItem } from '@/types';

function stableId(prefix: string, index: number, text: string): string {
  const slug = text.slice(0, 24).replace(/\s+/g, '-');
  return `${prefix}-${index}-${slug}`;
}

export function todosFromPlanSteps(steps: TurnPlanStep[]): TodoItem[] {
  const now = Date.now();
  return steps.map((step, index) => ({
    id: stableId('plan', index, step.step),
    text: step.step,
    completed: step.status === 'completed',
    createdAt: now + index,
  }));
}

/** 解析 plan item 的 Markdown 清单（- [ ] / - [x] / 数字列表） */
export function todosFromPlanText(text: string): TodoItem[] {
  const lines = text.split('\n');
  const now = Date.now();
  const items: TodoItem[] = [];
  let index = 0;

  for (const line of lines) {
    const trimmed = line.trim();
    const checkbox = trimmed.match(/^[-*]\s+\[([ xX])]\s+(.+)$/);
    if (checkbox) {
      items.push({
        id: stableId('plan-md', index, checkbox[2]),
        text: checkbox[2].trim(),
        completed: checkbox[1].toLowerCase() === 'x',
        createdAt: now + index,
      });
      index += 1;
      continue;
    }
    const numbered = trimmed.match(/^\d+[.)]\s+(.+)$/);
    if (numbered) {
      items.push({
        id: stableId('plan-num', index, numbered[1]),
        text: numbered[1].trim(),
        completed: false,
        createdAt: now + index,
      });
      index += 1;
    }
  }

  if (items.length === 0 && text.trim()) {
    items.push({
      id: stableId('plan', 0, text),
      text: text.trim().slice(0, 200),
      completed: false,
      createdAt: now,
    });
  }

  return items;
}

/** 是否来自 Agent plan（thread/plan 或 turn/planUpdated） */
export function isPlanTodoId(id: string): boolean {
  return id.startsWith('plan');
}

/** plan 文本是否含结构化步骤（checkbox / 编号列表） */
export function hasStructuredPlanSteps(text: string): boolean {
  const items = todosFromPlanText(text);
  if (items.length <= 1) {
    return false;
  }
  return items.some(
    (item) => item.id.includes('plan-md') || item.id.includes('plan-num'),
  );
}

/** 第一个未完成步骤的下标，无则 -1 */
export function activePlanStepIndex(steps: Pick<TodoItem, 'completed'>[]): number {
  return steps.findIndex((s) => !s.completed);
}

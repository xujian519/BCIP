/**
 * Plan item —— Agent 时间线内 plan 块（§8.3.3 + Codex plan 模式）
 */
import { cn } from '@/lib/utils';
import { ListTodo } from 'lucide-react';
import {
  activePlanStepIndex,
  hasStructuredPlanSteps,
  todosFromPlanText,
} from '@/lib/patentTodos';
import TodoRow from '@/components/todo/TodoRow';

interface PlanMessageBlockProps {
  content: string;
  timestamp?: string;
}

export default function PlanMessageBlock({
  content,
  timestamp,
}: PlanMessageBlockProps) {
  const steps = todosFromPlanText(content);
  const structured = hasStructuredPlanSteps(content);
  const activeIndex = activePlanStepIndex(steps);

  return (
    <div className="flex w-full flex-col">
      <div
        className={cn(
          'w-full overflow-hidden rounded-lg border border-[var(--plan-border)]',
          'border-l-2 border-l-[var(--plan-accent)]',
          'bg-[var(--plan-bg)]',
        )}
      >
        <div className="flex items-center gap-1.5 border-b border-[var(--plan-border)] px-2 py-1">
          <ListTodo size={12} className="shrink-0 text-[var(--plan-accent)]" />
          <span className="text-[11px] font-semibold uppercase tracking-wide text-[var(--plan-accent)]">
            Plan
          </span>
          {structured && (
            <span className="text-[11px] text-[var(--text-tertiary)]">
              {steps.filter((s) => s.completed).length}/{steps.length}
            </span>
          )}
          {timestamp && (
            <span className="ml-auto shrink-0 text-[10px] text-[var(--text-tertiary)] opacity-60">
              {timestamp}
            </span>
          )}
        </div>

        <div className="px-1 py-0.5">
          {structured ? (
            steps.map((step, index) => (
              <TodoRow
                key={step.id}
                text={step.text}
                completed={step.completed}
                active={index === activeIndex}
                readOnly
              />
            ))
          ) : (
            <pre className="whitespace-pre-wrap px-2 py-1 font-sans text-[13px] leading-normal text-[var(--text-primary)]">
              {content.trim()}
            </pre>
          )}
        </div>
      </div>
    </div>
  );
}

/**
 * ReasoningBlock —— 推理块（可折叠）
 * - 默认折叠
 * - 折叠状态：单行 "Thinking..." / "Reasoning..." + ChevronRight
 * - 展开状态：完整推理内容 + ChevronDown
 * - 点击切换，height 动画 200ms ease-in-out
 */
import { cn } from '@/lib/utils';
import { ChevronRight, ChevronDown } from 'lucide-react';
import { useState, useRef, useEffect } from 'react';

interface ReasoningBlockProps {
  content: string;
  label?: string;
  defaultExpanded?: boolean;
}

export default function ReasoningBlock({
  content,
  label = 'Thinking...',
  defaultExpanded = false,
}: ReasoningBlockProps) {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);
  const [height, setHeight] = useState<number | undefined>(defaultExpanded ? undefined : 24);
  const contentRef = useRef<HTMLDivElement>(null);
  const measureRef = useRef<HTMLDivElement>(null);

  // Measure content height
  useEffect(() => {
    if (measureRef.current) {
      const measuredHeight = measureRef.current.scrollHeight;
      if (isExpanded) {
        setHeight(measuredHeight);
      } else {
        setHeight(24);
      }
    }
  }, [isExpanded, content]);

  const toggle = () => setIsExpanded((prev) => !prev);

  return (
    <div className="my-0.5">
      {/* 折叠状态指示器（点击区域） */}
      <button
        onClick={toggle}
        className={cn(
          'flex items-center gap-1 w-full',
          'text-xs text-[var(--text-secondary)] italic',
          'hover:text-[var(--text-secondary)]',
          'transition-colors duration-150',
          'cursor-pointer select-none'
        )}
      >
        {isExpanded ? (
          <ChevronDown size={14} className="shrink-0 transition-transform duration-150" />
        ) : (
          <ChevronRight size={14} className="shrink-0 transition-transform duration-150" />
        )}
        <span className="truncate">
          {isExpanded ? 'Reasoning' : label}
        </span>
      </button>

      {/* 可展开的内容区域 */}
      <div
        className="overflow-hidden transition-[height] duration-250 ease-in-out"
        style={{ height }}
      >
        <div
          ref={contentRef}
          className={cn(
            'mt-1 ml-4 max-h-[400px] overflow-y-auto pl-2 custom-scrollbar',
            'rounded-md bg-[var(--bg-surface)]/50 p-2',
            'text-[13px] text-[var(--text-tertiary)] italic leading-relaxed',
            'border-l-2 border-[var(--border-default)]',
            'transition-opacity duration-150',
            isExpanded ? 'opacity-100' : 'opacity-0'
          )}
        >
          {content}
        </div>
      </div>

      {/* 隐藏的测量元素 */}
      <div
        ref={measureRef}
        className="absolute invisible pointer-events-none"
        style={{ width: '100%' }}
      >
        <div className={cn(
          'mt-1 ml-4 pl-2',
          'bg-[var(--bg-surface)]/50 rounded-md p-2',
          'text-xs text-[var(--text-tertiary)]/70 italic leading-relaxed',
          'border-l-2 border-[var(--border-default)]'
        )}>
          {content}
        </div>
      </div>
    </div>
  );
}

/**
 * 系统级时间线条目（plan、状态提示等）
 */
import { cn } from '@/lib/utils';

interface SystemNoticeProps {
  content: string;
  timestamp?: string;
}

export default function SystemNotice({ content, timestamp }: SystemNoticeProps) {
  return (
    <div className="flex justify-center py-0.5">
      <div
        className={cn(
          'max-w-[90%] rounded-md px-2.5 py-0.5 text-center text-[10px] italic',
          'text-[var(--text-tertiary)]',
        )}
      >
        {content}
        {timestamp && (
          <span className="ml-2 opacity-70">{timestamp}</span>
        )}
      </div>
    </div>
  );
}

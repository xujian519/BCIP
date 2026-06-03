import { useEffect, useRef } from 'react';
import { AtSign, Columns2, Rows2 } from 'lucide-react';

interface ContextMenuProps {
  x: number;
  y: number;
  filePath: string;
  onMention: (path: string) => void;
  onOpenSplitRight: (path: string) => void;
  onOpenSplitDown: (path: string) => void;
  onClose: () => void;
}

export default function FileTreeContextMenu({
  x,
  y,
  filePath,
  onMention,
  onOpenSplitRight,
  onOpenSplitDown,
  onClose,
}: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [onClose]);

  const fileName = filePath.split('/').pop() ?? filePath;

  const items = [
    {
      label: '在右侧分屏打开',
      icon: Columns2,
      action: () => onOpenSplitRight(filePath),
    },
    {
      label: '在下方分屏打开',
      icon: Rows2,
      action: () => onOpenSplitDown(filePath),
    },
    {
      label: '在聊天中引用',
      icon: AtSign,
      action: () => onMention(filePath),
    },
  ];

  return (
    <div
      ref={menuRef}
      className="fixed z-[100] rounded-lg border shadow-lg"
      style={{
        left: x,
        top: y,
        backgroundColor: 'var(--bg-elevated)',
        borderColor: 'var(--border-primary)',
        padding: 4,
        minWidth: 196,
      }}
    >
      <div
        className="truncate px-2 py-1.5 text-2xs"
        style={{ color: 'var(--text-tertiary)' }}
      >
        {fileName}
      </div>
      {items.map((item) => {
        const Icon = item.icon;
        return (
          <button
            key={item.label}
            type="button"
            onClick={() => {
              item.action();
              onClose();
            }}
            className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors"
            style={{ color: 'var(--text-primary)' }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent';
            }}
          >
            <Icon size={14} />
            {item.label}
          </button>
        );
      })}
    </div>
  );
}

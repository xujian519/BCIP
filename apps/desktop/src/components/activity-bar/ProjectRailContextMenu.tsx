import { useEffect, useRef } from 'react';
import { Trash2 } from 'lucide-react';

interface ProjectRailContextMenuProps {
  x: number;
  y: number;
  projectName: string;
  onRemove: () => void;
  onClose: () => void;
}

export default function ProjectRailContextMenu({
  x,
  y,
  projectName,
  onRemove,
  onClose,
}: ProjectRailContextMenuProps) {
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
        minWidth: 180,
      }}
    >
      <div
        className="truncate px-2 py-1.5 text-2xs"
        style={{ color: 'var(--text-tertiary)' }}
      >
        {projectName}
      </div>
      <button
        type="button"
        onClick={() => {
          onRemove();
          onClose();
        }}
        className="flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors"
        style={{ color: 'var(--status-error)' }}
        onMouseEnter={(e) => {
          e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.backgroundColor = 'transparent';
        }}
      >
        <Trash2 size={14} />
        从列表移除
      </button>
    </div>
  );
}

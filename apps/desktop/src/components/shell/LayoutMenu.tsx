import { useState, useRef, useEffect } from 'react';
import { Columns2, FileText, SplitSquareHorizontal } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import type { LayoutMode } from '@/types';

interface LayoutOption {
  id: LayoutMode;
  label: string;
  description: string;
  icon: typeof Columns2;
}

const layoutOptions: LayoutOption[] = [
  { id: 'three-column', label: '三栏布局', description: '工作区 + 聊天面板', icon: Columns2 },
  { id: 'document', label: '文档模式', description: '隐藏聊天面板', icon: FileText },
  { id: 'horizontal-split', label: '上下分屏', description: '工作区在上，聊天在下', icon: SplitSquareHorizontal },
];

export default function LayoutMenu() {
  const { state, dispatch } = useAppStore();
  const [open, setOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [open]);

  const handleSelect = (mode: LayoutMode) => {
    dispatch({ type: 'SET_LAYOUT_MODE', payload: mode });
    if (mode === 'document') {
      dispatch({ type: 'SET_AGENT_PANEL_OPEN', payload: false });
    } else if (!state.agentPanelOpen) {
      dispatch({ type: 'SET_AGENT_PANEL_OPEN', payload: true });
    }
    setOpen(false);
  };

  return (
    <div ref={menuRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="flex items-center justify-center transition-colors duration-150"
        style={{
          width: 28,
          height: 28,
          borderRadius: 6,
          color: open ? 'var(--accent-primary)' : 'var(--text-tertiary)',
          backgroundColor: open ? 'var(--bg-sidebar-active)' : 'transparent',
        }}
        onMouseEnter={(e) => {
          if (!open) {
            e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }
        }}
        onMouseLeave={(e) => {
          if (!open) {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }
        }}
        title="布局设置"
        aria-label="布局设置"
      >
        <Columns2 size={16} />
      </button>

      {open && (
        <div
          className="absolute right-0 top-full mt-1 z-50 rounded-lg border shadow-lg"
          style={{
            width: 220,
            backgroundColor: 'var(--bg-elevated)',
            borderColor: 'var(--border-primary)',
            padding: 4,
          }}
        >
          <div className="px-2 py-1.5 text-xs font-medium" style={{ color: 'var(--text-tertiary)' }}>
            布局设置
          </div>
          {layoutOptions.map((opt) => {
            const isActive = state.layoutMode === opt.id;
            const Icon = opt.icon;
            return (
              <button
                key={opt.id}
                type="button"
                onClick={() => handleSelect(opt.id)}
                className="w-full flex items-center gap-2.5 rounded-md transition-colors"
                style={{
                  padding: '8px',
                  backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
                  color: isActive ? 'var(--accent-primary)' : 'var(--text-primary)',
                }}
                onMouseEnter={(e) => {
                  if (!isActive) e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
                }}
                onMouseLeave={(e) => {
                  if (!isActive) e.currentTarget.style.backgroundColor = 'transparent';
                }}
              >
                <Icon size={16} />
                <div className="text-left">
                  <div className="text-xs font-medium">{opt.label}</div>
                  <div className="text-2xs" style={{ color: 'var(--text-tertiary)' }}>
                    {opt.description}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
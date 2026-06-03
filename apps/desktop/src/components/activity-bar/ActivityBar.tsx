import type { FC, MouseEvent } from 'react';
import { useState, useCallback } from 'react';
import { FolderOpen, Plus, Search, Zap, Bot, Settings, ListPlus } from 'lucide-react';
import { open } from '@tauri-apps/plugin-dialog';
import { useAppStore } from '@/hooks/useAppStore';
import { useProjects } from '@/hooks/useProjects';
import { addRecentProjectPath } from '@/lib/recentProjects';
import type { ActivityBarTab } from '@/types';

interface ActivityBarItem {
  id: ActivityBarTab;
  icon: FC<{ size?: number }>;
  label: string;
}

const topItems: ActivityBarItem[] = [
  { id: 'files', icon: FolderOpen, label: '资源管理器' },
  { id: 'new-task', icon: ListPlus, label: '新建任务' },
];

const bottomItems: ActivityBarItem[] = [
  { id: 'search', icon: Search, label: '搜索' },
  { id: 'skills', icon: Zap, label: '技能' },
  { id: 'bots', icon: Bot, label: 'AI 助手' },
];

const BUTTON_SIZE = 40;
const TOOLTIP_OFFSET = 6;
const TOOLTIP_DELAY = 400;
const TRANSITION_CURVE = 'cubic-bezier(0.34, 1.56, 0.64, 1)';

const buttonBaseStyle: React.CSSProperties = {
  width: BUTTON_SIZE,
  height: BUTTON_SIZE,
  borderRadius: 10,
  transitionTimingFunction: TRANSITION_CURVE,
};

function hoverHandlers(isActive: boolean) {
  return {
    onMouseEnter: (e: MouseEvent<HTMLButtonElement>) => {
      if (!isActive) {
        e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
        e.currentTarget.style.color = 'var(--text-secondary)';
      }
    },
    onMouseLeave: (e: MouseEvent<HTMLButtonElement>) => {
      if (!isActive) {
        e.currentTarget.style.backgroundColor = 'transparent';
        e.currentTarget.style.color = 'var(--text-tertiary)';
      }
    },
  };
}

// ========================================
// 自定义 Tooltip（替代原生 title）
// ========================================

function ActivityTooltip({ label, visible }: { label: string; visible: boolean }) {
  return (
    <div
      style={{
        position: 'absolute',
        left: BUTTON_SIZE + TOOLTIP_OFFSET,
        top: '50%',
        transform: visible ? 'translateY(-50%) scale(1)' : 'translateY(-50%) scale(0.92)',
        opacity: visible ? 1 : 0,
        pointerEvents: 'none',
        zIndex: 50,
        transition: `opacity 120ms ease, transform 120ms ${TRANSITION_CURVE}`,
      }}
    >
      <div
        style={{
          padding: '4px 10px',
          borderRadius: 6,
          fontSize: 12,
          fontWeight: 500,
          whiteSpace: 'nowrap',
          backgroundColor: 'var(--text-primary)',
          color: 'var(--text-inverse)',
          boxShadow: 'var(--shadow-floating)',
        }}
      >
        {label}
      </div>
    </div>
  );
}

// ========================================
// 按钮组件
// ========================================

function ActivityBarButton({
  item,
  isActive,
  onClick,
}: {
  item: ActivityBarItem;
  isActive: boolean;
  onClick: () => void;
}) {
  const Icon = item.icon;
  const [tooltipVisible, setTooltipVisible] = useState(false);
  const [timeoutId, setTimeoutId] = useState<ReturnType<typeof setTimeout> | null>(null);

  const handleMouseEnter = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    hoverHandlers(isActive).onMouseEnter(e);
    const id = setTimeout(() => setTooltipVisible(true), TOOLTIP_DELAY);
    setTimeoutId(id);
  }, [isActive]);

  const handleMouseLeave = useCallback((e: MouseEvent<HTMLButtonElement>) => {
    hoverHandlers(isActive).onMouseLeave(e);
    if (timeoutId) clearTimeout(timeoutId);
    setTooltipVisible(false);
  }, [isActive, timeoutId]);

  return (
    <button
      onClick={onClick}
      className="relative flex items-center justify-center transition-all duration-200"
      style={{
        ...buttonBaseStyle,
        color: isActive ? 'var(--accent-primary)' : 'var(--text-tertiary)',
        backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
      }}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      aria-label={item.label}
      type="button"
    >
      <Icon size={20} />
      {isActive && (
        <div
          className="absolute left-0 top-1/2 -translate-y-1/2 rounded-r-full transition-all duration-200"
          style={{
            width: 2.5,
            height: 20,
            backgroundColor: 'var(--accent-primary)',
            transitionTimingFunction: TRANSITION_CURVE,
          }}
        />
      )}
      <ActivityTooltip label={item.label} visible={tooltipVisible} />
    </button>
  );
}

function IconButton({
  icon: Icon,
  label,
  onClick,
}: {
  icon: FC<{ size?: number }>;
  label: string;
  onClick: () => void;
}) {
  const [tooltipVisible, setTooltipVisible] = useState(false);
  const [timeoutId, setTimeoutId] = useState<ReturnType<typeof setTimeout> | null>(null);

  const handleMouseEnter = useCallback(() => {
    const id = setTimeout(() => setTooltipVisible(true), TOOLTIP_DELAY);
    setTimeoutId(id);
  }, []);

  const handleMouseLeave = useCallback(() => {
    if (timeoutId) clearTimeout(timeoutId);
    setTooltipVisible(false);
  }, [timeoutId]);

  return (
    <button
      type="button"
      onClick={onClick}
      className="relative flex items-center justify-center transition-all duration-200"
      style={{ ...buttonBaseStyle, color: 'var(--text-tertiary)' }}
      onMouseEnter={(e) => {
        hoverHandlers(false).onMouseEnter(e);
        handleMouseEnter();
      }}
      onMouseLeave={(e) => {
        hoverHandlers(false).onMouseLeave(e);
        handleMouseLeave();
      }}
      aria-label={label}
    >
      <Icon size={20} />
      <ActivityTooltip label={label} visible={tooltipVisible} />
    </button>
  );
}

export default function ActivityBar() {
  const { state, dispatch } = useAppStore();
  const { createProject } = useProjects();

  const handleTabClick = (id: ActivityBarTab) => {
    dispatch({
      type: 'SET_ACTIVITY_BAR_TAB',
      payload: state.activityBarTab === id ? null : id,
    });
  };

  const handleAddProject = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '添加项目目录',
      });
      if (selected && typeof selected === 'string') {
        addRecentProjectPath(selected);
        await createProject(selected).catch(() => null);
        dispatch({ type: 'SWITCH_PROJECT', payload: selected });
        dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: 'files' });
        dispatch({ type: 'SET_PROJECT_RAIL_OPEN', payload: true });
      }
    } catch {
      // user cancelled
    }
  };

  return (
    <div
      className="flex shrink-0 flex-col items-center justify-between glass"
      style={{
        width: 48,
        borderRight: '1px solid var(--border-primary)',
        paddingTop: 8,
        paddingBottom: 8,
      }}
    >
      <div className="flex flex-col items-center gap-1">
        <IconButton icon={Plus} label="添加项目" onClick={() => void handleAddProject()} />
        {topItems.map((item) => (
          <ActivityBarButton
            key={item.id}
            item={item}
            isActive={state.activityBarTab === item.id}
            onClick={() => handleTabClick(item.id)}
          />
        ))}
      </div>

      <div className="flex flex-col items-center gap-1">
        {bottomItems.map((item) => (
          <ActivityBarButton
            key={item.id}
            item={item}
            isActive={state.activityBarTab === item.id}
            onClick={() => handleTabClick(item.id)}
          />
        ))}
        <IconButton
          icon={Settings}
          label="设置"
          onClick={() => dispatch({ type: 'OPEN_SETTINGS', payload: 'general' })}
        />
      </div>
    </div>
  );
}

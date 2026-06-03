import type { FC } from 'react';
import { FolderOpen, Plus, Search, Zap, Bot, Settings } from 'lucide-react';
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
];

const bottomItems: ActivityBarItem[] = [
  { id: 'search', icon: Search, label: '搜索' },
  { id: 'skills', icon: Zap, label: '技能' },
  { id: 'bots', icon: Bot, label: 'AI 助手' },
];

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
  return (
    <button
      onClick={onClick}
      className="relative flex items-center justify-center transition-all duration-200"
      style={{
        width: 40,
        height: 40,
        borderRadius: 10,
        color: isActive ? 'var(--accent-primary)' : 'var(--text-tertiary)',
        backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
        transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
      }}
      onMouseEnter={(e) => {
        if (!isActive) {
          e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
          e.currentTarget.style.color = 'var(--text-secondary)';
        }
      }}
      onMouseLeave={(e) => {
        if (!isActive) {
          e.currentTarget.style.backgroundColor = 'transparent';
          e.currentTarget.style.color = 'var(--text-tertiary)';
        }
      }}
      title={item.label}
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
            transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
          }}
        />
      )}
    </button>
  );
}

export default function ActivityBar() {
  const { state, dispatch } = useAppStore();
  const { createProject } = useProjects();

  const handleTabClick = (id: ActivityBarTab) => {
    if (state.activityBarTab === id) {
      dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: null });
    } else {
      dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: id });
    }
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
        <button
          type="button"
          onClick={() => void handleAddProject()}
          className="flex items-center justify-center transition-all duration-200"
          style={{
            width: 40,
            height: 40,
            borderRadius: 10,
            color: 'var(--text-tertiary)',
            transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }}
          title="添加项目"
          aria-label="添加项目"
        >
          <Plus size={20} />
        </button>
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
        <button
          onClick={() => dispatch({ type: 'OPEN_SETTINGS', payload: 'general' })}
          className="flex items-center justify-center transition-all duration-200"
          style={{
            width: 40,
            height: 40,
            borderRadius: 10,
            color: 'var(--text-tertiary)',
            transitionTimingFunction: 'cubic-bezier(0.34, 1.56, 0.64, 1)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }}
          title="设置"
          aria-label="设置"
          type="button"
        >
          <Settings size={20} />
        </button>
      </div>
    </div>
  );
}

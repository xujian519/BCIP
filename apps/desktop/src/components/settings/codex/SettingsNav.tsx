import type { CSSProperties } from 'react';
import type { LucideIcon } from 'lucide-react';
import { isTauri } from '@/api/tauri';
import { useAppStore } from '@/hooks/useAppStore';
import { cn } from '@/lib/utils';
import type { SettingsPage } from '@/types';
import {
  ChevronLeft,
  Cpu,
  Info,
  Keyboard,
  Palette,
  Plug,
  Puzzle,
  Settings,
  Shield,
  Zap,
} from 'lucide-react';
import { settingsTheme } from './settingsTheme';

const navItems: { id: SettingsPage; label: string; icon: LucideIcon }[] = [
  { id: 'general', label: '通用', icon: Settings },
  { id: 'model', label: '模型与推理', icon: Cpu },
  { id: 'approval', label: '审批与沙箱', icon: Shield },
  { id: 'mcp', label: 'MCP 服务器', icon: Plug },
  { id: 'skills', label: '技能', icon: Zap },
  { id: 'plugins', label: '插件', icon: Puzzle },
  { id: 'appearance', label: '编辑器与外观', icon: Palette },
  { id: 'shortcuts', label: '快捷键', icon: Keyboard },
  { id: 'about', label: '关于与诊断', icon: Info },
];

export default function SettingsNav() {
  const { state, dispatch } = useAppStore();
  const currentPage = state.settingsPage;

  const noDrag = { WebkitAppRegion: 'no-drag' } as CSSProperties;

  return (
    <nav
      className={cn(settingsTheme.navShell, isTauri() && 'pt-[52px]')}
      style={noDrag}
    >
      <button
        type="button"
        onClick={() => dispatch({ type: 'CLOSE_SETTINGS' })}
        style={noDrag}
        className={settingsTheme.navBack}
      >
        <ChevronLeft size={14} />
        <span>返回</span>
      </button>

      <div className="flex flex-col gap-0.5">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = currentPage === item.id;
          return (
            <button
              type="button"
              key={item.id}
              onClick={() =>
                dispatch({ type: 'SET_SETTINGS_PAGE', payload: item.id })
              }
              style={noDrag}
              className={cn(
                settingsTheme.navItem,
                isActive ? settingsTheme.navItemActive : settingsTheme.navItemIdle,
              )}
            >
              <Icon size={16} className="shrink-0" />
              <span className="truncate">{item.label}</span>
            </button>
          );
        })}
      </div>
    </nav>
  );
}

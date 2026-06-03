/**
 * TitleBar —— macOS 风格标题栏
 *
 * 精确设计参数（像素级对标 Codex）：
 * - 高度：40px (h-titlebar)
 * - 背景：透明 + backdrop-blur-md (glass)
 * - 底部边框：border-b border-border-subtle
 * - 左侧交通灯：三个圆点，每个 w-3 h-3 (12px)
 *   - 颜色：#FF5F57 (红/关闭)、#FFBD2E (黄/最小化)、#28C840 (绿/最大化)
 *   - 间距：gap-2 (8px)，距左 16px
 *   - 悬停效果：显示 ×/-/+ 图标（opacity 0→1 过渡 150ms）
 * - 中间标题："云熙智能体"，font-sm (13px)，font-medium，text-primary，绝对居中
 * - 右侧阶段指示器：4个 pill 按钮
 * - 整个标题栏是拖拽区（app-region-drag），但按钮不是（app-region-no-drag）
 */
import { useState } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import { isTauri } from '@/api/tauri';
import { isWindowsPlatform } from '@/lib/platform';
import { cn } from '@/lib/utils';
import type { StageInfo } from '@/types';
import LayoutMenu from './LayoutMenu';

/** macOS 透明标题栏下为系统交通灯预留的左侧内边距 */
const TAURI_TRAFFIC_LIGHTS_INSET = 'pl-[72px]';

// ========================================
// 交通灯按钮组
// ========================================

/** 单个交通灯按钮 */
function TrafficLight({
  color,
  hoverIcon,
  hoverIconColor,
  label,
}: {
  color: string;
  hoverIcon: string;
  hoverIconColor: string;
  label: string;
}) {
  const [isHovered, setIsHovered] = useState(false);

  return (
    <button
      type="button"
      className={cn(
        'relative w-3 h-3 rounded-full flex items-center justify-center',
        'transition-all duration-fast',
        color
      )}
      style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      title={label}
      aria-label={label}
    >
      {/* 悬停时显示的图标 */}
      <span
        className={cn(
          'absolute inset-0 flex items-center justify-center',
          'text-[10px] font-bold leading-none',
          'transition-opacity duration-normal',
          isHovered ? 'opacity-100' : 'opacity-0'
        )}
        style={{ color: hoverIconColor }}
      >
        {hoverIcon}
      </span>
    </button>
  );
}

/** 交通灯按钮组 */
function TrafficLights() {
  const lights = [
    {
      color: 'bg-[#FF5F57]',
      hoverIcon: '\u00D7', // ×
      hoverIconColor: '#4A0000',
      label: '关闭窗口',
    },
    {
      color: 'bg-[#FFBD2E]',
      hoverIcon: '\u2212', // −
      hoverIconColor: '#995700',
      label: '最小化到 Dock',
    },
    {
      color: 'bg-[#28C840]',
      hoverIcon: '+',
      hoverIconColor: '#006500',
      label: '最大化/全屏',
    },
  ];

  return (
    <div className="flex items-center gap-2 pl-4">
      {lights.map((light, i) => (
        <TrafficLight key={i} {...light} />
      ))}
    </div>
  );
}

// ========================================
// 阶段 Pill
// ========================================

/** 阶段 Pill 按钮 */
function StagePill({
  stage,
  onClick,
}: {
  stage: StageInfo;
  onClick: () => void;
}) {
  const { status, label } = stage;

  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        // 基础样式：h-7 (28px), px-3, rounded-full, text-2xs (11px), font-medium
        'inline-flex items-center justify-center h-7 px-3 rounded-full',
        'text-2xs font-medium tracking-wide whitespace-nowrap',
        'transition-all duration-normal cursor-pointer',
        // 未激活：描边 + 主色文字，保证在玻璃标题栏上可读
        status === 'pending' && [
          'bg-[var(--bg-elevated)]/60',
          'border border-[var(--border-default)]',
          'text-[var(--text-primary)]',
          'hover:bg-[var(--bg-hover)]',
        ],
        // 激活：品牌色底 + 反色字
        status === 'active' && [
          'bg-brand-500',
          'text-[var(--text-inverse)]',
          'border border-transparent',
          'shadow-sm',
        ],
        // 已完成：浅底 + 品牌色字
        status === 'completed' && [
          'bg-brand-500/15',
          'text-brand-500',
          'border border-brand-500/40',
        ]
      )}
      style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
    >
      {status === 'completed' && (
        <svg
          className="w-3 h-3 mr-1 shrink-0"
          viewBox="0 0 12 12"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <polyline points="2,6 5,9 10,3" />
        </svg>
      )}
      {label}
    </button>
  );
}

/** 阶段指示器 */
function StageIndicator() {
  const { state, dispatch } = useAppStore();

  const handleStageClick = (stage: StageInfo) => {
    if (stage.status === 'active') {
      return;
    }
    dispatch({ type: 'UPDATE_STAGE', payload: { id: stage.id, status: 'active' } });
  };

  return (
    <div
      className="flex items-center gap-2 pr-4"
      style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
    >
      {state.stages.map((stage) => (
        <StagePill
          key={stage.id}
          stage={stage}
          onClick={() => handleStageClick(stage)}
        />
      ))}
    </div>
  );
}

// ========================================
// 主标题栏
// ========================================

export default function TitleBar() {
  const useSystemChrome = isTauri() || isWindowsPlatform();
  const showCustomTrafficLights = !useSystemChrome;

  return (
    <header
      className={cn(
        'h-titlebar flex items-center justify-between shrink-0 relative',
        'glass-strong bg-[var(--bg-sidebar)]/80',
        'border-b border-[var(--border-default)]',
        'select-none',
        'z-10'
      )}
      style={{ WebkitAppRegion: 'drag' } as React.CSSProperties}
    >
      {/* Tauri 使用系统交通灯，避免与自定义圆点重复绘制 */}
      {showCustomTrafficLights ? (
        <TrafficLights />
      ) : (
        <div
          className={cn(
            'shrink-0',
            isWindowsPlatform() ? 'pl-4' : TAURI_TRAFFIC_LIGHTS_INSET,
          )}
          aria-hidden
        />
      )}

      {/* 中间：应用标题 —— 绝对居中 */}
      <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
        <span className="text-sm font-semibold text-[var(--text-primary)] tracking-tight">
          云熙智能体
        </span>
      </div>

      {/* 右侧：阶段指示器 */}
      <div
        className="flex items-center gap-2 pr-4"
        style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
      >
        <LayoutMenu />
        <StageIndicator />
      </div>
    </header>
  );
}

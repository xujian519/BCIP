import type { FC } from 'react';
import { useState } from 'react';
import { Sun, Moon, Zap } from 'lucide-react';
import { useTheme } from '@/hooks/useTheme';

interface TrafficLightProps {
  color: string;
  hoverColor: string;
  icon: React.ReactNode;
  onClick?: () => void;
}

const TrafficLight: FC<TrafficLightProps> = ({ color, hoverColor, icon, onClick }) => {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      className="relative flex items-center justify-center transition-colors duration-150"
      style={{
        width: 12,
        height: 12,
        borderRadius: '50%',
        backgroundColor: hovered ? hoverColor : color,
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      onClick={onClick}
      type="button"
    >
      {hovered && (
        <span className="flex items-center justify-center" style={{ color: 'rgba(0,0,0,0.5)', fontSize: 8, lineHeight: 1 }}>
          {icon}
        </span>
      )}
    </button>
  );
};

const CloseIcon = () => (
  <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
    <path d="M2 2L6 6M6 2L2 6" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
  </svg>
);

const MinimizeIcon = () => (
  <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
    <path d="M1.5 4H6.5" stroke="currentColor" strokeWidth="1" strokeLinecap="round" />
  </svg>
);

const MaximizeIcon = () => (
  <svg width="8" height="8" viewBox="0 0 8 8" fill="none">
    <path d="M2 2H6V6H2V2Z" stroke="currentColor" strokeWidth="0.8" />
  </svg>
);

interface TitleBarProps {
  title?: string;
  showTrafficLights?: boolean;
}

const TitleBar: FC<TitleBarProps> = ({ title = 'BCIP Agent', showTrafficLights = true }) => {
  const { resolvedTheme, setTheme } = useTheme();
  const isDark = resolvedTheme === 'dark';

  const toggleTheme = () => {
    setTheme(isDark ? 'light' : 'dark');
  };

  return (
    <header
      className="flex items-center justify-between select-none shrink-0"
      style={{
        height: 38,
        backgroundColor: 'var(--bg-surface)',
        borderBottom: '1px solid var(--border-primary)',
        userSelect: 'none',
        WebkitAppRegion: 'drag',
        backdropFilter: 'blur(20px) saturate(1.15)',
        WebkitBackdropFilter: 'blur(20px) saturate(1.15)',
      } as React.CSSProperties}
    >
      {/* Traffic lights + app branding */}
      <div className="flex items-center" style={{ gap: 0, paddingLeft: 12 }}>
        {showTrafficLights && (
          <div
            className="flex items-center"
            style={{
              gap: 8,
              width: 52,
              WebkitAppRegion: 'no-drag',
            } as React.CSSProperties}
          >
            <TrafficLight
              color="#FF5F57"
              hoverColor="#FF453A"
              icon={<CloseIcon />}
            />
            <TrafficLight
              color="#FFBD2E"
              hoverColor="#FFD60A"
              icon={<MinimizeIcon />}
            />
            <TrafficLight
              color="#28C840"
              hoverColor="#30D158"
              icon={<MaximizeIcon />}
            />
          </div>
        )}

        {/* App icon + title */}
        <div className="flex items-center" style={{ gap: 8, marginLeft: showTrafficLights ? 8 : 0 }}>
          <img
            src="./app-icon.png"
            alt="云熙智能体"
            style={{ width: 20, height: 20, borderRadius: 5, objectFit: 'cover' }}
          />
          <div className="flex flex-col">
            <span
              style={{
                fontSize: 13,
                fontWeight: 600,
                color: 'var(--text-primary)',
                letterSpacing: '-0.01em',
                lineHeight: 1.3,
              }}
            >
              {title}
            </span>
            <span
              style={{
                fontSize: 10,
                color: 'var(--text-tertiary)',
                letterSpacing: '0.02em',
                lineHeight: 1.2,
              }}
            >
              智能专利助手
            </span>
          </div>
        </div>
      </div>

      {/* Center - empty for drag area */}
      <div
        className="flex-1"
        style={{
          WebkitAppRegion: 'drag',
        } as React.CSSProperties}
      />

      {/* Right side - theme toggle + version */}
      <div
        className="flex items-center"
        style={{
          gap: 8,
          paddingRight: 12,
          WebkitAppRegion: 'no-drag',
        } as React.CSSProperties}
      >
        <div
          className="flex items-center gap-1 px-2 py-0.5 rounded-full"
          style={{
            backgroundColor: 'var(--bg-sidebar-active)',
            border: '1px solid var(--border-secondary)',
          }}
        >
          <Zap size={10} style={{ color: 'var(--accent-primary)' }} />
          <span style={{ fontSize: 10, color: 'var(--text-tertiary)', fontWeight: 500 }}>v0.1.0</span>
        </div>
        <button
          onClick={toggleTheme}
          className="p-1.5 rounded-md transition-colors"
          style={{ color: 'var(--text-tertiary)' }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }}
          title={isDark ? '切换到亮色模式' : '切换到暗色模式'}
        >
          {isDark ? <Sun size={14} /> : <Moon size={14} />}
        </button>
      </div>
    </header>
  );
};

export default TitleBar;
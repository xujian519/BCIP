import type { FC } from 'react';
import { useState, useEffect } from 'react';
import { Sun, Moon, Wifi, WifiOff, GitBranch, Cpu } from 'lucide-react';

const StatusBar: FC = () => {
  const [isDark, setIsDark] = useState(() => {
    if (typeof window !== 'undefined') {
      const saved = localStorage.getItem('theme');
      if (saved) return saved === 'dark';
      return window.matchMedia('(prefers-color-scheme: dark)').matches;
    }
    return false;
  });
  const [isOnline] = useState(true);
  const [cost] = useState({ used: 2.35, total: 10.0 });

  useEffect(() => {
    document.documentElement.classList.toggle('dark', isDark);
    localStorage.setItem('theme', isDark ? 'dark' : 'light');
  }, [isDark]);

  const toggleTheme = () => setIsDark((d) => !d);

  const costPercent = (cost.used / cost.total) * 100;
  const costColor =
    costPercent > 80 ? 'var(--status-error)' : costPercent > 50 ? 'var(--status-warning)' : 'var(--status-success)';

  return (
    <footer
      className="flex items-center justify-between select-none shrink-0 glass"
      style={{
        height: 32,
        borderTop: '1px solid var(--border-primary)',
        padding: '0 12px',
        fontSize: 11,
        fontWeight: 500,
        letterSpacing: '0.01em',
        color: 'var(--text-tertiary)',
      }}
    >
      {/* Left section - connection + cost */}
      <div className="flex items-center" style={{ gap: 16 }}>
        {/* Connection status */}
        <div className="flex items-center" style={{ gap: 4 }}>
          {isOnline ? (
            <Wifi size={12} style={{ color: 'var(--status-success)' }} />
          ) : (
            <WifiOff size={12} style={{ color: 'var(--status-error)' }} />
          )}
          <span>{isOnline ? '已连接' : '离线'}</span>
        </div>

        {/* Cost indicator */}
        <div className="flex items-center" style={{ gap: 6 }}>
          <span>费用</span>
          <div
            style={{
              width: 60,
              height: 4,
              borderRadius: 2,
              backgroundColor: 'var(--border-primary)',
              overflow: 'hidden',
            }}
          >
            <div
              style={{
                width: `${costPercent}%`,
                height: '100%',
                backgroundColor: costColor,
                borderRadius: 2,
                transition: 'width 0.3s ease, background-color 0.3s ease',
              }}
            />
          </div>
          <span>
            ¥{cost.used.toFixed(2)} / ¥{cost.total.toFixed(2)}
          </span>
        </div>
      </div>

      {/* Center - model info */}
      <div className="flex items-center" style={{ gap: 16 }}>
        <div className="flex items-center" style={{ gap: 4 }}>
          <GitBranch size={10} />
          <span>main</span>
        </div>
        <div className="flex items-center" style={{ gap: 4 }}>
          <Cpu size={10} />
          <span>DeepSeek-V3</span>
        </div>
      </div>

      {/* Right section - theme toggle */}
      <div className="flex items-center" style={{ gap: 8 }}>
        <button
          onClick={toggleTheme}
          className="flex items-center justify-center"
          style={{
            width: 20,
            height: 20,
            borderRadius: 4,
            color: 'var(--text-tertiary)',
            transition: 'color 0.15s ease, background-color 0.15s ease',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.color = 'var(--text-secondary)';
            e.currentTarget.style.backgroundColor = 'var(--bg-sidebar-active)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.color = 'var(--text-tertiary)';
            e.currentTarget.style.backgroundColor = 'transparent';
          }}
          title={isDark ? '切换到浅色模式' : '切换到深色模式'}
          type="button"
        >
          {isDark ? <Sun size={12} /> : <Moon size={12} />}
        </button>
      </div>
    </footer>
  );
};

export default StatusBar;

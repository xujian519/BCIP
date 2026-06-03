import { useEffect, useState } from 'react';
import type { LucideIcon } from 'lucide-react';
import { Monitor, Moon, Sun } from 'lucide-react';
import { Slider } from '@/components/ui/slider';
import { useAppStore } from '@/hooks/useAppStore';
import { useCodexConfig } from '@/hooks/useCodexConfig';
import {
  applyAccentColor,
  applyFontSizes,
  parseThemeMode,
  readDesktopField,
} from '@/lib/desktopAppearance';
import { isDesktopRpcReady } from '@/lib/configAccess';
import type { ThemeMode } from '@/types';
import {
  SettingRow,
  SettingsCard,
  SettingsRpcBanner,
} from '../SettingPrimitives';

type ThemeOption = ThemeMode;

export default function AppearanceSettings() {
  const { state, dispatch } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const { loading, error, saving, writeValue, get } = useCodexConfig(
    rpcReady,
    state.workspaceCwd,
  );

  const [theme, setTheme] = useState<ThemeOption>(state.theme);
  const [uiFontSize, setUiFontSize] = useState([14]);
  const [codeFontSize, setCodeFontSize] = useState([13]);
  const [accentColor, setAccentColor] = useState('#4A7C6F');

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- config-driven init
    setTheme(parseThemeMode(readDesktopField(get, 'theme')) ?? state.theme);
    const ui = readDesktopField(get, 'ui_font_size');
    const code = readDesktopField(get, 'code_font_size');
    const accent = readDesktopField(get, 'accent_color');
    if (typeof ui === 'number') {
      setUiFontSize([ui]);
    }
    if (typeof code === 'number') {
      setCodeFontSize([code]);
    }
    if (typeof accent === 'string') {
      setAccentColor(accent);
    }
  }, [get, state.theme]);

  const persistTheme = async (next: ThemeOption) => {
    setTheme(next);
    dispatch({ type: 'SET_THEME', payload: next });
    if (rpcReady) {
      await writeValue('desktop.theme', next, 'upsert');
    }
  };

  const persistUiFont = async (value: number[]) => {
    setUiFontSize(value);
    applyFontSizes(value[0], codeFontSize[0]);
    if (rpcReady) {
      await writeValue('desktop.ui_font_size', value[0], 'upsert');
    }
  };

  const persistCodeFont = async (value: number[]) => {
    setCodeFontSize(value);
    applyFontSizes(uiFontSize[0], value[0]);
    if (rpcReady) {
      await writeValue('desktop.code_font_size', value[0], 'upsert');
    }
  };

  const persistAccent = async (hex: string) => {
    setAccentColor(hex);
    applyAccentColor(hex);
    if (rpcReady) {
      await writeValue('desktop.accent_color', hex, 'upsert');
    }
  };

  const themeOptions: { key: ThemeOption; label: string; icon: LucideIcon }[] = [
    { key: 'light', label: '浅色', icon: Sun },
    { key: 'dark', label: '深色', icon: Moon },
    { key: 'system', label: '跟随系统', icon: Monitor },
  ];

  return (
    <div>
      <h1 className="text-2xl font-semibold text-[var(--text-primary)] mb-2">编辑器与外观</h1>
      <SettingsRpcBanner
        rpcReady={rpcReady}
        loading={loading}
        error={error}
        saving={saving}
      />
      <p className="text-xs text-[var(--text-secondary)] mb-4">
        外观写入 <span className="font-mono">config.toml</span> 的{' '}
        <span className="font-mono">[desktop]</span> 段（与 TUI 的{' '}
        <span className="font-mono">[tui]</span> 分离）。语法主题可继续在 TUI 用{' '}
        <span className="font-mono">tui.theme</span> 配置。
      </p>

      <SettingsCard title="主题">
        <div className="flex gap-3">
          {themeOptions.map((opt) => {
            const Icon = opt.icon;
            const isActive = theme === opt.key;
            return (
              <button
                key={opt.key}
                type="button"
                disabled={saving}
                onClick={() => void persistTheme(opt.key)}
                className={`w-[100px] h-[80px] flex flex-col items-center justify-center gap-2 rounded-lg border transition-all duration-150 ${
                  isActive
                    ? 'border-[var(--accent-primary)] shadow-[0_0_0_2px_rgba(74,124,111,0.3)]'
                    : 'border-[var(--border-default)] hover:border-[var(--border-hover)]'
                } bg-[var(--bg-base)]`}
              >
                <Icon
                  size={24}
                  className={isActive ? 'text-[var(--accent-primary)]' : 'text-[var(--text-secondary)]'}
                />
                <span
                  className={`text-xs font-medium ${isActive ? 'text-[var(--text-primary)]' : 'text-[var(--text-secondary)]'}`}
                >
                  {opt.label}
                </span>
              </button>
            );
          })}
        </div>
      </SettingsCard>

      <SettingsCard title="字体">
        <SettingRow
          label="UI 字体大小"
          description="desktop.ui_font_size"
        >
          <div className="w-[160px]">
            <Slider
              value={uiFontSize}
              onValueChange={(v) => void persistUiFont(v)}
              min={12}
              max={24}
              step={1}
              disabled={saving}
            />
          </div>
        </SettingRow>
        <SettingRow
          label="代码字体大小"
          description="desktop.code_font_size"
        >
          <div className="w-[160px]">
            <Slider
              value={codeFontSize}
              onValueChange={(v) => void persistCodeFont(v)}
              min={12}
              max={24}
              step={1}
              disabled={saving}
            />
          </div>
        </SettingRow>
      </SettingsCard>

      <SettingsCard title="强调色">
        <div className="flex items-center gap-3 flex-wrap">
          {['#4A7C6F', '#0066FF', '#B85C50', '#B8923A', '#8B5CF6'].map(
            (color) => (
              <button
                key={color}
                type="button"
                disabled={saving}
                onClick={() => void persistAccent(color)}
                className={`w-8 h-8 rounded-full transition-all duration-150 ${
                  accentColor === color
                    ? 'ring-2 ring-white ring-offset-2 ring-offset-[var(--bg-elevated)] scale-110'
                    : 'hover:scale-105'
                }`}
                style={{ backgroundColor: color }}
              />
            ),
          )}
          <input
            type="color"
            value={accentColor}
            disabled={saving}
            onChange={(e) => void persistAccent(e.target.value)}
            className="w-8 h-8 rounded-full overflow-hidden cursor-pointer border-0 p-0"
          />
        </div>
      </SettingsCard>
    </div>
  );
}

import { useState, useRef, useEffect, useCallback } from 'react';
import { Search } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import { useCodexConfig } from '@/hooks/useCodexConfig';
import { isDesktopRpcReady } from '@/lib/configAccess';
import {
  DEFAULT_DESKTOP_SHORTCUTS,
  shortcutsFromConfig,
  shortcutsToConfigValue,
  type DesktopShortcut,
} from '@/lib/desktopShortcuts';
import { SettingsRpcBanner } from '../SettingPrimitives';

export default function ShortcutsSettings() {
  const { state } = useAppStore();
  const rpcReady = isDesktopRpcReady(state.connectionStatus);
  const { loading, error, saving, writeValue, get } = useCodexConfig(
    rpcReady,
    state.workspaceCwd,
  );

  const [search, setSearch] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [shortcuts, setShortcuts] = useState<DesktopShortcut[]>(
    DEFAULT_DESKTOP_SHORTCUTS,
  );
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- config-driven init
    setShortcuts(shortcutsFromConfig(get));
  }, [get]);

  const persistShortcuts = useCallback(
    async (next: DesktopShortcut[]) => {
      setShortcuts(next);
      if (rpcReady) {
        await writeValue(
          'desktop.shortcuts',
          shortcutsToConfigValue(next),
          'upsert',
        );
      }
    },
    [rpcReady, writeValue],
  );

  const filteredShortcuts = shortcuts.filter(
    (s) =>
      s.action.toLowerCase().includes(search.toLowerCase()) ||
      s.category.toLowerCase().includes(search.toLowerCase()),
  );

  const grouped = filteredShortcuts.reduce<Record<string, DesktopShortcut[]>>(
    (acc, s) => {
      if (!acc[s.category]) {
        acc[s.category] = [];
      }
      acc[s.category].push(s);
      return acc;
    },
    {},
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!editingId) {
        return;
      }
      e.preventDefault();

      const keys: string[] = [];
      if (e.metaKey) {
        keys.push('⌘');
      }
      if (e.ctrlKey) {
        keys.push('Ctrl');
      }
      if (e.altKey) {
        keys.push('⌥');
      }
      if (e.shiftKey) {
        keys.push('⇧');
      }

      const key = e.key;
      if (key && !['Meta', 'Control', 'Alt', 'Shift'].includes(key)) {
        keys.push(key.length === 1 ? key.toUpperCase() : key);
      }

      if (keys.length > 0) {
        const newShortcut = keys.join('');
        const next = shortcuts.map((s) =>
          s.id === editingId ? { ...s, macKey: newShortcut } : s,
        );
        void persistShortcuts(next);
        setEditingId(null);
      }
    },
    [editingId, shortcuts, persistShortcuts],
  );

  useEffect(() => {
    if (editingId) {
      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
    }
  }, [editingId, handleKeyDown]);

  const handleReset = () => {
    void persistShortcuts(DEFAULT_DESKTOP_SHORTCUTS);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-2">
        <h1 className="text-2xl font-semibold text-[var(--text-primary)]">快捷键</h1>
        <button
          type="button"
          onClick={handleReset}
          disabled={!rpcReady || saving}
          className="text-xs text-[var(--text-secondary)] hover:text-[var(--text-primary)] disabled:opacity-50"
        >
          恢复默认
        </button>
      </div>

      <SettingsRpcBanner
        rpcReady={rpcReady}
        loading={loading}
        error={error}
        saving={saving}
      />

      <p className="text-xs text-[var(--text-secondary)] mb-4">
        桌面端快捷键保存在 <span className="font-mono">desktop.shortcuts</span>。
        TUI 完整键位映射见 config <span className="font-mono">tui.keymap</span>。
      </p>

      <div className="relative mb-4">
        <Search
          size={16}
          className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--text-secondary)]"
        />
        <input
          type="text"
          ref={inputRef}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="搜索快捷键..."
          className="w-full h-9 pl-9 pr-3 bg-[var(--bg-base)] rounded-lg border border-[var(--border-default)] text-sm text-[var(--text-primary)] placeholder:text-[var(--text-secondary)] focus:outline-none focus:border-[var(--border-focus)] transition-colors duration-150"
        />
      </div>

      <div className="bg-[var(--bg-elevated)] rounded-xl border border-[var(--border-default)] overflow-hidden">
        {Object.entries(grouped).map(([category, items]) => (
          <div key={category}>
            <div className="px-3 py-2 bg-[var(--bg-hover)]">
              <span className="text-[11px] font-semibold text-[var(--text-secondary)] uppercase tracking-wider">
                {category}
              </span>
            </div>
            {items.map((shortcut, index) => (
              <div
                key={shortcut.id}
                className={`flex items-center justify-between px-4 h-10 ${
                  index < items.length - 1
                    ? 'border-b border-[var(--border-default)]'
                    : ''
                } ${index % 2 === 1 ? 'bg-[var(--bg-hover)]' : ''}`}
              >
                <span className="text-[13px] text-[var(--text-primary)]">
                  {shortcut.action}
                </span>
                <button
                  type="button"
                  disabled={saving}
                  onClick={() => setEditingId(shortcut.id)}
                  className={`px-2 py-0.5 rounded font-mono text-[11px] transition-all duration-150 ${
                    editingId === shortcut.id
                      ? 'bg-[var(--accent-primary-muted)] text-[var(--accent-primary)] animate-pulse'
                      : 'bg-[var(--bg-base)] text-[var(--text-secondary)] border border-[var(--border-default)] hover:border-[var(--border-hover)]'
                  }`}
                >
                  {editingId === shortcut.id
                    ? '按下新快捷键...'
                    : shortcut.macKey}
                </button>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

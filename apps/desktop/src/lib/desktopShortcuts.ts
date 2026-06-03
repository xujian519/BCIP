import type { JsonValue } from '@/generated/app-server/serde_json/JsonValue';

export interface DesktopShortcut {
  id: string;
  action: string;
  category: string;
  macKey: string;
  winKey: string;
}

export const DEFAULT_DESKTOP_SHORTCUTS: DesktopShortcut[] = [
  { id: 'command-palette', action: '命令面板', category: '通用', macKey: '⌘⇧P', winKey: 'Ctrl+Shift+P' },
  { id: 'settings', action: '设置', category: '通用', macKey: '⌘,', winKey: 'Ctrl+,' },
  { id: 'new-thread', action: '新建线程', category: '线程', macKey: '⌘N', winKey: 'Ctrl+N' },
  { id: 'toggle-sidebar', action: '切换侧边栏', category: '工作区', macKey: '⌘B', winKey: 'Ctrl+B' },
  { id: 'focus-input', action: '聚焦输入框', category: '工作区', macKey: '⌘J', winKey: 'Ctrl+J' },
  { id: 'toggle-terminal', action: '切换终端', category: '工作区', macKey: '⌘⇧J', winKey: 'Ctrl+Shift+J' },
  { id: 'toggle-agent', action: '切换 Agent 面板', category: '工作区', macKey: '⌘⇧B', winKey: 'Ctrl+Shift+B' },
  { id: 'split-right', action: '当前标签右侧分屏', category: '工作区', macKey: '⌘\\', winKey: 'Ctrl+\\' },
  { id: 'split-left', action: '当前标签左侧分屏', category: '工作区', macKey: '⌘⇧\\', winKey: 'Ctrl+Shift+\\' },
  { id: 'split-down', action: '当前标签下方分屏', category: '工作区', macKey: '⌘⌥\\', winKey: 'Ctrl+Alt+\\' },
  { id: 'merge-pane', action: '关闭工作区分屏', category: '工作区', macKey: '⌘⌥M', winKey: 'Ctrl+Alt+M' },
];

function isShortcutRow(value: JsonValue): value is {
  id: string;
  action: string;
  category: string;
  macKey: string;
  winKey: string;
} {
  return (
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    typeof value.id === 'string' &&
    typeof value.action === 'string' &&
    typeof value.category === 'string' &&
    typeof value.macKey === 'string' &&
    typeof value.winKey === 'string'
  );
}

export function shortcutsFromConfig(
  get: (path: string) => JsonValue | undefined,
): DesktopShortcut[] {
  const raw = get('desktop.shortcuts');
  if (!Array.isArray(raw)) {
    return DEFAULT_DESKTOP_SHORTCUTS;
  }
  const overrides = raw.filter(isShortcutRow);
  if (overrides.length === 0) {
    return DEFAULT_DESKTOP_SHORTCUTS;
  }
  const byId = new Map(DEFAULT_DESKTOP_SHORTCUTS.map((s) => [s.id, { ...s }]));
  for (const row of overrides) {
    byId.set(row.id, row);
  }
  return [...byId.values()];
}

export function shortcutsToConfigValue(
  shortcuts: DesktopShortcut[],
): JsonValue {
  return shortcuts.map((s) => ({
    id: s.id,
    action: s.action,
    category: s.category,
    macKey: s.macKey,
    winKey: s.winKey,
  }));
}

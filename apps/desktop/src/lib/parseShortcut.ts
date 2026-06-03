import { isMacPlatform } from '@/lib/platform';
import type { DesktopShortcut } from '@/lib/desktopShortcuts';

export interface ParsedShortcut {
  key: string;
  meta: boolean;
  ctrl: boolean;
  shift: boolean;
  alt: boolean;
}

/** 将 desktop.shortcuts 中的 macKey / winKey 解析为键盘匹配条件 */
export function parseShortcutBinding(shortcut: DesktopShortcut): ParsedShortcut {
  const raw = isMacPlatform() ? shortcut.macKey : shortcut.winKey;
  return parseShortcutString(raw);
}

function parseShortcutString(raw: string): ParsedShortcut {
  const lower = raw.toLowerCase();
  const meta = raw.includes('⌘') || lower.includes('cmd') || lower.includes('meta');
  const ctrl = lower.includes('ctrl') || lower.includes('control');
  const shift = raw.includes('⇧') || lower.includes('shift');
  const alt = raw.includes('⌥') || lower.includes('alt') || lower.includes('option');

  const key = raw
    .replace(/⌘|⇧|⌥|ctrl\+|control\+|shift\+|alt\+|option\+|meta\+/gi, '')
    .replace(/\+/g, '')
    .trim()
    .toLowerCase();

  return { key, meta, ctrl, shift, alt };
}

function modifiersMatch(e: KeyboardEvent, binding: ParsedShortcut): boolean {
  const mac = isMacPlatform();
  const mod = mac ? e.metaKey : e.ctrlKey;
  const needsMod = binding.meta || binding.ctrl;
  if (needsMod && !mod) {
    return false;
  }
  if (!needsMod && (e.metaKey || e.ctrlKey) && binding.key !== ',') {
    return false;
  }
  if (binding.shift !== e.shiftKey) {
    return false;
  }
  if (binding.alt !== e.altKey) {
    return false;
  }
  return true;
}

/** macOS 按住 ⌥ 时 e.key 常为特殊字符（如 µ），需用物理键位 e.code 匹配 */
function physicalLetterMatches(bindingKey: string, code: string): boolean {
  if (bindingKey.length !== 1 || !/^[a-z]$/.test(bindingKey)) {
    return false;
  }
  return code === `Key${bindingKey.toUpperCase()}`;
}

export function keyboardEventMatches(
  e: KeyboardEvent,
  binding: ParsedShortcut,
): boolean {
  if (!modifiersMatch(e, binding)) {
    return false;
  }

  if (binding.key === ',' && (e.key === ',' || e.key === ',')) {
    return true;
  }
  if (
    binding.key === '\\' &&
    (e.key === '\\' || e.key === 'Backslash')
  ) {
    return true;
  }

  if (binding.alt && physicalLetterMatches(binding.key, e.code)) {
    return true;
  }

  const eventKey =
    e.key === 'Backslash' || e.key === '\\'
      ? '\\'
      : e.key.length === 1
        ? e.key.toLowerCase()
        : e.key.toLowerCase();
  return eventKey === binding.key;
}

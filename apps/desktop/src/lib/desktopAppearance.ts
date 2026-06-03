import type { JsonValue } from '@/generated/app-server/serde_json/JsonValue';
import type { ThemeMode } from '@/types';

const THEME_STORAGE_KEY = 'bcip-theme';

function desktopRecord(
  value: JsonValue | undefined,
): { [key: string]: JsonValue | undefined } | null {
  if (value && typeof value === 'object' && !Array.isArray(value)) {
    return value;
  }
  return null;
}

export function readDesktopField(
  get: (path: string) => JsonValue | undefined,
  key: string,
): JsonValue | undefined {
  const desktop = desktopRecord(get('desktop'));
  return desktop?.[key];
}

export function parseThemeMode(value: JsonValue | undefined): ThemeMode {
  if (value === 'light' || value === 'dark' || value === 'system') {
    return value;
  }
  return 'system';
}

export function resolveThemeIsDark(theme: ThemeMode): boolean {
  if (theme === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches;
  }
  return theme === 'dark';
}

/** 将主题同步到 `<html class="dark">` 与 localStorage（单一来源）。 */
export function applyThemeToDocument(theme: ThemeMode): boolean {
  const isDark = resolveThemeIsDark(theme);
  const root = document.documentElement;
  if (isDark) {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
  localStorage.setItem(THEME_STORAGE_KEY, theme);
  return isDark;
}

export function readStoredThemeMode(): ThemeMode {
  return parseThemeMode(localStorage.getItem(THEME_STORAGE_KEY) ?? undefined);
}

export function readInitialTheme(): { theme: ThemeMode; isDark: boolean } {
  const theme = readStoredThemeMode();
  const isDark = resolveThemeIsDark(theme);
  return { theme, isDark };
}

export function applyAccentColor(hex: string): void {
  document.documentElement.style.setProperty('--accent-primary', hex);
  document.documentElement.style.setProperty('--text-accent', hex);
  document.documentElement.style.setProperty('--ring', hex);
}

export function applyFontSizes(uiPx: number, codePx: number): void {
  document.documentElement.style.setProperty(
    '--desktop-ui-font-size',
    `${uiPx}px`,
  );
  document.documentElement.style.setProperty(
    '--desktop-code-font-size',
    `${codePx}px`,
  );
  document.documentElement.style.fontSize = `${uiPx}px`;
}

export function applyDesktopAppearance(
  get: (path: string) => JsonValue | undefined,
): ThemeMode | null {
  const theme = readDesktopField(get, 'theme');
  const accent = readDesktopField(get, 'accent_color');
  const uiFont = readDesktopField(get, 'ui_font_size');
  const codeFont = readDesktopField(get, 'code_font_size');

  if (typeof accent === 'string') {
    applyAccentColor(accent);
  }
  if (typeof uiFont === 'number' && typeof codeFont === 'number') {
    applyFontSizes(uiFont, codeFont);
  } else if (typeof uiFont === 'number') {
    applyFontSizes(uiFont, 13);
  }
  if (theme === 'light' || theme === 'dark' || theme === 'system') {
    return theme;
  }
  return null;
}

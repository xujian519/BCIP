import type { JsonValue } from '@/generated/app-server/serde_json/JsonValue';
import type { ThemeMode } from '@/types';
import { isTauri } from '@/api/tauri';

const THEME_STORAGE_KEY = 'bcip-theme';

/** 与 index.css 中 .dark / :root 背景一致，用于原生窗口底色 */
const WINDOW_BG = {
  light: [245, 242, 238, 255] as [number, number, number, number],
  dark: [28, 26, 24, 255] as [number, number, number, number],
};

const META_THEME_COLORS = {
  light: '#F5F2EE',
  dark: '#1C1A18',
} as const;

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

/** 同步浏览器 meta theme-color 与 Tauri 原生窗口底色，避免暗色下白边闪烁。 */
function syncNativeWindowChrome(isDark: boolean): void {
  const themeColor = isDark ? META_THEME_COLORS.dark : META_THEME_COLORS.light;
  let meta = document.querySelector('meta[name="theme-color"]');
  if (!meta) {
    meta = document.createElement('meta');
    meta.setAttribute('name', 'theme-color');
    document.head.appendChild(meta);
  }
  meta.setAttribute('content', themeColor);

  if (!isTauri()) {
    return;
  }
  void (async () => {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const bg = isDark ? WINDOW_BG.dark : WINDOW_BG.light;
      await getCurrentWindow().setBackgroundColor(bg);
    } catch {
      // 非 Tauri 或旧版 API 时忽略
    }
  })();
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
  syncNativeWindowChrome(isDark);
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

import { useState, useEffect, useCallback } from 'react';
import {
  applyThemeToDocument,
  readStoredThemeMode,
  resolveThemeIsDark,
} from '@/lib/desktopAppearance';

type Theme = 'light' | 'dark' | 'system';

export function useTheme() {
  const [theme, setTheme] = useState<Theme>(() => readStoredThemeMode());

  const [resolvedTheme, setResolvedTheme] = useState<'light' | 'dark'>(() =>
    resolveThemeIsDark(readStoredThemeMode()) ? 'dark' : 'light',
  );

  useEffect(() => {
    const isDark = applyThemeToDocument(theme);
    setResolvedTheme(isDark ? 'dark' : 'light');

    if (theme !== 'system') {
      return;
    }
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => {
      const nextDark = applyThemeToDocument('system');
      setResolvedTheme(nextDark ? 'dark' : 'light');
    };
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, [theme]);

  const setThemeValue = useCallback((newTheme: Theme) => {
    setTheme(newTheme);
  }, []);

  return {
    theme,
    resolvedTheme,
    setTheme: setThemeValue,
    isDark: resolvedTheme === 'dark',
  };
}

export function toggleTheme() {
  const stored = readStoredThemeMode();
  const isDark = resolveThemeIsDark(stored);
  applyThemeToDocument(isDark ? 'light' : 'dark');
}

export function getSystemTheme(): 'light' | 'dark' {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

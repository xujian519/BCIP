import { useEffect } from 'react';
import { applyThemeToDocument } from '@/lib/desktopAppearance';
import type { ThemeMode } from '@/types';

/** 将 store 中的 theme 同步到 documentElement，并监听 system 模式变化。 */
export function useThemeSync(theme: ThemeMode): void {
  useEffect(() => {
    applyThemeToDocument(theme);
  }, [theme]);

  useEffect(() => {
    if (theme !== 'system') {
      return;
    }
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => {
      applyThemeToDocument('system');
    };
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, [theme]);
}

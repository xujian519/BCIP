/**
 * 主应用：生产壳层 + Codex 对齐 Agent + 全局浮层
 */
import { AppProvider } from '@/hooks/useAppStore';
import DesktopShell from '@/components/shell/DesktopShell';
import GlobalOverlays from '@/components/shell/GlobalOverlays';
import DesktopBootOverlay from '@/components/boot/DesktopBootOverlay';
import { Toaster } from '@/components/ui/sonner';
import { useEffect } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import { useDesktopBoot } from '@/hooks/useDesktopBoot';
import { useE2eWalkthroughBridge } from '@/lib/e2eWalkthroughBridge';
import { BCIP_TOGGLE_TERMINAL } from '@/lib/desktopEvents';

function MainAppInner() {
  const { retryBoot, continueFileMode } = useDesktopBoot();
  const { state, dispatch } = useAppStore();
  useE2eWalkthroughBridge();

  useEffect(() => {
    const onToggleTerminal = () => {
      dispatch({ type: 'TOGGLE_TERMINAL_OVERLAY' });
    };
    window.addEventListener(BCIP_TOGGLE_TERMINAL, onToggleTerminal);
    return () => window.removeEventListener(BCIP_TOGGLE_TERMINAL, onToggleTerminal);
  }, [dispatch]);

  return (
    <>
      <DesktopShell />
      <DesktopBootOverlay onRetry={retryBoot} onContinueFileMode={continueFileMode} />
      <GlobalOverlays />
      <Toaster theme={state.isDark ? 'dark' : 'light'} position="bottom-right" />
    </>
  );
}

export default function MainApp() {
  return (
    <AppProvider>
      <MainAppInner />
    </AppProvider>
  );
}

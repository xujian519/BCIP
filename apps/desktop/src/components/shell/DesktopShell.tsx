import { useCallback, useRef } from 'react';
import { useGlobalKeyboardShortcuts } from '@/hooks/useGlobalKeyboardShortcuts';
import { useResponsiveShellLayout } from '@/hooks/useResponsiveShellLayout';
import { isWindowsPlatform } from '@/lib/platform';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/hooks/useAppStore';
import { useProjectBootstrap } from '@/hooks/useProjectBootstrap';
import { useBootstrapDesktopConfig } from '@/hooks/useBootstrapDesktopConfig';
import { useThemeSync } from '@/hooks/useThemeSync';
import { isDesktopRpcReady } from '@/lib/configAccess';
import TitleBar from './TitleBar';
import StatusBar from './StatusBar';
import ResizeHandle from './ResizeHandle';
import ActivityBar from '@/components/activity-bar/ActivityBar';
import SidePanel from '@/components/activity-bar/SidePanel';
import DocumentWorkspace from '@/components/workspace/DocumentWorkspace';
import AgentPanel from '@/components/agent/AgentPanel';

const MAX_SIDEBAR_WIDTH = 520;
const AGENT_MIN = 280;

export default function DesktopShell() {
  const { state, dispatch } = useAppStore();
  useProjectBootstrap();
  useBootstrapDesktopConfig(
    isDesktopRpcReady(state.connectionStatus),
    state.workspaceCwd,
    dispatch,
  );
  useThemeSync(state.theme);
  const centerRef = useRef<HTMLDivElement>(null);

  useGlobalKeyboardShortcuts();
  useResponsiveShellLayout(centerRef);

  const agentMax = Math.floor(window.innerWidth * 0.7);

  const handleLeftResize = useCallback(
    (width: number) => dispatch({ type: 'SET_LEFT_SIDEBAR_WIDTH', payload: width }),
    [dispatch],
  );

  const handleAgentResize = useCallback(
    (width: number) => dispatch({ type: 'SET_AGENT_PANEL_WIDTH', payload: width }),
    [dispatch],
  );

  const handleChatHeightResize = useCallback(
    (height: number) => dispatch({ type: 'SET_CHAT_PANEL_HEIGHT', payload: height }),
    [dispatch],
  );

  const sidePanelVisible = state.activityBarTab !== null;
  const showAgent = state.agentPanelOpen;
  const isHorizontalSplit = state.layoutMode === 'horizontal-split';

  return (
    <div
      className={cn(
        'h-[100dvh] w-full flex flex-col overflow-hidden',
        'bg-[var(--bg-base)] text-[var(--text-primary)]',
        isWindowsPlatform() && 'platform-windows',
      )}
      data-platform={isWindowsPlatform() ? 'windows' : 'mac'}
    >
      <TitleBar />

      <div className="flex-1 flex min-h-0">
        <ActivityBar />

        {sidePanelVisible && (
          <>
            <SidePanel />
            <ResizeHandle
              direction="horizontal"
              size={state.leftSidebarWidth}
              minSize={280}
              maxSize={MAX_SIDEBAR_WIDTH}
              onResize={handleLeftResize}
              position="left"
            />
          </>
        )}

        {isHorizontalSplit ? (
          <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
            <div style={{ height: `calc(100% - ${state.chatPanelHeight}px)` }}>
              <DocumentWorkspace />
            </div>
            <ResizeHandle
              direction="vertical"
              size={state.chatPanelHeight}
              minSize={Math.floor(window.innerHeight * 0.2)}
              maxSize={Math.floor(window.innerHeight * 0.8)}
              onResize={handleChatHeightResize}
              position="bottom"
            />
            <AgentPanel width={0} fillWidth />
          </div>
        ) : (
          <>
            <div
              ref={centerRef}
              className="flex min-w-0 flex-1 flex-col overflow-hidden"
              style={{ minWidth: 200 }}
            >
              <DocumentWorkspace />
            </div>

            {showAgent && (
              <ResizeHandle
                direction="horizontal"
                size={state.agentPanelWidth}
                minSize={AGENT_MIN}
                maxSize={agentMax}
                onResize={handleAgentResize}
                position="right"
              />
            )}

            {showAgent && (
              <AgentPanel width={state.agentPanelWidth} />
            )}
          </>
        )}
      </div>

      <StatusBar />
    </div>
  );
}
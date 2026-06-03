import { readInitialTheme } from '@/lib/desktopAppearance';
import type { AppState, StageInfo } from '@/types';

const defaultStages: StageInfo[] = [
  { id: 'search', label: '检索', status: 'active' },
  { id: 'compare', label: '对比', status: 'pending' },
  { id: 'review', label: '审查', status: 'pending' },
  { id: 'draft', label: '起草', status: 'pending' },
];

export function buildInitialState(): AppState {
  const { theme, isDark } =
    typeof window !== 'undefined'
      ? readInitialTheme()
      : { theme: 'system' as const, isDark: false };

  return {
    theme,
    isDark,

    leftSidebarOpen: true,
    leftSidebarWidth: 260,
    agentPanelOpen: true,
    agentPanelWidth: 380,
    threadListOpen: false,
    sidebarTab: 'project',
    activityBarTab: 'files',
    projectRailOpen: true,
    layoutMode: 'three-column',
    chatPanelHeight: typeof window !== 'undefined' ? Math.floor(window.innerHeight * 0.4) : 400,
    workspaceRoot: null,
    focusedPaneId: null,
    chatMentions: [],

    connectionStatus: 'disconnected',
    bootPhase: 'idle',
    bootError: null,
    bcipInstalled: null,
    bcipPath: null,
    bcipVersion: null,
    bcipSource: null,
    bootLogLines: [],
    appServerTransport: null,
    currentModel: 'claude-sonnet-4-20250514',
    usageMeter: null,

    threads: [],
    currentThreadId: null,
    messages: [],
    isStreaming: false,

    settingsOpen: false,
    settingsPage: 'general',

    workspaceCwd: null,
    currentFile: null,
    stages: defaultStages,
    todos: [],
    todoDockOpen: true,
    todoDockHeight: 140,
    terminalOverlayOpen: false,

    approvalDialog: null,
    toolUserInput: null,
    commandPaletteOpen: false,
    mcpElicitation: null,
    oAuthWaiting: null,
  };
}

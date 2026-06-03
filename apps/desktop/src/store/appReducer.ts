import type { AppState, AppAction } from '@/types';
import { addRecentProjectPath } from '@/lib/recentProjects';
import {
  collapseAllSplitsInLayout,
  closeTabInLayout,
  findFirstLeaf,
  findLeaf,
  findLeafByTabId,
  mergePaneInLayout,
  moveTabInLayout,
  openTabInLayout,
  reorderTabInLayout,
  setActiveTabInLayout,
  setSplitRatioInLayout,
  splitTabInLayout,
} from '@/lib/workspaceLayout';
import { loadWorkspaceLayout } from '@/lib/workspaceLayoutStorage';
import type { WorkspaceNode } from '@/types';

function filePathForTab(root: WorkspaceNode | null, tabId: string): string | null {
  const leaf = findLeafByTabId(root, tabId);
  if (!leaf) {
    return null;
  }
  return leaf.tabs.find((tab) => tab.id === tabId)?.filePath ?? null;
}

function activeFileInPane(
  root: WorkspaceNode | null,
  paneId: string | null,
): string | null {
  if (!root || !paneId) {
    return null;
  }
  const pane = findLeaf(root, paneId);
  if (!pane?.activeTabId) {
    return null;
  }
  return filePathForTab(root, pane.activeTabId);
}

export function appReducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'SET_THEME': {
      const isDark =
        action.payload === 'system'
          ? window.matchMedia('(prefers-color-scheme: dark)').matches
          : action.payload === 'dark';
      return { ...state, theme: action.payload, isDark };
    }
    case 'TOGGLE_DARK': {
      const nextTheme = state.isDark ? 'light' : 'dark';
      return {
        ...state,
        theme: nextTheme,
        isDark: nextTheme === 'dark',
      };
    }

    case 'TOGGLE_LEFT_SIDEBAR':
      if (state.activityBarTab !== null) {
        return {
          ...state,
          activityBarTab: null,
          leftSidebarOpen: false,
        };
      }
      return {
        ...state,
        activityBarTab: 'files',
        leftSidebarOpen: true,
      };
    case 'SET_LEFT_SIDEBAR_OPEN':
      return { ...state, leftSidebarOpen: action.payload };
    case 'SET_LEFT_SIDEBAR_WIDTH':
      return { ...state, leftSidebarWidth: action.payload };
    case 'TOGGLE_AGENT_PANEL':
      return { ...state, agentPanelOpen: !state.agentPanelOpen };
    case 'SET_AGENT_PANEL_OPEN':
      return { ...state, agentPanelOpen: action.payload };
    case 'SET_AGENT_PANEL_WIDTH':
      return { ...state, agentPanelWidth: action.payload };
    case 'TOGGLE_THREAD_LIST':
      return { ...state, threadListOpen: !state.threadListOpen };
    case 'SET_THREAD_LIST_OPEN':
      return { ...state, threadListOpen: action.payload };
    case 'SET_SIDEBAR_TAB':
      return { ...state, sidebarTab: action.payload };
    case 'SET_ACTIVITY_BAR_TAB': {
      const tab = action.payload;
      return {
        ...state,
        activityBarTab: tab,
        leftSidebarOpen: tab !== null,
      };
    }
    case 'TOGGLE_PROJECT_RAIL':
      return { ...state, projectRailOpen: !state.projectRailOpen };
    case 'SET_PROJECT_RAIL_OPEN':
      return { ...state, projectRailOpen: action.payload };
    case 'SET_LAYOUT_MODE':
      return { ...state, layoutMode: action.payload };
    case 'SET_CHAT_PANEL_HEIGHT':
      return { ...state, chatPanelHeight: action.payload };
    case 'OPEN_TAB': {
      const opened = openTabInLayout(
        state.workspaceRoot,
        state.focusedPaneId,
        action.payload,
      );
      return {
        ...state,
        workspaceRoot: opened.root,
        focusedPaneId: opened.focusedPaneId,
        currentFile: action.payload.filePath,
      };
    }
    case 'CLOSE_TAB': {
      const closingPath = filePathForTab(state.workspaceRoot, action.payload);
      const closed = closeTabInLayout(state.workspaceRoot, action.payload);
      const currentFile =
        closingPath && state.currentFile === closingPath
          ? activeFileInPane(closed.root, closed.focusedPaneId)
          : state.currentFile;
      return {
        ...state,
        workspaceRoot: closed.root,
        focusedPaneId: closed.focusedPaneId,
        currentFile,
      };
    }
    case 'SET_ACTIVE_TAB': {
      const nextRoot = setActiveTabInLayout(
        state.workspaceRoot,
        action.payload.paneId,
        action.payload.tabId,
      );
      return {
        ...state,
        workspaceRoot: nextRoot,
        focusedPaneId: action.payload.paneId,
        currentFile:
          filePathForTab(nextRoot, action.payload.tabId) ?? state.currentFile,
      };
    }
    case 'SET_FOCUSED_PANE':
      return { ...state, focusedPaneId: action.payload };
    case 'SPLIT_TAB': {
      if (!state.workspaceRoot) {
        return state;
      }
      const split = splitTabInLayout(
        state.workspaceRoot,
        action.payload.paneId,
        action.payload.tabId,
        action.payload.side,
      );
      return {
        ...state,
        workspaceRoot: split.root,
        focusedPaneId: split.focusedPaneId,
      };
    }
    case 'SPLIT_ACTIVE_TAB': {
      if (!state.workspaceRoot || !state.focusedPaneId) {
        return state;
      }
      const leaf = findLeaf(state.workspaceRoot, state.focusedPaneId);
      if (!leaf?.activeTabId) {
        return state;
      }
      const split = splitTabInLayout(
        state.workspaceRoot,
        state.focusedPaneId,
        leaf.activeTabId,
        action.payload.side,
      );
      return {
        ...state,
        workspaceRoot: split.root,
        focusedPaneId: split.focusedPaneId,
      };
    }
    case 'OPEN_TAB_SPLIT': {
      const opened = openTabInLayout(
        state.workspaceRoot,
        state.focusedPaneId,
        action.payload.tab,
      );
      const split = splitTabInLayout(
        opened.root,
        opened.focusedPaneId,
        opened.activeTabId,
        action.payload.side,
      );
      return {
        ...state,
        workspaceRoot: split.root,
        focusedPaneId: split.focusedPaneId,
        currentFile: action.payload.tab.filePath,
      };
    }
    case 'MERGE_FOCUSED_PANE': {
      if (!state.workspaceRoot) {
        return state;
      }
      const paneId =
        state.focusedPaneId ?? findFirstLeaf(state.workspaceRoot)?.id ?? null;
      if (!paneId) {
        return state;
      }
      const merged = mergePaneInLayout(state.workspaceRoot, paneId);
      if (!merged) {
        return state;
      }
      return {
        ...state,
        workspaceRoot: merged.root,
        focusedPaneId: merged.focusedPaneId,
        currentFile:
          activeFileInPane(merged.root, merged.focusedPaneId) ??
          state.currentFile,
      };
    }
    case 'COLLAPSE_WORKSPACE_SPLITS': {
      if (!state.workspaceRoot) {
        return state;
      }
      const collapsed = collapseAllSplitsInLayout(state.workspaceRoot);
      return {
        ...state,
        workspaceRoot: collapsed.root,
        focusedPaneId: collapsed.focusedPaneId,
        currentFile:
          activeFileInPane(collapsed.root, collapsed.focusedPaneId) ??
          state.currentFile,
      };
    }
    case 'MOVE_TAB': {
      if (!state.workspaceRoot) {
        return state;
      }
      const moved = moveTabInLayout(
        state.workspaceRoot,
        action.payload.tabId,
        action.payload.targetPaneId,
        action.payload.insertIndex,
      );
      return {
        ...state,
        workspaceRoot: moved.root,
        focusedPaneId: moved.focusedPaneId,
        currentFile:
          filePathForTab(moved.root, action.payload.tabId) ?? state.currentFile,
      };
    }
    case 'REORDER_TAB': {
      if (!state.workspaceRoot) {
        return state;
      }
      return {
        ...state,
        workspaceRoot: reorderTabInLayout(
          state.workspaceRoot,
          action.payload.paneId,
          action.payload.tabId,
          action.payload.toIndex,
        ),
      };
    }
    case 'SET_WORKSPACE_SPLIT_RATIO': {
      if (!state.workspaceRoot) {
        return state;
      }
      return {
        ...state,
        workspaceRoot: setSplitRatioInLayout(
          state.workspaceRoot,
          action.payload.splitId,
          action.payload.ratio,
        ),
      };
    }
    case 'INSERT_CHAT_MENTION':
      return { ...state, chatMentions: [...state.chatMentions, action.payload] };
    case 'CLEAR_CHAT_MENTIONS':
      return { ...state, chatMentions: [] };

    case 'SET_CONNECTION_STATUS':
      return { ...state, connectionStatus: action.payload };
    case 'SET_BOOT_PHASE':
      return { ...state, bootPhase: action.payload };
    case 'SET_BOOT_ERROR':
      return { ...state, bootError: action.payload };
    case 'SET_BCIP_INSTALLED':
      return { ...state, bcipInstalled: action.payload };
    case 'SET_BCIP_RESOLUTION':
      return {
        ...state,
        bcipPath: action.payload.path,
        bcipVersion: action.payload.version,
        bcipSource: action.payload.source,
      };
    case 'APPEND_BOOT_LOG':
      return {
        ...state,
        bootLogLines: [...state.bootLogLines, action.payload],
      };
    case 'CLEAR_BOOT_LOG':
      return { ...state, bootLogLines: [] };
    case 'SET_APP_SERVER_TRANSPORT':
      return { ...state, appServerTransport: action.payload };
    case 'SET_CURRENT_MODEL':
      return { ...state, currentModel: action.payload };
    case 'SET_USAGE_METER':
      return { ...state, usageMeter: action.payload };

    case 'SET_THREADS':
      return { ...state, threads: action.payload };
    case 'SET_CURRENT_THREAD':
      return { ...state, currentThreadId: action.payload };
    case 'ADD_MESSAGE':
      return { ...state, messages: [...state.messages, action.payload] };
    case 'UPDATE_MESSAGE': {
      const { id, updates } = action.payload;
      return {
        ...state,
        messages: state.messages.map((m) =>
          m.id === id ? { ...m, ...updates } : m,
        ),
      };
    }
    case 'SET_STREAMING':
      return { ...state, isStreaming: action.payload };
    case 'SET_MESSAGES':
      return { ...state, messages: action.payload };
    case 'APPEND_MESSAGE_DELTA': {
      const { id, delta } = action.payload;
      const exists = state.messages.some((m) => m.id === id);
      if (!exists) {
        return {
          ...state,
          messages: [
            ...state.messages,
            {
              id,
              role: 'agent',
              content: delta,
              timestamp: Date.now(),
              status: 'streaming',
              itemKind: 'agent',
            },
          ],
        };
      }
      return {
        ...state,
        messages: state.messages.map((m) =>
          m.id === id
            ? { ...m, content: m.content + delta, status: 'streaming' }
            : m,
        ),
      };
    }

    case 'OPEN_SETTINGS':
      return { ...state, settingsOpen: true, settingsPage: action.payload };
    case 'CLOSE_SETTINGS':
      return { ...state, settingsOpen: false };
    case 'SET_SETTINGS_PAGE':
      return { ...state, settingsPage: action.payload };

    case 'SET_WORKSPACE_CWD':
      return { ...state, workspaceCwd: action.payload };
    case 'SWITCH_PROJECT': {
      addRecentProjectPath(action.payload);
      const restored = loadWorkspaceLayout(action.payload);
      return {
        ...state,
        workspaceCwd: action.payload,
        workspaceRoot: restored?.root ?? null,
        focusedPaneId: restored?.focusedPaneId ?? null,
        currentFile: restored?.currentFile ?? null,
        chatMentions: [],
      };
    }
    case 'REMOVE_THREAD': {
      const remaining = state.threads.filter((t) => t.id !== action.payload);
      const currentThreadId =
        state.currentThreadId === action.payload
          ? (remaining[0]?.id ?? null)
          : state.currentThreadId;
      return {
        ...state,
        threads: remaining,
        currentThreadId,
        messages: state.currentThreadId === action.payload ? [] : state.messages,
        isStreaming: state.currentThreadId === action.payload ? false : state.isStreaming,
      };
    }
    case 'SET_CURRENT_FILE':
      return { ...state, currentFile: action.payload };
    case 'SET_STAGES':
      return { ...state, stages: action.payload };
    case 'UPDATE_STAGE': {
      const { id, status } = action.payload;
      return {
        ...state,
        stages: state.stages.map((s) => {
          if (s.id === id) {
            return { ...s, status };
          }
          if (status === 'active' && s.status === 'active') {
            return { ...s, status: 'pending' as const };
          }
          return s;
        }),
      };
    }
    case 'SET_TODOS':
      return { ...state, todos: action.payload };
    case 'ADD_TODO':
      return { ...state, todos: [...state.todos, action.payload] };
    case 'UPDATE_TODO': {
      const { id, completed } = action.payload;
      return {
        ...state,
        todos: state.todos.map((t) =>
          t.id === id ? { ...t, completed } : t,
        ),
      };
    }
    case 'DELETE_TODO':
      return {
        ...state,
        todos: state.todos.filter((t) => t.id !== action.payload),
      };
    case 'TOGGLE_TODO_DOCK':
      return { ...state, todoDockOpen: !state.todoDockOpen };
    case 'SET_TODO_DOCK_HEIGHT':
      return {
        ...state,
        todoDockHeight: Math.max(36, Math.min(240, action.payload)),
        todoDockOpen: action.payload > 36 ? true : state.todoDockOpen,
      };
    case 'TOGGLE_TERMINAL_OVERLAY':
      return { ...state, terminalOverlayOpen: !state.terminalOverlayOpen };
    case 'SET_TERMINAL_OVERLAY_OPEN':
      return { ...state, terminalOverlayOpen: action.payload };

    case 'SET_APPROVAL_DIALOG':
      return { ...state, approvalDialog: action.payload };
    case 'SET_TOOL_USER_INPUT':
      return { ...state, toolUserInput: action.payload };
    case 'TOGGLE_COMMAND_PALETTE':
      return { ...state, commandPaletteOpen: !state.commandPaletteOpen };
    case 'SET_MCP_ELICITATION':
      return { ...state, mcpElicitation: action.payload };
    case 'SET_OAUTH_WAITING':
      return { ...state, oAuthWaiting: action.payload };
    case 'CLEAR_SESSION':
      return {
        ...state,
        threads: [],
        currentThreadId: null,
        messages: [],
        isStreaming: false,
        workspaceRoot: null,
        focusedPaneId: null,
        chatMentions: [],
        currentFile: null,
      };

    default:
      return state;
  }
}

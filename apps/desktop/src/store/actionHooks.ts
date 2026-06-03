import { useCallback } from 'react';
import type {
  ThemeMode,
  SidebarTab,
  ActivityBarTab,
  LayoutMode,
  SettingsPage,
  Message,
  WorkStage,
  StageInfo,
  ApprovalRequest,
  TodoItem,
} from '@/types';
import { useAppStore } from './AppStoreContext';

export function useThemeActions() {
  const { dispatch } = useAppStore();

  const setTheme = useCallback(
    (theme: ThemeMode) => dispatch({ type: 'SET_THEME', payload: theme }),
    [dispatch],
  );

  const toggleDark = useCallback(
    () => dispatch({ type: 'TOGGLE_DARK' }),
    [dispatch],
  );

  return { setTheme, toggleDark };
}

export function useLayoutActions() {
  const { dispatch } = useAppStore();

  const toggleLeftSidebar = useCallback(
    () => dispatch({ type: 'TOGGLE_LEFT_SIDEBAR' }),
    [dispatch],
  );

  const setLeftSidebarWidth = useCallback(
    (width: number) =>
      dispatch({ type: 'SET_LEFT_SIDEBAR_WIDTH', payload: width }),
    [dispatch],
  );

  const toggleAgentPanel = useCallback(
    () => dispatch({ type: 'TOGGLE_AGENT_PANEL' }),
    [dispatch],
  );

  const setAgentPanelWidth = useCallback(
    (width: number) =>
      dispatch({ type: 'SET_AGENT_PANEL_WIDTH', payload: width }),
    [dispatch],
  );

  const setSidebarTab = useCallback(
    (tab: SidebarTab) => dispatch({ type: 'SET_SIDEBAR_TAB', payload: tab }),
    [dispatch],
  );

  const setActivityBarTab = useCallback(
    (tab: ActivityBarTab | null) => dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: tab }),
    [dispatch],
  );

  const setLayoutMode = useCallback(
    (mode: LayoutMode) => dispatch({ type: 'SET_LAYOUT_MODE', payload: mode }),
    [dispatch],
  );

  return {
    toggleLeftSidebar,
    setLeftSidebarWidth,
    toggleAgentPanel,
    setAgentPanelWidth,
    setSidebarTab,
    setActivityBarTab,
    setLayoutMode,
  };
}

export function useThreadActions() {
  const { dispatch } = useAppStore();

  const setCurrentThread = useCallback(
    (id: string | null) =>
      dispatch({ type: 'SET_CURRENT_THREAD', payload: id }),
    [dispatch],
  );

  const addMessage = useCallback(
    (message: Message) => dispatch({ type: 'ADD_MESSAGE', payload: message }),
    [dispatch],
  );

  const setStreaming = useCallback(
    (streaming: boolean) =>
      dispatch({ type: 'SET_STREAMING', payload: streaming }),
    [dispatch],
  );

  return { setCurrentThread, addMessage, setStreaming };
}

export function useSettingsActions() {
  const { dispatch } = useAppStore();

  const openSettings = useCallback(
    (page: SettingsPage = 'general') =>
      dispatch({ type: 'OPEN_SETTINGS', payload: page }),
    [dispatch],
  );

  const closeSettings = useCallback(
    () => dispatch({ type: 'CLOSE_SETTINGS' }),
    [dispatch],
  );

  return { openSettings, closeSettings };
}

export function useStageActions() {
  const { dispatch } = useAppStore();

  const updateStage = useCallback(
    (id: WorkStage, status: StageInfo['status']) =>
      dispatch({ type: 'UPDATE_STAGE', payload: { id, status } }),
    [dispatch],
  );

  const setStages = useCallback(
    (stages: StageInfo[]) => dispatch({ type: 'SET_STAGES', payload: stages }),
    [dispatch],
  );

  return { updateStage, setStages };
}

export function useTodoActions() {
  const { dispatch } = useAppStore();

  const addTodo = useCallback(
    (todo: TodoItem) => dispatch({ type: 'ADD_TODO', payload: todo }),
    [dispatch],
  );

  const updateTodo = useCallback(
    (id: string, completed: boolean) =>
      dispatch({ type: 'UPDATE_TODO', payload: { id, completed } }),
    [dispatch],
  );

  const deleteTodo = useCallback(
    (id: string) => dispatch({ type: 'DELETE_TODO', payload: id }),
    [dispatch],
  );

  const toggleTodoDock = useCallback(
    () => dispatch({ type: 'TOGGLE_TODO_DOCK' }),
    [dispatch],
  );

  return { addTodo, updateTodo, deleteTodo, toggleTodoDock };
}

export function useApprovalActions() {
  const { dispatch } = useAppStore();

  const setApprovalDialog = useCallback(
    (request: ApprovalRequest | null) =>
      dispatch({ type: 'SET_APPROVAL_DIALOG', payload: request }),
    [dispatch],
  );

  return { setApprovalDialog };
}

/**
 * 设计 spec §8.2 全局快捷键
 */
import { useEffect } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import { sendApprovalDecision } from '@/lib/approvalRespond';
import { DEFAULT_DESKTOP_SHORTCUTS } from '@/lib/desktopShortcuts';
import { focusComposer, requestNewThread } from '@/lib/desktopEvents';
import { keyboardEventMatches, parseShortcutBinding } from '@/lib/parseShortcut';

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const tag = target.tagName;
  return (
    tag === 'INPUT' ||
    tag === 'TEXTAREA' ||
    tag === 'SELECT' ||
    target.isContentEditable
  );
}

export function useGlobalKeyboardShortcuts() {
  const { state, dispatch } = useAppStore();

  useEffect(() => {
    const bindings = DEFAULT_DESKTOP_SHORTCUTS.map((s) => ({
      id: s.id,
      binding: parseShortcutBinding(s),
    }));

    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (state.commandPaletteOpen) {
          e.preventDefault();
          dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
          return;
        }
        if (state.settingsOpen) {
          e.preventDefault();
          dispatch({ type: 'CLOSE_SETTINGS' });
          return;
        }
        if (state.approvalDialog) {
          e.preventDefault();
          void sendApprovalDecision(state.approvalDialog, 'decline').finally(() => {
            dispatch({ type: 'SET_APPROVAL_DIALOG', payload: null });
          });
          return;
        }
        if (state.mcpElicitation) {
          e.preventDefault();
          dispatch({ type: 'SET_MCP_ELICITATION', payload: null });
          return;
        }
        return;
      }

      const inField = isEditableTarget(e.target);
      for (const { id, binding } of bindings) {
        if (!keyboardEventMatches(e, binding)) {
          continue;
        }
        if (inField && id !== 'command-palette' && id !== 'focus-input') {
          continue;
        }
        e.preventDefault();
        switch (id) {
          case 'command-palette':
            dispatch({ type: 'TOGGLE_COMMAND_PALETTE' });
            break;
          case 'settings':
            dispatch({ type: 'OPEN_SETTINGS', payload: 'general' });
            break;
          case 'new-thread':
            requestNewThread();
            break;
          case 'toggle-sidebar':
            dispatch({ type: 'TOGGLE_LEFT_SIDEBAR' });
            break;
          case 'focus-input':
            focusComposer();
            break;
          case 'toggle-agent':
            dispatch({ type: 'TOGGLE_AGENT_PANEL' });
            break;
          case 'toggle-terminal':
            dispatch({ type: 'TOGGLE_TERMINAL_OVERLAY' });
            break;
          case 'split-right':
            dispatch({ type: 'SPLIT_ACTIVE_TAB', payload: { side: 'right' } });
            break;
          case 'split-left':
            dispatch({ type: 'SPLIT_ACTIVE_TAB', payload: { side: 'left' } });
            break;
          case 'split-down':
            dispatch({ type: 'SPLIT_ACTIVE_TAB', payload: { side: 'bottom' } });
            break;
          case 'merge-pane':
            dispatch({ type: 'COLLAPSE_WORKSPACE_SPLITS' });
            break;
          default:
            break;
        }
        return;
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [
    dispatch,
    state.commandPaletteOpen,
    state.settingsOpen,
    state.approvalDialog,
    state.mcpElicitation,
  ]);
}

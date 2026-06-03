/** 跨组件桌面事件（快捷键、命令面板触发） */

export const BCIP_FOCUS_COMPOSER = 'bcip:focus-composer';
export const BCIP_NEW_THREAD = 'bcip:new-thread';
export const BCIP_FILE_TREE_REFRESH = 'bcip:file-tree-refresh';
export const BCIP_PREVIEW_RELOAD = 'bcip:preview-reload';
export const BCIP_TOGGLE_TERMINAL = 'bcip:toggle-terminal';

export function focusComposer(): void {
  window.dispatchEvent(new CustomEvent(BCIP_FOCUS_COMPOSER));
}

export function requestNewThread(): void {
  window.dispatchEvent(new CustomEvent(BCIP_NEW_THREAD));
}

export function refreshFileTree(): void {
  window.dispatchEvent(new CustomEvent(BCIP_FILE_TREE_REFRESH));
}

export function reloadPreview(paths?: string[]): void {
  window.dispatchEvent(
    new CustomEvent(BCIP_PREVIEW_RELOAD, { detail: { paths: paths ?? [] } }),
  );
}

export function toggleTerminalOverlay(): void {
  window.dispatchEvent(new CustomEvent(BCIP_TOGGLE_TERMINAL));
}

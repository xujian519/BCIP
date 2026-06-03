import { invoke } from '@tauri-apps/api/core';

export interface TerminalSession {
  id: string;
  websocketUrl: string;
  command: string;
}

export interface BcipCheckResult {
  installed: boolean;
  version?: string;
  path?: string;
}

export async function checkBcipInstalled(): Promise<BcipCheckResult> {
  return await invoke<BcipCheckResult>('check_bcip_installed');
}

export async function spawnTerminal(
  command: string = 'bcip',
  args: string[] = ['tui'],
  cwd?: string
): Promise<TerminalSession> {
  const session = await invoke<{ id: string; websocket_url: string; command: string }>('pty_spawn', {
    command,
    args,
    cwd,
  });
  
  return {
    id: session.id,
    websocketUrl: session.websocket_url,
    command: session.command,
  };
}

export async function writeToTerminal(sessionId: string, data: string): Promise<void> {
  await invoke('pty_write', { sessionId, data });
}

export async function resizeTerminal(
  sessionId: string,
  cols: number,
  rows: number
): Promise<void> {
  await invoke('pty_resize', { sessionId, cols, rows });
}

export async function killTerminal(sessionId: string): Promise<void> {
  await invoke('pty_kill', { sessionId });
}
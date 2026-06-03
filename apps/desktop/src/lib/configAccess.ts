import { isTauri } from '@/api/tauri';
import type { Config } from '@/generated/app-server/v2/Config';
import type { JsonValue } from '@/generated/app-server/serde_json/JsonValue';

/** 从有效 config 对象按点分路径读取（如 `tui.notifications`） */
export function configGet(
  config: Config | null,
  keyPath: string,
): JsonValue | undefined {
  if (!config) {
    return undefined;
  }
  const parts = keyPath.split('.');
  let cur: JsonValue = config as JsonValue;
  for (const part of parts) {
    if (cur === null || typeof cur !== 'object' || Array.isArray(cur)) {
      return undefined;
    }
    const next = cur[part];
    if (next === undefined) {
      return undefined;
    }
    cur = next;
  }
  return cur;
}

export function isDesktopRpcReady(connectionStatus: string): boolean {
  return connectionStatus === 'connected' && isTauri();
}

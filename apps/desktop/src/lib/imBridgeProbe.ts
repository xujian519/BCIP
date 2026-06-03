export type ImBridgeProbeState = 'checking' | 'online' | 'offline';

const DEFAULT_PROBE_TIMEOUT_MS = 2500;

export function resolveImBridgeUrl(raw: unknown): string {
  if (typeof raw === 'string' && raw.trim().length > 0) {
    return raw.trim();
  }
  return 'ws://127.0.0.1:3456';
}

/** 探测 IM Bridge WebSocket 是否可达（短连接后立即关闭） */
export function probeImBridge(
  url: string,
  timeoutMs = DEFAULT_PROBE_TIMEOUT_MS,
): Promise<boolean> {
  if (typeof WebSocket === 'undefined') {
    return Promise.resolve(false);
  }

  return new Promise((resolve) => {
    let settled = false;
    const finish = (ok: boolean) => {
      if (settled) {
        return;
      }
      settled = true;
      resolve(ok);
    };

    let ws: WebSocket;
    try {
      ws = new WebSocket(url);
    } catch {
      finish(false);
      return;
    }

    const timer = window.setTimeout(() => {
      ws.close();
      finish(false);
    }, timeoutMs);

    ws.onopen = () => {
      window.clearTimeout(timer);
      ws.close();
      finish(true);
    };

    ws.onerror = () => {
      window.clearTimeout(timer);
      finish(false);
    };
  });
}

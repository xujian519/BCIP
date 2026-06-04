/**
 * app-server 客户端：经 Tauri stdio 桥接收发 JSON-RPC（JSONL）。
 * 类型生成后迁至 `generated/app-server`（`bcip app-server generate-ts`）。
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface AppServerStatus {
  connected: boolean;
  transport: string;
  error?: string | null;
}

export interface JsonRpcRequest {
  jsonrpc: '2.0';
  id: string | number;
  method: string;
  params?: unknown;
}

export interface JsonRpcResponse {
  jsonrpc: '2.0';
  id: string | number;
  result?: unknown;
  error?: { code: number; message: string; data?: unknown };
}

export interface JsonRpcNotification {
  jsonrpc: '2.0';
  method: string;
  params?: unknown;
}

/** 服务端发起的 JSON-RPC 请求（如审批），需客户端 respond */
export interface JsonRpcServerRequest {
  jsonrpc: '2.0';
  id: string | number;
  method: string;
  params?: unknown;
}

type WireMessage = JsonRpcResponse | JsonRpcNotification | JsonRpcServerRequest;

export type ConnectionStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export interface AppServerClientOptions {
  onStatusChange?: (status: ConnectionStatus) => void;
  onNotification?: (notification: JsonRpcNotification) => void;
  onServerRequest?: (request: JsonRpcServerRequest) => void;
  onResponse?: (response: JsonRpcResponse) => void;
  onTransportError?: (message: string) => void;
}

const HANDSHAKE_TIMEOUT_MS = 45_000;

export class AppServerClient {
  private status: ConnectionStatus = 'disconnected';
  private unlisten: UnlistenFn | null = null;
  private pending = new Map<string | number, (response: JsonRpcResponse) => void>();
  private nextId = 1;
  private options: AppServerClientOptions;
  private initialized = false;

  constructor(options: AppServerClientOptions = {}) {
    this.options = options;
  }

  /** 注册或更新通知/状态回调（供 useAppServerSession 使用） */
  mergeHandlers(partial: Partial<AppServerClientOptions>): void {
    this.options = { ...this.options, ...partial };
  }

  getConnectionStatus(): ConnectionStatus {
    return this.status;
  }

  isInitialized(): boolean {
    return this.initialized;
  }

  async connect(): Promise<AppServerStatus> {
    // 如果已经连接且已初始化，直接返回
    if (this.status === 'connected' && this.initialized) {
      console.log('[appServerClient] already connected, reusing');
      return {
        connected: true,
        transport: 'stdio',
        error: null,
      };
    }

    this.setStatus('connecting');
    try {
      // 始终重新注册 listener（WebView 重启后旧 listener 会失效）
      if (this.unlisten) {
        try { await this.unlisten(); } catch { /* already unregistered */ }
        this.unlisten = null;
      }
      this.unlisten = await listen<string>('app-server-message', (event) => {
        this.handleWireLine(event.payload);
      });

      const status = await invoke<AppServerStatus>('app_server_connect');
      console.log('[appServerClient] invoke result:', status);
      if (!status.connected) {
        throw new Error(status.error ?? 'app-server 连接失败');
      }
      this.setStatus('connected');
      await this.handshake();
      return status;
    } catch (err) {
      this.setStatus('error');
      throw err;
    }
  }

  async disconnect(): Promise<void> {
    if (this.unlisten) {
      await this.unlisten();
      this.unlisten = null;
    }
    await invoke('app_server_disconnect');
    this.pending.clear();
    this.initialized = false;
    this.setStatus('disconnected');
  }

  async request<T = unknown>(method: string, params?: unknown): Promise<T> {
    const id = this.nextId++;
    const payload: JsonRpcRequest = {
      jsonrpc: '2.0',
      id,
      method,
      params,
    };
    const response = await this.sendRequest(payload);
    if (response.error) {
      throw new Error(response.error.message);
    }
    return response.result as T;
  }

  /** 响应服务端下发的请求（审批等） */
  async respond(id: string | number, result: unknown): Promise<void> {
    const line = JSON.stringify({
      jsonrpc: '2.0',
      id,
      result,
    });
    return invoke('app_server_send', { line });
  }

  notify(method: string, params?: unknown): Promise<void> {
    const line = JSON.stringify({
      jsonrpc: '2.0',
      method,
      params,
    } satisfies JsonRpcNotification);
    return invoke('app_server_send', { line });
  }

  private async handshake(): Promise<void> {
    console.log('[appServerClient] handshake: sending initialize…');
    await this.request('initialize', {
      clientInfo: {
        name: 'bcip-desktop',
        title: 'BCIP Agent Desktop',
        version: '0.1.0',
      },
    });
    console.log('[appServerClient] handshake: initialize OK, sending initialized…');
    await this.notify('initialized');
    this.initialized = true;
    console.log('[appServerClient] handshake complete');
  }

  private sendRequest(request: JsonRpcRequest): Promise<JsonRpcResponse> {
    return new Promise((resolve, reject) => {
      this.pending.set(request.id, resolve);
      const line = JSON.stringify(request);
      invoke('app_server_send', { line }).catch((err) => {
        this.pending.delete(request.id);
        reject(err);
      });
      window.setTimeout(() => {
        if (this.pending.has(request.id)) {
          this.pending.delete(request.id);
          reject(
            new Error(
              `请求超时（${HANDSHAKE_TIMEOUT_MS / 1000}s）: ${request.method}。请确认 bcip 已编译且 sidecar 为真实二进制（npm run prepare-sidecar）`,
            ),
          );
        }
      }, request.method === 'initialize' ? HANDSHAKE_TIMEOUT_MS : 120_000);
    });
  }

  private handleWireLine(line: string): void {
    let parsed: WireMessage;
    try {
      parsed = JSON.parse(line) as WireMessage;
    } catch (err) {
      console.warn('[appServerClient] 无法解析 wire 消息:', err instanceof Error ? err.message : String(err));
      return;
    }

    if ('method' in parsed && parsed.method === 'bcip/desktop/transportError') {
      const message =
        typeof parsed.params === 'object' &&
        parsed.params !== null &&
        'message' in parsed.params
          ? String((parsed.params as { message: string }).message)
          : '传输错误';
      this.options.onTransportError?.(message);
      this.setStatus('error');
      return;
    }

    if ('id' in parsed && parsed.id !== undefined && parsed.id !== null) {
      const hasMethod = 'method' in parsed && typeof parsed.method === 'string';
      const isServerRequest =
        hasMethod && !('result' in parsed) && !('error' in parsed);

      if (isServerRequest) {
        this.options.onServerRequest?.(parsed as JsonRpcServerRequest);
        return;
      }

      const handler = this.pending.get(parsed.id);
      if (handler) {
        this.pending.delete(parsed.id);
        handler(parsed as JsonRpcResponse);
        this.options.onResponse?.(parsed as JsonRpcResponse);
      }
      return;
    }

    if ('method' in parsed) {
      this.options.onNotification?.(parsed as JsonRpcNotification);
    }
  }

  private setStatus(status: ConnectionStatus): void {
    this.status = status;
    this.options.onStatusChange?.(status);
  }
}

let singleton: AppServerClient | null = null;

export function getAppServerClient(options?: AppServerClientOptions): AppServerClient {
  if (!singleton) {
    singleton = new AppServerClient(options);
  } else if (options) {
    singleton.mergeHandlers(options);
  }
  return singleton;
}

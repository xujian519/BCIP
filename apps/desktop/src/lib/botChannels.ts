import type { JsonValue } from '@/generated/app-server/serde_json/JsonValue';

export type BotChannelId = 'wechat' | 'feishu' | 'dingtalk';

export type BotChannelStatus = 'disconnected' | 'configured' | 'connected';

export interface BotChannelDefinition {
  id: BotChannelId;
  name: string;
  description: string;
  /** 与 codex-im-feishu 等 crate 对齐的 config 字段说明 */
  configHint: string;
}

export interface BotChannelState {
  enabled: boolean;
  webhookUrl: string;
  appId: string;
  appSecret: string;
  status: BotChannelStatus;
}

const STORAGE_KEY = 'bcip.desktop.botChannels';

export const BOT_CHANNEL_DEFINITIONS: BotChannelDefinition[] = [
  {
    id: 'wechat',
    name: '企业微信',
    description: '通过 Webhook 接收与回复会话消息',
    configHint: 'desktop.bot_channels.wechat',
  },
  {
    id: 'feishu',
    name: '飞书',
    description: 'App ID + App Secret（对接 codex-im-feishu）',
    configHint: 'desktop.bot_channels.feishu',
  },
  {
    id: 'dingtalk',
    name: '钉钉',
    description: '接入钉钉自定义机器人 Webhook',
    configHint: 'desktop.bot_channels.dingtalk',
  },
];

function defaultChannelState(): Omit<BotChannelState, 'status'> {
  return {
    enabled: false,
    webhookUrl: '',
    appId: '',
    appSecret: '',
  };
}

function getStorage(): Storage | null {
  if (typeof window === 'undefined') {
    return null;
  }
  return window.localStorage;
}

function readStore(): Partial<Record<BotChannelId, Partial<BotChannelState>>> {
  const storage = getStorage();
  if (!storage) {
    return {};
  }
  try {
    const raw = storage.getItem(STORAGE_KEY);
    if (!raw) {
      return {};
    }
    return JSON.parse(raw) as Partial<Record<BotChannelId, Partial<BotChannelState>>>;
  } catch {
    return {};
  }
}

function writeStore(value: Record<BotChannelId, BotChannelState>): void {
  getStorage()?.setItem(STORAGE_KEY, JSON.stringify(value));
}

export function deriveBotChannelStatus(state: Omit<BotChannelState, 'status'>): BotChannelStatus {
  if (!state.enabled) {
    return 'disconnected';
  }
  if (state.webhookUrl.trim().length > 0) {
    return 'configured';
  }
  if (state.appId.trim().length > 0 && state.appSecret.trim().length > 0) {
    return 'configured';
  }
  return 'disconnected';
}

function normalizeChannelState(
  partial: Partial<BotChannelState> | undefined,
): BotChannelState {
  const base = { ...defaultChannelState(), ...partial };
  return {
    ...base,
    status: deriveBotChannelStatus(base),
  };
}

export function loadBotChannelStates(): Record<BotChannelId, BotChannelState> {
  const stored = readStore();
  const merged = {} as Record<BotChannelId, BotChannelState>;
  for (const channel of BOT_CHANNEL_DEFINITIONS) {
    merged[channel.id] = normalizeChannelState(stored[channel.id]);
  }
  return merged;
}

export function applyBotChannelPatch(
  current: Record<BotChannelId, BotChannelState>,
  id: BotChannelId,
  patch: Partial<Omit<BotChannelState, 'status'>>,
): Record<BotChannelId, BotChannelState> {
  const next = normalizeChannelState({ ...current[id], ...patch });
  return { ...current, [id]: next };
}

export function persistBotChannelStates(
  states: Record<BotChannelId, BotChannelState>,
): void {
  writeStore(states);
}

export function saveBotChannelState(
  id: BotChannelId,
  patch: Partial<Omit<BotChannelState, 'status'>>,
): Record<BotChannelId, BotChannelState> {
  const all = applyBotChannelPatch(loadBotChannelStates(), id, patch);
  writeStore(all);
  return all;
}

function isRecord(value: JsonValue): value is Record<string, JsonValue> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

export function botChannelsFromConfig(value: JsonValue | undefined): Record<BotChannelId, BotChannelState> | null {
  if (value === undefined || !isRecord(value)) {
    return null;
  }
  const merged = {} as Record<BotChannelId, BotChannelState>;
  for (const channel of BOT_CHANNEL_DEFINITIONS) {
    const entry = value[channel.id];
    if (!isRecord(entry)) {
      merged[channel.id] = normalizeChannelState(undefined);
      continue;
    }
    merged[channel.id] = normalizeChannelState({
      enabled: entry.enabled === true,
      webhookUrl: typeof entry.webhookUrl === 'string' ? entry.webhookUrl : '',
      appId: typeof entry.appId === 'string' ? entry.appId : '',
      appSecret: typeof entry.appSecret === 'string' ? entry.appSecret : '',
    });
  }
  return merged;
}

export function botChannelsToConfigValue(
  states: Record<BotChannelId, BotChannelState>,
): JsonValue {
  const out: Record<string, JsonValue> = {};
  for (const channel of BOT_CHANNEL_DEFINITIONS) {
    const state = states[channel.id];
    out[channel.id] = {
      enabled: state.enabled,
      webhookUrl: state.webhookUrl,
      appId: state.appId,
      appSecret: state.appSecret,
    };
  }
  return out;
}

export const IM_BRIDGE_DEFAULT_URL = 'ws://127.0.0.1:3456';

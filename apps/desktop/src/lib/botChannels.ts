export type BotChannelId = 'wechat' | 'feishu' | 'dingtalk';

export type BotChannelStatus = 'disconnected' | 'configured' | 'connected';

export interface BotChannelDefinition {
  id: BotChannelId;
  name: string;
  description: string;
}

export interface BotChannelState {
  enabled: boolean;
  webhookUrl: string;
  status: BotChannelStatus;
}

const STORAGE_KEY = 'bcip.desktop.botChannels';

export const BOT_CHANNEL_DEFINITIONS: BotChannelDefinition[] = [
  {
    id: 'wechat',
    name: '企业微信',
    description: '通过 Webhook 接收与回复会话消息',
  },
  {
    id: 'feishu',
    name: '飞书',
    description: '绑定飞书机器人，同步群聊 @ 消息',
  },
  {
    id: 'dingtalk',
    name: '钉钉',
    description: '接入钉钉自定义机器人推送',
  },
];

function defaultChannelState(): BotChannelState {
  return {
    enabled: false,
    webhookUrl: '',
    status: 'disconnected',
  };
}

function getStorage(): Storage | null {
  if (typeof window === 'undefined') {
    return null;
  }
  return window.localStorage;
}

function readStore(): Partial<Record<BotChannelId, BotChannelState>> {
  const storage = getStorage();
  if (!storage) {
    return {};
  }
  try {
    const raw = storage.getItem(STORAGE_KEY);
    if (!raw) {
      return {};
    }
    return JSON.parse(raw) as Partial<Record<BotChannelId, BotChannelState>>;
  } catch {
    return {};
  }
}

function writeStore(value: Record<BotChannelId, BotChannelState>): void {
  getStorage()?.setItem(STORAGE_KEY, JSON.stringify(value));
}

function deriveStatus(state: BotChannelState): BotChannelStatus {
  if (!state.enabled) {
    return 'disconnected';
  }
  if (state.webhookUrl.trim().length > 0) {
    return 'configured';
  }
  return 'disconnected';
}

export function loadBotChannelStates(): Record<BotChannelId, BotChannelState> {
  const stored = readStore();
  const merged = {} as Record<BotChannelId, BotChannelState>;
  for (const channel of BOT_CHANNEL_DEFINITIONS) {
    const base = { ...defaultChannelState(), ...stored[channel.id] };
    merged[channel.id] = {
      ...base,
      status: deriveStatus(base),
    };
  }
  return merged;
}

export function saveBotChannelState(
  id: BotChannelId,
  patch: Partial<BotChannelState>,
): Record<BotChannelId, BotChannelState> {
  const current = loadBotChannelStates();
  const next = {
    ...current[id],
    ...patch,
  };
  next.status = deriveStatus(next);
  const all = { ...current, [id]: next };
  writeStore(all);
  return all;
}

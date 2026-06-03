import { describe, expect, it, beforeEach } from 'vitest';
import {
  applyBotChannelPatch,
  botChannelsFromConfig,
  botChannelsToConfigValue,
  deriveBotChannelStatus,
  effectiveBotChannelStatus,
  loadBotChannelStates,
  saveBotChannelState,
} from './botChannels';

class MemoryStorage implements Storage {
  private store = new Map<string, string>();

  get length() {
    return this.store.size;
  }

  clear(): void {
    this.store.clear();
  }

  getItem(key: string): string | null {
    return this.store.get(key) ?? null;
  }

  key(index: number): string | null {
    return [...this.store.keys()][index] ?? null;
  }

  removeItem(key: string): void {
    this.store.delete(key);
  }

  setItem(key: string, value: string): void {
    this.store.set(key, value);
  }
}

describe('botChannels', () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, 'window', {
      value: { localStorage: new MemoryStorage() },
      configurable: true,
    });
  });

  it('persists webhook url to localStorage', () => {
    const all = saveBotChannelState('dingtalk', {
      enabled: true,
      webhookUrl: 'https://example.com/hook',
    });
    expect(all.dingtalk.enabled).toBe(true);
    expect(all.dingtalk.status).toBe('configured');

    const reloaded = loadBotChannelStates();
    expect(reloaded.dingtalk.webhookUrl).toBe('https://example.com/hook');
  });

  it('marks feishu configured when app credentials present', () => {
    const status = deriveBotChannelStatus({
      enabled: true,
      webhookUrl: '',
      appId: 'cli_abc',
      appSecret: 'secret',
    });
    expect(status).toBe('configured');
  });

  it('promotes configured channel to connected when bridge online', () => {
    const state = applyBotChannelPatch(loadBotChannelStates(), 'dingtalk', {
      enabled: true,
      webhookUrl: 'https://example.com/hook',
    }).dingtalk;
    expect(effectiveBotChannelStatus(state, false)).toBe('configured');
    expect(effectiveBotChannelStatus(state, true)).toBe('connected');
  });

  it('round-trips config json', () => {
    const states = applyBotChannelPatch(loadBotChannelStates(), 'feishu', {
      enabled: true,
      appId: 'id',
      appSecret: 'sec',
    });
    const json = botChannelsToConfigValue(states);
    const parsed = botChannelsFromConfig(json);
    expect(parsed?.feishu.appId).toBe('id');
    expect(parsed?.feishu.status).toBe('configured');
  });
});

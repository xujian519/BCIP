import { describe, expect, it, beforeEach } from 'vitest';
import { loadBotChannelStates, saveBotChannelState } from './botChannels';

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
    const storage = new MemoryStorage();
    Object.defineProperty(globalThis, 'window', {
      value: { localStorage: storage },
      configurable: true,
    });
  });

  it('persists enabled flag and webhook url', () => {
    const all = saveBotChannelState('feishu', {
      enabled: true,
      webhookUrl: 'https://example.com/hook',
    });
    expect(all.feishu.enabled).toBe(true);
    expect(all.feishu.status).toBe('configured');

    const reloaded = loadBotChannelStates();
    expect(reloaded.feishu.webhookUrl).toBe('https://example.com/hook');
  });
});

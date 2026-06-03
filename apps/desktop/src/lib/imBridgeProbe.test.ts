import { describe, expect, it } from 'vitest';
import { resolveImBridgeUrl } from './imBridgeProbe';

describe('resolveImBridgeUrl', () => {
  it('uses default when config missing', () => {
    expect(resolveImBridgeUrl(undefined)).toBe('ws://127.0.0.1:3456');
  });

  it('trims configured url', () => {
    expect(resolveImBridgeUrl('  ws://127.0.0.1:4000  ')).toBe('ws://127.0.0.1:4000');
  });
});

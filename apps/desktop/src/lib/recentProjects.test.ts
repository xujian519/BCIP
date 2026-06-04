import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
  addRecentProjectPath,
  loadHiddenProjectPaths,
  loadRecentProjectPaths,
  removeProjectFromRail,
} from './recentProjects';

function createStorage(): Storage {
  const data = new Map<string, string>();
  return {
    get length() {
      return data.size;
    },
    clear: () => data.clear(),
    getItem: (key: string) => data.get(key) ?? null,
    key: (index: number) => [...data.keys()][index] ?? null,
    removeItem: (key: string) => {
      data.delete(key);
    },
    setItem: (key: string, value: string) => {
      data.set(key, value);
    },
  };
}

describe('recentProjects', () => {
  beforeEach(() => {
    vi.stubGlobal('localStorage', createStorage());
  });

  it('removeProjectFromRail 从最近列表移除并加入隐藏集', () => {
    addRecentProjectPath('/tmp/a');
    addRecentProjectPath('/tmp/b');
    removeProjectFromRail('/tmp/a');

    expect(loadRecentProjectPaths()).toEqual(['/tmp/b']);
    expect(loadHiddenProjectPaths()).toContain('/tmp/a');
  });

  it('addRecentProjectPath 会取消隐藏', () => {
    removeProjectFromRail('/tmp/a');
    addRecentProjectPath('/tmp/a');

    expect(loadHiddenProjectPaths()).not.toContain('/tmp/a');
    expect(loadRecentProjectPaths()[0]).toBe('/tmp/a');
  });
});

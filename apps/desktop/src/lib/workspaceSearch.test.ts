import { describe, expect, it } from 'vitest';
import { rankFilenameMatch } from './workspaceSearch';

describe('rankFilenameMatch', () => {
  it('matches case-insensitive substrings', () => {
    expect(rankFilenameMatch('PatentDraft.md', 'draft')).toBe(true);
    expect(rankFilenameMatch('README', 'read')).toBe(true);
    expect(rankFilenameMatch('foo.txt', 'bar')).toBe(false);
  });
});

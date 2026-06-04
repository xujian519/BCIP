import { describe, expect, it } from 'vitest';
import {
  mergeModelCatalog,
  reasoningOptionsForModel,
  STANDARD_REASONING_EFFORTS,
} from './bcipModelPresets';

describe('bcipModelPresets', () => {
  it('falls back to presets when catalog is empty', () => {
    const merged = mergeModelCatalog([], 'kimi-for-coding');
    expect(merged.some((m) => m.model === 'glm-5.1')).toBe(true);
    expect(merged.some((m) => m.model === 'kimi-for-coding')).toBe(true);
  });

  it('includes unknown config model in fallback list', () => {
    const merged = mergeModelCatalog([], 'custom-model-x');
    expect(merged.some((m) => m.model === 'custom-model-x')).toBe(true);
  });

  it('returns standard reasoning when catalog model has no efforts', () => {
    const merged = mergeModelCatalog([], 'glm-5.1');
    const opts = reasoningOptionsForModel(merged[0]);
    expect(opts.map((o) => o.reasoningEffort)).toEqual(STANDARD_REASONING_EFFORTS);
  });
});

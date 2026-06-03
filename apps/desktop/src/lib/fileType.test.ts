import { describe, expect, it } from 'vitest';
import { getFileType, isPreviewable } from './fileType';

describe('fileType', () => {
  it('distinguishes doc and docx', () => {
    expect(getFileType('report.doc')).toBe('doc');
    expect(getFileType('report.docx')).toBe('docx');
  });

  it('marks doc as previewable', () => {
    expect(isPreviewable('legacy.doc')).toBe(true);
  });
});

import { describe, expect, it } from 'vitest';
import {
  approvalDocumentPath,
  approvalIdFromDocumentPath,
  buildApprovalMarkdown,
  isApprovalDocumentPath,
} from './approvalDocument';
import type { ApprovalRequest } from '@/types';

const sampleApproval: ApprovalRequest = {
  id: 'item-42',
  type: 'command',
  title: '允许执行命令？',
  description: 'Agent 请求执行 shell 命令',
  riskLevel: 'medium',
  command: 'npm test',
  cwd: '/tmp/project',
  isDangerous: false,
};

describe('approvalDocument', () => {
  it('builds markdown summary with frontmatter', () => {
    const md = buildApprovalMarkdown({ approval: sampleApproval });
    expect(md).toContain('bcip_approval: true');
    expect(md).toContain('decision: pending');
    expect(md).toContain('```bash');
    expect(md).toContain('npm test');
    expect(md).toContain('/tmp/project');
  });

  it('detects approval document paths', () => {
    const path = approvalDocumentPath('/tmp/project', 'item-42');
    expect(path).toBe('/tmp/project/.bcip/approvals/approval-item-42.md');
    expect(isApprovalDocumentPath(path)).toBe(true);
    expect(isApprovalDocumentPath('/tmp/project/readme.md')).toBe(false);
  });

  it('extracts approval id from path', () => {
    expect(
      approvalIdFromDocumentPath('/tmp/.bcip/approvals/approval-item-42.md'),
    ).toBe('item-42');
  });

  it('includes file change list for patch approvals', () => {
    const md = buildApprovalMarkdown({
      approval: {
        ...sampleApproval,
        type: 'file',
        title: '允许应用补丁？',
        command: '2 个文件变更',
      },
      patchParams: {
        conversationId: 't1',
        callId: 'c1',
        reason: null,
        grantRoot: null,
        fileChanges: {
          'src/a.rs': { type: 'update', unified_diff: '@@\n+line', move_path: null },
          'src/b.rs': { type: 'add', content: 'hello' },
        },
      },
    });
    expect(md).toContain('**修改** `src/a.rs`');
    expect(md).toContain('**新增** `src/b.rs`');
  });
});

/**
 * 审批摘要文档：写入工作区 .bcip/approvals/ 并在文档区展示
 */
import type { ApplyPatchApprovalParams } from '@/generated/app-server/ApplyPatchApprovalParams';
import type { FileChange } from '@/generated/app-server/FileChange';
import type { ApprovalRequest } from '@/types';

export const APPROVAL_DIR = '.bcip/approvals';
export const APPROVAL_FILE_PREFIX = 'approval-';

export function joinPath(base: string, ...parts: string[]): string {
  const normalized = [base, ...parts]
    .filter(Boolean)
    .join('/')
    .replace(/\\/g, '/')
    .replace(/\/+/g, '/');
  return normalized;
}

export function approvalDocumentPath(workspaceCwd: string, approvalId: string): string {
  const safeId = approvalId.replace(/[^a-zA-Z0-9_-]/g, '_');
  return joinPath(workspaceCwd, APPROVAL_DIR, `${APPROVAL_FILE_PREFIX}${safeId}.md`);
}

export function isApprovalDocumentPath(filePath: string): boolean {
  const normalized = filePath.replace(/\\/g, '/');
  return (
    normalized.includes(`/${APPROVAL_DIR}/`) &&
    normalized.includes(APPROVAL_FILE_PREFIX) &&
    normalized.endsWith('.md')
  );
}

export function approvalIdFromDocumentPath(filePath: string): string | null {
  const name = filePath.replace(/\\/g, '/').split('/').pop() ?? '';
  if (!name.startsWith(APPROVAL_FILE_PREFIX) || !name.endsWith('.md')) {
    return null;
  }
  return name.slice(APPROVAL_FILE_PREFIX.length, -3);
}

function summarizeFileChange(path: string, change: FileChange): string {
  switch (change.type) {
    case 'add':
      return `- **新增** \`${path}\`（${change.content.length} 字符）`;
    case 'delete':
      return `- **删除** \`${path}\``;
    case 'update': {
      const diffLines = change.unified_diff.split('\n').length;
      const moved =
        change.move_path && change.move_path !== path
          ? ` → \`${change.move_path}\``
          : '';
      return `- **修改** \`${path}\`${moved}（diff 约 ${diffLines} 行）`;
    }
    default:
      return `- \`${path}\``;
  }
}

function fileChangesSection(changes: Record<string, FileChange | undefined>): string {
  const entries = Object.entries(changes).filter(
    (entry): entry is [string, FileChange] => entry[1] !== undefined,
  );
  if (entries.length === 0) {
    return '_（无文件明细）_\n';
  }
  return `${entries.map(([path, change]) => summarizeFileChange(path, change)).join('\n')}\n`;
}

export interface ApprovalDocumentInput {
  approval: ApprovalRequest;
  patchParams?: ApplyPatchApprovalParams;
}

export function buildApprovalMarkdown(input: ApprovalDocumentInput): string {
  const { approval, patchParams } = input;
  const lines: string[] = [
    '---',
    'bcip_approval: true',
    `approval_id: ${approval.id}`,
    `approval_type: ${approval.type}`,
    'decision: pending',
    '---',
    '',
    `# ${approval.title}`,
    '',
    approval.description ? `> ${approval.description}` : '',
    '',
    '## 摘要',
    '',
    '| 项目 | 内容 |',
    '| --- | --- |',
    `| 类型 | ${approval.type === 'command' ? '命令' : approval.type === 'file' ? '文件变更' : 'MCP'} |`,
    `| 风险 | ${approval.riskLevel === 'high' ? '高' : approval.riskLevel === 'medium' ? '中' : '低'} |`,
  ];

  if (approval.isDangerous) {
    lines.push('| 警告 | 此操作具有破坏性 |');
  }

  lines.push('', '## 详情', '');

  if (approval.type === 'command') {
    lines.push('```bash', approval.command, '```', '');
    if (approval.cwd) {
      lines.push(`**工作目录：** \`${approval.cwd}\``, '');
    }
  } else if (approval.type === 'file') {
    if (patchParams?.fileChanges) {
      lines.push('### 变更文件', '', fileChangesSection(patchParams.fileChanges));
    } else {
      lines.push(approval.command, '');
    }
    if (approval.details ?? patchParams?.grantRoot) {
      lines.push(`**附加说明：** ${approval.details ?? patchParams?.grantRoot ?? ''}`, '');
    }
  }

  if (approval.details && approval.type === 'command') {
    lines.push(`**附加说明：** ${approval.details}`, '');
  }

  lines.push(
    '## 备注',
    '',
    '（可在此记录审批意见；点击下方按钮完成确认）',
    '',
    '## 操作',
    '',
    '- **拒绝**：不允许本次操作',
    '- **允许一次**：仅允许本次',
    '- **始终允许**：同类型操作不再询问',
    '',
  );

  return lines.filter((line, index, arr) => !(line === '' && arr[index - 1] === '')).join('\n');
}

export function approvalTabTitle(approval: ApprovalRequest): string {
  if (approval.type === 'file') {
    return '文件审批';
  }
  if (approval.isDangerous) {
    return '危险操作审批';
  }
  return '命令审批';
}

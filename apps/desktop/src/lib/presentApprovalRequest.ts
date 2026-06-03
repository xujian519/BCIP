/**
 * 将审批请求呈现到文档工作区（摘要 Markdown + 打开标签）
 */
import type { ApplyPatchApprovalParams } from '@/generated/app-server/ApplyPatchApprovalParams';
import type { Dispatch } from 'react';
import {
  approvalDocumentPath,
  approvalTabTitle,
  buildApprovalMarkdown,
  joinPath,
  APPROVAL_DIR,
} from '@/lib/approvalDocument';
import { createDirectory, writeFile } from '@/lib/fileSystem';
import type { AppAction, ApprovalRequest } from '@/types';

export interface PresentApprovalOptions {
  workspaceCwd: string | null;
  patchParams?: ApplyPatchApprovalParams;
}

export async function presentApprovalRequest(
  dispatch: Dispatch<AppAction>,
  approval: ApprovalRequest,
  options: PresentApprovalOptions,
): Promise<ApprovalRequest> {
  const cwd = options.workspaceCwd;
  if (!cwd) {
    dispatch({ type: 'SET_APPROVAL_DIALOG', payload: approval });
    return approval;
  }

  const documentPath = approvalDocumentPath(cwd, approval.id);
  const markdown = buildApprovalMarkdown({
    approval,
    patchParams: options.patchParams,
  });

  try {
    await createDirectory(joinPath(cwd, APPROVAL_DIR));
    await writeFile(documentPath, markdown);
  } catch (err) {
    console.warn('写入审批摘要失败，回退为仅状态展示', err);
    dispatch({ type: 'SET_APPROVAL_DIALOG', payload: approval });
    return approval;
  }

  const enriched: ApprovalRequest = { ...approval, documentPath };
  dispatch({ type: 'SET_APPROVAL_DIALOG', payload: enriched });
  dispatch({
    type: 'OPEN_TAB',
    payload: {
      id: `approval-${approval.id}`,
      filePath: documentPath,
      title: approvalTabTitle(approval),
    },
  });

  return enriched;
}

/**
 * 仅 DEV + VITE_DEV_MOCK：供 Playwright 走查截图触发浮层状态
 */
import { useEffect } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import { mockMessagesThread1, mockThreads } from '@/data/desktopMockMessages';
import {
  approvalDocumentPath,
  buildApprovalMarkdown,
  joinPath,
  APPROVAL_DIR,
} from '@/lib/approvalDocument';
import type { ApprovalRequest } from '@/types';

const mockApproval: ApprovalRequest = {
  id: 'e2e-approval',
  type: 'command',
  title: '允许执行命令？',
  description: 'npm run test',
  riskLevel: 'medium',
  command: 'npm test',
  isDangerous: false,
};

const E2E_WORKSPACE = '/tmp/bcip-e2e-workspace';

export function useE2eWalkthroughBridge(): void {
  const { dispatch } = useAppStore();

  useEffect(() => {
    if (import.meta.env.VITE_DEV_MOCK !== '1') {
      return;
    }

    const onApproval = async () => {
      dispatch({ type: 'SWITCH_PROJECT', payload: E2E_WORKSPACE });
      const documentPath = approvalDocumentPath(E2E_WORKSPACE, mockApproval.id);
      const markdown = buildApprovalMarkdown({ approval: mockApproval });
      try {
        const { createDirectory, writeFile } = await import('@/lib/fileSystem');
        await createDirectory(joinPath(E2E_WORKSPACE, APPROVAL_DIR));
        await writeFile(documentPath, markdown);
      } catch {
        // mock 环境无 Tauri 时仍展示文档区 UI（内容可能加载失败）
      }
      const enriched = { ...mockApproval, documentPath };
      dispatch({ type: 'SET_APPROVAL_DIALOG', payload: enriched });
      dispatch({
        type: 'OPEN_TAB',
        payload: {
          id: `approval-${mockApproval.id}`,
          filePath: documentPath,
          title: '命令审批',
        },
      });
    };
    const onRichThread = () => {
      dispatch({ type: 'SET_THREADS', payload: mockThreads });
      dispatch({ type: 'SET_CURRENT_THREAD', payload: 'thread-1' });
      dispatch({ type: 'SET_MESSAGES', payload: mockMessagesThread1 });
    };

    window.addEventListener('bcip-e2e-show-approval', () => void onApproval());
    window.addEventListener('bcip-e2e-rich-thread', onRichThread);
    return () => {
      window.removeEventListener('bcip-e2e-show-approval', () => void onApproval());
      window.removeEventListener('bcip-e2e-rich-thread', onRichThread);
    };
  }, [dispatch]);
}

import type { ReviewDecision } from '@/generated/app-server/ReviewDecision';
import type { CommandExecutionApprovalDecision } from '@/generated/app-server/v2/CommandExecutionApprovalDecision';
import type { FileChangeApprovalDecision } from '@/generated/app-server/v2/FileChangeApprovalDecision';
import type { ToolRequestUserInputResponse } from '@/generated/app-server/v2/ToolRequestUserInputResponse';
import {
  respondLegacyReviewDecision,
  respondPermissionsApproval,
  respondPermissionsDecline,
} from '@/lib/appServerServerRequest';
import { getAppServerClient } from '@/lib/appServerClient';
import type { ApprovalRequest } from '@/types';

export async function sendApprovalDecision(
  approval: ApprovalRequest,
  decision:
    | CommandExecutionApprovalDecision
    | FileChangeApprovalDecision
    | ReviewDecision,
): Promise<void> {
  const { rpcId, rpcMethod } = approval;
  if (rpcId === undefined) {
    return;
  }

  const client = getAppServerClient();

  switch (rpcMethod) {
    case 'item/commandExecution/requestApproval':
      await client.respond(rpcId, { decision });
      return;
    case 'item/fileChange/requestApproval':
      await client.respond(rpcId, { decision });
      return;
    case 'item/permissions/requestApproval': {
      if (!approval.permissionsParams) {
        throw new Error('缺少 permissions 参数');
      }
      if (decision === 'decline' || decision === 'cancel') {
        await respondPermissionsDecline(rpcId);
        return;
      }
      const scope =
        decision === 'acceptForSession' ? 'session' : 'turn';
      await respondPermissionsApproval(
        rpcId,
        approval.permissionsParams,
        scope,
      );
      return;
    }
    case 'execCommandApproval':
    case 'applyPatchApproval': {
      const review: ReviewDecision =
        decision === 'decline' || decision === 'cancel'
          ? 'denied'
          : decision === 'acceptForSession'
            ? 'approved_for_session'
            : 'approved';
      await respondLegacyReviewDecision(rpcId, review);
      return;
    }
    default:
      throw new Error(`不支持的审批类型: ${rpcMethod ?? 'unknown'}`);
  }
}

export async function sendToolUserInputResponse(
  rpcId: string | number,
  response: ToolRequestUserInputResponse,
): Promise<void> {
  await getAppServerClient().respond(rpcId, response);
}

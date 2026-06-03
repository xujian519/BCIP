/**
 * 处理 app-server 下发的 JSON-RPC 请求（审批、工具输入等）
 */
import type { ApplyPatchApprovalParams } from '@/generated/app-server/ApplyPatchApprovalParams';
import type { ApplyPatchApprovalResponse } from '@/generated/app-server/ApplyPatchApprovalResponse';
import type { ExecCommandApprovalParams } from '@/generated/app-server/ExecCommandApprovalParams';
import type { ExecCommandApprovalResponse } from '@/generated/app-server/ExecCommandApprovalResponse';
import type { ReviewDecision } from '@/generated/app-server/ReviewDecision';
import type { AttestationGenerateResponse } from '@/generated/app-server/v2/AttestationGenerateResponse';
import type { ChatgptAuthTokensRefreshResponse } from '@/generated/app-server/v2/ChatgptAuthTokensRefreshResponse';
import type { CommandExecutionRequestApprovalParams } from '@/generated/app-server/v2/CommandExecutionRequestApprovalParams';
import type { DynamicToolCallParams } from '@/generated/app-server/v2/DynamicToolCallParams';
import type { DynamicToolCallResponse } from '@/generated/app-server/v2/DynamicToolCallResponse';
import type { FileChangeRequestApprovalParams } from '@/generated/app-server/v2/FileChangeRequestApprovalParams';
import type { McpServerElicitationRequestParams } from '@/generated/app-server/v2/McpServerElicitationRequestParams';
import type { PermissionsRequestApprovalParams } from '@/generated/app-server/v2/PermissionsRequestApprovalParams';
import type { PermissionsRequestApprovalResponse } from '@/generated/app-server/v2/PermissionsRequestApprovalResponse';
import type { ToolRequestUserInputParams } from '@/generated/app-server/v2/ToolRequestUserInputParams';
import {
  approvalFromApplyPatch,
  approvalFromCommandExecution,
  approvalFromExecCommand,
  approvalFromFileChange,
  approvalFromPermissions,
} from '@/lib/approvalFromRpc';
import { getAppServerClient, type JsonRpcServerRequest } from '@/lib/appServerClient';
import { grantedPermissionProfileFromRequest } from '@/lib/grantedPermissions';
import { mcpElicitationFromRpc } from '@/lib/mcpElicitationFromRpc';
import { presentApprovalRequest } from '@/lib/presentApprovalRequest';
import type { Dispatch } from 'react';
import type { AppAction, ToolUserInputRequest } from '@/types';

export interface AppServerRequestContext {
  workspaceCwd: string | null;
}

export function handleAppServerRequest(
  request: JsonRpcServerRequest,
  dispatch: Dispatch<AppAction>,
  context: AppServerRequestContext,
): void {
  const { method, id: rpcId, params } = request;

  switch (method) {
    case 'item/commandExecution/requestApproval': {
      void presentApprovalRequest(
        dispatch,
        approvalFromCommandExecution(
          rpcId,
          params as CommandExecutionRequestApprovalParams,
        ),
        { workspaceCwd: context.workspaceCwd },
      );
      break;
    }
    case 'item/fileChange/requestApproval': {
      void presentApprovalRequest(
        dispatch,
        approvalFromFileChange(
          rpcId,
          params as FileChangeRequestApprovalParams,
        ),
        { workspaceCwd: context.workspaceCwd },
      );
      break;
    }
    case 'item/permissions/requestApproval': {
      void presentApprovalRequest(
        dispatch,
        approvalFromPermissions(
          rpcId,
          params as PermissionsRequestApprovalParams,
        ),
        { workspaceCwd: context.workspaceCwd },
      );
      break;
    }
    case 'execCommandApproval': {
      void presentApprovalRequest(
        dispatch,
        approvalFromExecCommand(
          rpcId,
          params as ExecCommandApprovalParams,
        ),
        { workspaceCwd: context.workspaceCwd },
      );
      break;
    }
    case 'applyPatchApproval': {
      const patchParams = params as ApplyPatchApprovalParams;
      void presentApprovalRequest(
        dispatch,
        approvalFromApplyPatch(rpcId, patchParams),
        { workspaceCwd: context.workspaceCwd, patchParams },
      );
      break;
    }
    case 'mcpServer/elicitation/request': {
      dispatch({
        type: 'SET_MCP_ELICITATION',
        payload: mcpElicitationFromRpc(
          rpcId,
          params as McpServerElicitationRequestParams,
        ),
      });
      break;
    }
    case 'item/tool/requestUserInput': {
      const p = params as ToolRequestUserInputParams;
      const payload: ToolUserInputRequest = {
        rpcId,
        threadId: p.threadId,
        turnId: p.turnId,
        itemId: p.itemId,
        questions: p.questions,
      };
      dispatch({ type: 'SET_TOOL_USER_INPUT', payload });
      break;
    }
    case 'item/tool/call': {
      void (async () => {
        const p = params as DynamicToolCallParams;
        const result: DynamicToolCallResponse = {
          success: false,
          contentItems: [],
        };
        await getAppServerClient().respond(rpcId, result);
        dispatch({
          type: 'ADD_MESSAGE',
          payload: {
            id: `sys-${Date.now()}`,
            role: 'system',
            content: `动态工具「${p.tool}」尚未在桌面端实现，已自动拒绝。`,
            timestamp: Date.now(),
            status: 'complete',
          },
        });
      })();
      break;
    }
    case 'attestation/generate': {
      void (async () => {
        const result: AttestationGenerateResponse = { token: '' };
        await getAppServerClient().respond(rpcId, result);
      })();
      break;
    }
    case 'account/chatgptAuthTokens/refresh': {
      void (async () => {
        const result: ChatgptAuthTokensRefreshResponse = {
          accessToken: '',
          chatgptAccountId: '',
          chatgptPlanType: null,
        };
        await getAppServerClient().respond(rpcId, result);
        dispatch({
          type: 'SET_OAUTH_WAITING',
          payload: {
            serverName: 'ChatGPT',
            phase: 'failed',
            error: '请在终端完成账号登录后重试',
          },
        });
      })();
      break;
    }
    default: {
      console.warn(`app-server 未处理的服务端请求: ${method}`);
      break;
    }
  }
}

export async function respondPermissionsApproval(
  rpcId: string | number,
  params: PermissionsRequestApprovalParams,
  scope: 'turn' | 'session',
): Promise<void> {
  const result: PermissionsRequestApprovalResponse = {
    permissions: grantedPermissionProfileFromRequest(params.permissions),
    scope,
  };
  await getAppServerClient().respond(rpcId, result);
}

export async function respondPermissionsDecline(
  rpcId: string | number,
): Promise<void> {
  const result: PermissionsRequestApprovalResponse = {
    permissions: {},
    scope: 'turn',
  };
  await getAppServerClient().respond(rpcId, result);
}

export async function respondLegacyReviewDecision(
  rpcId: string | number,
  decision: ReviewDecision,
): Promise<void> {
  const result: ExecCommandApprovalResponse | ApplyPatchApprovalResponse = {
    decision,
  };
  await getAppServerClient().respond(rpcId, result);
}

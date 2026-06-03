/**
 * 将 app-server 审批 RPC 转为 UI ApprovalRequest
 */
import type { ApplyPatchApprovalParams } from '@/generated/app-server/ApplyPatchApprovalParams';
import type { ExecCommandApprovalParams } from '@/generated/app-server/ExecCommandApprovalParams';
import type { CommandExecutionRequestApprovalParams } from '@/generated/app-server/v2/CommandExecutionRequestApprovalParams';
import type { FileChangeRequestApprovalParams } from '@/generated/app-server/v2/FileChangeRequestApprovalParams';
import type { PermissionsRequestApprovalParams } from '@/generated/app-server/v2/PermissionsRequestApprovalParams';
import type { ApprovalRequest } from '@/types';

const DANGEROUS_PATTERNS = [
  /\brm\s+-rf\b/i,
  /\bsudo\b/i,
  /\bmkfs\b/i,
  /\bdd\s+if=/i,
  />\s*\/dev\//i,
];

function isDangerousCommand(command: string): boolean {
  return DANGEROUS_PATTERNS.some((re) => re.test(command));
}

export function approvalFromCommandExecution(
  rpcId: string | number,
  params: CommandExecutionRequestApprovalParams,
): ApprovalRequest {
  const command = params.command ?? params.reason ?? '（命令待确认）';
  const cwd = params.cwd ?? undefined;

  return {
    id: params.itemId,
    type: 'command',
    title: '允许执行命令？',
    description: params.reason ?? 'Agent 请求执行 shell 命令',
    riskLevel: isDangerousCommand(command) ? 'high' : 'medium',
    command,
    cwd,
    isDangerous: isDangerousCommand(command),
    rpcId,
    rpcMethod: 'item/commandExecution/requestApproval',
  };
}

export function approvalFromFileChange(
  rpcId: string | number,
  params: FileChangeRequestApprovalParams,
): ApprovalRequest {
  const details = params.grantRoot
    ? `会话内写入权限：${params.grantRoot}`
    : params.reason ?? undefined;

  return {
    id: params.itemId,
    type: 'file',
    title: '允许修改文件？',
    description: params.reason ?? 'Agent 请求应用文件变更',
    riskLevel: 'medium',
    command: details ?? '文件变更',
    details,
    isDangerous: false,
    rpcId,
    rpcMethod: 'item/fileChange/requestApproval',
  };
}

export function approvalFromPermissions(
  rpcId: string | number,
  params: PermissionsRequestApprovalParams,
): ApprovalRequest {
  const parts: string[] = [];
  if (params.permissions.network?.enabled) {
    parts.push('网络访问');
  }
  if (params.permissions.fileSystem) {
    parts.push('文件系统扩展权限');
  }
  const summary = parts.length > 0 ? parts.join('、') : '额外沙箱权限';

  return {
    id: params.itemId,
    type: 'command',
    title: '允许额外权限？',
    description: params.reason ?? 'Agent 请求超出当前沙箱的权限',
    riskLevel: 'high',
    command: summary,
    cwd: params.cwd,
    isDangerous: true,
    rpcId,
    rpcMethod: 'item/permissions/requestApproval',
    permissionsParams: params,
  };
}

export function approvalFromExecCommand(
  rpcId: string | number,
  params: ExecCommandApprovalParams,
): ApprovalRequest {
  const command = params.command.join(' ');
  return {
    id: params.callId,
    type: 'command',
    title: '允许执行命令？',
    description: params.reason ?? '（旧版 execCommandApproval）',
    riskLevel: isDangerousCommand(command) ? 'high' : 'medium',
    command,
    cwd: params.cwd,
    isDangerous: isDangerousCommand(command),
    rpcId,
    rpcMethod: 'execCommandApproval',
  };
}

export function approvalFromApplyPatch(
  rpcId: string | number,
  params: ApplyPatchApprovalParams,
): ApprovalRequest {
  const count = Object.keys(params.fileChanges).length;
  return {
    id: params.callId,
    type: 'file',
    title: '允许应用补丁？',
    description: params.reason ?? '（旧版 applyPatchApproval）',
    riskLevel: 'medium',
    command: `${count} 个文件变更`,
    details: params.grantRoot ?? undefined,
    isDangerous: false,
    rpcId,
    rpcMethod: 'applyPatchApproval',
  };
}

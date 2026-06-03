import type { GrantedPermissionProfile } from '@/generated/app-server/v2/GrantedPermissionProfile';
import type { RequestPermissionProfile } from '@/generated/app-server/v2/RequestPermissionProfile';

/** 将请求的权限配置转为批准响应（与 TUI granted_permission_profile_from_request 一致） */
export function grantedPermissionProfileFromRequest(
  requested: RequestPermissionProfile,
): GrantedPermissionProfile {
  return {
    network: requested.network
      ? { enabled: requested.network.enabled }
      : undefined,
    fileSystem: requested.fileSystem ?? undefined,
  };
}

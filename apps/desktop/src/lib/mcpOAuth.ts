import type { McpServerOauthLoginResponse } from '@/generated/app-server/v2/McpServerOauthLoginResponse';
import { getAppServerClient } from '@/lib/appServerClient';
import type { OAuthWaitingState } from '@/types';

/** 发起 MCP OAuth 登录，返回等待态供 UI 展示 */
export async function startMcpOauthLogin(
  serverName: string,
): Promise<OAuthWaitingState> {
  const client = getAppServerClient();
  const res = await client.request<McpServerOauthLoginResponse>(
    'mcpServer/oauth/login',
    { name: serverName },
  );
  return {
    serverName,
    authUrl: res.authorizationUrl,
    phase: 'idle',
  };
}

export function openOAuthAuthorizationUrl(authUrl: string | undefined): void {
  if (!authUrl) {
    return;
  }
  window.open(authUrl, '_blank', 'noopener,noreferrer');
}

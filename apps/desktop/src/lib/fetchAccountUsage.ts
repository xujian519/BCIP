import type { GetAccountRateLimitsResponse } from '@/generated/app-server/v2/GetAccountRateLimitsResponse';
import { getAppServerClient } from '@/lib/appServerClient';
import {
  usageMeterFromRateLimits,
  type UsageMeterState,
} from '@/lib/usageMeter';

/** 读取账户额度（与 TUI 共用 app-server） */
export async function fetchAccountUsageMeter(): Promise<UsageMeterState | null> {
  const client = getAppServerClient();
  if (!client.isInitialized()) {
    return null;
  }
  try {
    const res = await client.request<GetAccountRateLimitsResponse>(
      'account/rateLimits/read',
      {},
    );
    return usageMeterFromRateLimits(res.rateLimits);
  } catch {
    return null;
  }
}

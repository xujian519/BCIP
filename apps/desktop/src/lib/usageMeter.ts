/**
 * 状态栏用量展示：account/rateLimits 与 thread/tokenUsage
 */
import type { RateLimitSnapshot } from '@/generated/app-server/v2/RateLimitSnapshot';
import type { ThreadTokenUsage } from '@/generated/app-server/v2/ThreadTokenUsage';

export interface UsageMeterState {
  label: string;
  used: number;
  max: number;
  /** 附加说明（如 credits 余额文案） */
  hint?: string;
}

export function usageMeterFromRateLimits(
  snapshot: RateLimitSnapshot,
): UsageMeterState | null {
  if (snapshot.primary && snapshot.primary.usedPercent >= 0) {
    return {
      label: '额度',
      used: snapshot.primary.usedPercent,
      max: 100,
      hint: snapshot.limitName ?? undefined,
    };
  }
  const credits = snapshot.credits;
  if (credits?.balance) {
    const balance = parseFloat(credits.balance);
    if (!Number.isNaN(balance)) {
      return {
        label: '余额',
        used: Math.min(balance, 100),
        max: 100,
        hint: credits.unlimited ? '不限' : `$${credits.balance}`,
      };
    }
  }
  return null;
}

export function usageMeterFromTokenUsage(
  usage: ThreadTokenUsage,
): UsageMeterState {
  const used = usage.total.totalTokens;
  const max = usage.modelContextWindow ?? Math.max(used, 128_000);
  return {
    label: 'tokens',
    used,
    max,
  };
}

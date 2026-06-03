import type { ThreadResumeResponse } from '@/generated/app-server/v2/ThreadResumeResponse';
import { getAppServerClient } from '@/lib/appServerClient';

/** 线程尚未写入 rollout（常见于 thread/start 后、首条 turn 前） */
export function isThreadNotMaterializedError(message: string): boolean {
  return (
    message.includes('no rollout found for thread id') ||
    message.includes('not materialized yet') ||
    message.includes('includeTurns is unavailable before first user message')
  );
}

/**
 * 尝试恢复线程。空线程 resume 会失败，此时返回 false，turn/start 仍可直接使用 threadId。
 */
export async function tryResumeThread(
  threadId: string,
  cwd: string | null,
): Promise<boolean> {
  const client = getAppServerClient();
  try {
    await client.request<ThreadResumeResponse>('thread/resume', {
      threadId,
      cwd,
    });
    return true;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    if (isThreadNotMaterializedError(message)) {
      return false;
    }
    throw err;
  }
}

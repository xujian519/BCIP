import type { Thread } from '@/generated/app-server/v2/Thread';
import type { Message } from '@/types';
import { threadItemToMessage } from '@/lib/threadItemMapper';

/** 从 thread/read 的 turns 重建时间线消息 */
export function messagesFromThreadHistory(thread: Thread): Message[] {
  const messages: Message[] = [];
  for (const turn of thread.turns) {
    const ts =
      (turn.completedAt ?? turn.startedAt ?? Math.floor(Date.now() / 1000)) *
      1000;
    for (const item of turn.items) {
      const mapped = threadItemToMessage(item, ts);
      if (mapped) {
        messages.push(mapped);
      }
    }
  }
  return messages;
}

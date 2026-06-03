/** 线程列表相对时间 —— 规范 §9.2.2（2h / 1d 格式） */
export function formatRelativeTime(timestamp: number): string {
  const now = Date.now();
  const diff = now - timestamp;

  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;
  const week = 7 * day;

  if (diff < minute) {
    return '刚刚';
  }
  if (diff < hour) {
    return `${Math.floor(diff / minute)}m`;
  }
  if (diff < day) {
    return `${Math.floor(diff / hour)}h`;
  }
  if (diff < week) {
    return `${Math.floor(diff / day)}d`;
  }
  return '更早';
}

/** 预览文本截断 —— 规范约 30 字符 */
export function truncateThreadPreview(text: string, maxLen = 30): string {
  const trimmed = text.trim();
  if (trimmed.length <= maxLen) {
    return trimmed;
  }
  return `${trimmed.slice(0, maxLen)}…`;
}

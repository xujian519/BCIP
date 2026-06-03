/** 运行平台探测（Tauri / 浏览器） */

export function isMacPlatform(): boolean {
  if (typeof navigator === 'undefined') {
    return true;
  }
  return /Mac|iPhone|iPad|iPod/i.test(navigator.platform);
}

export function isWindowsPlatform(): boolean {
  if (typeof navigator === 'undefined') {
    return false;
  }
  return /Win/i.test(navigator.platform);
}

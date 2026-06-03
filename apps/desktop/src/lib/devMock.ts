/** 显式开启浏览器演示 mock（`npm run dev` 默认 VITE_DEV_MOCK=1） */
export function isDevMockEnv(): boolean {
  return import.meta.env.VITE_DEV_MOCK === '1';
}

/**
 * 专利演示种子数据与 mock 对话（仅 VITE_DEV_MOCK=1）。
 * 生产 Tauri 构建不得设置该变量。
 */
export function isPatentMockDataEnabled(): boolean {
  return isDevMockEnv();
}

/**
 * 中心区专利 mock 视图（SearchView 等）：未连接 app-server 或显式 dev mock。
 */
export function shouldShowPatentMockViews(rpcReady: boolean): boolean {
  return !rpcReady || isDevMockEnv();
}

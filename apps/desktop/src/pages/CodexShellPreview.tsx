/**
 * 兼容旧书签 `#/preview/codex-shell`；与 MainApp 相同生产壳层。
 * 专利 mock 中心区需 `VITE_DEV_MOCK=1`（`npm run dev`）或未连接 app-server。
 */
export { default } from './MainApp';

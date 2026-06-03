# Codex 像素走查（C01–C12）

对照 [`docs/plans/2026-05-30-desktop-design-spec.md`](../../../docs/plans/2026-05-30-desktop-design-spec.md) §7。  
截图存档目录建议：`apps/desktop/docs/walkthrough-screenshots/`（按 `C01.png` … `C12.png` 命名）。

## 如何截取

| 环境 | 命令 | 说明 |
|------|------|------|
| **自动（推荐）** | `npm run walkthrough:capture` | Playwright 写入 `docs/walkthrough-screenshots/C01.png` … `C12.png` |
| 浏览器 mock | `cd apps/desktop && npm run dev` | 默认 `VITE_DEV_MOCK=1`；未连接时可见专利 mock 中心区 |
| 真机对话 | `npm run tauri:dev` | 连接 app-server 后走 C04–C05、C11–C12 |

打开主界面 `/` 或兼容路由 `#/preview/codex-shell`（与 MainApp 相同）。

## 走查表

| ID | 组件 | 实现路径 | 状态 | 截图要点 |
|----|------|----------|------|----------|
| C01 | ThreadListDrawer | `components/agent/ThreadListDrawer.tsx` | ✅ | 行高 `h-9`、预览 `truncate`、相对时间 `刚刚/m/h/d` |
| C02 | UserBubble | `components/agent/UserBubble.tsx` | ✅ | `max-w-[85%]`、`rounded-xl rounded-tr-sm` |
| C03 | AgentBlock | `components/agent/AgentBlock.tsx` | ✅ | 流式 `border-l-2` + cyan 左边框 |
| C04 | ToolCallCard | `components/agent/ToolCallCard.tsx` | ✅ | 展开/收起 + 状态图标 |
| C05 | ApprovalDialog | `components/overlays/ApprovalDialog.tsx` | ✅ | 拒绝 / 允许一次 / 始终允许 三按钮层级 |
| C06 | McpServersSettings | `components/settings/codex/pages/McpServersSettings.tsx` | ✅ | starting / ready / failed 色标 |
| C07 | SettingsNav | `components/settings/codex/SettingsNav.tsx` | ✅ | 与 Codex 分组顺序可 diff |
| C08 | ModelSettings | `components/settings/codex/pages/ModelSettings.tsx` | ✅ | `useModelCatalog` 加载占位，无写死品牌列表 |
| C09 | UsageStrip | `components/agent/AgentHeader.tsx` | ✅ | 顶栏展示 `usageMeter` / 传入 `usageText` |
| C10 | Composer | `components/agent/Composer.tsx` | ✅ | slash palette、附件、发送 |
| C11 | ReasoningBlock | `components/agent/ReasoningBlock.tsx` | ✅ | `defaultExpanded={false}` |
| C12 | AgentFooter | `components/agent/AgentFooter.tsx` | ✅ | 断开/失败时「重试」 |

## BCIP-only（不要求与 Codex 一致）

| ID | 组件 | 路径 |
|----|------|------|
| P01 | StageIndicator | `components/stage/StageIndicator.tsx` |
| P02 | TodoDock | `components/todo/TodoList.tsx` |
| P03 | PdfAnnotation | `components/preview/PdfPreview.tsx` |
| P04 | ProjectTree | `components/sidebar/LeftSidebar.tsx` |

## 专利 mock 门控

中心区专利 mock 视图（SearchView / CompareView 等）**仅**在以下情况通过 dev mock 数据展示：

- app-server **未连接**（`!isDesktopRpcReady`），或
- 构建时 **`VITE_DEV_MOCK=1`**（`npm run dev`）

主壳层中间区已统一为 `DocumentWorkspace`（多标签文档工作区）；`CenterPanel` / `AgentWorkPane` 已移除。

## DoD

- [x] 工程侧：`npm run walkthrough:capture` → `docs/walkthrough-screenshots/C01–C12.png`
- [ ] 设计侧：与 Codex 参考并排 diff 评审
- [x] 工程侧：上表组件已实现并对齐 spec 注释
- [x] CI：主 `.github/workflows/ci.yml` 路径门控 desktop 任务 + `scripts/check-generated-app-server.sh`

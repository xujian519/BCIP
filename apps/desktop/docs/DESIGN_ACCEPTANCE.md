# BCIP 桌面端设计验收指南

> 对应规范：[`docs/codex-desktop-pixel-perfect-design-spec.md`](../../../docs/codex-desktop-pixel-perfect-design-spec.md) **第 28 章**  
> 工程走查：[`CODEX_PIXEL_WALKTHROUGH.md`](./CODEX_PIXEL_WALKTHROUGH.md)  
> 验收清单数据：[`walkthrough-checklist.json`](./walkthrough-checklist.json)

本文档描述 **设计 / QA / 前端** 如何完成 C01–C12 像素走查闭环：自动截图 → 并排对比 → 勾选签收 → 归档 PR。

---

## 1. 角色与产出

| 角色 | 职责 | 产出 |
|------|------|------|
| **前端** | 跑截图脚本、修 regressions | `walkthrough-screenshots/C01–C12.png` |
| **设计** | 提供 Codex 参考图、并排评审 | `codex-ref/C01–C12.png`（可选入仓） |
| **QA / 设计** | 在 HTML 评审页勾选、导出签收 | `DESIGN_SIGNOFF.md` 或 PR 评论附件 |

---

## 2. 一键流程（推荐）

在 `apps/desktop` 目录：

```bash
# 1) 截取 BCIP 当前 UI（Playwright，1440×900，VITE_DEV_MOCK=1）
npm run walkthrough:capture

# 2) 校验 C01–C12 是否齐全
npm run walkthrough:check

# 3) 生成并排评审页 review.html
npm run walkthrough:report

# 或合并为一步：
npm run walkthrough:acceptance
```

在浏览器打开：

```bash
open docs/walkthrough-screenshots/review.html
```

（Linux/Windows 用系统默认浏览器打开该路径即可。）

---

## 3. Codex 参考图

1. 按 [`walkthrough-screenshots/codex-ref/README.md`](./walkthrough-screenshots/codex-ref/README.md) 命名放入 `codex-ref/`。
2. 重新运行 `npm run walkthrough:report`。
3. 评审页右侧显示 Codex 参考；缺失时显示虚线占位，仍可仅评 BCIP 与清单项。

可选 CI 严格模式（要求参考图齐全）：

```bash
node scripts/check-walkthrough-screenshots.mjs --strict-ref
```

---

## 4. 评审页功能

`review.html` 为**自包含静态页**（无服务器）：

- **并排图**：BCIP 当前 vs Codex 参考
- **验收子项**：来自 `walkthrough-checklist.json`，可勾选
- **设计结论**：通过 / 轻微偏差 / 需修复 / 不适用
- **备注**：间距、色值、动画等说明
- **导出 Markdown**：复制签收单到剪贴板，粘贴至 PR 或 `docs/walkthrough-screenshots/DESIGN_SIGNOFF.md`
- **本地进度**：`localStorage` 键 `bcip-walkthrough-review`

---

## 5. 验收标准摘要（C01–C12）

完整条目见 [`walkthrough-checklist.json`](./walkthrough-checklist.json) 与规范 §28.1。

| ID | 组件 | 关键对齐点 |
|----|------|------------|
| C01 | ThreadListDrawer | 行高 32–36px、truncate、相对时间、选中竖线 |
| C02 | UserBubble | max-width 85%、非对称圆角、右对齐 |
| C03 | AgentBlock | border-l-2、流式 cyan 边框 |
| C04 | ToolCallCard | 展开 250ms、Chevron、状态图标 |
| C05 | ApprovalDialog | 三按钮层级、520px、命令预览等宽 |
| C06 | McpServersSettings | starting/ready/failed 色标 |
| C07 | SettingsNav | 宽 200–220px、13px 文案、选中态 |
| C08 | ModelSettings | Pill 选择器、动态 model list、Skeleton |
| C09 | UsageStrip | Header 右侧、~0.xx/1.0 格式 |
| C10 | Composer | slash palette、min/max 高度、发送 disabled |
| C11 | ReasoningBlock | 默认折叠、Thinking 摘要 |
| C12 | AgentFooter | 断线重试、连接状态文案 |

**BCIP-only（P01–P04）** 不要求与 Codex 一致，见评审页底部列表。

---

## 6. 签收与 DoD

### 6.1 通过条件

- [ ] C01–C12 BCIP 截图已更新（与当前 PR 代码一致）
- [ ] 12 项均有设计结论（允许「轻微偏差」但需备注）
- [ ] **需修复** 项已开 issue 或同 PR 修复
- [ ] 签收 Markdown 已附 PR 或提交至 `docs/walkthrough-screenshots/DESIGN_SIGNOFF.md`

### 6.2 签收单模板

复制 [`DESIGN_SIGNOFF.template.md`](./DESIGN_SIGNOFF.template.md)，或由评审页 **导出 Markdown** 自动生成。

### 6.3 与工程 DoD 的关系

[`CODEX_PIXEL_WALKTHROUGH.md`](./CODEX_PIXEL_WALKTHROUGH.md) DoD 中「设计侧并排 diff」由本流程满足。

---

## 7. 真机 / 主题变体

| 场景 | 命令 | 说明 |
|------|------|------|
| Mock 走查（默认） | `npm run walkthrough:capture` | 无需 app-server，覆盖 C01–C12 |
| 真机对话 | `npm run tauri:dev` | C04–C05、C11–C12 可手动画补充 |
| 浅色主题 | `npm run dev` 切换 Appearance 后重跑 capture | 签收单注明 theme=light |

---

## 8. 故障排查

| 现象 | 处理 |
|------|------|
| `walkthrough:capture` 失败 | 确认 Chromium：`npm run test:e2e:install` |
| 某张图为空白/过小 | 检查 `e2e/walkthrough-capture.spec.ts` 选择器；mock 事件 `bcip-e2e-rich-thread` |
| review.html 图片裂图 | 须在 `walkthrough-screenshots/` 目录下打开 HTML，或先 capture |
| Codex 参考缺失 | 仅评 BCIP + 清单；或从设计稿导出 PNG |

---

## 9. 相关脚本

| npm script | 脚本文件 |
|------------|----------|
| `walkthrough:capture` | `e2e/walkthrough-capture.spec.ts` |
| `walkthrough:check` | `scripts/check-walkthrough-screenshots.mjs` |
| `walkthrough:report` | `scripts/generate-walkthrough-review.mjs` |
| `walkthrough:acceptance` | 上述三步串联 |

---

## 10. 变更记录

| 日期 | 说明 |
|------|------|
| 2026-06-03 | 初版：清单 JSON、HTML 评审页、验收文档与 npm scripts |

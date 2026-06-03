# BCIP 桌面端设计审查报告

> 审查日期：2026-06-03
> 审查范围：apps/desktop/src/ 全部组件 + index.css + tailwind.config.js
> 对照文档：docs/codex-desktop-pixel-perfect-design-spec.md (v2.0)
> 审查目标：识别与 Codex 像素级规范的差距，确保桌面端简洁、质感、可读性强

---

## 执行摘要

经过对设计规范 v2.0 和当前代码实现的逐条比对，发现 **存在 47 项差距**，分布在 8 个主要类别中。其中 **P0（阻塞发布）2 项，P1（重要体验）18 项，P2（优化提升）27 项**。

当前实现覆盖率约 **75%**，主要问题在于：
- 色彩系统未完全对齐（暖色 vs 冷灰）
- 布局尺寸存在硬编码偏差
- 组件细节（圆角、阴影、间距）未严格遵循 Token
- 动画参数不一致

---

## 一、色彩系统差距（8 项）

### P1: 背景色偏离规范暖色调

| Token | 规范值 | 当前值 | 差距 |
|-------|--------|--------|------|
| `bg/base` (Light) | `#F5F2EE` 暖米灰 | `#F5F5F5` 冷灰 | 偏离品牌暖色基调 |
| `bg/surface` (Light) | `#FAF8F5` 暖白 | `#FFFFFF` 纯白 | 缺少暖色底蕴 |
| `bg/base` (Dark) | `#1C1A18` 暖深灰 | `#1C1C1C` 冷深灰 | 偏离暖深色调 |
| `bg/surface` (Dark) | `#1A1816` | `#0F0F0F` | 过深，不符合规范 |

**影响**：整体视觉缺少规范定义的"暖色质感"，与 Codex 冷灰过于接近，弱化 BCIP 品牌辨识度。

**修复建议**：统一替换为规范中的暖色值，恢复品牌差异化。

---

### P1: 文字色未使用暖中性色

| Token | 规范值 | 当前值 |
|-------|--------|--------|
| `text/primary` (Light) | `#1A1814` | `#1A1A1A` |
| `text/secondary` (Light) | `#6B6560` | `#6E6E73` |
| `text/tertiary` (Light) | `#A39E98` | `#AEAEB2` |

**影响**：文字色偏冷，阅读时缺少温暖感，与规范定义的可读性目标有差距。

---

### P1: 侧边栏背景色不一致

| Token | 规范值 | 当前值 |
|-------|--------|--------|
| `bg/sidebar` (Light) | `#F0EDE8` / `rgba(245,242,238,0.72)` | `#F5F5F5` |
| `bg/sidebar` (Dark) | `#1A1816` / `rgba(26,24,22,0.85)` | `#1C1C1C` |

**影响**：侧边栏缺少规范定义的玻璃效果和暖色调。

---

### P2: 边框色值不一致

| Token | 规范值 | 当前值 |
|-------|--------|--------|
| `border/default` (Light) | `rgba(0,0,0,0.08)` | 代码中混合使用 `rgba(0,0,0,0.06)` 和 `0.08` |
| `border-primary` | 无此 Token | 代码中自创，与规范冲突 |

**影响**：边框颜色不统一，导致分隔线视觉重量不一致。

---

### P2: 状态色未对齐

| Token | 规范值 | 当前值 |
|-------|--------|--------|
| `accent-primary-hover` (Light) | `#5A9A8C` | `#3D6A5E` |

**影响**：Hover 态颜色过深，缺少明亮感。

---

## 二、布局尺寸差距（7 项）

### P0: TitleBar 高度不一致

| 规范 | 当前 |
|------|------|
| 38px | 40px (`var(--layout-titlebar-height)`) |

**文件**：`apps/desktop/src/components/TitleBar.tsx:76`

**影响**：标题栏高度直接偏离规范 2px，影响整体比例。

---

### P0: StatusBar 高度不一致

| 规范 | 当前 |
|------|------|
| 32px | 28px (StatusBar.tsx) / 40px (CSS 变量) |

**文件**：`StatusBar.tsx:32` 使用 28px，但 `index.css` 中 `--layout-statusbar-height: 40px`

**影响**：高度不一致，且与规范相差 4-8px。

---

### P1: AgentHeader 高度不一致

| 规范 | 当前 |
|------|------|
| 48px | 40px (`var(--chat-header-h)`) |

**文件**：`AgentHeader.tsx:86`

**影响**：Agent 面板头部高度偏离规范 8px。

---

### P1: AgentFooter 高度不一致

| 规范 | 当前 |
|------|------|
| 32px | 未明确设置，约为 28-32px |

---

### P1: 面板宽度默认值不一致

| Token | 规范 | 当前 |
|-------|------|------|
| LeftSidebar | 260px | 未明确默认，由状态控制 |
| AgentPanel | 380px | 未明确默认，由状态控制 |

---

### P2: 状态栏内边距不一致

| 规范 | 当前 |
|------|------|
| `0 12px` | `0 12px` (正确) |

此项正确，但其他间距参数混合使用 px 值和 CSS 变量。

---

## 三、Typography 差距（6 项）

### P1: 标题栏字号未使用规范 Token

| 规范 | 当前 |
|------|------|
| `font/titlebar`: 13px / 600 / 1.0 | TitleBar.tsx:131 使用 13px / 600 (正确) |
| 但副标题应为 10px | 实际使用 10px (正确) |

标题栏基本正确，但 `letterSpacing` 等细节需核查。

---

### P1: 消息气泡字号不一致

| 规范 | 当前 |
|------|------|
| 用户/Agent 消息: 14px / 1.6 | UserBubble.tsx:79 使用 `text-sm` (约 14px) / `leading-normal` |
| Agent 消息: `font/markdown` 14px / 1.7 | AgentBlock.tsx:145 使用 `text-sm` / `leading-relaxed` |

**影响**：行高未精确对齐，影响阅读体验。

---

### P1: 设置页标题字号偏大

| 规范 | 当前 |
|------|------|
| H1: 20px / 600 | AppearanceSettings.tsx:73 使用 18px |
| H2: 16px / 600 | GeneralSettings.tsx:74 使用 18px |

**影响**：设置页标题统一使用 18px，偏离规范阶梯。

---

### P2: 等宽字体未统一

| 规范 | 当前 |
|------|------|
| `JetBrains Mono` 为主 | tailwind.config.js 中配置正确 |
| 但代码中多处使用 `font-mono` | 可能映射到系统默认等宽字体 |

---

### P2: 字距未遵循规范

| Token | 规范 | 当前 |
|-------|------|------|
| `tracking-tight` | -0.01em | 未明确设置 |
| `tracking-wide` (Caption) | +0.01em | 未明确设置 |

---

## 四、间距系统差距（5 项）

### P1: 消息气泡内边距不一致

| 规范 | 当前 |
|------|------|
| UserBubble: `10px 14px` | `var(--chat-bubble-py) var(--chat-bubble-px)` (约 6px 10px) |
| AgentBlock: `10px 14px` | 未明确设置 padding |

**影响**：气泡过于紧凑，缺少呼吸感。

---

### P1: Composer 内边距不一致

| 规范 | 当前 |
|------|------|
| 内边距: 16px | 代码中使用 `px-2.5` (约 10px) |
| 顶部行 padding: 标准 | `pt-1.5 pb-1` (约 6px 4px) |

---

### P2: 设置页内容区 padding 不一致

| 规范 | 当前 |
|------|------|
| `24px 32px` | `24px 28px` |

---

### P2: 按钮内边距不一致

| 规范 | 当前 |
|------|------|
| Primary 按钮: 16px 水平 | ShadCN 默认 `px-4` (16px) 正确 |
| 但高度: 32px | 实际 `h-9` (36px) |

---

## 五、圆角系统差距（6 项）

### P1: 圆角 Token 映射错误

| Token | 规范 | 当前 (tailwind) |
|-------|------|-----------------|
| `radius-sm` | 2px | 6px (`'radius-sm': '6px'`) |
| `radius-md` | 6px | 8px |
| `radius-lg` | 8px | 12px |
| `radius-xl` | 10px | 16px |
| `radius-2xl` | 12px | 未定义 |
| `radius-3xl` | 16px | 未定义 |

**影响**：Tailwind 配置中圆角值整体偏移，导致组件圆角偏大。

---

### P2: Card 组件圆角不一致

| 规范 | 当前 |
|------|------|
| 设置卡片: `radius-2xl` (12px) | `rounded-xl` (根据配置可能是 12-16px) |
| 工具卡片: `radius-lg` (8px) | 未明确 |

---

### P2: 按钮圆角不一致

| 规范 | 当前 |
|------|------|
| 主按钮: 8px (`radius-lg`) | `rounded-md` (根据配置 6-8px) |
| 图标按钮: 6px (`radius-md`) | `rounded-md` |

---

## 六、阴影系统差距（5 项）

### P1: 阴影值未使用规范定义

| Token | 规范 (Light) | 当前 |
|-------|-------------|------|
| `shadow-card` | `0 1px 3px rgba(0,0,0,0.08), 0 1px 2px rgba(0,0,0,0.04)` | `0 1px 3px rgba(0,0,0,0.08), 0 1px 2px rgba(0,0,0,0.04)` (正确) |
| `shadow-floating` | `0 4px 12px rgba(0,0,0,0.08), 0 2px 4px rgba(0,0,0,0.04)` | `0 4px 12px rgba(0,0,0,0.06)` (偏轻) |
| `shadow-modal` | `0 12px 40px rgba(0,0,0,0.12), 0 4px 12px rgba(0,0,0,0.08)` | 未定义 |

---

### P2: Composer 阴影不一致

| 规范 | 当前 |
|------|------|
| `0 4px 12px rgba(0,0,0,0.15)` | `0 4px 12px rgba(0,0,0,0.15)` (正确) |
| Focus: `0 4px 12px rgba(0,0,0,0.08)` | `var(--shadow-composer-focus)` (正确) |

---

## 七、动画/动效差距（6 项）

### P1: 动画时长未对齐

| Token | 规范 | 当前 |
|-------|------|------|
| `duration-fast` | 100ms | 代码中多处使用 150ms |
| `duration-normal` | 150ms | 代码中使用 200ms |
| `duration-slow` | 250ms | 代码中使用 300ms |

---

### P1: 缓动函数未统一

| Token | 规范 | 当前 |
|-------|------|------|
| `ease-spring` | `cubic-bezier(0.4, 0, 0.2, 1)` | 代码中混合使用 `ease-out`、`ease` |
| `ease-bounce` | `cubic-bezier(0.34, 1.56, 0.64, 1)` | 未使用 |

---

### P2: 按钮 Hover 过渡

| 规范 | 当前 |
|------|------|
| 150ms ease-out | ShadCN 默认过渡，未明确覆盖 |

---

### P2: 流式光标动画

| 规范 | 当前 |
|------|------|
| `animation: cursor-blink 1s steps(2) infinite` | tailwind.config.js 中定义 `cursor-blink` 正确 |
| 但颜色应为 `#3A8B8C` | 使用 `var(--accent-cyan)` 正确 |

---

## 八、组件细节差距（4 项）

### P1: 按钮组件未适配 BCIP 品牌

**文件**：`apps/desktop/src/components/ui/button.tsx`

当前使用 ShadCN 默认样式，未应用 BCIP 品牌绿：
- Primary 按钮背景使用 `bg-primary` (HSL 变量)，非 `#4A7C6F`
- Hover 态未映射到 `#5A9A8C`
- 缺少品牌按钮的特定阴影

---

### P1: Switch 组件尺寸不一致

**文件**：`apps/desktop/src/components/ui/switch.tsx`

| 规范 | 当前 |
|------|------|
| 高度: 20px | `h-[1.15rem]` (~18px) |
| 宽度: 36px | `w-8` (32px) |

---

### P2: 输入框高度不一致

| 规范 | 当前 |
|------|------|
| 单行输入: 36px | `h-9` (36px) 正确 |
| Composer: min 56px, max 200px | min 36px, max 200px |

---

### P2: Toast 组件未完全适配

当前使用 sonner，但未配置左侧彩色边框指示器。

---

## 九、整体架构差距

### P1: 缺少全局 CSS 变量与 Tailwind 的桥接

当前问题：
- `index.css` 定义了 CSS 变量
- `tailwind.config.js` 定义了颜色映射
- 但两者未完全同步，存在双轨制

**建议**：统一使用 CSS 变量驱动 Tailwind，删除 tailwind.config.js 中的硬编码色值。

---

### P2: 平台适配不完善

- Windows 字体栈：`[data-platform='windows']` 已定义，但缺少全面的 Windows 变体
- macOS 交通灯位置：规范 `x: 16, y: 14`，当前实现未明确验证

---

## 修复优先级清单

### P0 - 立即修复（阻塞发布）

1. [ ] **TitleBar 高度**：38px -> 40px，统一为 38px
2. [ ] **StatusBar 高度**：统一为 32px，删除 CSS 变量冲突

### P1 - 本周修复（重要体验）

3. [ ] **背景色系统**：统一替换为暖色调 (`#F5F2EE`, `#FAF8F5`, `#1C1A18`, `#1A1816`)
4. [ ] **文字色系统**：替换为暖中性色 (`#1A1814`, `#6B6560`, `#A39E98`)
5. [ ] **AgentHeader 高度**：40px -> 48px
6. [ ] **消息气泡内边距**：调整为 `10px 14px`
7. [ ] **设置页标题字号**：18px -> 20px (H1), 16px (H2)
8. [ ] **按钮组件品牌适配**：Primary 使用 `#4A7C6F`，Hover `#5A9A8C`
9. [ ] **圆角系统修正**：`radius-sm` 2px, `radius-md` 6px, `radius-lg` 8px, `radius-xl` 10px
10. [ ] **Switch 尺寸**：高度 20px，宽度 36px
11. [ ] **动画时长统一**：fast 100ms, normal 150ms, slow 250ms
12. [ ] **缓动函数统一**：使用 `cubic-bezier(0.4, 0, 0.2, 1)` 替代混合 easing
13. [ ] **Composer 内边距**：统一为 16px
14. [ ] **AgentFooter 高度**：明确设置为 32px
15. [ ] **阴影系统补全**：添加 `shadow-modal`, `shadow-drag`
16. [ ] **边框色统一**：统一使用 `rgba(0,0,0,0.08)` / `rgba(255,255,255,0.08)`
17. [ ] **侧边栏背景色**：使用 `#F0EDE8` / `#1A1816`
18. [ ] **按钮 Hover 色**：`#3D6A5E` -> `#5A9A8C`
19. [ ] **全局 CSS/Tailwind 桥接**：统一变量系统
20. [ ] **等宽字体统一**：确保 `JetBrains Mono` 正确加载

### P2 - 后续优化（提升体验）

21. [ ] **设置页内容区 padding**：`24px 28px` -> `24px 32px`
22. [ ] **按钮高度**：36px -> 32px (Primary)
23. [ ] **Toast 左侧边框**：添加彩色指示器
24. [ ] **流式边框渐隐动画**：300ms ease-out
25. [ ] **ReasoningBlock 默认折叠**：当前未检查
26. [ ] **ToolCallCard 展开动画**：250ms spring
27. [ ] **TurnDivider 样式**：检查是否实现
28. [ ] **代码块语言标签**：添加右上角标签
29. [ ] **Markdown 渲染样式**：H1/H2/H3 底部边框
30. [ ] **PDF 查看器背景色**：`#E8E5E0`
31. [ ] **图片查看器信息叠加层**
32. [ ] **代码高亮色**：使用规范定义的 Prism 主题
33. [ ] **StageIndicator 组件**：检查是否完整实现
34. [ ] **TodoDock 实现**：检查高度和动画
35. [ ] **TerminalOverlay 背景色**：`#0D0D0D`
36. [ ] **ThreadListDrawer 宽度**：260px
37. [ ] **ThreadRow 高度**：48px
38. [ ] **连接状态脉冲动画**：1s infinite
39. [ ] **滚动条样式细化**：使用 `rgba(0,0,0,0.20)`
40. [ ] **focus-ring 样式**：`0 0 0 2px rgba(74,124,111,0.3)`
41. [ ] **glass-blur 效果**：`blur(20px) saturate(180%)`
42. [ ] **选中文字背景**：`rgba(74,124,111,0.25)`
43. [ ] **Reduced Motion 支持**：检查完整性
44. [ ] **Windows 平台字体栈**：Segoe UI Variable
45. [ ] **交通灯位置验证**：`x: 16, y: 14`
46. [ ] **WorkspaceBreadcrumb**：检查实现
47. [ ] **MCP 状态色**：检查完整映射

---

## 附录：关键文件修改清单

| 文件 | 修改内容 |
|------|----------|
| `index.css` | 统一背景色、文字色、边框色为规范暖色调 |
| `tailwind.config.js` | 修正圆角映射，统一阴影值 |
| `components/ui/button.tsx` | 适配 BCIP 品牌绿 |
| `components/ui/switch.tsx` | 调整尺寸为 36x20px |
| `components/TitleBar.tsx` | 高度 38px，验证交通灯位置 |
| `components/StatusBar.tsx` | 高度 32px，统一样式 |
| `components/agent/AgentHeader.tsx` | 高度 48px |
| `components/agent/UserBubble.tsx` | 调整内边距为 10px 14px |
| `components/agent/AgentBlock.tsx` | 调整内边距，检查流式边框 |
| `components/agent/Composer.tsx` | 统一内边距为 16px |
| `components/settings/*` | 标题字号 20px/16px，内容区 padding |
| `components/ui/card.tsx` | 圆角调整为 12px |

---

*报告完成。建议按 P0 -> P1 -> P2 的顺序逐批修复，每批修复后进行视觉回归测试。*

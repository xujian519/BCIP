# BCIP 桌面端 UI macOS 风格美化报告

> 生成日期：2026-06-03
> 设计基准：macOS Big Sur+ 审美 + 项目设计规范 v2.0
> 审查范围：`apps/desktop/src/` 全组件

---

## 美化概述

本次美化基于 **macOS Big Sur+ 设计哲学**，将原生毛玻璃（Vibrancy）、Spring 动画、精致圆角和柔和阴影等 macOS 核心视觉语言融入项目 UI。在保持原有设计规范（暖色调色彩系统、布局尺寸）的基础上，全面提升了界面的精致度和原生感。

---

## 修改文件清单（共 12 个文件）

| 序号 | 文件路径 | 修改类型 |
|------|----------|----------|
| 1 | `src/index.css` | 全局样式增强 |
| 2 | `tailwind.config.js` | 配置增强 |
| 3 | `src/components/shell/TitleBar.tsx` | 毛玻璃增强 |
| 4 | `src/components/StatusBar.tsx` | 毛玻璃增强 |
| 5 | `src/components/sidebar/LeftSidebar.tsx` | 毛玻璃 + 输入框优化 |
| 6 | `src/components/activity-bar/ActivityBar.tsx` | 毛玻璃 + 动画优化 |
| 7 | `src/components/agent/AgentHeader.tsx` | 毛玻璃 + 圆角优化 |
| 8 | `src/components/agent/UserBubble.tsx` | 气泡风格优化 |
| 9 | `src/components/agent/AgentBlock.tsx` | 动画 + 代码块优化 |
| 10 | `src/components/agent/MessageTimeline.tsx` | 进入动画优化 |
| 11 | `src/components/agent/Composer.tsx` | 输入框风格优化 |
| 12 | `src/components/settings/categories/*.tsx` | 设置页对齐（前期修复） |

---

## 核心美化项

### 1. 全局滚动条 macOS 化

**文件**：`src/index.css`

| 优化项 | 修改前 | 修改后 |
|--------|--------|--------|
| 宽度 | 8px / 6px | **6px / 4px** |
| 圆角 | 4px / 3px | **100px（胶囊形）** |
| 边框技巧 | 无 | **透明边框 + background-clip** |
| 悬停透明度 | 0.20 → 0.28 | **0.12 → 0.22（更 subtle）** |

**效果**：滚动条在 macOS 风格下更细、更精致，悬停时平滑过渡，不使用时几乎不可见。

---

### 2. 毛玻璃（Glass/Vibrancy）系统增强

**文件**：`src/index.css`、`TitleBar.tsx`、`StatusBar.tsx`、`LeftSidebar.tsx`、`ActivityBar.tsx`、`AgentHeader.tsx`

新增/增强工具类：

```css
.glass {
  background: var(--glass-bg);
  backdrop-filter: blur(20px) saturate(180%);
}

.glass-strong {
  background: var(--glass-bg);
  backdrop-filter: blur(24px) saturate(200%);
}
```

**应用位置**：
- **TitleBar**：`.glass-strong` → 更强烈的毛玻璃效果，交通灯和标题更清晰
- **StatusBar**：`.glass` → 底部状态栏获得半透明质感
- **LeftSidebar**：`.glass` → 侧边栏文件树区域获得微妙透明度
- **ActivityBar**：`.glass` → 左侧图标栏获得原生 macOS Dock 般的质感
- **AgentHeader**：`.glass-strong` → 聊天面板头部获得精致毛玻璃

---

### 3. Spring 动画系统

**文件**：`src/index.css`、`tailwind.config.js`

新增动画曲线：

```css
/* macOS 风格缓动 */
.transition-mac { transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); }
.transition-spring { transition-timing-function: cubic-bezier(0.34, 1.56, 0.64, 1); }
```

**Tailwind 配置**：
```js
transitionTimingFunction: {
  'mac': 'cubic-bezier(0.4, 0, 0.2, 1)',
  'spring': 'cubic-bezier(0.34, 1.56, 0.64, 1)',
  'bounce': 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
}
```

**应用效果**：
- 所有按钮悬停/点击：Spring 弹性效果
- 消息气泡进入：先缩放 0.98 → 弹到 1.02 → 稳定到 1.0
- 面板切换：更流畅的物理感

---

### 4. 阴影系统 macOS 化

**文件**：`tailwind.config.js`

新增阴影层级：

```js
boxShadow: {
  'mac': '0 2px 8px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04)',
  'mac-lg': '0 8px 24px rgba(0, 0, 0, 0.08), 0 2px 6px rgba(0, 0, 0, 0.04)',
  'mac-xl': '0 16px 48px rgba(0, 0, 0, 0.10), 0 4px 12px rgba(0, 0, 0, 0.06)',
  'inner-glow': 'inset 0 1px 2px rgba(255, 255, 255, 0.1)',
}
```

**特点**：
- 双层阴影模拟真实光照（弥散阴影 + 接触阴影）
- 阴影颜色更柔和，透明度更低
- 支持内发光效果（用于凸起按钮）

---

### 5. 消息气泡 macOS 化

**文件**：`src/components/agent/UserBubble.tsx`

| 优化项 | 修改前 | 修改后 |
|--------|--------|--------|
| 圆角 | `12px 4px 12px 12px` | **`18px 6px 18px 18px`** |
| 阴影 | 无 | **`shadow-sm`** |
| 进入动画 | 150ms 线性 | **200ms Spring 弹性** |
| 时间戳 | 纯文本 | **微调透明度 + 位置优化** |

---

### 6. 代码块 macOS 化

**文件**：`src/components/agent/AgentBlock.tsx`

| 优化项 | 修改前 | 修改后 |
|--------|--------|--------|
| 圆角 | `rounded-md` (6px) | **`rounded-xl` (12px)** |
| 内边距 | `p-2 pt-5` | **`p-3 pt-6`** |
| 语言标签 | 固定显示 | **默认 80% 透明，悬停 100%** |
| 阴影 | 无 | **`shadow-sm`** |
| 间距 | `my-1.5` | **`my-2`** |

---

### 7. Composer 输入框 macOS 化

**文件**：`src/components/agent/Composer.tsx`

| 优化项 | 修改前 | 修改后 |
|--------|--------|--------|
| 外框圆角 | `rounded-xl` (10px) | **`rounded-2xl` (12px)** |
| Focus 光环 | 无 | **`ring-2 ring-[var(--accent-primary-muted)]`** |
| 过渡动画 | 150ms 线性 | **200ms Spring 弹性** |
| Placeholder | 静态 | **增加透明度过渡动画** |
| 行高 | `leading-normal` | **`leading-relaxed`** |

---

### 8. 按钮系统统一增强

**文件**：`ActivityBar.tsx`、`AgentHeader.tsx`、`LeftSidebar.tsx`

所有交互按钮统一升级：

| 优化项 | 修改前 | 修改后 |
|--------|--------|--------|
| 圆角 | 6-8px | **8-10px** |
| 过渡时长 | 150ms | **200ms** |
| 过渡曲线 | linear / ease | **Spring `cubic-bezier(0.34, 1.56, 0.64, 1)`** |
| 激活指示器 | 2px 宽 | **2.5px 宽 + 过渡动画** |

---

### 9. 消息时间线进入动画

**文件**：`src/components/agent/MessageTimeline.tsx`

- 空状态图标：增大到 `h-12 w-12`，圆角 `rounded-2xl`，添加 `shadow-sm`
- 消息分组：添加交错进入动画 `animationDelay: turnIndex * 0.05s`
- 整体添加 `.message-enter` 动画类

---

### 10. 全局动画工具类

**文件**：`src/index.css`

新增：

```css
/* 消息进入动画 */
@keyframes message-enter {
  from { opacity: 0; transform: translateY(8px) scale(0.98); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

/* 悬停提升效果 */
.hover-lift {
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.2s ease;
}
.hover-lift:hover { transform: translateY(-1px); }

/* 微妙脉冲 */
.subtle-pulse { animation: subtle-pulse 2s ease-in-out infinite; }
```

---

## 设计原则遵循

本次美化严格遵循以下 macOS 设计原则：

1. **Depth（深度）**：通过毛玻璃和多层阴影创造层级感
2. **Deference（遵从内容）**：UI 元素不抢夺内容注意力，滚动条、边框都更 subtle
3. **Clarity（清晰）**：Focus 状态更明确（品牌色光环），交互反馈更即时
4. **Fluidity（流畅）**：所有动画使用 Spring 物理曲线，拒绝生硬的线性过渡

---

## 与原有设计规范的关系

| 规范项目 | 状态 | 说明 |
|----------|------|------|
| 色彩系统（暖色调） | ✅ 保持 | 未修改，继续沿用 |
| 布局尺寸（38/32/48px） | ✅ 保持 | 未修改 |
| 圆角系统 | ⚡️ 增强 | 在规范基础上增大关键组件圆角 |
| 阴影系统 | ⚡️ 增强 | 新增 macOS 风格双层阴影 |
| 动画时长 | ⚡️ 增强 | 新增 Spring 曲线，时长微调 |
| Typography | ✅ 保持 | 未修改 |
| 间距系统 | ✅ 保持 | 未修改 |

---

## 视觉对比摘要

| 区域 | 修改前 | 修改后 |
|------|--------|--------|
| **TitleBar** | 纯色背景 | **毛玻璃 + 背景模糊 24px** |
| **StatusBar** | 纯色背景 | **毛玻璃 + 背景模糊 20px** |
| **侧边栏** | 纯色背景 | **毛玻璃 + 微妙透明度** |
| **消息气泡** | 直角圆角，无阴影 | **大圆角 18px + 柔和阴影** |
| **代码块** | 小圆角 | **大圆角 12px + 悬停语言标签** |
| **输入框** | 普通边框聚焦 | **品牌色光环 + Spring 动画** |
| **按钮悬停** | 线性过渡 | **Spring 弹性 + 微缩放** |
| **滚动条** | 标准样式 | **胶囊形 + 更 subtle** |

---

*美化完成 — 共修改 12 个文件，增强 10 大视觉系统*

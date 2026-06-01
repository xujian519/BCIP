# 桌面版布局重构设计

> 日期：2026-06-01
> 状态：草案

## 1. 目标

重构桌面版（`apps/desktop`）的布局系统，实现：

1. 左侧栏改为 VSCode 风格 Activity Bar + 可展开侧面板
2. 中间区断开与 Agent 实时输出的耦合，改为多功能文档工作区
3. 右侧聊天面板放宽宽度调节范围
4. 右上角增加布局设置，支持多种布局预设
5. 文件树右键支持 @引用文件到聊天区

## 2. 当前架构

```
DesktopShell
├── TitleBar (交通灯 | "云熙智能体" | 阶段Pill)
├── flex row
│   ├── LeftSidebar (文件树 + 折叠按钮, 48~400px)
│   ├── ResizeHandle
│   ├── CenterPanel (AgentWorkPane / 文件预览 / 阶段视图)
│   ├── ResizeHandle
│   └── AgentPanel (聊天面板, 280~520px)
└── StatusBar
```

关键文件：
- `src/components/shell/DesktopShell.tsx` — 主壳层布局
- `src/components/sidebar/LeftSidebar.tsx` — 左侧栏（当前为文件树）
- `src/components/sidebar/FileTree.tsx` — 文件树组件
- `src/components/center/CenterPanel.tsx` — 中间区
- `src/components/center/AgentWorkPane.tsx` — 中间区 Agent 输出（将移除）
- `src/components/agent/AgentPanel.tsx` — 右侧聊天面板
- `src/components/shell/TitleBar.tsx` — 标题栏
- `src/components/shell/ResizeHandle.tsx` — 拖拽调节手柄
- `src/hooks/useResponsiveShellLayout.ts` — 响应式断点逻辑
- `src/store/appReducer.ts` — 状态管理

## 3. 目标架构

```
DesktopShell
├── TitleBar (交通灯 | "云熙智能体" | 布局设置⚙️)
├── flex row
│   ├── ActivityBar (48px 固定, 5个图标)
│   ├── SidePanel (可展开/收起, 显示当前活跃图标对应的面板)
│   ├── ResizeHandle (可选, 仅侧面板展开时)
│   ├── DocumentWorkspace (文档工作区, 多标签+分屏)
│   │   或 (上下分屏模式时)
│   │   ├── DocumentWorkspace (上半)
│   │   ├── ResizeHandle (水平)
│   │   └── AgentPanel (下半)
│   ├── ResizeHandle (可选, 三栏模式时)
│   └── AgentPanel (聊天面板, 280px~70%屏幕宽度)
└── StatusBar
```

## 4. 左侧 Activity Bar + 侧面板

### 4.1 Activity Bar

固定 48px 宽度，从上至下 5 个图标：

| 序号 | 图标 | 名称 | 展开面板 |
|------|------|------|----------|
| 1 | FolderOpen (📁) | 文件浏览器 | 文件树（复用现有 `FileTree` 组件） |
| 2 | PlusSquare (➕) | 新建任务 | 线程列表 + 新建会话按钮 |
| 3 | Search (🔍) | 搜索 | 文件内容搜索（输入框 + 结果列表） |
| 4 | Zap (⚡) | 技能 | Skill 管理器（技能列表、搜索、启用/禁用） |
| 5 | Bot (🤖) | 机器人 | 外接渠道管理（微信/飞书/钉钉配置） |

交互规则：
- 点击未激活图标 → 展开对应面板，图标高亮
- 点击已激活图标 → 收起面板，图标取消高亮
- 同一时刻最多展开一个面板
- 底部保留设置按钮（齿轮图标），点击打开设置浮层

### 4.2 侧面板

侧面板宽度和内容随 Activity Bar 选择变化：
- 文件浏览器：复用 `FileTree`，180~400px
- 新建任务：线程列表，200~360px
- 搜索：搜索框 + 结果，240~400px
- 技能：技能列表，240~400px
- 机器人：渠道配置，280~440px

### 4.3 文件树右键 @引用

在 `FileTree` 组件上增加右键上下文菜单：

- 右键文件 → 菜单项："在聊天中引用"
- 右键文件夹 → 菜单项："在聊天中引用"
- 点击后效果：
  1. 自动聚焦聊天输入框
  2. 在输入框中插入 `@相对路径` 文本
  3. 文件内容作为上下文附件暂存，随消息一并发送（对用户透明，显示为引用标签）

实现要点：
- 新建 `src/components/sidebar/FileTreeContextMenu.tsx`
- 通过 dispatch 发送 `INSERT_CHAT_MENTION` action
- `Composer` 组件监听 mention 插入事件

## 5. 中间文档工作区

### 5.1 断开与 Agent 输出的耦合

移除 `AgentWorkPane` 在中间区的渲染。中间区不再显示实时智能体输出，仅做文档相关操作。

### 5.2 多标签页系统

新建 `src/components/workspace/` 目录：

- `WorkspaceTabs.tsx` — 标签栏（可滚动，显示已打开文件列表）
- `WorkspaceView.tsx` — 单文件视图容器
- `WorkspaceSplit.tsx` — 分屏容器

标签页行为：
- 点击文件树中的文件 → 在工作区新开标签（已打开则切换到该标签）
- 标签栏显示文件名，可关闭（×按钮）
- 拖拽标签可分屏（MVP 阶段可选，后续迭代）

### 5.3 支持的文件类型

| 文件类型 | 组件 | 功能 |
|----------|------|------|
| Markdown | `MarkdownPreview` | 编辑 + 实时预览 |
| PDF | `PdfPreview` | 预览 + 标注 |
| 图片 | `ImagePreview` | 预览 + 标注 |
| DOCX | `DocxEditorView` / `DocxPreview` | 编辑（docx-editor） |
| 代码/文本 | `TextPreview` | 语法高亮预览 |

### 5.4 空状态

无文件打开时显示欢迎页：
- 工作区名称
- 快速操作按钮（新建文件、最近打开）
- 快捷键提示

## 6. 右侧聊天面板

### 6.1 宽度调节范围

- 最小宽度：280px
- 默认宽度：380px（保持现状）
- 最大宽度：70% 屏幕宽度

修改 `DesktopShell.tsx` 中 `AGENT_MAX` 常量，改为动态计算：
```typescript
const AGENT_MAX = Math.floor(window.innerWidth * 0.7);
```

### 6.2 面板内容不变

`AgentPanel` 内部结构（Header、MessageTimeline、Composer、Footer）保持不变。

## 7. 布局设置

### 7.1 入口

TitleBar 右侧（阶段 Pill 旁边）添加 ⚙️ 布局按钮，点击弹出下拉菜单。

### 7.2 布局预设

| 预设 | 结构 | 说明 |
|------|------|------|
| **默认三栏** | ActivityBar + SidePanel + 工作区 + AgentPanel | 标准左右排列 |
| **文档模式** | ActivityBar + SidePanel + 工作区 | 隐藏聊天面板 |
| **上下分屏** | ActivityBar + SidePanel + (工作区↑ / AgentPanel↓) | 聊天区在底部 |

### 7.3 上下分屏细节

- 工作区和聊天区在同一列，中间用水平 ResizeHandle 分隔
- 聊天区默认占 40% 高度
- 拖拽范围：20%~80%
- 两栏均不可折叠（分屏模式下两者始终可见）
- 上下分屏模式下，聊天面板宽度自动填满可用列宽，不受三栏模式的宽度限制

## 8. 状态管理变更

### 8.1 AppState 新增字段

```typescript
// Activity Bar
activityBarTab: 'files' | 'new-task' | 'search' | 'skills' | 'bots' | null;

// 布局模式
layoutMode: 'three-column' | 'document' | 'horizontal-split';

// 聊天面板（上下分屏模式的高度）
chatPanelHeight: number; // 默认 window.innerHeight * 0.4

// 文档工作区标签
openTabs: WorkspaceTab[];
activeTabId: string | null;
```

### 8.2 新增 Action 类型

```typescript
| { type: 'SET_ACTIVITY_BAR_TAB'; payload: ActivityBarTab | null }
| { type: 'SET_LAYOUT_MODE'; payload: LayoutMode }
| { type: 'SET_CHAT_PANEL_HEIGHT'; payload: number }
| { type: 'OPEN_TAB'; payload: WorkspaceTab }
| { type: 'CLOSE_TAB'; payload: string } // tab id
| { type: 'SET_ACTIVE_TAB'; payload: string | null }
| { type: 'INSERT_CHAT_MENTION'; payload: { path: string; content?: string } }
```

### 8.3 WorkspaceTab 类型

```typescript
interface WorkspaceTab {
  id: string;
  filePath: string;
  title: string;
}
```

## 9. 响应式断点调整

`useResponsiveShellLayout.ts` 需要适配新布局：

- 窄屏（<900px）：上下分屏模式自动生效，Activity Bar 面板以浮层形式展示
- 中屏（900~1200px）：隐藏线程列表抽屉
- 宽屏（>1200px）：完整三栏体验

聊天面板的 `AGENT_MAX` 改为响应式计算，随窗口大小变化动态更新。

## 10. 文件变更清单

### 新建文件

| 文件 | 说明 |
|------|------|
| `src/components/activity-bar/ActivityBar.tsx` | Activity Bar 图标栏 |
| `src/components/activity-bar/ActivityBarIcon.tsx` | 单个图标按钮 |
| `src/components/sidebar/FileTreeContextMenu.tsx` | 文件树右键菜单 |
| `src/components/sidebar/SearchPanel.tsx` | 文件内容搜索面板 |
| `src/components/sidebar/NewTaskPanel.tsx` | 新建任务面板 |
| `src/components/sidebar/SkillsPanel.tsx` | Skill 管理器面板 |
| `src/components/sidebar/BotsPanel.tsx` | 外接渠道管理面板 |
| `src/components/workspace/WorkspaceTabs.tsx` | 工作区标签栏 |
| `src/components/workspace/WorkspaceView.tsx` | 工作区视图容器 |
| `src/components/workspace/WorkspaceSplit.tsx` | 分屏容器 |
| `src/components/workspace/WelcomeScreen.tsx` | 空状态欢迎页 |
| `src/components/shell/LayoutMenu.tsx` | 布局设置下拉菜单 |

### 修改文件

| 文件 | 变更 |
|------|------|
| `src/components/shell/DesktopShell.tsx` | 重构为 Activity Bar + 新布局逻辑 |
| `src/components/sidebar/LeftSidebar.tsx` | 改造为通用 SidePanel 容器 |
| `src/components/sidebar/FileTree.tsx` | 增加右键菜单支持 |
| `src/components/shell/TitleBar.tsx` | 添加布局设置按钮 |
| `src/components/center/CenterPanel.tsx` | 改造为文档工作区（多标签） |
| `src/components/agent/AgentPanel.tsx` | 支持上下分屏模式 |
| `src/components/agent/Composer.tsx` | 支持 mention 插入 |
| `src/hooks/useResponsiveShellLayout.ts` | 适配新布局断点 |
| `src/store/appReducer.ts` | 新增 action 处理 |
| `src/types/index.ts` | 新增类型定义 |
| `src/types/desktopApp.ts` | 新增 AppState 字段 |

### 可移除文件

| 文件 | 原因 |
|------|------|
| `src/components/center/AgentWorkPane.tsx` | 中间区不再显示 Agent 输出 |

## 11. 实施分阶段建议

### Phase 1：Activity Bar + 侧面板重构
- 新建 ActivityBar 组件
- 改造 LeftSidebar 为 SidePanel
- 文件浏览器面板（复用 FileTree）
- 新建任务面板
- 状态管理更新

### Phase 2：中间文档工作区
- 多标签页系统
- WorkspaceTabs + WorkspaceView
- WelcomeScreen 空状态
- 移除 AgentWorkPane

### Phase 3：布局设置 + 聊天面板
- LayoutMenu 布局切换
- 聊天面板宽度范围放宽
- 上下分屏模式
- 响应式断点适配

### Phase 4：搜索面板 + 技能 + 机器人 + 右键@
- SearchPanel 文件内容搜索
- SkillsPanel 技能管理器
- BotsPanel 外接渠道
- FileTreeContextMenu 右键@引用

### Phase 5：打磨与测试
- 分屏拖拽完善
- 键盘快捷键（Ctrl+B 切换侧栏, Ctrl+J 切换聊天）
- E2E 测试更新
- 快照测试更新

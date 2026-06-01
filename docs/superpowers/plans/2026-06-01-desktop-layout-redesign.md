# 桌面版布局重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构桌面版布局为 VSCode 风格 Activity Bar + 多标签文档工作区 + 可缩放聊天面板 + 多布局预设。

**Architecture:** 在 `apps/desktop/src` 下新增 `activity-bar/`、`workspace/` 目录，改造 `DesktopShell` 为新布局容器，状态管理新增 Activity Bar、布局模式、工作区标签等字段。分 5 个 Phase 依次实施，每个 Phase 独立可测。

**Tech Stack:** React 19 + TypeScript + Tailwind CSS + Framer Motion + Tauri 2.x + Lucide Icons

**Design Spec:** `docs/superpowers/specs/2026-06-01-desktop-layout-redesign.md`

---

## Phase 1: 类型系统 + 状态管理基础

### Task 1: 扩展类型定义

**Files:**
- Modify: `apps/desktop/src/types/desktopApp.ts`

- [ ] **Step 1: 新增类型定义**

在 `desktopApp.ts` 文件中，在 `SidebarTab` 类型之后添加：

```typescript
export type ActivityBarTab = 'files' | 'new-task' | 'search' | 'skills' | 'bots';

export type LayoutMode = 'three-column' | 'document' | 'horizontal-split';

export interface WorkspaceTab {
  id: string;
  filePath: string;
  title: string;
}

export interface ChatMention {
  path: string;
  content?: string;
}
```

- [ ] **Step 2: 扩展 AppState 接口**

在 `AppState` 接口的 `// —— 布局 ——` 区域，替换现有布局字段为：

```typescript
  // —— 布局 ——
  leftSidebarOpen: boolean;
  leftSidebarWidth: number;
  agentPanelOpen: boolean;
  agentPanelWidth: number;
  threadListOpen: boolean;
  sidebarTab: SidebarTab;
  activityBarTab: ActivityBarTab | null;
  layoutMode: LayoutMode;
  chatPanelHeight: number;

  // —— 工作区标签 ——
  openTabs: WorkspaceTab[];
  activeTabId: string | null;
  chatMentions: ChatMention[];
```

- [ ] **Step 3: 扩展 AppAction 类型**

在 `AppAction` 联合类型中，在 `// 布局` 区域添加：

```typescript
  | { type: 'SET_ACTIVITY_BAR_TAB'; payload: ActivityBarTab | null }
  | { type: 'SET_LAYOUT_MODE'; payload: LayoutMode }
  | { type: 'SET_CHAT_PANEL_HEIGHT'; payload: number }
  // 工作区标签
  | { type: 'OPEN_TAB'; payload: WorkspaceTab }
  | { type: 'CLOSE_TAB'; payload: string }
  | { type: 'SET_ACTIVE_TAB'; payload: string | null }
  | { type: 'INSERT_CHAT_MENTION'; payload: ChatMention }
  | { type: 'CLEAR_CHAT_MENTIONS' }
```

- [ ] **Step 4: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -30`
Expected: 可能有缺少字段的错误（下一步修复），但不应有类型语法错误。

---

### Task 2: 更新 Reducer 和初始状态

**Files:**
- Modify: `apps/desktop/src/store/appReducer.ts`
- Modify: `apps/desktop/src/store/buildInitialState.ts`
- Modify: `apps/desktop/src/store/actionHooks.ts`

- [ ] **Step 1: 更新初始状态**

在 `buildInitialState.ts` 中，在 `sidebarTab: 'project',` 之后添加：

```typescript
    activityBarTab: 'files',
    layoutMode: 'three-column',
    chatPanelHeight: typeof window !== 'undefined' ? Math.floor(window.innerHeight * 0.4) : 400,
    openTabs: [],
    activeTabId: null,
    chatMentions: [],
```

- [ ] **Step 2: 在 appReducer.ts 中添加新 case**

在 `appReducer.ts` 的 `SET_SIDEBAR_TAB` case 之后添加：

```typescript
    case 'SET_ACTIVITY_BAR_TAB': {
      const tab = action.payload;
      return {
        ...state,
        activityBarTab: tab,
        leftSidebarOpen: tab !== null,
      };
    }
    case 'SET_LAYOUT_MODE':
      return { ...state, layoutMode: action.payload };
    case 'SET_CHAT_PANEL_HEIGHT':
      return { ...state, chatPanelHeight: action.payload };
    case 'OPEN_TAB': {
      const tab = action.payload;
      const exists = state.openTabs.some((t) => t.filePath === tab.filePath);
      if (exists) {
        return { ...state, activeTabId: state.openTabs.find((t) => t.filePath === tab.filePath)!.id };
      }
      return {
        ...state,
        openTabs: [...state.openTabs, tab],
        activeTabId: tab.id,
      };
    }
    case 'CLOSE_TAB': {
      const remaining = state.openTabs.filter((t) => t.id !== action.payload);
      const newActiveId = state.activeTabId === action.payload
        ? (remaining.length > 0 ? remaining[remaining.length - 1].id : null)
        : state.activeTabId;
      return { ...state, openTabs: remaining, activeTabId: newActiveId };
    }
    case 'SET_ACTIVE_TAB':
      return { ...state, activeTabId: action.payload };
    case 'INSERT_CHAT_MENTION':
      return { ...state, chatMentions: [...state.chatMentions, action.payload] };
    case 'CLEAR_CHAT_MENTIONS':
      return { ...state, chatMentions: [] };
```

- [ ] **Step 3: 更新 actionHooks.ts**

在 `useLayoutActions` 中添加新的 action creator：

```typescript
  const setActivityBarTab = useCallback(
    (tab: ActivityBarTab | null) => dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: tab }),
    [dispatch],
  );

  const setLayoutMode = useCallback(
    (mode: LayoutMode) => dispatch({ type: 'SET_LAYOUT_MODE', payload: mode }),
    [dispatch],
  );
```

在 return 对象中添加 `setActivityBarTab` 和 `setLayoutMode`。同时更新文件顶部的 import 添加 `ActivityBarTab`, `LayoutMode`。

- [ ] **Step 4: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -30`
Expected: PASS（可能有组件层面的类型错误，后续 Task 修复）

---

## Phase 2: Activity Bar 组件

### Task 3: 创建 ActivityBar 组件

**Files:**
- Create: `apps/desktop/src/components/activity-bar/ActivityBar.tsx`

- [ ] **Step 1: 创建 ActivityBar 目录和组件**

创建文件 `apps/desktop/src/components/activity-bar/ActivityBar.tsx`：

```tsx
import type { FC } from 'react';
import {
  FolderOpen,
  PlusSquare,
  Search,
  Zap,
  Bot,
  Settings,
} from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import type { ActivityBarTab } from '@/types';

interface ActivityBarItem {
  id: ActivityBarTab;
  icon: FC<{ size?: number }>;
  label: string;
}

const items: ActivityBarItem[] = [
  { id: 'files', icon: FolderOpen, label: '文件浏览器' },
  { id: 'new-task', icon: PlusSquare, label: '新建任务' },
  { id: 'search', icon: Search, label: '搜索' },
  { id: 'skills', icon: Zap, label: '技能' },
  { id: 'bots', icon: Bot, label: '机器人' },
];

export default function ActivityBar() {
  const { state, dispatch } = useAppStore();

  const handleClick = (id: ActivityBarTab) => {
    if (state.activityBarTab === id) {
      dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: null });
    } else {
      dispatch({ type: 'SET_ACTIVITY_BAR_TAB', payload: id });
    }
  };

  return (
    <div
      className="flex flex-col items-center justify-between shrink-0"
      style={{
        width: 48,
        backgroundColor: 'var(--bg-sidebar)',
        borderRight: '1px solid var(--border-primary)',
        paddingTop: 8,
        paddingBottom: 8,
      }}
    >
      <div className="flex flex-col items-center gap-1">
        {items.map((item) => {
          const isActive = state.activityBarTab === item.id;
          const Icon = item.icon;
          return (
            <button
              key={item.id}
              onClick={() => handleClick(item.id)}
              className="relative flex items-center justify-center transition-colors duration-150"
              style={{
                width: 40,
                height: 40,
                borderRadius: 8,
                color: isActive ? 'var(--accent-primary)' : 'var(--text-tertiary)',
                backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
              }}
              onMouseEnter={(e) => {
                if (!isActive) {
                  e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
                  e.currentTarget.style.color = 'var(--text-secondary)';
                }
              }}
              onMouseLeave={(e) => {
                if (!isActive) {
                  e.currentTarget.style.backgroundColor = 'transparent';
                  e.currentTarget.style.color = 'var(--text-tertiary)';
                }
              }}
              title={item.label}
              aria-label={item.label}
              type="button"
            >
              <Icon size={20} />
              {isActive && (
                <div
                  className="absolute left-0 top-1/2 -translate-y-1/2 rounded-r-full"
                  style={{
                    width: 2,
                    height: 20,
                    backgroundColor: 'var(--accent-primary)',
                  }}
                />
              )}
            </button>
          );
        })}
      </div>

      <div className="flex flex-col items-center gap-1">
        <button
          onClick={() => dispatch({ type: 'OPEN_SETTINGS', payload: 'general' })}
          className="flex items-center justify-center transition-colors duration-150"
          style={{
            width: 40,
            height: 40,
            borderRadius: 8,
            color: 'var(--text-tertiary)',
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }}
          title="设置"
          aria-label="设置"
          type="button"
        >
          <Settings size={20} />
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 2: 创建侧面板容器**

创建文件 `apps/desktop/src/components/activity-bar/SidePanel.tsx`：

```tsx
import { AnimatePresence, motion } from 'framer-motion';
import { useAppStore } from '@/hooks/useAppStore';
import FileTree from '@/components/sidebar/FileTree';
import { useFileSystem } from '@/hooks/useFileSystem';
import { useProjects } from '@/hooks/useProjects';
import { useState, useEffect, useRef } from 'react';

function FilesPanel() {
  const { state } = useAppStore();
  const { projects } = useProjects();
  const defaultRoot = state.workspaceCwd ?? projects[0]?.path ?? '';
  const {
    files,
    loading,
    error,
    selectedPath,
    expandedPaths,
    selectFile,
    toggleExpanded,
    loadChildren,
    refresh,
    setRootPath,
  } = useFileSystem(defaultRoot);

  useEffect(() => {
    if (state.workspaceCwd) setRootPath(state.workspaceCwd);
  }, [state.workspaceCwd, setRootPath]);

  const handleSelect = (path: string) => {
    selectFile(path);
  };

  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        资源管理器
      </div>
      {defaultRoot ? (
        <FileTree
          rootPath={defaultRoot}
          entries={files}
          selectedPath={selectedPath}
          expandedPaths={expandedPaths}
          onSelect={handleSelect}
          onToggleExpand={toggleExpanded}
          onLoadChildren={loadChildren}
        />
      ) : (
        <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
          打开一个工作区目录
        </div>
      )}
    </div>
  );
}

function NewTaskPanel() {
  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        新建任务
      </div>
      <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
        在当前项目创建新会话
      </div>
    </div>
  );
}

function SearchPanel() {
  const [query, setQuery] = useState('');
  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        搜索
      </div>
      <div className="px-3 pb-2">
        <input
          type="text"
          placeholder="搜索文件内容..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="w-full rounded-md border px-2 py-1 text-xs outline-none"
          style={{
            backgroundColor: 'var(--bg-elevated)',
            borderColor: 'var(--border-primary)',
            color: 'var(--text-primary)',
            height: 30,
          }}
        />
      </div>
      {!query && (
        <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
          输入关键词搜索文件内容
        </div>
      )}
    </div>
  );
}

function SkillsPanel() {
  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        技能管理
      </div>
      <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
        技能列表将在此显示
      </div>
    </div>
  );
}

function BotsPanel() {
  return (
    <div className="h-full overflow-auto">
      <div className="px-3 py-2 text-xs font-medium" style={{ color: 'var(--text-secondary)' }}>
        外接渠道
      </div>
      <div className="p-4 text-center text-xs" style={{ color: 'var(--text-tertiary)' }}>
        管理微信、飞书、钉钉等外接渠道
      </div>
    </div>
  );
}

const panelMap: Record<string, () => JSX.Element> = {
  files: FilesPanel,
  'new-task': NewTaskPanel,
  search: SearchPanel,
  skills: SkillsPanel,
  bots: BotsPanel,
};

export default function SidePanel() {
  const { state } = useAppStore();
  const tab = state.activityBarTab;

  if (!tab) return null;

  const Panel = panelMap[tab];
  if (!Panel) return null;

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={tab}
        initial={{ opacity: 0, x: -8 }}
        animate={{ opacity: 1, x: 0 }}
        exit={{ opacity: 0, x: -8 }}
        transition={{ duration: 0.15 }}
        className="h-full shrink-0 overflow-hidden"
        style={{
          width: state.leftSidebarWidth,
          backgroundColor: 'var(--bg-sidebar)',
          borderRight: '1px solid var(--border-primary)',
        }}
      >
        <Panel />
      </motion.div>
    </AnimatePresence>
  );
}
```

- [ ] **Step 3: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -20`
Expected: 新文件无错误

---

### Task 4: 改造 DesktopShell 集成 Activity Bar

**Files:**
- Modify: `apps/desktop/src/components/shell/DesktopShell.tsx`

- [ ] **Step 1: 重写 DesktopShell**

将 `DesktopShell.tsx` 的内容替换为：

```tsx
import { useState, useCallback, useRef, useEffect } from 'react';
import { BCIP_PREVIEW_RELOAD } from '@/lib/desktopEvents';
import { pathMatchesAny } from '@/lib/workspacePaths';
import { useGlobalKeyboardShortcuts } from '@/hooks/useGlobalKeyboardShortcuts';
import { isWindowsPlatform } from '@/lib/platform';
import { cn } from '@/lib/utils';
import { useAppStore } from '@/hooks/useAppStore';
import { useBootstrapDesktopConfig } from '@/hooks/useBootstrapDesktopConfig';
import { isDesktopRpcReady } from '@/lib/configAccess';
import TitleBar from './TitleBar';
import StatusBar from './StatusBar';
import ResizeHandle from './ResizeHandle';
import ActivityBar from '@/components/activity-bar/ActivityBar';
import SidePanel from '@/components/activity-bar/SidePanel';
import DocumentWorkspace from '@/components/workspace/DocumentWorkspace';
import AgentPanel from '@/components/agent/AgentPanel';

const MAX_SIDEBAR_WIDTH = 400;
const AGENT_MIN = 280;

export default function DesktopShell() {
  const { state, dispatch } = useAppStore();
  useBootstrapDesktopConfig(
    isDesktopRpcReady(state.connectionStatus),
    state.workspaceCwd,
    dispatch,
  );
  const centerRef = useRef<HTMLDivElement>(null);

  useGlobalKeyboardShortcuts();

  const agentMax = Math.floor(window.innerWidth * 0.7);

  const handleWorkspaceCwd = useCallback(
    (cwd: string) => dispatch({ type: 'SET_WORKSPACE_CWD', payload: cwd }),
    [dispatch],
  );

  const handleLeftResize = useCallback(
    (width: number) => dispatch({ type: 'SET_LEFT_SIDEBAR_WIDTH', payload: width }),
    [dispatch],
  );

  const handleAgentResize = useCallback(
    (width: number) => dispatch({ type: 'SET_AGENT_PANEL_WIDTH', payload: width }),
    [dispatch],
  );

  const handleChatHeightResize = useCallback(
    (height: number) => dispatch({ type: 'SET_CHAT_PANEL_HEIGHT', payload: height }),
    [dispatch],
  );

  const sidePanelVisible = state.activityBarTab !== null;
  const showAgent = state.agentPanelOpen;
  const isHorizontalSplit = state.layoutMode === 'horizontal-split';

  return (
    <div
      className={cn(
        'h-[100dvh] w-full flex flex-col overflow-hidden',
        'bg-[var(--bg-base)] text-[var(--text-primary)]',
        state.isDark ? 'dark' : '',
        isWindowsPlatform() && 'platform-windows',
      )}
      data-platform={isWindowsPlatform() ? 'windows' : 'mac'}
    >
      <TitleBar />

      <div className="flex-1 flex min-h-0">
        <ActivityBar />

        {sidePanelVisible && (
          <>
            <SidePanel />
            <ResizeHandle
              direction="horizontal"
              size={state.leftSidebarWidth}
              minSize={180}
              maxSize={MAX_SIDEBAR_WIDTH}
              onResize={handleLeftResize}
              position="left"
            />
          </>
        )}

        {isHorizontalSplit ? (
          <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
            <div style={{ height: `calc(100% - ${state.chatPanelHeight}px)` }}>
              <DocumentWorkspace />
            </div>
            <ResizeHandle
              direction="vertical"
              size={state.chatPanelHeight}
              minSize={Math.floor(window.innerHeight * 0.2)}
              maxSize={Math.floor(window.innerHeight * 0.8)}
              onResize={handleChatHeightResize}
              position="bottom"
            />
            <AgentPanel width={0} fillWidth />
          </div>
        ) : (
          <>
            <div
              ref={centerRef}
              className="flex min-w-0 flex-1 flex-col overflow-hidden"
              style={{ minWidth: 200 }}
            >
              <DocumentWorkspace />
            </div>

            {showAgent && (
              <ResizeHandle
                direction="horizontal"
                size={state.agentPanelWidth}
                minSize={AGENT_MIN}
                maxSize={agentMax}
                onResize={handleAgentResize}
                position="right"
              />
            )}

            {showAgent && (
              <AgentPanel width={state.agentPanelWidth} />
            )}
          </>
        )}
      </div>

      <StatusBar />
    </div>
  );
}
```

- [ ] **Step 2: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -20`
Expected: `DocumentWorkspace` 和 `AgentPanel fillWidth` 报错（后续 Task 解决）

---

## Phase 3: 文档工作区

### Task 5: 创建 DocumentWorkspace 组件

**Files:**
- Create: `apps/desktop/src/components/workspace/DocumentWorkspace.tsx`
- Create: `apps/desktop/src/components/workspace/WorkspaceTabs.tsx`

- [ ] **Step 1: 创建 WorkspaceTabs**

创建文件 `apps/desktop/src/components/workspace/WorkspaceTabs.tsx`：

```tsx
import type { FC } from 'react';
import { X, FileText } from 'lucide-react';
import type { WorkspaceTab as WTab } from '@/types';

interface WorkspaceTabsProps {
  tabs: WTab[];
  activeTabId: string | null;
  onSelect: (id: string) => void;
  onClose: (id: string) => void;
}

const WorkspaceTabs: FC<WorkspaceTabsProps> = ({ tabs, activeTabId, onSelect, onClose }) => {
  if (tabs.length === 0) return null;

  return (
    <div
      className="flex shrink-0 overflow-x-auto"
      style={{
        height: 36,
        backgroundColor: 'var(--bg-elevated)',
        borderBottom: '1px solid var(--border-primary)',
      }}
    >
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        return (
          <div
            key={tab.id}
            className="group flex items-center gap-1.5 shrink-0 cursor-pointer select-none"
            style={{
              padding: '0 12px',
              height: 36,
              fontSize: 12,
              color: isActive ? 'var(--text-primary)' : 'var(--text-tertiary)',
              backgroundColor: isActive ? 'var(--bg-surface)' : 'transparent',
              borderRight: '1px solid var(--border-primary)',
              maxWidth: 180,
            }}
            onClick={() => onSelect(tab.id)}
          >
            <FileText size={12} style={{ flexShrink: 0 }} />
            <span className="truncate flex-1">{tab.title}</span>
            <button
              onClick={(e) => { e.stopPropagation(); onClose(tab.id); }}
              className="opacity-0 group-hover:opacity-100 transition-opacity shrink-0 rounded p-0.5 hover:bg-[var(--bg-hover)]"
              style={{ color: 'var(--text-tertiary)' }}
              type="button"
              aria-label={`关闭 ${tab.title}`}
            >
              <X size={12} />
            </button>
          </div>
        );
      })}
    </div>
  );
};

export default WorkspaceTabs;
```

- [ ] **Step 2: 创建 WelcomeScreen**

创建文件 `apps/desktop/src/components/workspace/WelcomeScreen.tsx`：

```tsx
import { FolderOpen } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';

export default function WelcomeScreen() {
  const { state } = useAppStore();
  const hasWorkspace = !!state.workspaceCwd;

  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 px-8">
      <FolderOpen size={36} style={{ color: 'var(--text-tertiary)', opacity: 0.4 }} />
      {hasWorkspace ? (
        <>
          <p className="text-sm" style={{ color: 'var(--text-secondary)' }}>
            从左侧文件浏览器选择文件开始工作
          </p>
          <p className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
            支持 Markdown 编辑、PDF 预览、DOCX 编辑、代码查看
          </p>
        </>
      ) : (
        <>
          <p className="text-sm" style={{ color: 'var(--text-secondary)' }}>
            打开一个工作区目录开始工作
          </p>
          <p className="text-xs" style={{ color: 'var(--text-tertiary)' }}>
            点击左侧文件浏览器图标打开目录
          </p>
        </>
      )}
    </div>
  );
}
```

- [ ] **Step 3: 创建 DocumentWorkspace**

创建文件 `apps/desktop/src/components/workspace/DocumentWorkspace.tsx`：

```tsx
import { useCallback, useEffect } from 'react';
import { useAppStore } from '@/hooks/useAppStore';
import WorkspaceTabs from './WorkspaceTabs';
import WelcomeScreen from './WelcomeScreen';
import FilePreviewRouter from '@/components/preview/FilePreviewRouter';
import { BCIP_FILE_TREE_REFRESH, BCIP_PREVIEW_RELOAD } from '@/lib/desktopEvents';
import { pathMatchesAny } from '@/lib/workspacePaths';
import type { WorkspaceTab } from '@/types';

export default function DocumentWorkspace() {
  const { state, dispatch } = useAppStore();

  const handleSelectTab = useCallback(
    (id: string) => dispatch({ type: 'SET_ACTIVE_TAB', payload: id }),
    [dispatch],
  );

  const handleCloseTab = useCallback(
    (id: string) => dispatch({ type: 'CLOSE_TAB', payload: id }),
    [dispatch],
  );

  const activeTab = state.openTabs.find((t) => t.id === state.activeTabId) ?? null;

  return (
    <div
      className="flex h-full flex-col overflow-hidden"
      style={{ backgroundColor: 'var(--bg-surface)' }}
    >
      <WorkspaceTabs
        tabs={state.openTabs}
        activeTabId={state.activeTabId}
        onSelect={handleSelectTab}
        onClose={handleCloseTab}
      />

      <div className="relative min-h-0 flex-1 overflow-hidden">
        {activeTab ? (
          <FilePreviewRouter key={activeTab.filePath} filePath={activeTab.filePath} />
        ) : (
          <WelcomeScreen />
        )}
      </div>
    </div>
  );
}
```

- [ ] **Step 4: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -20`
Expected: AgentPanel `fillWidth` prop 报错（下一个 Task 解决）

---

### Task 6: 改造 AgentPanel 支持上下分屏

**Files:**
- Modify: `apps/desktop/src/components/agent/AgentPanel.tsx`

- [ ] **Step 1: AgentPanel 增加 fillWidth 模式**

在 `AgentPanel.tsx` 中，修改组件签名，添加 `fillWidth` 可选 prop：

将 `export default function AgentPanel({ width }: { width: number })` 改为：

```typescript
export default function AgentPanel({ width, fillWidth }: { width: number; fillWidth?: boolean }) {
```

将 `<aside>` 的 `style={{ width }}` 改为：

```typescript
      style={fillWidth ? { flex: 1 } : { width }}
```

并在 `<aside>` 的 className 中，当 `fillWidth` 时去掉 `shrink-0`：

```typescript
      className={cn(
        'h-full flex flex-col',
        !fillWidth && 'shrink-0',
        'bg-[var(--bg-surface)]',
        fillWidth ? '' : 'border-l border-[var(--border-default)]',
      )}
```

- [ ] **Step 2: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -20`
Expected: 可能仍有 TitleBar 布局菜单相关错误（Phase 4 解决）

---

## Phase 4: 布局设置菜单

### Task 7: 创建 LayoutMenu 并集成到 TitleBar

**Files:**
- Create: `apps/desktop/src/components/shell/LayoutMenu.tsx`
- Modify: `apps/desktop/src/components/shell/TitleBar.tsx`

- [ ] **Step 1: 创建 LayoutMenu**

创建文件 `apps/desktop/src/components/shell/LayoutMenu.tsx`：

```tsx
import { useState, useRef, useEffect } from 'react';
import { Columns2, FileText, SplitSquareHorizontal } from 'lucide-react';
import { useAppStore } from '@/hooks/useAppStore';
import type { LayoutMode } from '@/types';

interface LayoutOption {
  id: LayoutMode;
  label: string;
  description: string;
  icon: typeof Columns2;
}

const layoutOptions: LayoutOption[] = [
  { id: 'three-column', label: '三栏布局', description: '工作区 + 聊天面板', icon: Columns2 },
  { id: 'document', label: '文档模式', description: '隐藏聊天面板', icon: FileText },
  { id: 'horizontal-split', label: '上下分屏', description: '工作区在上，聊天在下', icon: SplitSquareHorizontal },
];

export default function LayoutMenu() {
  const { state, dispatch } = useAppStore();
  const [open, setOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [open]);

  const handleSelect = (mode: LayoutMode) => {
    dispatch({ type: 'SET_LAYOUT_MODE', payload: mode });
    if (mode === 'document') {
      dispatch({ type: 'SET_AGENT_PANEL_OPEN', payload: false });
    } else if (!state.agentPanelOpen) {
      dispatch({ type: 'SET_AGENT_PANEL_OPEN', payload: true });
    }
    setOpen(false);
  };

  return (
    <div ref={menuRef} className="relative">
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="flex items-center justify-center transition-colors duration-150"
        style={{
          width: 28,
          height: 28,
          borderRadius: 6,
          color: open ? 'var(--accent-primary)' : 'var(--text-tertiary)',
          backgroundColor: open ? 'var(--bg-sidebar-active)' : 'transparent',
        }}
        onMouseEnter={(e) => {
          if (!open) {
            e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
            e.currentTarget.style.color = 'var(--text-secondary)';
          }
        }}
        onMouseLeave={(e) => {
          if (!open) {
            e.currentTarget.style.backgroundColor = 'transparent';
            e.currentTarget.style.color = 'var(--text-tertiary)';
          }
        }}
        title="布局设置"
        aria-label="布局设置"
      >
        <Columns2 size={16} />
      </button>

      {open && (
        <div
          className="absolute right-0 top-full mt-1 z-50 rounded-lg border shadow-lg"
          style={{
            width: 220,
            backgroundColor: 'var(--bg-elevated)',
            borderColor: 'var(--border-primary)',
            padding: 4,
          }}
        >
          <div className="px-2 py-1.5 text-xs font-medium" style={{ color: 'var(--text-tertiary)' }}>
            布局设置
          </div>
          {layoutOptions.map((opt) => {
            const isActive = state.layoutMode === opt.id;
            const Icon = opt.icon;
            return (
              <button
                key={opt.id}
                type="button"
                onClick={() => handleSelect(opt.id)}
                className="w-full flex items-center gap-2.5 rounded-md transition-colors"
                style={{
                  padding: '8px 8px',
                  backgroundColor: isActive ? 'var(--bg-sidebar-active)' : 'transparent',
                  color: isActive ? 'var(--accent-primary)' : 'var(--text-primary)',
                }}
                onMouseEnter={(e) => {
                  if (!isActive) e.currentTarget.style.backgroundColor = 'var(--bg-hover)';
                }}
                onMouseLeave={(e) => {
                  if (!isActive) e.currentTarget.style.backgroundColor = 'transparent';
                }}
              >
                <Icon size={16} />
                <div className="text-left">
                  <div className="text-xs font-medium">{opt.label}</div>
                  <div className="text-2xs" style={{ color: 'var(--text-tertiary)' }}>
                    {opt.description}
                  </div>
                </div>
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: 集成到 TitleBar**

在 `TitleBar.tsx` 中：

1. 在文件顶部添加 import：
```typescript
import LayoutMenu from './LayoutMenu';
```

2. 在 `<header>` 的 StageIndicator 之前，添加布局按钮。找到 `{/* 右侧：阶段指示器 */}` 注释，在其之前插入：

```tsx
      <div
        className="flex items-center gap-2 pr-2"
        style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
      >
        <LayoutMenu />
      </div>
```

然后把 `{/* 右侧：阶段指示器 */}` 的 `<StageIndicator />` 包裹容器和新建的 LayoutMenu 容器合并为一个右侧 flex 容器：

```tsx
      <div
        className="flex items-center gap-2 pr-4"
        style={{ WebkitAppRegion: 'no-drag' } as React.CSSProperties}
      >
        <LayoutMenu />
        <StageIndicator />
      </div>
```

替换掉原来的单独 `<StageIndicator />` 所在的 div。

- [ ] **Step 3: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -20`
Expected: PASS

---

### Task 8: 串联文件选择 → 工作区标签

**Files:**
- Modify: `apps/desktop/src/components/activity-bar/SidePanel.tsx`

- [ ] **Step 1: 文件选择时打开标签**

在 `SidePanel.tsx` 的 `FilesPanel` 组件中，修改 `handleSelect` 使其 dispatch `OPEN_TAB`：

```typescript
function FilesPanel() {
  const { state, dispatch } = useAppStore();
  const { projects } = useProjects();
  const defaultRoot = state.workspaceCwd ?? projects[0]?.path ?? '';
  const {
    files,
    loading,
    error,
    selectedPath,
    expandedPaths,
    selectFile,
    toggleExpanded,
    loadChildren,
    refresh,
    setRootPath,
  } = useFileSystem(defaultRoot);

  useEffect(() => {
    if (state.workspaceCwd) setRootPath(state.workspaceCwd);
  }, [state.workspaceCwd, setRootPath]);

  const handleSelect = (path: string) => {
    selectFile(path);
    dispatch({ type: 'SET_CURRENT_FILE', payload: path });
    const fileName = path.split('/').pop() ?? path;
    dispatch({
      type: 'OPEN_TAB',
      payload: {
        id: `tab-${path}`,
        filePath: path,
        title: fileName,
      },
    });
  };
```

- [ ] **Step 2: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -20`
Expected: PASS

---

## Phase 5: 文件树右键菜单 + @引用

### Task 9: 创建 FileTreeContextMenu

**Files:**
- Create: `apps/desktop/src/components/sidebar/FileTreeContextMenu.tsx`
- Modify: `apps/desktop/src/components/sidebar/FileTree.tsx`

- [ ] **Step 1: 创建右键菜单组件**

创建文件 `apps/desktop/src/components/sidebar/FileTreeContextMenu.tsx`：

```tsx
import { useEffect, useRef } from 'react';
import { AtSign, Copy } from 'lucide-react';

interface ContextMenuProps {
  x: number;
  y: number;
  filePath: string;
  onMention: (path: string) => void;
  onClose: () => void;
}

export default function FileTreeContextMenu({ x, y, filePath, onMention, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [onClose]);

  const fileName = filePath.split('/').pop() ?? filePath;

  return (
    <div
      ref={menuRef}
      className="fixed z-[100] rounded-lg border shadow-lg"
      style={{
        left: x,
        top: y,
        backgroundColor: 'var(--bg-elevated)',
        borderColor: 'var(--border-primary)',
        padding: 4,
        minWidth: 180,
      }}
    >
      <div className="px-2 py-1.5 text-2xs truncate" style={{ color: 'var(--text-tertiary)' }}>
        {fileName}
      </div>
      <button
        type="button"
        onClick={() => { onMention(filePath); onClose(); }}
        className="w-full flex items-center gap-2 rounded-md px-2 py-1.5 text-xs transition-colors"
        style={{ color: 'var(--text-primary)' }}
        onMouseEnter={(e) => { e.currentTarget.style.backgroundColor = 'var(--bg-hover)'; }}
        onMouseLeave={(e) => { e.currentTarget.style.backgroundColor = 'transparent'; }}
      >
        <AtSign size={14} />
        在聊天中引用
      </button>
    </div>
  );
}
```

- [ ] **Step 2: 在 FileTree 中添加右键事件**

需要先读取 `FileTree.tsx` 了解其结构，在文件条目的 `onClick` 同级添加 `onContextMenu` 事件处理。这需要修改 `FileTree.tsx` 中的文件条目渲染部分，添加右键事件并在顶层渲染 `FileTreeContextMenu`。

具体修改：

1. 在 `FileTree.tsx` 顶部添加 imports：
```typescript
import { useState } from 'react';
import FileTreeContextMenu from './FileTreeContextMenu';
import { useAppStore } from '@/hooks/useAppStore';
```

2. 在 `FileTree` 组件内添加状态：
```typescript
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; path: string } | null>(null);
  const { dispatch } = useAppStore();
```

3. 在每个文件条目的容器 `<div>` 上添加 `onContextMenu`：
```typescript
          onContextMenu={(e) => {
            e.preventDefault();
            setContextMenu({ x: e.clientX, y: e.clientY, path: entry.path });
          }}
```

4. 在组件最外层 div 的末尾（闭合标签前）添加：
```tsx
      {contextMenu && (
        <FileTreeContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          filePath={contextMenu.path}
          onMention={(path) => {
            dispatch({ type: 'INSERT_CHAT_MENTION', payload: { path } });
            window.dispatchEvent(new CustomEvent('bcip:focus-composer'));
          }}
          onClose={() => setContextMenu(null)}
        />
      )}
```

- [ ] **Step 3: Composer 监听 mention 插入**

在 `Composer.tsx` 的 `BCIP_FOCUS_COMPOSER` 事件监听旁边，增加对 mentions 的处理。在 `useEffect` 中添加：

```typescript
  useEffect(() => {
    const onFocus = () => textareaRef.current?.focus();
    window.addEventListener(BCIP_FOCUS_COMPOSER, onFocus);
    return () => window.removeEventListener(BCIP_FOCUS_COMPOSER, onFocus);
  }, []);
```

这段代码已存在。需要在旁边添加 mention 附件显示。在 Composer 的顶部行（附件区域）之后添加：

```tsx
        {state.chatMentions.length > 0 && (
          <div className="flex flex-wrap gap-1 px-3 pb-1">
            {state.chatMentions.map((m, i) => (
              <span
                key={i}
                className="inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-2xs"
                style={{
                  backgroundColor: 'var(--accent-primary-muted)',
                  color: 'var(--accent-primary)',
                  border: '1px solid var(--accent-primary)/30',
                }}
              >
                @{m.path.split('/').pop()}
              </span>
            ))}
          </div>
        )}
```

并在 `handleSend` 回调中发送后清除 mentions：

在 `handleSend` 函数中，`setText('')` 之后添加：
```typescript
    dispatch({ type: 'CLEAR_CHAT_MENTIONS' });
```

- [ ] **Step 4: 验证编译**

Run: `cd apps/desktop && npx tsc --noEmit 2>&1 | head -20`
Expected: PASS

---

## Phase 6: 清理 + 响应式

### Task 10: 移除旧组件 + 更新响应式逻辑

**Files:**
- Modify: `apps/desktop/src/hooks/useResponsiveShellLayout.ts`
- Delete (可选): `apps/desktop/src/components/center/AgentWorkPane.tsx`

- [ ] **Step 1: 更新响应式断点逻辑**

将 `useResponsiveShellLayout.ts` 替换为：

```typescript
import { useEffect, useRef, useState } from 'react';
import { useAppStore } from '@/hooks/useAppStore';

const BP_NARROW_OVERLAY = 900;
const BP_HIDE_THREAD_LIST = 1200;

export function useResponsiveShellLayout(centerRef: React.RefObject<HTMLElement | null>) {
  const { state, dispatch } = useAppStore();
  const [isNarrowViewport, setIsNarrowViewport] = useState(
    () => typeof window !== 'undefined' && window.innerWidth < BP_NARROW_OVERLAY,
  );

  useEffect(() => {
    const applyBreakpoint = () => {
      const w = window.innerWidth;
      setIsNarrowViewport(w < BP_NARROW_OVERLAY);
      const band = w < BP_NARROW_OVERLAY ? 'narrow' : w < BP_HIDE_THREAD_LIST ? 'medium' : 'wide';

      if (band !== 'wide' && state.threadListOpen) {
        dispatch({ type: 'SET_THREAD_LIST_OPEN', payload: false });
      }

      if (band === 'narrow' && state.layoutMode !== 'horizontal-split') {
        dispatch({ type: 'SET_LAYOUT_MODE', payload: 'horizontal-split' });
      }
    };

    applyBreakpoint();
    window.addEventListener('resize', applyBreakpoint);
    return () => window.removeEventListener('resize', applyBreakpoint);
  }, [dispatch, state.threadListOpen, state.layoutMode]);

  return { isNarrowViewport };
}
```

- [ ] **Step 2: 验证完整编译**

Run: `cd apps/desktop && npx tsc --noEmit`
Expected: PASS

- [ ] **Step 3: 验证构建**

Run: `cd apps/desktop && npm run build`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat(desktop): 布局重构 — Activity Bar + 文档工作区 + 多布局预设"
```

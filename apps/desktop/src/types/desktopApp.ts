/**
 * BCIP Agent 桌面端 —— 全局类型定义
 * 所有组件共享的核心类型
 */
import type { PermissionsRequestApprovalParams } from '@/generated/app-server/v2/PermissionsRequestApprovalParams';
import type { ToolRequestUserInputQuestion } from '@/generated/app-server/v2/ToolRequestUserInputQuestion';

// ========================================
// 连接状态
// ========================================
export type ConnectionStatus = 'connected' | 'connecting' | 'disconnected' | 'error';

/** 状态栏用量条数据 */
export interface UsageMeterSnapshot {
  label: string;
  used: number;
  max: number;
  hint?: string;
}

// ========================================
// 消息系统
// ========================================
export type MessageRole = 'user' | 'agent' | 'system';
export type MessageStatus = 'sending' | 'streaming' | 'complete' | 'error';

/** 工具调用 */
export interface ToolCall {
  id: string;
  name: string;
  status: 'running' | 'success' | 'error';
  kind?: 'shell' | 'mcp' | 'patch';
  /** 命令行、MCP 工具路径或变更摘要 */
  detail?: string;
  /** 调用参数（JSON 文本） */
  args?: string;
  output?: string;
  error?: string;
}

/** 聊天消息 */
/** 时间线条目子类型（对应 app-server ThreadItem） */
export type MessageItemKind = 'plan' | 'agent' | 'tool';

export interface Message {
  id: string;
  role: MessageRole;
  content: string;
  timestamp: number;
  status?: MessageStatus;
  reasoning?: string;
  toolCalls?: ToolCall[];
  itemKind?: MessageItemKind;
}

// ========================================
// 线程
// ========================================
export interface Thread {
  id: string;
  title: string;
  preview: string;
  timestamp: number;
  status: 'active' | 'archived';
}

// ========================================
// 侧边栏
// ========================================
export type SidebarTab = 'project' | 'files';

export type ActivityBarTab = 'files' | 'new-task' | 'search' | 'skills' | 'bots';

export type LayoutMode = 'three-column' | 'document' | 'horizontal-split';

export interface WorkspaceTab {
  id: string;
  filePath: string;
  title: string;
}

export interface WorkspaceLeafNode {
  type: 'leaf';
  id: string;
  tabs: WorkspaceTab[];
  activeTabId: string | null;
}

export type WorkspaceSplitDirection = 'horizontal' | 'vertical';

export type WorkspaceSplitSide = 'left' | 'right' | 'top' | 'bottom';

export interface WorkspaceSplitNode {
  type: 'split';
  id: string;
  direction: WorkspaceSplitDirection;
  /** 第一侧 pane 占比 0–1（horizontal=宽度，vertical=高度） */
  ratio: number;
  first: WorkspaceNode;
  second: WorkspaceNode;
}

export type WorkspaceNode = WorkspaceLeafNode | WorkspaceSplitNode;

export interface ChatMention {
  path: string;
  content?: string;
}

// ========================================
// 设置页
// ========================================
export type SettingsPage =
  | 'general'
  | 'model'
  | 'approval'
  | 'mcp'
  | 'skills'
  | 'plugins'
  | 'appearance'
  | 'shortcuts'
  | 'about';

// ========================================
// 主题
// ========================================
export type ThemeMode = 'light' | 'dark' | 'system';

// ========================================
// MCP 服务器
// ========================================
export interface McpServer {
  id: string;
  name: string;
  status: 'starting' | 'ready' | 'failed' | 'cancelled';
  toolCount?: number;
  oauth?: boolean;
}

// ========================================
// 工作阶段
// ========================================
export type WorkStage = 'search' | 'compare' | 'review' | 'draft';

export interface StageInfo {
  id: WorkStage;
  label: string;
  status: 'pending' | 'active' | 'completed';
}

// ========================================
// 审批请求
// ========================================
export interface ApprovalRequest {
  id: string;
  type: 'command' | 'file' | 'mcp';
  title: string;
  description: string;
  riskLevel: 'low' | 'medium' | 'high';
  details?: string;
  command: string;
  cwd?: string;
  isDangerous?: boolean;
  /** app-server 下发的 JSON-RPC 请求 id */
  rpcId?: string | number;
  rpcMethod?: string;
  /** item/permissions/requestApproval 原始参数 */
  permissionsParams?: PermissionsRequestApprovalParams;
  /** 写入工作区的审批摘要 Markdown 路径 */
  documentPath?: string;
}

/** Agent 工具向用户提问（item/tool/requestUserInput） */
export interface ToolUserInputRequest {
  rpcId: string | number;
  threadId: string;
  turnId: string;
  itemId: string;
  questions: ToolRequestUserInputQuestion[];
}

// ========================================
// MCP 请求弹窗
// ========================================
export interface ElicitationField {
  id: string;
  label: string;
  type: 'text' | 'password' | 'select';
  description?: string;
  options?: string[];
  value?: string;
}

export interface McpElicitation {
  rpcId: string | number;
  rpcMethod: 'mcpServer/elicitation/request';
  serverName: string;
  message: string;
  mode: 'form' | 'url';
  fields: ElicitationField[];
  url?: string;
  elicitationId?: string;
}

// ========================================
// OAuth 等待状态
// ========================================
export type OAuthWaitingPhase = 'idle' | 'waiting' | 'completed' | 'failed';

export interface OAuthWaitingState {
  serverName: string;
  authUrl?: string;
  phase?: OAuthWaitingPhase;
  error?: string;
}

// ========================================
// 待办事项
// ========================================
export interface TodoItem {
  id: string;
  text: string;
  completed: boolean;
  createdAt: number;
}

// ========================================
// 应用状态（用于全局 Store）
// ========================================
/** 桌面端启动 / 接入 app-server 的阶段 */
export type BootPhase =
  | 'idle'
  | 'checking'
  | 'no_cli'
  | 'connecting'
  | 'ready'
  | 'fault';

export interface AppState {
  // —— 主题 ——
  theme: ThemeMode;
  isDark: boolean;

  // —— 布局 ——
  leftSidebarOpen: boolean;
  leftSidebarWidth: number;
  agentPanelOpen: boolean;
  agentPanelWidth: number;
  /** Agent 面板内左侧会话列表是否展开 */
  threadListOpen: boolean;
  sidebarTab: SidebarTab;
  activityBarTab: ActivityBarTab | null;
  /** 资源管理器内左侧项目栏是否展开 */
  projectRailOpen: boolean;
  layoutMode: LayoutMode;
  chatPanelHeight: number;
  workspaceRoot: WorkspaceNode | null;
  focusedPaneId: string | null;
  chatMentions: ChatMention[];

  // —— 连接 ——
  connectionStatus: ConnectionStatus;
  bootPhase: BootPhase;
  bootError: string | null;
  bcipInstalled: boolean | null;
  /** 解析后的 bcip 路径（PATH 或 sidecar） */
  bcipPath: string | null;
  bcipVersion: string | null;
  bcipSource: 'path' | 'sidecar' | 'workspace' | null;
  appServerTransport: string | null;
  /** Boot 过程日志（可展开） */
  bootLogLines: string[];
  currentModel: string;
  /** 状态栏用量（RPC 驱动；未连接时为 null 则显示占位） */
  usageMeter: UsageMeterSnapshot | null;

  // —— 线程 ——
  threads: Thread[];
  currentThreadId: string | null;
  messages: Message[];
  isStreaming: boolean;

  // —— 设置 ——
  settingsOpen: boolean;
  settingsPage: SettingsPage;

  // —— 工作区 ——
  /** app-server thread/start.cwd：当前选中项目根或文件所属目录 */
  workspaceCwd: string | null;
  currentFile: string | null;
  stages: StageInfo[];
  todos: TodoItem[];
  todoDockOpen: boolean;
  todoDockHeight: number;
  terminalOverlayOpen: boolean;

  // —— 覆盖层 ——
  approvalDialog: ApprovalRequest | null;
  toolUserInput: ToolUserInputRequest | null;
  commandPaletteOpen: boolean;
  mcpElicitation: McpElicitation | null;
  oAuthWaiting: OAuthWaitingState | null;
}

// ========================================
// App Action（用于 useReducer）
// ========================================
export type AppAction =
  // 主题
  | { type: 'SET_THEME'; payload: ThemeMode }
  | { type: 'TOGGLE_DARK' }
  // 布局
  | { type: 'TOGGLE_LEFT_SIDEBAR' }
  | { type: 'SET_LEFT_SIDEBAR_OPEN'; payload: boolean }
  | { type: 'SET_LEFT_SIDEBAR_WIDTH'; payload: number }
  | { type: 'TOGGLE_AGENT_PANEL' }
  | { type: 'SET_AGENT_PANEL_OPEN'; payload: boolean }
  | { type: 'SET_AGENT_PANEL_WIDTH'; payload: number }
  | { type: 'TOGGLE_THREAD_LIST' }
  | { type: 'SET_THREAD_LIST_OPEN'; payload: boolean }
  | { type: 'SET_SIDEBAR_TAB'; payload: SidebarTab }
  | { type: 'SET_ACTIVITY_BAR_TAB'; payload: ActivityBarTab | null }
  | { type: 'TOGGLE_PROJECT_RAIL' }
  | { type: 'SET_PROJECT_RAIL_OPEN'; payload: boolean }
  | { type: 'SET_LAYOUT_MODE'; payload: LayoutMode }
  | { type: 'SET_CHAT_PANEL_HEIGHT'; payload: number }
  | { type: 'OPEN_TAB'; payload: WorkspaceTab }
  | { type: 'CLOSE_TAB'; payload: string }
  | { type: 'SET_ACTIVE_TAB'; payload: { paneId: string; tabId: string } }
  | { type: 'SET_FOCUSED_PANE'; payload: string }
  | { type: 'SPLIT_TAB'; payload: { paneId: string; tabId: string; side: WorkspaceSplitSide } }
  | { type: 'SPLIT_ACTIVE_TAB'; payload: { side: WorkspaceSplitSide } }
  | { type: 'OPEN_TAB_SPLIT'; payload: { tab: WorkspaceTab; side: WorkspaceSplitSide } }
  | { type: 'MERGE_FOCUSED_PANE' }
  | { type: 'COLLAPSE_WORKSPACE_SPLITS' }
  | { type: 'MOVE_TAB'; payload: { tabId: string; targetPaneId: string; insertIndex?: number } }
  | { type: 'REORDER_TAB'; payload: { paneId: string; tabId: string; toIndex: number } }
  | { type: 'SET_WORKSPACE_SPLIT_RATIO'; payload: { splitId: string; ratio: number } }
  | { type: 'INSERT_CHAT_MENTION'; payload: ChatMention }
  | { type: 'CLEAR_CHAT_MENTIONS' }
  | { type: 'CLEAR_SESSION' }
  // 连接
  | { type: 'SET_CONNECTION_STATUS'; payload: ConnectionStatus }
  | { type: 'SET_BOOT_PHASE'; payload: BootPhase }
  | { type: 'SET_BOOT_ERROR'; payload: string | null }
  | { type: 'SET_BCIP_INSTALLED'; payload: boolean }
  | {
      type: 'SET_BCIP_RESOLUTION';
      payload: {
        path: string | null;
        version: string | null;
        source: 'path' | 'sidecar' | 'workspace' | null;
      };
    }
  | { type: 'APPEND_BOOT_LOG'; payload: string }
  | { type: 'CLEAR_BOOT_LOG' }
  | { type: 'SET_APP_SERVER_TRANSPORT'; payload: string | null }
  | { type: 'SET_CURRENT_MODEL'; payload: string }
  | { type: 'SET_USAGE_METER'; payload: UsageMeterSnapshot | null }
  // 线程
  | { type: 'SET_THREADS'; payload: Thread[] }
  | { type: 'SET_CURRENT_THREAD'; payload: string | null }
  | { type: 'ADD_MESSAGE'; payload: Message }
  | { type: 'UPDATE_MESSAGE'; payload: { id: string; updates: Partial<Message> } }
  | { type: 'SET_STREAMING'; payload: boolean }
  | { type: 'SET_MESSAGES'; payload: Message[] }
  | { type: 'APPEND_MESSAGE_DELTA'; payload: { id: string; delta: string } }
  // 设置
  | { type: 'OPEN_SETTINGS'; payload: SettingsPage }
  | { type: 'CLOSE_SETTINGS' }
  | { type: 'SET_SETTINGS_PAGE'; payload: SettingsPage }
  // 工作区
  | { type: 'SET_WORKSPACE_CWD'; payload: string | null }
  | { type: 'SWITCH_PROJECT'; payload: string }
  | { type: 'REMOVE_THREAD'; payload: string }
  | { type: 'SET_CURRENT_FILE'; payload: string | null }
  | { type: 'SET_STAGES'; payload: StageInfo[] }
  | { type: 'UPDATE_STAGE'; payload: { id: WorkStage; status: StageInfo['status'] } }
  | { type: 'SET_TODOS'; payload: TodoItem[] }
  | { type: 'ADD_TODO'; payload: TodoItem }
  | { type: 'UPDATE_TODO'; payload: { id: string; completed: boolean } }
  | { type: 'DELETE_TODO'; payload: string }
  | { type: 'TOGGLE_TODO_DOCK' }
  | { type: 'SET_TODO_DOCK_HEIGHT'; payload: number }
  | { type: 'TOGGLE_TERMINAL_OVERLAY' }
  | { type: 'SET_TERMINAL_OVERLAY_OPEN'; payload: boolean }
  // 覆盖层
  | { type: 'SET_APPROVAL_DIALOG'; payload: ApprovalRequest | null }
  | { type: 'SET_TOOL_USER_INPUT'; payload: ToolUserInputRequest | null }
  | { type: 'TOGGLE_COMMAND_PALETTE' }
  | { type: 'SET_MCP_ELICITATION'; payload: McpElicitation | null }
  | { type: 'SET_OAUTH_WAITING'; payload: OAuthWaitingState | null };

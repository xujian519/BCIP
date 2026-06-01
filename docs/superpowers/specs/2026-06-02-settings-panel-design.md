# TUI 统一设置面板设计

## 背景

当前 TUI 的设置功能分散在多个独立的 `/` 斜杠命令弹窗中（`/model`、`/permissions`、`/theme`、`/experimental` 等），且许多 ConfigToml 中已有的配置项（动画、通知、工具提示等）没有任何 UI 暴露。用户需要记忆不同的命令，操作路径长，体验不统一。

## 目标

构建一个统一的 `/settings` 设置面板，将所有配置项按分类集中管理，同时保留现有斜杠命令作为快捷入口。

## 分期方案

### 第一期：设置面板骨架 + 简单开关（当前范围）

- 搭建 `/settings` 统一设置面板（分页 Tab 布局）
- 新增通用页签：动画开关、通知设置、原始输出模式、工具提示开关
- 将现有 `/model`、`/permissions`、`/theme` 等弹窗内容以页签形式整合进面板
- 后续期次见本设计文档末尾的"后续规划"部分

### 第二期：LLM + 搜索引擎 + MCP/Skill 配置

涵盖模型提供商选择、API Key 管理、搜索配置、MCP 服务器状态管理、Skill 启停等。**不在本文档范围内。**

### 第三期：知识体系配置

涵盖知识库、知识图谱、语义库的接入和配置。**不在本文档范围内。**

## 交互设计

### 入口

- **主要入口**：`/settings` 斜杠命令
- **未来可加**：快捷键（如 `Cmd+,`）

### 面板结构

```
┌──────────────────────────────────────────┐
│ /settings 设置                            │
│ ┌──────┬──────┬──────┬──────┐            │
│ │ 通用  │ 模型  │ 权限  │ 外观  │            │
│ ├──────┴──────┴──────┴──────┤            │
│ │                             │            │
│ │  ☐ 启用动画                │            │
│ │  ☑ 显示启动提示            │            │
│ │  ☐ 默认原始输出模式        │            │
│ │  ...                        │            │
│ └─────────────────────────────┘           │
└──────────────────────────────────────────┘
```

### 快捷键

| 按键 | 操作 |
|------|------|
| Tab / Shift+Tab | 切换页签 |
| ↑↓ | 在设置项间导航 |
| Enter / Space | 切换开关 / 确认选择 |
| Esc | 关闭面板 |

### 交互原则

- **变更即时生效**：通过 `config/batchWrite` RPC 写入，遵循现有 ExperimentalFeaturesView 模式
- **乐观更新**：UI 先切换状态，异步写入失败时通过 Toast 通知
- **无需确认**：Esc 直接关闭，不弹"是否保存"对话框
- **互斥**：属于 BottomPaneView，与其他弹出视图互斥

## 架构设计

### 文件变更

**新增 5 个文件：**

| 文件 | 职责 |
|------|------|
| `bottom_pane/settings_panel.rs` | 主组件：布局 + Tab 切换 + 键盘事件路由 |
| `bottom_pane/settings_general.rs` | 通用页签：动画/通知/工具提示等开关 |
| `bottom_pane/settings_model.rs` | 模型页签：包装现有 ModelPopup 内容 |
| `bottom_pane/settings_permissions.rs` | 权限页签：包装现有 PermissionPopup 内容 |
| `bottom_pane/settings_appearance.rs` | 外观页签：包装现有 Theme/PetPicker 内容 |

**修改文件：**

| 文件 | 修改内容 |
|------|---------|
| `slash_command.rs` | 新增 `/settings` 命令 |
| `slash_dispatch.rs` | 路由 `/settings` 到打开面板 |
| `chatwidget/settings.rs` | 新增 SettingsState，可选面板状态 |
| `bottom_pane/mod.rs` | 注册 SettingsPanel |

### 组件树

```
SettingsPanel (BottomPaneView)
├── SettingsGeneral    — 动画/通知/工具提示等
├── SettingsModel      — 复用 ModelPopup 内容
├── SettingsPermissions — 复用 PermissionPopup 内容
└── SettingsAppearance — 复用 Theme/PetPicker 内容
```

### 数据流

```
用户切换开关
  → SettingsPanel 更新内部状态（乐观更新）
  → AppEvent 发送到 EventDispatch
  → config_persistence.rs 调用 config/batchWrite RPC
  → App 内 Config 更新 + 线程同步
```

与现有 ExperimentalFeaturesView、MemoriesSettingsView 完全一致的模式。

### 状态模型

```rust
struct SettingsPanel {
    active_tab: SettingsTab,
    items: Vec<SettingsItem>,
    focus_index: usize,
}

enum SettingsTab {
    General,
    Model,
    Permissions,
    Appearance,
}

struct SettingsItem {
    label: String,
    description: String,
    kind: SettingsItemKind,  // Toggle | Select | Action
    enabled: bool,
}
```

## 第一期页签详情

### 通用（General）

| 配置项 | Config 字段 | 类型 |
|--------|-------------|------|
| 启用动画 | `tui.animations` | bool toggle |
| 显示启动提示 | `tui.show_tooltips` | bool toggle |
| 默认原始输出模式 | `tui.raw_output_mode` | bool toggle |
| 代理回合完成时通知 | `tui.notification_settings` | bool toggle |
| 审批请求时通知 | `tui.notification_settings` | bool toggle |

### 模型（Model）

复用现有 `/model` 弹窗内容：
- 模型选择列表
- 推理强度调整
- 服务层级选择

### 权限（Permissions）

复用现有 `/permissions` 弹窗内容：
- 审批策略选择
- 沙箱模式选择
- 权限配置列表

### 外观（Appearance）

复用现有 `/theme` 和 `/pets` 弹窗内容：
- 主题选择
- 宠物选择
- 状态栏配置

## 边界情况

- **弹窗互斥**：打开 SettingsPanel 时关闭其他 BottomPaneView，反之亦然
- **写入失败**：乐观更新 UI，异步写入失败时通过 Toast 反馈
- **关闭策略**：Esc 直接关闭，变更已即时写入
- **空状态**：某页签无设置项时显示"暂无设置项"

## 测试策略

- **单元测试**：Tab 切换、焦点导航、Enter/Space 切换逻辑
- **快照测试**：每个页签的 insta 快照，不同焦点位置的快照
- **集成测试**：`/settings` 命令路由、开关变更 → AppEvent → config 写入链路

## 后续规划

### 第二期

- LLM 配置页签：模型提供商、API Key、模型选择、上下文窗口
- 搜索引擎配置页签：默认搜索模式、API Key、自定义端点
- MCP/Skill 管理页签：服务器状态、Skill 启停

### 第三期

- 知识库页签：向量数据库连接、索引管理
- 知识图谱页签：图谱启用/禁用、重建索引
- 语义库页签：语义引擎选择、嵌入模型选择

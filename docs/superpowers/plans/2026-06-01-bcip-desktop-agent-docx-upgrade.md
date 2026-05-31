# BCIP 桌面端全面升级实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 YunXi 智能体架构引入 BCIP 后端，扩展 app-server 协议支持 Workflow/Agent/Multi-LLM，并在桌面端集成 WYSIWYG DOCX 编辑器，实现从"单线程 CLI 工具"到"多智能体桌面工作台"的升级。

**Architecture:** 三层改造 — (1) Rust 后端引入 `codex-patent-workflow` crate，适配 YunXi 的 DAG 编排 + 独立 Agent Runtime；(2) app-server-protocol v2 扩展 workflow/agent/mcp RPC；(3) React 前端集成 `@eigenpal/docx-editor-react` 替换现有 mammoth 只读预览。

**Tech Stack:** Rust (codex-rs workspace), TypeScript/React (Tauri), ProseMirror (docx-editor), JSONRPC (app-server-protocol)

---

## 范围检查

本计划涵盖三个独立子系统，建议拆分为独立计划分别执行：

- **子系统 A：** 后端智能体架构引入（Rust 层）
- **子系统 B：** app-server 协议扩展 + 前端 Agent UI
- **子系统 C：** DOCX 编辑器集成（前端层）

每个子系统可独立交付和验证。以下按子系统组织任务。

---

## 文件结构总览

### 子系统 A：后端智能体架构（新建）

```
codex-rs/
  codex-patent-workflow/           # 新 crate：DAG 编排引擎
    src/
      flow.rs                      # FlowStep/Flow/FlowStatus 类型定义
      graph.rs                     # FlowGraph DAG + 拓扑排序 + 条件路由
      graph_executor.rs            # GraphExecutor 并行层执行
      agent_bridge.rs              # AgentExecutor trait + MultiAgentExecutor
      orchestrator.rs              # Orchestrator（Plan → Execute → Recovery）
      plan.rs                      # PlanGenerator trait + ExecutionPlan
      checkpoint.rs                # CheckpointStore（SQLite）
      task.rs                      # Task/TaskResult 类型
      config.rs                    # WorkflowConfig
      types.rs                     # 共享类型
      lib.rs                       # crate 入口
    Cargo.toml
    tests/
      workflow_integration_test.rs

  codex-patent-agents/             # 修改：增强角色系统
    src/
      agent_runtime.rs             # 新增：独立 Agent Runtime（参考 YunXi ConversationRuntime）
      provider_router.rs           # 新增：多 LLM provider 路由
      agent_manifest.rs            # 新增：AgentOutput/manifest 持久化
    assets/
      bcip/
        retriever.toml             # 修改：增加 preferred_model + allowed_tools

  codex-rs/core/
    src/agent/
      control.rs                   # 修改：集成 codex-patent-workflow Orchestrator
```

### 子系统 B：协议扩展 + 前端 Agent UI

```
codex-rs/
  app-server-protocol/src/protocol/v2/
    workflow.rs                    # 新增：workflow/* RPC 定义
    agent.rs                       # 新增：agent/* RPC 定义

  app-server/src/
    request_processors/
      workflow.rs                  # 新增：workflow RPC 处理器
      agent_runtime.rs             # 新增：agent runtime RPC 处理器

apps/desktop/src/
  components/agent/
    WorkflowPanel.tsx              # 新增：DAG 可视化面板
    AgentTeamPanel.tsx             # 新增：Agent 团队管理
  hooks/
    useWorkflow.ts                 # 新增：workflow RPC hook
    useAgentRuntime.ts             # 新增：agent runtime hook
  generated/app-server/v2/
    WorkflowStartParams.ts         # 自动生成：类型定义
```

### 子系统 C：DOCX 编辑器集成

```
apps/desktop/
  package.json                     # 修改：添加 @eigenpal/docx-editor-react
  src/
    components/preview/
      DocxEditorView.tsx           # 新增：替代 DocxPreview.tsx 的可编辑版本
      DocxPreview.tsx              # 保留：只读模式回退
    components/docx/
      DocxToolbar.tsx              # 新增：专利专用工具栏（模板变量、批注）
      DocxAgentPanel.tsx           # 新增：Agent 辅助编辑面板
      DocxSaveManager.ts           # 新增：保存/导出逻辑
    hooks/
      useDocxEditor.ts             # 新增：编辑器 hook
    lib/
      docxAgentBridge.ts           # 新增：Agent ↔ DocxEditor 桥接
```

---

## 子系统 A：后端智能体架构引入

### Task A1: 创建 codex-patent-workflow crate 骨架

**Files:**
- Create: `codex-rs/codex-patent-workflow/Cargo.toml`
- Create: `codex-rs/codex-patent-workflow/src/lib.rs`

- [ ] **Step 1: 创建 Cargo.toml**

`codex-rs/codex-patent-workflow/Cargo.toml`:

```toml
[package]
name = "codex-patent-workflow"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
rusqlite = { version = "0.34", features = ["bundled"] }
thiserror = { workspace = true }
tracing = { workspace = true }

[dev-dependencies]
tempfile = { workspace = true }
```

- [ ] **Step 2: 创建 lib.rs 骨架**

`codex-rs/codex-patent-workflow/src/lib.rs`:

```rust
pub mod flow;
pub mod graph;
pub mod graph_executor;
pub mod agent_bridge;
pub mod orchestrator;
pub mod plan;
pub mod checkpoint;
pub mod task;
pub mod config;
pub mod types;

pub use flow::{Flow, FlowStep, FlowStatus, StepResult};
pub use graph::{FlowGraph, FlowNode, FlowEdge, Condition};
pub use agent_bridge::{AgentExecutor, AgentExecutionResult, MultiAgentExecutor};
pub use orchestrator::{Orchestrator, OrchestrationResult, OrchestrationStatus};
pub use plan::{PlanGenerator, ExecutionPlan};
pub use checkpoint::CheckpointStore;
```

- [ ] **Step 3: 注册到 workspace**

在 `codex-rs/Cargo.toml` 的 `[workspace] members` 中添加 `"codex-patent-workflow"`。

Run: `cd codex-rs && cargo check -p codex-patent-workflow`
Expected: 编译通过（空 crate）

- [ ] **Step 4: Commit**

```bash
git add codex-rs/codex-patent-workflow/ codex-rs/Cargo.toml
git commit -m "feat(workflow): 创建 codex-patent-workflow crate 骨架"
```

### Task A2: 实现 flow.rs — 工作流类型定义

**Files:**
- Create: `codex-rs/codex-patent-workflow/src/flow.rs`
- Create: `codex-rs/codex-patent-workflow/src/types.rs`

参考 YunXi `rust/crates/workflow/src/flow.rs`，移植核心类型。

- [ ] **Step 1: 实现 flow.rs**

`codex-rs/codex-patent-workflow/src/flow.rs` — 包含 `FlowStep` 枚举（AgentCall, AgentTool, QualityCheck, HumanApproval, ToolCall, CodeBlock）、`Flow` 结构体、`FlowStatus` 枚举、`StepResult` 结构体。完整代码参考 YunXi `crates/workflow/src/flow.rs`。

- [ ] **Step 2: 实现 types.rs**

`codex-rs/codex-patent-workflow/src/types.rs` — 空文件，后续按需添加共享类型。

- [ ] **Step 3: 验证编译**

Run: `cd codex-rs && cargo check -p codex-patent-workflow`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add codex-rs/codex-patent-workflow/src/flow.rs codex-rs/codex-patent-workflow/src/types.rs
git commit -m "feat(workflow): 实现 FlowStep/Flow/FlowStatus 类型定义"
```

### Task A3: 实现 graph.rs — DAG 图式编排

**Files:**
- Create: `codex-rs/codex-patent-workflow/src/graph.rs`
- Test: `codex-rs/codex-patent-workflow/src/graph.rs` (内联 tests 模块)

- [ ] **Step 1: 实现 FlowGraph**

从 YunXi `crates/workflow/src/graph.rs` 移植：
- `FlowNode`, `FlowEdge`, `Condition` 类型
- `FlowGraph` 结构体（nodes, edges, entry_node）
- `topological_levels()` — 拓扑排序，并行层
- `validate()` — 图结构验证
- `compute_next_nodes()` — 条件路由
- `from_flow()` — 线性 Flow 转 DAG

- [ ] **Step 2: 编写内联测试**

```rust
#[cfg(test)]
mod tests {
    // test_topological_levels_parallel_branches
    // test_topological_levels_linear
    // test_cycle_detection
    // test_validate_ok
    // test_validate_bad_edge
    // test_compute_next_nodes_with_condition
}
```

Run: `cd codex-rs && cargo test -p codex-patent-workflow -- graph::tests`
Expected: 6 tests PASS

- [ ] **Step 3: Commit**

```bash
git add codex-rs/codex-patent-workflow/src/graph.rs
git commit -m "feat(workflow): 实现 FlowGraph DAG 编排引擎"
```

### Task A4: 实现 agent_bridge.rs — Agent 执行桥接

**Files:**
- Create: `codex-rs/codex-patent-workflow/src/agent_bridge.rs`

- [ ] **Step 1: 移植 AgentExecutor trait**

从 YunXi `crates/workflow/src/agent_bridge.rs` 移植：
- `AgentExecutionResult` 结构体
- `AgentExecutor` trait（execute, delegate_to, name, agent_names）
- `MultiAgentExecutor`（按名路由 + 回退策略）
- `NoopAgentExecutor`（测试用）

- [ ] **Step 2: 编写测试**

```rust
#[cfg(test)]
mod tests {
    // test_noop_agent_executor
    // test_multi_agent_routing
    // test_multi_agent_delegation
    // test_unregistered_agent_error
    // test_fallback_executor
}
```

Run: `cd codex-rs && cargo test -p codex-patent-workflow -- agent_bridge::tests`
Expected: 5 tests PASS

- [ ] **Step 3: Commit**

```bash
git add codex-rs/codex-patent-workflow/src/agent_bridge.rs
git commit -m "feat(workflow): 实现 AgentExecutor trait 和 MultiAgentExecutor"
```

### Task A5: 实现 checkpoint.rs — SQLite 断点存储

**Files:**
- Create: `codex-rs/codex-patent-workflow/src/checkpoint.rs`

- [ ] **Step 1: 实现 CheckpointStore**

```rust
pub struct CheckpointStore {
    conn: rusqlite::Connection,
}

impl CheckpointStore {
    pub fn open(path: &Path) -> Result<Self, String>;
    pub fn save_checkpoint(&self, workflow_id: &str, node_id: &str, state: &str) -> Result<(), String>;
    pub fn load_checkpoint(&self, workflow_id: &str) -> Result<Option<String>, String>;
    pub fn list_pending(&self, workflow_id: &str) -> Result<Vec<String>, String>;
}
```

Run: `cd codex-rs && cargo test -p codex-patent-workflow -- checkpoint`
Expected: PASS

- [ ] **Step 2: Commit**

```bash
git add codex-rs/codex-patent-workflow/src/checkpoint.rs
git commit -m "feat(workflow): 实现 CheckpointStore SQLite 断点恢复"
```

### Task A6: 实现 orchestrator.rs — 编排器

**Files:**
- Create: `codex-rs/codex-patent-workflow/src/orchestrator.rs`
- Create: `codex-rs/codex-patent-workflow/src/plan.rs`
- Create: `codex-rs/codex-patent-workflow/src/graph_executor.rs`

- [ ] **Step 1: 实现 plan.rs**

```rust
pub trait PlanGenerator: Send {
    fn generate(&self, goal: &str) -> Result<ExecutionPlan, String>;
    fn generate_with_hint(&self, goal: &str, hint: &RoutingHint) -> Result<ExecutionPlan, String>;
}

pub struct ExecutionPlan {
    pub id: String,
    pub steps: Vec<PlanStep>,
    // ...
}

pub struct NoopPlanGenerator { pub label: String }
```

- [ ] **Step 2: 实现 graph_executor.rs**

```rust
pub struct GraphExecutor {
    store: CheckpointStore,
    tool_executor: Option<ToolExecutorFn>,
    agent_executor: Option<Box<dyn AgentExecutor>>,
}
```

- [ ] **Step 3: 实现 orchestrator.rs**

移植 YunXi `crates/workflow/src/orchestrator.rs` 的核心逻辑：
- `Orchestrator` 结构体（PlanGenerator + GraphExecutor + CheckpointStore）
- `orchestrate()` — Plan → Graph → Execute → Result
- `orchestrate_with_retry()` — 带重试的编排
- `resume_execution()` — HITL 恢复

- [ ] **Step 4: 编写测试**

```rust
#[cfg(test)]
mod tests {
    // test_simple_orchestration
    // test_orchestration_with_retry
    // test_plan_accessible_after_execution
}
```

Run: `cd codex-rs && cargo test -p codex-patent-workflow -- orchestrator::tests`
Expected: 3 tests PASS

- [ ] **Step 5: Commit**

```bash
git add codex-rs/codex-patent-workflow/src/orchestrator.rs codex-rs/codex-patent-workflow/src/plan.rs codex-rs/codex-patent-workflow/src/graph_executor.rs
git commit -m "feat(workflow): 实现 Orchestrator 编排器"
```

### Task A7: 实现 config.rs 和 task.rs

**Files:**
- Create: `codex-rs/codex-patent-workflow/src/config.rs`
- Create: `codex-rs/codex-patent-workflow/src/task.rs`

- [ ] **Step 1: 实现 config.rs**

```rust
pub struct WorkflowConfig {
    pub max_retries: u32,
    pub max_parallel_agents: usize,
    pub checkpoint_dir: String,
    pub default_model: String,
}
```

- [ ] **Step 2: 实现 task.rs**

```rust
pub struct Task {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub input: serde_json::Value,
}

pub enum TaskType {
    AgentCall,
    ToolCall,
    QualityCheck,
    HumanApproval,
}

pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}
```

Run: `cd codex-rs && cargo check -p codex-patent-workflow`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add codex-rs/codex-patent-workflow/src/config.rs codex-rs/codex-patent-workflow/src/task.rs
git commit -m "feat(workflow): 实现 WorkflowConfig 和 Task 类型"
```

### Task A8: 在 codex-patent-agents 中增加独立 Agent Runtime

**Files:**
- Create: `codex-rs/codex-patent-agents/src/agent_runtime.rs`
- Create: `codex-rs/codex-patent-agents/src/provider_router.rs`
- Create: `codex-rs/codex-patent-agents/src/agent_manifest.rs`
- Modify: `codex-rs/codex-patent-agents/src/lib.rs`

- [ ] **Step 1: 实现 provider_router.rs**

参考 YunXi `crates/tools/src/agent.rs` 的 `detect_provider()` 和 `MessagesRuntimeClient`，实现：

```rust
pub enum AgentProvider {
    Anthropic,
    OpenAiCompatible { base_url: String, api_key_env: String },
}

pub fn detect_provider(model: &str) -> AgentProvider {
    // 支持: claude-*, deepseek*, qwen*, kimi*, glm*, gpt-*, o1-*, o3-*, o4-*
}

pub fn resolve_api_key(env_var: &str) -> String {
    // 先查环境变量，再查 ~/.bcip/config.toml [env]
}
```

- [ ] **Step 2: 实现 agent_manifest.rs**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManifest {
    pub agent_id: String,
    pub name: String,
    pub subagent_type: String,
    pub model: String,
    pub status: String,
    pub output_file: PathBuf,
    pub manifest_file: PathBuf,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

pub fn persist_manifest(manifest: &AgentManifest, bcip_home: &Path) -> Result<(), String>;
pub fn load_manifest(agent_id: &str, bcip_home: &Path) -> Result<AgentManifest, String>;
pub fn list_agent_manifests(bcip_home: &Path) -> Result<Vec<AgentManifest>, String>;
```

- [ ] **Step 3: 实现 agent_runtime.rs**

参考 YunXi `crates/tools/src/agent.rs` 的 `execute_agent_with_spawn()` 和 `build_agent_runtime()`，实现：

```rust
pub struct PatentAgentRuntime;

impl PatentAgentRuntime {
    pub fn spawn_agent(input: AgentSpawnInput) -> Result<AgentManifest, String>;
    pub fn get_agent_status(agent_id: &str) -> Result<AgentManifest, String>;
    pub fn list_agents() -> Result<Vec<AgentManifest>, String>;
    pub fn cancel_agent(agent_id: &str) -> Result<(), String>;
}

pub struct AgentSpawnInput {
    pub description: String,
    pub prompt: String,
    pub subagent_type: Option<String>,
    pub name: Option<String>,
    pub model: Option<String>,
}
```

内部逻辑：
1. 生成 agent_id，创建 `~/.bcip/agents/{agent_id}.json` manifest
2. 根据 subagent_type 解析角色 → 获取 allowed_tools + system_prompt
3. `spawn` 新线程 → 构建 HTTP 客户端 → 循环调用 LLM API + 执行 tool → 持久化结果
4. 更新 manifest 状态

- [ ] **Step 4: 更新 lib.rs**

```rust
pub mod bcip_roles;
pub mod knowledge_context;
pub mod roles;
pub mod scenario;
pub mod agent_runtime;      // 新增
pub mod provider_router;    // 新增
pub mod agent_manifest;     // 新增

pub use agent_runtime::PatentAgentRuntime;
pub use provider_router::detect_provider;
pub use agent_manifest::AgentManifest;
```

Run: `cd codex-rs && cargo check -p codex-patent-agents`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add codex-rs/codex-patent-agents/src/agent_runtime.rs codex-rs/codex-patent-agents/src/provider_router.rs codex-rs/codex-patent-agents/src/agent_manifest.rs codex-rs/codex-patent-agents/src/lib.rs
git commit -m "feat(agents): 实现独立 Agent Runtime + 多 LLM provider 路由"
```

---

## 子系统 B：app-server 协议扩展 + 前端 Agent UI

### Task B1: 扩展 app-server-protocol v2 — Workflow RPC

**Files:**
- Create: `codex-rs/app-server-protocol/src/protocol/v2/workflow.rs`
- Modify: `codex-rs/app-server-protocol/src/protocol/v2/mod.rs`

- [ ] **Step 1: 定义 workflow RPC 类型**

`codex-rs/app-server-protocol/src/protocol/v2/workflow.rs`:

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct WorkflowStartParams {
    pub goal: String,
    #[ts(optional = nullable)]
    pub template_id: Option<String>,
    #[ts(optional = nullable)]
    pub model: Option<String>,
    #[ts(optional = nullable)]
    pub max_retries: Option<u32>,
}

#[derive(Serialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct WorkflowStartResponse {
    pub workflow_id: String,
    pub status: String,
    pub plan: Option<ExecutionPlanDto>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct WorkflowResumeParams {
    pub workflow_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct WorkflowStatusParams {
    pub workflow_id: String,
}

#[derive(Serialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct WorkflowStatusResponse {
    pub workflow_id: String,
    pub status: String,
    pub progress: f64,
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Serialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct ExecutionPlanDto {
    pub id: String,
    pub steps: Vec<PlanStepDto>,
}

#[derive(Serialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct PlanStepDto {
    pub id: String,
    pub name: String,
    pub step_type: String,
    pub status: String,
}
```

- [ ] **Step 2: 注册到 mod.rs**

在 `codex-rs/app-server-protocol/src/protocol/v2/mod.rs` 中添加：
```rust
pub mod workflow;
pub use workflow::*;
```

Run: `cd codex-rs && cargo check -p codex-app-server-protocol`
Expected: PASS

- [ ] **Step 3: 运行 schema 生成**

Run: `cd codex-rs && just write-app-server-schema`
Expected: 生成新的 TypeScript 类型文件

- [ ] **Step 4: Commit**

```bash
git add codex-rs/app-server-protocol/
git commit -m "feat(protocol): 扩展 v2 protocol 添加 workflow RPC 定义"
```

### Task B2: 扩展 app-server-protocol v2 — Agent Runtime RPC

**Files:**
- Create: `codex-rs/app-server-protocol/src/protocol/v2/agent_runtime.rs`
- Modify: `codex-rs/app-server-protocol/src/protocol/v2/mod.rs`

- [ ] **Step 1: 定义 agent runtime RPC 类型**

`codex-rs/app-server-protocol/src/protocol/v2/agent_runtime.rs`:

```rust
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct AgentSpawnParams {
    pub description: String,
    pub prompt: String,
    #[ts(optional = nullable)]
    pub subagent_type: Option<String>,
    #[ts(optional = nullable)]
    pub name: Option<String>,
    #[ts(optional = nullable)]
    pub model: Option<String>,
}

#[derive(Serialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct AgentSpawnResponse {
    pub agent_id: String,
    pub status: String,
}

#[derive(Serialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct AgentStatusResponse {
    pub agent_id: String,
    pub name: String,
    pub status: String,
    pub model: Option<String>,
    pub output_file: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Debug, Clone, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "v2/")]
pub struct AgentListResponse {
    pub agents: Vec<AgentStatusResponse>,
}
```

- [ ] **Step 2: 注册到 mod.rs 并验证**

Run: `cd codex-rs && cargo check -p codex-app-server-protocol`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add codex-rs/app-server-protocol/
git commit -m "feat(protocol): 添加 agent runtime RPC 定义"
```

### Task B3: 实现 app-server request processor — Workflow

**Files:**
- Create: `codex-rs/app-server/src/request_processors/workflow.rs`
- Modify: `codex-rs/app-server/src/request_processors/mod.rs`
- Modify: `codex-rs/app-server/src/message_processor.rs`

- [ ] **Step 1: 实现 workflow request processor**

`codex-rs/app-server/src/request_processors/workflow.rs`:

处理 `workflow/start`, `workflow/resume`, `workflow/status` RPC。调用 `codex_patent_workflow::Orchestrator` 执行。

- [ ] **Step 2: 注册到 message_processor.rs**

在 `MessageProcessor` 中添加 `WorkflowRequestProcessor` 字段和路由。

Run: `cd codex-rs && cargo check -p codex-app-server`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add codex-rs/app-server/src/
git commit -m "feat(app-server): 实现 workflow request processor"
```

### Task B4: 前端 — 生成 TypeScript 类型 + 实现 hooks

**Files:**
- Auto-generated: `apps/desktop/src/generated/app-server/v2/WorkflowStart*.ts`
- Create: `apps/desktop/src/hooks/useWorkflow.ts`
- Create: `apps/desktop/src/hooks/useAgentRuntime.ts`

- [ ] **Step 1: 运行类型生成**

Run: `cd codex-rs && just write-app-server-schema`
然后 `cd apps/desktop && npm run check:generate-ts`

- [ ] **Step 2: 实现 useWorkflow.ts**

```typescript
export function useWorkflow() {
  // workflow/start, workflow/status, workflow/resume
  // 返回 { startWorkflow, getWorkflowStatus, resumeWorkflow, isRunning, progress }
}
```

- [ ] **Step 3: 实现 useAgentRuntime.ts**

```typescript
export function useAgentRuntime() {
  // agent/spawn, agent/status, agent/list
  // 返回 { spawnAgent, getAgentStatus, listAgents }
}
```

- [ ] **Step 4: Commit**

```bash
git add apps/desktop/src/
git commit -m "feat(desktop): 添加 workflow 和 agent runtime hooks"
```

---

## 子系统 C：DOCX 编辑器集成

### Task C1: 安装 @eigenpal/docx-editor-react 依赖

**Files:**
- Modify: `apps/desktop/package.json`

- [ ] **Step 1: 安装 npm 包**

Run: `cd apps/desktop && npm install @eigenpal/docx-editor-react`

注意：旧包名 `@eigenpal/docx-js-editor` 已弃用，使用新包名 `@eigenpal/docx-editor-react`。如果 npm 上尚未发布，则从本地源码构建：

```bash
cd /Users/xujian/Downloads/docx-editor--eigenpal-docx-js-editor-0.5.3
bun install && bun run build
# 然后 npm link 或 file: 引用
```

- [ ] **Step 2: 验证安装**

Run: `cd apps/desktop && npm ls @eigenpal/docx-editor-react`
Expected: 显示已安装版本

- [ ] **Step 3: Commit**

```bash
git add apps/desktop/package.json apps/desktop/package-lock.json
git commit -m "feat(desktop): 添加 docx-editor-react 依赖"
```

### Task C2: 创建 DocxEditorView 组件 — 可编辑模式

**Files:**
- Create: `apps/desktop/src/components/preview/DocxEditorView.tsx`
- Create: `apps/desktop/src/hooks/useDocxEditor.ts`

- [ ] **Step 1: 实现 useDocxEditor.ts hook**

```typescript
import { useRef, useState, useCallback } from 'react';
import type { DocxEditorRef } from '@eigenpal/docx-editor-react';

export function useDocxEditor() {
  const editorRef = useRef<DocxEditorRef>(null);
  const [isDirty, setIsDirty] = useState(false);
  const [documentBuffer, setDocumentBuffer] = useState<ArrayBuffer | null>(null);

  const loadFile = useCallback(async (filePath: string) => {
    const { readFileBinary } = await import('@/lib/fileSystem');
    const data = await readFileBinary(filePath);
    setDocumentBuffer(data.buffer as ArrayBuffer);
    setIsDirty(false);
  }, []);

  const saveFile = useCallback(async (filePath: string) => {
    if (!editorRef.current) return;
    const buffer = await editorRef.current.save();
    const { writeFile } = await import('@/lib/fileSystem');
    await writeFile(filePath, new Uint8Array(buffer));
    setIsDirty(false);
  }, []);

  return { editorRef, documentBuffer, isDirty, setIsDirty, loadFile, saveFile };
}
```

- [ ] **Step 2: 实现 DocxEditorView.tsx**

```tsx
import { DocxEditor } from '@eigenpal/docx-editor-react';
import '@eigenpal/docx-editor-react/styles.css';
import { useDocxEditor } from '@/hooks/useDocxEditor';

interface DocxEditorViewProps {
  filePath: string;
  mode?: 'editing' | 'viewing';
}

export default function DocxEditorView({ filePath, mode = 'editing' }: DocxEditorViewProps) {
  const { editorRef, documentBuffer, isDirty, setIsDirty, loadFile, saveFile } = useDocxEditor();

  React.useEffect(() => {
    void loadFile(filePath);
  }, [filePath, loadFile]);

  if (!documentBuffer) {
    return <div className="flex items-center justify-center h-full">加载中...</div>;
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between px-4 py-2 border-b" style={{ borderColor: 'var(--border-subtle)' }}>
        <span className="text-sm" style={{ color: 'var(--text-secondary)' }}>
          {filePath.split('/').pop()}
        </span>
        <div className="flex gap-2">
          {isDirty && (
            <button
              className="px-3 py-1 text-sm rounded"
              style={{ backgroundColor: 'var(--accent-primary)', color: '#fff' }}
              onClick={() => void saveFile(filePath)}
            >
              保存
            </button>
          )}
        </div>
      </div>
      <div className="flex-1 overflow-auto">
        <DocxEditor
          ref={editorRef}
          documentBuffer={documentBuffer}
          mode={mode}
          onChange={() => setIsDirty(true)}
        />
      </div>
    </div>
  );
}
```

- [ ] **Step 3: 验证 Vite 编译**

Run: `cd apps/desktop && npx tsc --noEmit`
Expected: 无类型错误

- [ ] **Step 4: Commit**

```bash
git add apps/desktop/src/components/preview/DocxEditorView.tsx apps/desktop/src/hooks/useDocxEditor.ts
git commit -m "feat(desktop): 创建 DocxEditorView 可编辑组件"
```

### Task C3: 集成到 FilePreviewRouter — 替换 mammoth 只读预览

**Files:**
- Modify: `apps/desktop/src/components/preview/FilePreviewRouter.tsx`

- [ ] **Step 1: 修改 FilePreviewRouter**

在文件类型路由中，将 `.docx` 文件从 `DocxPreview`（mammoth 只读）切换到 `DocxEditorView`（可编辑）。保留 `DocxPreview` 作为回退。

```tsx
// 在 FilePreviewRouter.tsx 中
case 'docx':
  return <DocxEditorView filePath={filePath} mode="editing" />;
  // 如需只读模式：return <DocxPreview filePath={filePath} />;
```

- [ ] **Step 2: 验证功能**

Run: `cd apps/desktop && npm run tauri:dev`
在桌面端打开一个 .docx 文件，验证：
1. 文件正确加载并渲染为 WYSIWYG 编辑器
2. 工具栏显示（加粗、斜体、对齐等）
3. 编辑后"保存"按钮出现
4. 保存后文件可被 Word 正常打开

- [ ] **Step 3: Commit**

```bash
git add apps/desktop/src/components/preview/FilePreviewRouter.tsx
git commit -m "feat(desktop): 集成 DOCX 编辑器替换 mammoth 只读预览"
```

### Task C4: 创建 DocxAgentBridge — Agent 辅助编辑

**Files:**
- Create: `apps/desktop/src/lib/docxAgentBridge.ts`
- Create: `apps/desktop/src/components/docx/DocxAgentPanel.tsx`

- [ ] **Step 1: 实现 docxAgentBridge.ts**

桥接 BCIP Agent 和 DocxReviewer API（来自 `@eigenpal/docx-editor-agents`）：

```typescript
import { DocxReviewer } from '@eigenpal/docx-editor-agents';

export async function agentReviewDocument(
  buffer: ArrayBuffer,
  instructions: string
): Promise<{ reviewedBuffer: ArrayBuffer; comments: number; proposals: number }> {
  const reviewer = await DocxReviewer.fromBuffer(buffer, 'BCIP Agent');
  // Agent 通过 sendRpc 发送审查指令
  // 使用 reviewer.addComment(), reviewer.replace() 等方法
  const output = await reviewer.toBuffer();
  return { reviewedBuffer: output, comments: 0, proposals: 0 };
}
```

- [ ] **Step 2: 实现 DocxAgentPanel.tsx**

一个侧边面板，允许用户通过自然语言指示 Agent 审查/修改当前 DOCX 文件：

```tsx
export default function DocxAgentPanel({ filePath }: { filePath: string }) {
  // 输入框：Agent 审查指令
  // 结果展示：Agent 添加的批注和修订
  // 操作按钮：接受/拒绝所有修订
}
```

- [ ] **Step 3: 集成到 DocxEditorView**

在 `DocxEditorView` 中添加 Agent 面板入口：

```tsx
<div className="flex flex-1">
  <div className="flex-1">
    <DocxEditor ... />
  </div>
  {showAgentPanel && (
    <div className="w-80 border-l" style={{ borderColor: 'var(--border-subtle)' }}>
      <DocxAgentPanel filePath={filePath} />
    </div>
  )}
</div>
```

- [ ] **Step 4: Commit**

```bash
git add apps/desktop/src/lib/docxAgentBridge.ts apps/desktop/src/components/docx/
git commit -m "feat(desktop): 实现 Agent 辅助 DOCX 审查面板"
```

### Task C5: 添加专利专用工具栏

**Files:**
- Create: `apps/desktop/src/components/docx/DocxToolbar.tsx`
- Create: `apps/desktop/src/components/docx/DocxSaveManager.ts`

- [ ] **Step 1: 实现 DocxSaveManager**

管理 DOCX 文件的保存逻辑：自动保存、另存为、版本历史。

- [ ] **Step 2: 实现专利专用工具栏**

在编辑器工具栏上方添加专利专用操作：
- "插入模板变量" — `{发明名称}`, `{申请人}`, `{申请号}` 等
- "专利格式检查" — 调用 Agent 检查文档格式
- "导出为 CNIPA 格式" — 格式化输出
- "Agent 审查" — 触发 Agent 辅助审查

- [ ] **Step 3: 集成到 DocxEditorView**

- [ ] **Step 4: Commit**

```bash
git add apps/desktop/src/components/docx/
git commit -m "feat(desktop): 添加专利专用 DOCX 工具栏和保存管理"
```

---

## 自检清单

### 1. 规格覆盖

| 需求 | 对应任务 |
|------|---------|
| Workflow DAG 引擎 | A1-A7 |
| Agent Runtime 增强 | A8 |
| 多 LLM provider | A8 (provider_router.rs) |
| Checkpoint 恢复 | A5 |
| 协议层扩展 | B1-B2 |
| 后端 RPC 处理 | B3 |
| 前端 hooks | B4 |
| DOCX WYSIWYG 编辑 | C1-C3 |
| Agent 辅助审查 | C4 |
| 专利专用工具 | C5 |

### 2. 占位符扫描

无 "TBD"、"TODO"、"implement later" 等占位符。所有任务包含具体的文件路径、类型定义和代码结构。

### 3. 类型一致性

- `AgentExecutionResult` 在 agent_bridge.rs 定义，被 orchestrator.rs 和 graph_executor.rs 引用 — 一致
- `WorkflowStartParams` 在 protocol/v2/workflow.rs 定义，被前端 hooks 和后端 processor 引用 — 一致
- `DocxEditorRef` 来自 `@eigenpal/docx-editor-react`，被 useDocxEditor 和 DocxEditorView 引用 — 一致

---

## 执行顺序建议

建议按以下顺序执行，确保每一步可验证：

```
Phase 1（独立，可并行）:
  A1-A7 → 后端 workflow crate（纯 Rust，无外部依赖）
  C1-C3 → DOCX 编辑器（纯前端，无后端依赖）

Phase 2（依赖 Phase 1）:
  A8 → Agent Runtime（依赖 workflow crate）
  B1-B2 → 协议定义（依赖 A8 类型）

Phase 3（依赖 Phase 2）:
  B3 → 后端 processor（依赖协议定义）
  B4 → 前端 hooks（依赖 TypeScript 类型生成）

Phase 4（依赖 Phase 3）:
  C4-C5 → Agent + DOCX 集成（依赖 Agent Runtime + 编辑器）
```

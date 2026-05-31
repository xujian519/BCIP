# 智能体接入架构差距分析：YunXi vs BCIP

> **For agentic workers:** 本文档为研究分析文档，不是实施计划。用于指导后续实施规划的优先级排序。

## 目标

对比 YunXi 和 BCIP 两个项目的智能体接入架构，识别 BCIP 的关键差距，为后续开发提供清晰的路线图。

---

## 1. 架构对比总览

### 1.1 项目定位

| 维度 | YunXi | BCIP |
|------|-------|------|
| 定位 | 独立智能体产品，自研全栈 | 基于 Codex 的专利领域定制版 |
| Rust crates | ~18 个精简 crate（`crates/`） | ~126 个 crate（含 Codex 完整生态） |
| Python 层 | 完整的 Python 适配层（`src/`） | 无 Python 层 |
| 构建系统 | Cargo only | Cargo + Bazel |
| 前端 | TUI（自研） | TUI + 桌面端（Tauri） |

### 1.2 智能体核心架构对比

| 能力 | YunXi | BCIP | 差距 |
|------|-------|------|------|
| **Agent Runtime** | 自研 `ConversationRuntime<ApiClient, ToolExecutor>` | 依赖 Codex `core::Session` + `TurnContext` | BCIP 受限于 Codex runtime |
| **多 LLM 支持** | 内置 Anthropic + OpenAI Compatible（DeepSeek/Qwen/Kimi/GLM/GPT） | 通过 Codex model-provider | BCIP 无独立多模型路由 |
| **Agent 角色系统** | `AgentRole` enum + XML 系统提示 + TOML 配置 | `codex-patent-agents` TOML 配置（9 角色） | 架构类似，BCIP 已有 |
| **Agent 执行引擎** | `execute_agent_with_spawn()` — 线程级子 Agent | `core::agent::control` — 线程级 | BCIP 已有，更成熟 |
| **Tool Dispatch** | `dispatch.rs` 统一分发（150+ 行） | `codex-patent-tools` HashMap 注册（50+ 工具） | 架构类似，BCIP 更丰富 |
| **Agent Team** | `agent_team.rs` — TeamCreate/Delete/List | 无 | **BCIP 缺失** |
| **Workflow DAG** | `workflow` crate — FlowGraph/Orchestrator/Checkpoint | 无对等实现 | **BCIP 缺失** |
| **Multi-Agent Executor** | `MultiAgentExecutor` — 按名路由 + Agent-as-Tool 委托 | `core::agent::control` 基于线程 | BCIP 有但非 DAG 感知 |
| **Plan Generator** | `PlanGenerator` trait + LLM 驱动 | 无 | **BCIP 缺失** |
| **Graph Executor** | `GraphExecutor` — DAG 拓扑 + 并行层执行 | 无 | **BCIP 缺失** |
| **Checkpoint/Recovery** | `CheckpointStore`（SQLite） | 无 | **BCIP 缺失** |
| **Agent-as-Tool** | `AgentExecutor::delegate_to()` | `core::tools::handlers::agent_jobs` | BCIP 已有（CSV 批量模式） |
| **Skill 系统** | `skill.rs` + `resolve_includes()` | `codex-patent-skills`（12 个技能） | 架构类似 |
| **宪法规则引擎** | `constitutional-engine` crate | `codex-patent-constitutional`（35 条规则） | BCIP 更完整 |
| **知识图谱** | `knowledge` crate | `codex-patent-knowledge`（SQLite KG + 法律库） | BCIP 更完整 |

---

## 2. YunXi 智能体接入核心模式分析

### 2.1 Agent 生命周期（YunXi 独有）

YunXi 的 `agent.rs` 实现了完整的 Agent 生命周期：

```
AgentInput → execute_agent_with_spawn()
  ├─ make_agent_id() → 唯一 ID
  ├─ agent_store_dir() → ~/.yunxi-agents/
  ├─ build_agent_system_prompt() → 加载角色 XML + Athena 能力
  ├─ allowed_tools_for_subagent() → 按角色配置工具白名单
  ├─ spawn_agent_job() → 新线程
  │   └─ run_agent_job()
  │       ├─ build_agent_runtime() → ConversationRuntime
  │       ├─ runtime.run_turn() → 执行对话轮次
  │       └─ persist_agent_terminal_state()
  └─ AgentOutput（manifest JSON + output markdown）
```

**关键设计：**
- 每个 Agent 有独立的 manifest（JSON）和 output（Markdown）
- Agent 状态持久化到文件系统
- 支持多 LLM provider 自动检测（`detect_provider()`）
- 按 Agent 角色动态配置 allowed_tools
- 子 Agent 拥有独立的 `ConversationRuntime` 和 `Session`

### 2.2 Workflow DAG 编排（YunXi 独有）

```
Orchestrator
  ├─ PlanGenerator（trait）— LLM 驱动的计划生成
  │   └─ generate_with_hint() → ExecutionPlan
  ├─ ExecutionPlan → FlowGraph（DAG）
  │   └─ topological_levels() → 并行层级
  ├─ GraphExecutor
  │   ├─ ToolExecutorFn — 工具执行
  │   ├─ AgentExecutor — Agent 执行桥接
  │   └─ CodeExecutor — 代码执行
  └─ CheckpointStore（SQLite）— 断点恢复
```

**FlowStep 类型：**
- `AgentCall` — 调用指定 Agent
- `AgentTool` — Agent 作为工具
- `QualityCheck` — 质量门控
- `HumanApproval` — HITL 人工审批
- `ToolCall` — 直接工具调用
- `CodeBlock` — 代码执行

**条件路由：** `Condition::Always / OnSuccess / OnFailure`

### 2.3 Agent Bridge（Multi-Agent 委托）

```
AgentExecutor（trait）
  ├─ execute() → 直接执行 Agent
  ├─ delegate_to() → Agent-as-Tool 模式
  └─ agent_names() → 发现可用 Agent

MultiAgentExecutor
  ├─ register() → 注册 Agent 执行器
  ├─ with_default_fallback() → 未注册时的回退
  └─ 按 name 路由到具体 AgentExecutor
```

### 2.4 Provider 多模型路由（YunXi 独有）

```rust
detect_provider(model) → SubagentProvider
  ├─ Anthropic（Claude 系列）
  ├─ OpenAI Compatible
  │   ├─ DeepSeek（api.deepseek.com）
  │   ├─ Qwen（dashscope.aliyuncs.com）
  │   ├─ Kimi/Moonshot（api.moonshot.cn）
  │   ├─ GLM/ChatGLM（open.bigmodel.cn）
  │   └─ GPT/o1/o3/o4（api.openai.com）
  └─ 默认回退 Anthropic
```

每个 provider 使用独立的流式解析逻辑（SSE）。

---

## 3. BCIP 差距分析

### 3.1 关键缺失能力

#### GAP-1: Workflow DAG 编排引擎
- **YunXi 有：** 完整的 `workflow` crate（FlowGraph + Orchestrator + Checkpoint）
- **BCIP 缺失：** 无 DAG 编排能力，所有任务线性执行
- **影响：** 无法实现复杂的并行专利分析流水线（如"检索并行 → 汇聚分析 → 审查模拟"）
- **优先级：** 高

#### GAP-2: 独立 Agent Runtime（多模型）
- **YunXi 有：** `ConversationRuntime` + 内置多 LLM provider
- **BCIP 现状：** 依赖 Codex `core::Session`，模型路由由 Codex 控制
- **影响：** BCIP 的 Agent 只能使用 Codex 配置的单一模型，无法按角色选择不同 LLM
- **优先级：** 中高

#### GAP-3: Agent Team 管理
- **YunXi 有：** `AgentTeam` 持久化存储（JSON），支持创建/删除/列出
- **BCIP 缺失：** 无 Team 概念
- **影响：** 无法定义和管理 Agent 团队（如"检索组 + 分析组 + 撰写组"）
- **优先级：** 中

#### GAP-4: Plan Generator（LLM 驱动的任务规划）
- **YunXi 有：** `PlanGenerator` trait + LLM 生成计划
- **BCIP 缺失：** 无计划生成能力
- **影响：** 用户需要手动指定步骤，无法自动分解复杂专利任务
- **优先级：** 中高

#### GAP-5: Checkpoint/Recovery
- **YunXi 有：** SQLite CheckpointStore，支持 HITL 恢复
- **BCIP 缺失：** 无断点恢复
- **影响：** 长时间运行的专利分析任务中断后无法恢复
- **优先级：** 中

### 3.2 BCIP 已有但需增强的能力

#### ENHANCE-1: Agent 角色系统
- BCIP 已有 9 个 TOML 角色，但缺少：
  - XML 系统提示模板（YunXi 用 XML + `<include>` 引入技能）
  - `preferred_model()` 按角色选模型
  - 角色级 `allowed_tools` 白名单

#### ENHANCE-2: Tool Dispatch
- BCIP 的 `codex-patent-tools` 有 50+ 工具，但：
  - 缺少 `ToolSearch`（YunXi 的工具发现）
  - 缺少 `Skill` 工具（运行时技能加载）
  - 工具注册是静态的，无法动态发现

#### ENHANCE-3: Agent Job 管理
- BCIP 有 `agent_jobs.rs`（CSV 批量），但：
  - 缺少单个 Agent 的 manifest/output 持久化
  - 缺少 Agent 状态文件系统
  - 无 `AgentOutput` 结构化输出

---

## 4. 建议实施路线图

### Phase 1: Agent Runtime 增强（1-2 周）
- [ ] 在 `codex-patent-agents` 中增加 `allowed_tools` 白名单
- [ ] 增加 `preferred_model()` 角色配置
- [ ] Agent 状态持久化（manifest JSON + output markdown）
- [ ] `ToolSearch` 工具发现

### Phase 2: Workflow DAG 引擎（2-3 周）
- [ ] 新建 `codex-patent-workflow` crate
- [ ] 实现 `FlowGraph`（DAG + 拓扑排序 + 条件路由）
- [ ] 实现 `Orchestrator`（Plan → Execute → Recovery）
- [ ] 实现 `CheckpointStore`（SQLite）
- [ ] 内置专利领域 Workflow 模板

### Phase 3: 多模型 Agent（1-2 周）
- [ ] Agent 角色级模型选择
- [ ] 多 LLM provider 路由（DeepSeek/Qwen/Kimi/GLM）
- [ ] 独立 `ConversationRuntime` per Agent

### Phase 4: Plan Generator + Agent Team（2 周）
- [ ] `PlanGenerator` trait + LLM 驱动计划
- [ ] `AgentTeam` 管理（创建/注册/路由）
- [ ] `MultiAgentExecutor` 集成

---

## 5. 结论

BCIP 的**领域深度**（50+ 工具、35 条宪法规则、知识图谱）远超 YunXi，但在**智能体编排能力**上存在结构性差距：

1. **最大差距：** Workflow DAG 引擎（YunXi 的核心竞争力）
2. **第二差距：** 独立 Agent Runtime + 多模型路由
3. **第三差距：** 计划生成 + Agent Team

YunXi 的智能体接入模式是**自底向上**的（Runtime → Agent → Workflow），每个层次解耦且可独立测试。BCIP 应借鉴这一设计，在 `codex-patent-*` crate 层面构建类似的分层架构，同时保留 Codex 底座的优势。

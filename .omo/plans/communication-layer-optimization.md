# BCIP 通信层优化计划

> 基于 13 层通信架构深度分析，按 P0-P3 优先级拆解为 4 个 Phase、28 个任务单元。
> 每个任务含：目标、涉及文件、具体改动、验收标准、依赖关系。

---

## 架构现状概要

### 当前问题

| # | 问题 | 严重度 | 影响 |
|---|------|--------|------|
| P0-1 | 双轨 Agent 体系：BCIP Core AgentControl vs PatentAgentRuntime(std::thread)，完全隔离 | 严重 | Agent 无法跨体系通信，工具/状态不共享 |
| P0-2 | IM 通道(5 crate)编译通过但零接入 | 严重 | Telegram/Feishu 适配器闲置 |
| P1-1 | 无 AgentBus：Agent 间通信仅 InputQueue mailbox，无广播/订阅 | 高 | 跨角色协作靠手动传递 |
| P1-2 | 配置缺失：ConfigToml 无 `[im]` 段，无环境变量规范 | 高 | IM 无法配置启用 |
| P2-1 | 仅 polling 无 webhook：Telegram/Feishu 适配器只有长轮询 | 中 | 无法接收推送通知 |
| P2-2 | DingTalk 定义未实现：`codex-im-protocol` 有枚举但无适配器 | 中 | 缺少国内主流 IM |
| P3-1 | 无消息重试/加密/可观测性 | 低 | 生产环境可靠性不足 |

### 关键接口分析

#### 三层 Trait 层次
```
ToolExecutor<ToolInvocation>  (codex-tools)
    ↑ implements
CoreToolRuntime               (codex-core, internal)
    ↑ implements
PatentToolHandler             (codex-core, patent_tools mod)
```

#### AgentControl 核心方法
- `spawn_agent_internal()` — 预留 slot → 准备元数据 → 创建线程 → 发送初始 Op → 启动完成监听
- `send_inter_agent_communication()` — 包装 InterAgentCommunication → send_op()
- `subscribe_status()` — 返回 watch::Receiver<AgentStatus>

#### PatentAgentRuntime 现状
- `spawn_agent()` → 创建 manifest → 写入文件 → std::thread::spawn
- `run_agent_job()` → 加载 TOML → 构建 prompt → 调用 LLM → 记录反馈
- **无**与 AgentControl 的任何连接

---

## Phase 1: 双轨 Agent 合并 [P0]

> 目标：消除 PatentAgentRuntime 的独立线程体系，将其接入 BCIP Core AgentControl。

### 1.1 创建 PatentAgentBridge 工具

**目标**：让 Patent Agent Runtime 的 `spawn_agent()` 能通过 AgentControl 创建子 Agent。

**涉及文件**：
- `codex-rs/core/src/tools/handlers/patent_tools/` — 新建 `agent_bridge.rs`
- `codex-rs/core/src/tools/handlers/patent_tools/mod.rs` — 注册新工具
- `codex-rs/core/src/tools/spec_plan.rs` — 添加工具注册入口

**具体改动**：

```rust
// 新文件: codex-rs/core/src/tools/handlers/patent_tools/agent_bridge.rs

/// PatentAgentBridge 将 PatentAgentRuntime 的 agent 调用映射为 BCIP AgentControl 的 spawn 操作。
/// 它实现了 ToolExecutor<ToolInvocation>，因此可以作为一个"工具"被 LLM 调用。
pub struct PatentAgentBridge {
    agent_control: AgentControl,
    session_source: SessionSource,
    tool_registry: Arc<PatentToolRegistry>,
}

impl PatentAgentBridge {
    /// 从当前会话上下文创建 bridge
    pub fn from_session(services: &SessionServices) -> Result<Self, BridgeError> {
        let role = services.session_source.get_agent_role();
        Ok(Self {
            agent_control: services.agent_control.clone(),
            session_source: services.session_source.clone(),
            tool_registry: PatentToolRegistry::for_role(role),
        })
    }
}

impl ToolExecutor<ToolInvocation> for PatentAgentBridge {
    fn tool_name(&self) -> ToolName { ToolName::from_static("patent_spawn_agent") }
    
    fn spec(&self) -> ToolSpec {
        ToolSpec::new(
            "patent_spawn_agent",
            "Spawn a patent specialist agent to handle a subtask",
            json!({
                "type": "object",
                "properties": {
                    "role": { "type": "string", "enum": [
                        "retriever", "analyzer", "writer", "novelty_checker",
                        "creativity_checker", "infringement_checker", "invalidity_checker",
                        "reviewer", "quality_checker"
                    ]},
                    "task": { "type": "string", "description": "Task description for the agent" },
                    "fork_turns": { "type": "integer", "description": "Optional: fork N turns of history" }
                },
                "required": ["role", "task"]
            })
        )
    }

    async fn handle(&self, invocation: ToolInvocation) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let args: SpawnPatentAgentArgs = parse_args(&invocation)?;
        
        // 1. 解析角色 → 应用 role config
        let role = PatentAgentRole::from_str(&args.role)?;
        let mut config = self.base_config_for_role(&role);
        
        // 2. 构建 SessionSource::SubAgent
        let spawn_source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id: self.current_thread_id(),
            depth: self.session_source.child_depth() + 1,
            agent_path: self.agent_control.resolve_path(&args.role),
            agent_nickname: self.agent_control.generate_nickname(),
            agent_role: args.role.clone(),
        });
        
        // 3. 注册角色感知工具
        let tools = PatentToolHandler::create_adapters_for_role(Some(role));
        
        // 4. 通过 AgentControl spawn
        let agent_id = self.agent_control.spawn_agent_with_metadata(
            config, Op::UserInput { items: vec![args.task.into()], .. },
            spawn_source, SpawnOptions::default()
        ).await.map_err(|e| FunctionCallError::ToolError(e.to_string()))?;
        
        // 5. 返回 agent ID 给调用者
        Ok(Box::new(FunctionToolOutput::from_text(
            json!({ "agent_id": agent_id.to_string(), "role": args.role }).to_string()
        )))
    }
}
```

**验收标准**：
- [ ] `cargo check -p codex-core` 通过
- [ ] `patent_spawn_agent` 工具出现在 spec_plan 注册列表中
- [ ] 角色感知工具裁剪（primary=Direct, secondary=Deferred, 其他=Hidden）正确映射
- [ ] 单元测试：bridge 能解析 9 种 PatentAgentRole
- [ ] 集成测试：spawn 的 agent 通过 AgentControl 生命周期管理

**依赖**：无（Phase 1 首个任务）

---

### 1.2 迁移 PatentAgentRuntime 到 AgentControl spawn

**目标**：将 `codex-patent-agents/src/agent_runtime/spawn.rs` 中的 `std::thread::spawn` 替换为 `AgentControl::spawn_agent_internal()`。

**涉及文件**：
- `codex-rs/codex-patent-agents/src/agent_runtime/spawn.rs` — 核心改动
- `codex-rs/codex-patent-agents/src/agent_runtime/mod.rs` — 接口调整
- `codex-rs/codex-patent-workflow/src/agent_bridge.rs` — 更新 AgentExecutor 实现

**具体改动**：

```rust
// spawn.rs 改动要点：

// 旧代码：
pub fn spawn_agent(job: AgentJob) -> Result<JoinHandle<()>> {
    std::thread::spawn(move || { run_agent_job(job) })
}

// 新代码：
pub async fn spawn_agent_via_control(
    agent_control: &AgentControl,
    job: AgentJob,
    session_source: &SessionSource,
) -> Result<ThreadId> {
    let role = PatentAgentRole::from_str(&job.role_name)?;
    let config = build_config_for_role(&role, &job.model_overrides);
    
    let spawn_source = SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
        parent_thread_id: session_source.thread_id(),
        depth: session_source.child_depth() + 1,
        agent_path: agent_control.resolve_path(&job.role_name),
        agent_nickname: agent_control.generate_nickname(),
        agent_role: job.role_name.clone(),
    });
    
    let initial_op = Op::UserInput {
        items: vec![ResponseInputItem::Message {
            role: Role::User,
            content: vec![ContentItem::Text { text: job.task_description }],
            ..Default::default()
        }],
        environments: vec![],
        is_meta: false,
        source: InputSource::User,
    };
    
    agent_control.spawn_agent_with_metadata(
        config, initial_op, spawn_source, SpawnOptions::default()
    ).await
}
```

**验收标准**：
- [ ] `spawn_agent_via_control` 能通过 AgentControl 创建子 agent
- [ ] 子 agent 的工具集按角色域映射正确裁剪
- [ ] 子 agent 完成后通过 Completion Watcher 通知父 agent
- [ ] 原有 `run_agent_job` 的 LLM 调用逻辑保持不变
- [ ] `cargo check -p codex-patent-agents -p codex-patent-workflow` 通过

**依赖**：1.1

---

### 1.3 角色配置统一

**目标**：将 Patent Agent 的 9 个 TOML 角色配置统一到 BCIP Core 的 role 系统中。

**涉及文件**：
- `codex-rs/core/src/agent/role.rs` — 已有 9 个 patent role，验证完整性
- `codex-rs/core/src/agent/builtins/` — 创建 patent role TOML 文件
- `codex-rs/codex-patent-agents/src/roles.rs` — 确认域映射一致性

**具体改动**：

1. 在 `codex-rs/core/src/agent/builtins/patent/` 创建 9 个 TOML 文件：
   - `retriever.toml`, `analyzer.toml`, `writer.toml`, `novelty_checker.toml`, ...
   - 每个文件含：`model`, `temperature`, `developer_instructions`, `tool_domains`

2. 验证 `role.rs` 中已有的 `inject_patent_role_context()` 正确注入知识上下文

3. 确认 `PatentAgentRole::primary_domains/secondary_domains` 与 `ToolDomain` 枚举完全对应

**验收标准**：
- [ ] 9 个 TOML 文件创建完毕，格式正确
- [ ] `resolve_role_config("retriever")` 能正确加载配置
- [ ] 域映射表验证通过（见下方检查清单）
- [ ] `just write-config-schema` 生成的 schema 包含新角色

**依赖**：1.1

---

### 1.4 删除旧 PatentAgentRuntime 线程管理代码

**目标**：清理 `std::thread::spawn` 相关代码，统一使用 AgentControl。

**涉及文件**：
- `codex-rs/codex-patent-agents/src/agent_runtime/spawn.rs` — 移除旧 spawn 函数
- `codex-rs/codex-patent-agents/src/agent_runtime/mod.rs` — 移除 thread handles 管理
- `codex-rs/codex-patent-workflow/src/` — 更新 workflow 对 agent 生命周期管理的调用

**验收标准**：
- [ ] 无 `std::thread::spawn` 用于 agent 创建
- [ ] 所有 agent 通过 AgentControl 管理
- [ ] `cargo clippy` 无新警告
- [ ] 现有测试全部通过

**依赖**：1.2, 1.3

---

### 1.5 双轨合并集成测试

**目标**：端到端验证 Patent Agent 通过 AgentControl 正常工作。

**涉及文件**：
- `codex-rs/codex-patent-agents/tests/` — 新增集成测试
- `codex-rs/codex-patent-workflow/tests/` — 新增集成测试

**测试场景**：
1. Retriever agent spawn → 搜索工具可用 → 返回结果
2. Writer agent spawn → 撰写工具可用 → 返回结果
3. Retriever → Analyzer 链式委托 → 结果传递
4. Agent 异常时 Completion Watcher 正确通知父 agent
5. 超过 max_threads 限制时 spawn 失败

**验收标准**：
- [ ] 5 个集成测试全部通过
- [ ] 无 panic、无 dead code 警告

**依赖**：1.4

---

## Phase 2: IM 通道接入 [P0]

> 目标：将 codex-im-* 5 个 crate 集成到 app-server 运行时，实现可配置的 IM 通知。

### 2.1 添加 IM 配置到 ConfigToml

**目标**：在配置系统中增加 `[im]` 配置段。

**涉及文件**：
- `codex-rs/config/src/types.rs` — 新增 `ImConfigToml` 结构体
- `codex-rs/config/src/config_toml.rs` — 新增 `pub im: Option<ImConfigToml>` 字段
- `codex-rs/config/src/lib.rs` — 导出新类型
- `codex-rs/core/src/config/mod.rs` — 运行时 Config 添加 im 字段 + wire

**具体改动**：

```rust
// codex-rs/config/src/types.rs 新增：

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct ImConfigToml {
    /// 全局启用/禁用
    #[serde(default)]
    pub enabled: bool,
    
    /// Telegram 配置
    pub telegram: Option<TelegramConfigToml>,
    
    /// 飞书配置
    pub feishu: Option<FeishuConfigToml>,
    
    /// 钉钉配置（未来）
    pub dingtalk: Option<DingtalkConfigToml>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct TelegramConfigToml {
    pub bot_token: String,
    #[serde(default)]
    pub allowed_users: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct FeishuConfigToml {
    pub app_id: String,
    pub app_secret: String,
    #[serde(default)]
    pub allowed_users: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
#[schemars(deny_unknown_fields)]
pub struct DingtalkConfigToml {
    pub app_key: Option<String>,
    pub app_secret: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}
```

**ConfigToml 新增字段**（`config_toml.rs`）：
```rust
/// IM (Instant Messaging) notification configuration.
#[serde(default)]
pub im: Option<ImConfigToml>,
```

**Config wire**（`core/src/config/mod.rs`，约 line 3381）：
```rust
im: cfg.im,
```

**验收标准**：
- [ ] `cargo check -p codex-config -p codex-core` 通过
- [ ] `just write-config-schema` 生成包含 `[im]` 段的 schema
- [ ] TOML 反序列化测试通过
- [ ] `"im"` 添加到 `PROJECT_LOCAL_CONFIG_DENYLIST`（安全考虑）

**依赖**：无（可与 Phase 1 并行）

---

### 2.2 创建 IM Service 初始化器

**目标**：在 app-server 启动流程中集成 IM 服务。

**涉及文件**：
- `codex-rs/codex-im-bridge/src/lib.rs` — 可能需要调整 BridgeConfig 以接受 ConfigToml
- `codex-rs/app-server/src/lib.rs` — 在 `run_main_with_transport_options` 中添加 IM 初始化
- `codex-rs/app-server/Cargo.toml` — 添加 codex-im-* 依赖

**具体改动**：

```rust
// app-server/src/lib.rs, 在 run_main_with_transport_options 中
// 约 line 815 之后（MessageProcessor 创建之后）添加：

// IM Service 初始化
if let Some(ref im_config) = config.im {
    if im_config.enabled {
        im_service = init_im_service(im_config, &state_db).await?;
        // spawn 为后台任务
        let im_handle = tokio::spawn(async move {
            if let Err(e) = im_service.run().await {
                tracing::error!("IM service error: {e}");
            }
        });
        // 注册 shutdown hook
        shutdown_hooks.push(im_handle);
    }
}

async fn init_im_service(
    config: &ImConfigToml,
    state_db: &StateDbHandle,
) -> Result<ImService, ImError> {
    let bridge_config = BridgeConfig {
        server_url: "ws://127.0.0.1:3456".to_string(),  // 可配置化
        session_db_path: state_db.path().to_path_buf(),
        ..Default::default()
    };
    let bridge = Arc::new(ImBridge::new(bridge_config).await?);
    
    let mut adapters: Vec<Box<dyn ImAdapter>> = vec![];
    
    if let Some(ref tg) = config.telegram {
        if tg.enabled {
            adapters.push(Box::new(
                TelegramAdapter::new(tg.into(), bridge.clone())
            ));
        }
    }
    
    if let Some(ref fs) = config.feishu {
        if fs.enabled {
            adapters.push(Box::new(
                FeishuAdapter::new(fs.into(), bridge.clone())
            ));
        }
    }
    
    Ok(ImService::new(bridge, adapters))
}
```

**验收标准**：
- [ ] `cargo check -p codex-app-server` 通过
- [ ] IM 配置启用时，adapter 被正确创建
- [ ] IM 配置禁用时，跳过初始化（无错误）
- [ ] adapter 的 `run()` 作为后台 tokio task 运行
- [ ] app-server shutdown 时 IM service 正确关闭

**依赖**：2.1

---

### 2.3 IM 配置环境变量支持

**目标**：支持通过环境变量覆盖 IM 配置（安全凭证不在 TOML 文件中）。

**涉及文件**：
- `codex-rs/config/src/types.rs` — 添加环境变量别名
- `codex-rs/config/src/loader/mod.rs` — 环境变量展开逻辑

**环境变量规范**：
```bash
BCIP_IM_ENABLED=true
BCIP_IM_TELEGRAM_BOT_TOKEN=xxx
BCIP_IM_TELEGRAM_ALLOWED_USERS=user1,user2
BCIP_IM_FEISHU_APP_ID=xxx
BCIP_IM_FEISHU_APP_SECRET=xxx
BCIP_IM_BRIDGE_SERVER_URL=ws://127.0.0.1:3456
```

**验收标准**：
- [ ] 环境变量覆盖 TOML 配置
- [ ] 无 TOML 配置时，纯环境变量可启动
- [ ] 凭证字段优先从环境变量读取（安全性）

**依赖**：2.1

---

### 2.4 IM 适配器冒烟测试

**目标**：验证 Telegram/Feishu 适配器能正确初始化和运行。

**涉及文件**：
- `codex-rs/codex-im-telegram/tests/` — 新增冒烟测试
- `codex-rs/codex-im-feishu/tests/` — 新增冒烟测试

**测试场景**：
1. 无效 bot_token 时 graceful 错误
2. Adapter 能连接 ImBridge（mock）
3. 消息收发基本流程

**验收标准**：
- [ ] 两个适配器各有 3 个冒烟测试
- [ ] 无真实 API 调用（mock）

**依赖**：2.2

---

## Phase 3: AgentBus 统一消息系统 [P1]

> 目标：建立跨角色 Agent 通信总线，支持广播/订阅/点对点消息。

### 3.1 设计 AgentBus 消息信封

**目标**：统一所有 Agent 间通信的消息格式。

**涉及文件**：
- `codex-rs/protocol/src/agent_bus.rs` — 新文件

**具体改动**：

```rust
/// 统一消息信封
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentBusMessage {
    /// 消息唯一 ID
    pub id: Uuid,
    /// 发送者 AgentPath
    pub from: AgentPath,
    /// 接收者（点对点、广播、主题订阅）
    pub to: AgentBusRecipient,
    /// 消息类型
    pub message_type: AgentBusMessageType,
    /// 消息负载
    pub payload: serde_json::Value,
    /// 时间戳
    pub timestamp: i64,
    /// 关联的任务/会话 ID
    pub correlation_id: Option<Uuid>,
    /// 优先级
    pub priority: MessagePriority,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AgentBusRecipient {
    /// 点对点
    Direct(AgentPath),
    /// 广播到所有 agent
    Broadcast,
    /// 主题订阅（如 "patent.search.results"）
    Topic(String),
    /// 角色广播（发送给某角色的所有 agent）
    Role(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AgentBusMessageType {
    /// 任务请求
    TaskRequest,
    /// 任务结果
    TaskResult,
    /// 进度更新
    Progress,
    /// 错误通知
    Error,
    /// 系统事件（agent spawn/shutdown）
    SystemEvent,
    /// 自定义
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}
```

**验收标准**：
- [ ] 消息信封能序列化/反序列化
- [ ] 与现有 InterAgentCommunication 兼容（提供转换方法）
- [ ] `cargo check -p codex-protocol` 通过

**依赖**：Phase 1.1

---

### 3.2 实现 AgentBus 核心

**目标**：基于 tokio broadcast channel 实现消息总线。

**涉及文件**：
- `codex-rs/core/src/agent/bus.rs` — 新文件
- `codex-rs/core/src/agent/mod.rs` — 导出

**具体改动**：

```rust
pub struct AgentBus {
    /// 广播通道
    tx: broadcast::Sender<AgentBusMessage>,
    /// 主题订阅映射
    topics: Arc<RwLock<HashMap<String, Vec<AgentPath>>>>,
    /// 消息历史（可选，用于调试/审计）
    history: Arc<RwLock<VecDeque<AgentBusMessage>>>,
    /// 最大历史长度
    max_history: usize,
}

impl AgentBus {
    pub fn new(buffer_size: usize) -> Self;
    
    /// 订阅总线（返回 Receiver）
    pub fn subscribe(&self) -> broadcast::Receiver<AgentBusMessage>;
    
    /// 订阅特定主题
    pub fn subscribe_topic(&self, topic: &str, agent: AgentPath);
    
    /// 发送消息
    pub fn send(&self, message: AgentBusMessage) -> Result<(), SendError>;
    
    /// 便捷方法：点对点发送
    pub fn send_direct(&self, from: AgentPath, to: AgentPath, payload: Value);
    
    /// 便捷方法：主题广播
    pub fn publish(&self, from: AgentPath, topic: &str, payload: Value);
    
    /// 获取消息历史
    pub fn history(&self, filter: MessageFilter) -> Vec<AgentBusMessage>;
}

/// AgentBus 集成到 SessionServices
impl SessionServices {
    pub fn agent_bus(&self) -> &AgentBus;
}
```

**验收标准**：
- [ ] broadcast channel 能正确路由消息
- [ ] 主题订阅/取消订阅正常工作
- [ ] 历史记录不超过 max_history
- [ ] 并发安全（多 agent 同时发送/接收）
- [ ] 单元测试覆盖 > 90%

**依赖**：3.1

---

### 3.3 集成 AgentBus 到 AgentControl

**目标**：AgentControl 的 spawn/message 方法通过 AgentBus 增强。

**涉及文件**：
- `codex-rs/core/src/agent/control.rs` — 在 spawn/message 流程中发送 AgentBus 事件

**具体改动**：

```rust
// spawn_agent_internal 中添加：
self.agent_bus.publish(
    AgentPath::root(),
    "agent.lifecycle.spawned",
    json!({ "agent_id": thread_id, "role": role, "parent": parent_id })
);

// send_inter_agent_communication 中添加：
self.agent_bus.send_direct(
    author.clone(),
    recipient.clone(),
    json!({ "content": content, "type": "inter_agent_comm" })
);

// Completion Watcher 中添加：
self.agent_bus.publish(
    agent_path,
    "agent.lifecycle.completed",
    json!({ "agent_id": thread_id, "status": status })
);
```

**验收标准**：
- [ ] Agent spawn/shutdown/message 事件通过 AgentBus 广播
- [ ] 现有 InterAgentCommunication 逻辑不受影响
- [ ] 订阅者能收到生命周期事件

**依赖**：3.2

---

### 3.4 Patent Agent 角色间协作

**目标**：利用 AgentBus 实现 Retriever → Analyzer → Writer 的协作流程。

**涉及文件**：
- `codex-rs/codex-patent-workflow/src/collaboration.rs` — 新文件
- `codex-rs/codex-patent-workflow/src/lib.rs` — 导出

**协作场景示例**：
```
用户: "检索 X 领域的专利并分析"
→ Retriever: 搜索专利 → publish("patent.search.results", results)
→ Analyzer: 订阅 "patent.search.results" → 分析 → publish("patent.analysis.complete", analysis)
→ Writer: 订阅 "patent.analysis.complete" → 撰写报告 → 返回给用户
```

**验收标准**：
- [ ] 3 个角色通过 AgentBus 完成端到端协作
- [ ] 中间结果正确传递
- [ ] 异常场景（某角色失败）能通过 AgentBus 通知其他角色

**依赖**：3.3, Phase 1.2

---

## Phase 4: 基础设施增强 [P2-P3]

> 目标：提升生产环境可靠性、安全性和可观测性。

### 4.1 Webhook 支持

**目标**：为 IM 适配器添加 webhook 接收能力。

**涉及文件**：
- `codex-rs/codex-im-common/src/lib.rs` — 新增 WebhookReceiver trait
- `codex-rs/codex-im-telegram/src/webhook.rs` — 新文件
- `codex-rs/codex-im-feishu/src/webhook.rs` — 新文件

**验收标准**：
- [ ] Telegram webhook 验证流程实现
- [ ] 飞书事件订阅回调实现
- [ ] 可配置选择 polling 或 webhook 模式

**依赖**：Phase 2.2

---

### 4.2 DingTalk 适配器实现

**目标**：实现 `codex-im-protocol` 中已定义的 DingTalk 枚举。

**涉及文件**：
- `codex-rs/codex-im-dingtalk/` — 新 crate
- `codex-rs/Cargo.toml` — 添加 workspace member
- `codex-rs/codex-im-protocol/src/messages.rs` — 验证 DingTalk 消息格式

**验收标准**：
- [ ] DingTalk 适配器实现 ImAdapter trait
- [ ] 支持文本/Markdown 消息发送
- [ ] 支持消息接收（webhook）

**依赖**：Phase 2.1

---

### 4.3 消息重试与死信队列

**目标**：为 AgentBus 和 IM Bridge 添加可靠消息传递。

**涉及文件**：
- `codex-rs/core/src/agent/bus.rs` — 添加重试逻辑
- `codex-rs/codex-im-bridge/src/lib.rs` — 添加重连/重试

**具体改动**：
- AgentBus: `send_with_retry(msg, max_retries, backoff)`
- ImBridge: 指数退避重连 + 死信队列（SQLite 持久化）
- IM Adapter: 消息发送失败时写入死信队列

**验收标准**：
- [ ] AgentBus 消息重试最多 3 次
- [ ] IM Bridge 断连后自动重连
- [ ] 死信消息可通过管理接口查看/重发

**依赖**：3.2, 2.2

---

### 4.4 消息加密

**目标**：敏感通信（如审查意见讨论）支持端到端加密。

**涉及文件**：
- `codex-rs/codex-im-common/src/crypto.rs` — 新文件
- `codex-rs/core/src/agent/bus.rs` — 可选加密层

**验收标准**：
- [ ] AES-256-GCM 加密/解密
- [ ] 密钥通过配置管理（不硬编码）
- [ ] 加密对 AgentBus 透明（自动加解密）

**依赖**：3.2

---

### 4.5 可观测性（OpenTelemetry 集成）

**目标**：AgentBus 和 IM Bridge 操作集成 OTel tracing/metrics。

**涉及文件**：
- `codex-rs/core/src/agent/bus.rs` — 添加 span/metrics
- `codex-rs/codex-im-bridge/src/lib.rs` — 添加 span/metrics
- `codex-rs/codex-im-*/src/lib.rs` — 各 adapter 添加 tracing

**验收标准**：
- [ ] AgentBus 每条消息有 trace_id
- [ ] IM 消息收发有 span 覆盖
- [ ] Metrics: 消息吞吐量、延迟、错误率

**依赖**：3.2, 2.2

---

### 4.6 部署配置清单

**目标**：生成标准化的部署配置模板。

**涉及文件**：
- `scripts/config/bcip-default-config.toml` — 更新默认配置
- `docs/deployment/im-setup.md` — 新文件（如用户要求）

**验收标准**：
- [ ] 默认配置包含 IM 配置注释模板
- [ ] 部署文档包含环境变量说明

**依赖**：2.1

---

## 依赖关系图

```
Phase 1: 双轨 Agent 合并
  1.1 PatentAgentBridge ──→ 1.2 迁移 spawn ──→ 1.4 删除旧代码 ──→ 1.5 集成测试
                      └──→ 1.3 角色配置统一 ──────────────────────→ 1.5
                                                           ↑
Phase 3: AgentBus ─────────────────────────────────────────┘
  3.1 消息信封 ──→ 3.2 AgentBus 核心 ──→ 3.3 集成到 AgentControl ──→ 3.4 角色协作
                       ↑
                   1.1 PatentAgentBridge

Phase 2: IM 通道接入 (可并行)
  2.1 IM ConfigToml ──→ 2.2 IM Service 初始化 ──→ 2.4 冒烟测试
                    └──→ 2.3 环境变量支持 ──→ 2.4

Phase 4: 基础设施 (依赖 Phase 2+3)
  4.1 Webhook ←── 2.2
  4.2 DingTalk ←── 2.1
  4.3 重试/死信 ←── 3.2, 2.2
  4.4 加密 ←── 3.2
  4.5 可观测性 ←── 3.2, 2.2
  4.6 部署配置 ←── 2.1
```

## 关键风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Phase 1 迁移导致现有工作流中断 | 高 | 保持旧 API 兼容，渐进式迁移，feature flag 控制 |
| AgentControl spawn 性能瓶颈 | 中 | 预热 agent pool，异步 spawn，性能基准测试 |
| IM 适配器在生产环境不稳定 | 中 | 熔断器模式，graceful degradation，不影响核心功能 |
| AgentBus 广播风暴 | 低 | 消息速率限制，主题细分，背压控制 |
| 配置安全（IM token 泄露） | 高 | 环境变量优先，denylist 阻止项目级配置 |

## 预估工时

| Phase | 任务数 | 预估工时 | 可并行度 |
|-------|--------|----------|----------|
| Phase 1 | 5 | 15-20 天 | 中（1.1 先行，1.2+1.3 可并行） |
| Phase 2 | 4 | 8-10 天 | 高（2.1 先行，2.2+2.3 可并行） |
| Phase 3 | 4 | 12-15 天 | 低（强顺序依赖） |
| Phase 4 | 6 | 10-12 天 | 高（各任务独立） |
| **总计** | **19** | **45-57 天** | |

**Phase 1+2 可并行推进**（核心改动不冲突），总关键路径约 30 天。

---

## 附录：检查清单

### Phase 1 检查清单

- [ ] `PatentAgentBridge` 实现 `ToolExecutor<ToolInvocation>`
- [ ] 9 种 PatentAgentRole 都能通过 bridge spawn
- [ ] 角色域映射正确（primary=Direct, secondary=Deferred, 其他=Hidden）
- [ ] Completion Watcher 能通知父 agent
- [ ] 无 `std::thread::spawn` 用于 agent 创建
- [ ] 所有测试通过
- [ ] Clippy 无警告
- [ ] `cargo nextest run -p codex-patent-agents -p codex-patent-workflow` 通过

### Phase 2 检查清单

- [ ] `ImConfigToml` 定义完毕
- [ ] ConfigToml 有 `im` 字段
- [ ] Config wire 正确
- [ ] `config.schema.json` 包含 `[im]` 段
- [ ] `"im"` 在 PROJECT_LOCAL_CONFIG_DENYLIST
- [ ] app-server 启动时条件创建 IM service
- [ ] 环境变量覆盖工作正常
- [ ] IM disabled 时不初始化
- [ ] adapter `run()` 作为 tokio task 运行
- [ ] shutdown hook 正确

### Phase 3 检查清单

- [ ] `AgentBusMessage` 序列化/反序列化
- [ ] 与 `InterAgentCommunication` 兼容
- [ ] broadcast channel 正确路由
- [ ] 主题订阅/取消正常
- [ ] Agent 生命周期事件广播
- [ ] 3 角色端到端协作测试
- [ ] 无 dead letter（正常场景）

### Phase 4 检查清单

- [ ] Telegram webhook 验证
- [ ] 飞书事件订阅
- [ ] polling/webhook 可配置
- [ ] DingTalk 适配器 crate
- [ ] AgentBus 重试最多 3 次
- [ ] IM Bridge 自动重连
- [ ] 死信队列持久化
- [ ] AES-256-GCM 加密
- [ ] OTel span/metrics
- [ ] 默认配置模板更新

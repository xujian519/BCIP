# BCIP 容错与弹性机制改进计划

> 审计日期: 2026-06-04
> 状态: 规划阶段
> 审计评分: 单点故障 7/10 | Agent恢复 3/10 | 超时重试 9/10

---

## 一、审计发现概览

### 已有机制（健康）

| 机制 | 位置 | 状态 |
|------|------|------|
| Panic捕获 | spawn.rs, tui.rs, exec/lib.rs | ✅ 完整 |
| AgentBus消息重试+DLQ | core/src/agent/bus.rs | ✅ 完整 |
| 工作流编排重试+检查点 | patent-workflow/orchestrator.rs, checkpoint.rs | ⚠️ 检查点未激活 |
| DAG步骤重试+工具退避 | patent-workflow/graph_executor.rs | ✅ 完整 |
| 工具级重试+降级链 | core/src/tools/retry_config.rs | ✅ 完整 |
| LLM调用重试 | patent-agents/llm.rs | ⚠️ 无jitter |
| SSE/WebSocket/HTTP重连 | session/turn.rs, websocket.rs, client/retry.rs | ✅ 完整 |
| Guardian断路器 | core/src/guardian/review.rs | ✅ 仅用于denial |
| CancellationToken全栈 | core/src/tasks/mod.rs | ✅ 完整 |
| 心跳活性追踪 | bus.rs check_liveness() | ⚠️ 未接入恢复 |

### 关键缺口（9项）

| # | 缺口 | 优先级 | 当前状态 | 影响 |
|---|------|--------|----------|------|
| G1 | 无全局熔断器 | P0 | 5条外部API路径仅重试+降级 | 持续故障耗尽重试预算，雪崩 |
| G2 | Agent无自动重启 | P0 | panic后仅标记failed | Agent死后无法恢复 |
| G3 | 心跳未接入恢复 | P0 | 标记Unresponsive但无动作 | 死Agent永远占位 |
| G4 | 任务无法转移 | P1 | 无reassign_task() | Agent死亡后任务丢失 |
| G5 | 无Turn中间检查点 | P1 | 仅turn边界持久化 | Turn崩溃丢失所有内部工作 |
| G6 | 无舱壁隔离 | P2 | std::thread::scope无限制 | 资源耗尽影响全局 |
| G7 | 5处backoff实现未统一 | P2 | 不同公式，部分无jitter | 维护困难，行为不一致 |
| G8 | HITL无超时 | P3 | Suspended状态无限等待 | 任务永远挂起 |
| G9 | 无健康检查端点 | P3 | 无/health | 无法监控编排状态 |

---

## 二、架构设计

### 2.1 新增共享原语（Phase 1）

#### 2.1.1 统一退避策略 `ExponentialBackoff`

```rust
// 位置: core/src/resilience/backoff.rs（新模块）
// 或: codex-patent-domain/src/resilience/backoff.rs

/// 统一的指数退避策略，替代现有5处分散实现
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExponentialBackoff {
    /// 初始延迟（毫秒）
    pub base_delay_ms: u64,
    /// 最大延迟（毫秒）
    pub max_delay_ms: u64,
    /// 退避倍数，通常为2.0
    pub multiplier: f64,
    /// jitter范围 [1.0 - jitter, 1.0 + jitter]，0.0表示无jitter
    pub jitter_range: f64,
}

impl ExponentialBackoff {
    /// 计算第N次尝试的延迟
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let exp = self.multiplier.powi(attempt as i32);
        let delay = self.base_delay_ms as f64 * exp;
        let jitter = 1.0 + (rand::random::<f64>() * 2.0 - 1.0) * self.jitter_range;
        let final_delay = (delay * jitter) as u64;
        Duration::from_millis(final_delay.min(self.max_delay_ms))
    }

    // 预设配置
    pub fn aggressive() -> Self {       // 工具调用: 100ms base
        Self { base_delay_ms: 100, max_delay_ms: 5_000, multiplier: 2.0, jitter_range: 0.1 }
    }
    pub fn standard() -> Self {         // LLM/API: 1000ms base
        Self { base_delay_ms: 1_000, max_delay_ms: 30_000, multiplier: 2.0, jitter_range: 0.1 }
    }
    pub fn conservative() -> Self {     // 外部服务: 2000ms base
        Self { base_delay_ms: 2_000, max_delay_ms: 60_000, multiplier: 2.0, jitter_range: 0.15 }
    }
}
```

#### 2.1.2 通用熔断器 `CircuitBreaker`

```rust
// 位置: core/src/resilience/circuit_breaker.rs（新模块）

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// 正常，请求可通过
    Closed,
    /// 熔断中，请求被拒绝
    Open,
    /// 半开，允许探测请求通过以测试恢复
    HalfOpen,
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 触发熔断的连续失败次数
    pub failure_threshold: u32,
    /// 触发熔断的滑动窗口内失败率（0.0~1.0）
    pub failure_rate_threshold: f64,
    /// 滑动窗口大小（请求次数）
    pub window_size: usize,
    /// Open → HalfOpen 的等待时间
    pub reset_timeout: Duration,
    /// HalfOpen 状态允许通过的探测请求数
    pub half_open_max_calls: u32,
}

/// 熔断器 trait
pub trait CircuitBreaker: Send + Sync {
    /// 当前是否允许请求通过
    fn allow_request(&self) -> bool;
    /// 记录成功调用
    fn record_success(&self);
    /// 记录失败调用
    fn record_failure(&self);
    /// 获取当前状态
    fn state(&self) -> CircuitState;
    /// 强制重置为 Closed
    fn reset(&self);
    /// 获取统计信息（用于健康检查）
    fn stats(&self) -> CircuitBreakerStats;
}

/// 线程安全的熔断器实现（基于 tokio::sync 或 std::sync）
pub struct StdCircuitBreaker {
    config: CircuitBreakerConfig,
    state: Mutex<CircuitBreakerInner>,
}

/// 熔断器注册表 — 每个外部服务一个实例
pub struct CircuitBreakerRegistry {
    breakers: HashMap<String, Arc<dyn CircuitBreaker>>,
}

impl CircuitBreakerRegistry {
    /// 获取指定服务的熔断器（不存在则创建）
    pub fn get_or_create(&self, service: &str, config: CircuitBreakerConfig) -> Arc<dyn CircuitBreaker>;
    /// 获取所有熔断器状态（用于健康检查）
    pub fn all_stats(&self) -> HashMap<String, CircuitBreakerStats>;
}

/// 熔断器统计
#[derive(Debug, Clone, serde::Serialize)]
pub struct CircuitBreakerStats {
    pub service: String,
    pub state: CircuitState,
    pub total_calls: u64,
    pub total_failures: u64,
    pub consecutive_failures: u32,
    pub last_failure_time: Option<i64>,
}
```

#### 2.1.3 Agent恢复策略 `RecoveryPolicy`

```rust
// 位置: core/src/resilience/recovery.rs（新模块）

/// Agent恢复策略配置
#[derive(Debug, Clone)]
pub struct RecoveryPolicy {
    /// 最大自动重启次数（超过后标记为 Unrecoverable）
    pub max_restarts: u32,
    /// 重启间隔（退避）
    pub restart_backoff: ExponentialBackoff,
    /// Unresponsive 超时后是否自动重启
    pub restart_on_unresponsive: bool,
    /// Unresponsive 检测超时
    pub unresponsive_timeout: Duration,
    /// 是否允许任务转移
    pub allow_task_reassignment: bool,
}

/// Agent恢复上下文（存储在 AgentMetadata 中）
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// 当前重启计数
    pub restart_count: u32,
    /// 上次重启时间
    pub last_restart_at: Option<Instant>,
    /// 上次失败原因
    pub last_failure_reason: Option<String>,
    /// 当前状态
    pub state: AgentRecoveryState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRecoveryState {
    /// 正常运行
    Healthy,
    /// 等待重启
    PendingRestart,
    /// 正在重启中
    Restarting,
    /// 超过最大重启次数，不可恢复
    Unrecoverable,
}
```

---

### 2.2 各缺口的具体改进设计

#### G1: 全局熔断器（P0）

**目标**: 为所有外部API调用添加熔断器，防止级联故障

**实现方案**:

1. **熔断器注册表** 放入 `SessionServices`（与现有 `guardian_rejection_circuit_breaker` 并列）
2. **包装外部调用**: 创建 `ResilientClient` wrapper，在每次调用前检查熔断器
3. **注册5个服务的熔断器**:

| 服务 | 熔断器配置 | 位置 |
|------|-----------|------|
| LLM Responses API | failure_threshold=5, window=20, reset=30s | core/client.rs |
| Agent LLM (blocking) | failure_threshold=3, window=10, reset=60s | patent-agents/llm.rs |
| Google Patents | failure_threshold=5, window=20, reset=120s | patent-tools/google_patents.rs |
| Embedding Service | failure_threshold=3, window=10, reset=60s | embedding_client.rs |
| Guardian Review | 复用LLM熔断器 | — |

**改动文件**:
- `core/src/resilience/mod.rs` — 新模块
- `core/src/resilience/circuit_breaker.rs` — 熔断器实现
- `core/src/state/service.rs` — SessionServices 添加 registry
- `core/src/client.rs` — 包装LLM调用
- `codex-patent-agents/src/agent_runtime/llm.rs` — 包装Agent LLM调用
- `codex-patent-tools/src/google_patents.rs` — 包装Google Patents调用
- `codex-patent-tools/src/embedding_client.rs` — 包装Embedding调用

**集成扩展点**: EP3（bus.rs心跳回调可触发熔断器检查）

#### G2: Agent自动重启（P0）

**目标**: Agent panic/crash后自动重启，恢复工作

**实现方案**:

1. **AgentMetadata 扩展**: 添加 `recovery_context: Option<RecoveryContext>` 字段
2. **AgentStatus 扩展**: 添加 `Restarting` 变体
3. **重启循环**: 在 EP8（spawn.rs catch_unwind后）检查 restart_count < max_restarts，满足则重新 spawn
4. **退避间隔**: 使用 `ExponentialBackoff::standard()` 计算重启间隔
5. **资源保留**: 重启期间保留 agent_path 和 agent_nickname（EP6 restart mode）

**关键逻辑**（伪代码）:
```rust
// spawn.rs: spawn_agent_thread() 中 catch_unwind 后
match result {
    Err(panic_payload) => {
        let recovery = metadata.recovery_context.get_or_insert_default();
        if recovery.restart_count < policy.max_restarts {
            recovery.state = AgentRecoveryState::Restarting;
            recovery.restart_count += 1;
            let delay = policy.restart_backoff.delay_for_attempt(recovery.restart_count);
            thread::sleep(delay);
            // 重新调用 run_agent_job()（递归，但有限深度）
            continue; // 在循环中重试
        } else {
            recovery.state = AgentRecoveryState::Unrecoverable;
            // 通知 AgentBus 记录死亡事件
            break;
        }
    }
    Ok(()) => break,
}
```

**改动文件**:
- `core/src/agent/registry.rs` — AgentMetadata 添加 recovery_context
- `core/src/agent/control.rs` — AgentStatus 添加 Restarting
- `codex-patent-agents/src/agent_runtime/spawn.rs` — 重启循环
- `codex-patent-agents/src/agent_runtime/spawn.rs` — 传递 RecoveryPolicy

**集成扩展点**: EP4（注册时初始化recovery）, EP5（释放时检查restart）, EP6（restart mode）, EP8（catch_unwind后重启）

#### G3: 心跳接入恢复（P0）

**目标**: Unresponsive 检测触发自动恢复动作

**实现方案**:

1. **AgentBus 扩展**: `check_liveness()` 添加回调 `on_unresponsive: Box<dyn Fn(&str) + Send + Sync>`
2. **回调注册**: 在 AgentControl 初始化时注册回调，回调逻辑：
   - 检查 RecoveryContext，如果 restart_count < max_restarts → 触发重启
   - 否则标记 Unrecoverable + 记录死亡事件到 DLQ
3. **事件传播**: 新增 `AgentEvent::UnresponsiveDetected { agent_id, missed_count }` 事件
4. **与熔断器联动**: 如果该Agent依赖的外部服务熔断器已Open，跳过重启（避免无意义重试）

**改动文件**:
- `core/src/agent/bus.rs` — check_liveness 添加回调 + 事件
- `core/src/agent/control.rs` — 注册回调 + 处理逻辑

**集成扩展点**: EP3（check_liveness回调）, EP2（completion_watcher检测）

#### G4: 任务转移（P1）

**目标**: Agent死亡后将其未完成任务转移到其他Agent

**实现方案**:

1. **任务追踪**: 在 AgentBus 中添加 `pending_tasks: HashMap<agent_id, VecDeque<TaskDescriptor>>`
2. **TaskDescriptor**: 包含 task_type、payload、priority、deadline
3. **转移逻辑**: 
   - Agent被标记Unrecoverable时，其 pending_tasks 移入全局重分配队列
   - `reassign_orphaned_tasks()` 方法：遍历队列，寻找具有相同 role 的可用Agent
   - 如果无可用Agent → 任务放入 DLQ 等待人工干预
4. **通知**: 转移成功/失败都通过 AgentEvent 广播

**改动文件**:
- `core/src/agent/bus.rs` — 添加 pending_tasks + reassign_orphaned_tasks()
- `core/src/agent/registry.rs` — 按 role 查询可用Agent

**集成扩展点**: EP9（DLQ扩展包含orphaned tasks）

#### G5: Turn中间检查点（P1）

**目标**: Turn执行过程中间状态持久化，崩溃后可恢复

**实现方案**:

1. **激活 CheckpointStore**: 移除 GraphExecutor 中 `#[allow(dead_code)]`，在 execute_step 完成后写入检查点
2. **Turn级检查点**: 在 run_turn() 的 sampling request 循环中，每完成一个 request 保存中间状态
3. **增量保存**: 仅保存新增的 conversation items（避免全量序列化）
4. **恢复流程**: Turn崩溃后，从最后一个检查点恢复 conversation items，跳过已完成的步骤

**改动文件**:
- `codex-patent-workflow/src/graph_executor.rs` — 激活 checkpoint_store，execute_step后写入
- `codex-patent-workflow/src/checkpoint.rs` — 添加 turn-level checkpoint 支持
- `core/src/session/turn.rs` — sampling request 循环中保存检查点

**集成扩展点**: 无（新功能，不依赖现有EP）

#### G6: 舱壁隔离（P2）

**目标**: 限制并发资源使用，防止资源耗尽

**实现方案**:

1. **工作流级别**: 替换 `std::thread::scope` 为有界线程池 `rayon::ThreadPool` 或 `tokio::sync::Semaphore`
2. **工具级别**: 为高资源消耗工具（如OCR、Embedding）添加 `Semaphore` 许可
3. **Agent级别**: 限制同时活跃Agent数量（已通过 `max_agents` 部分实现）

**改动文件**:
- `codex-patent-workflow/src/graph_executor.rs` — 并行执行添加信号量
- `core/src/tools/retry_dispatch.rs` — 工具执行添加信号量

#### G7: 退避统一（P2）

**目标**: 所有退避实现统一使用 `ExponentialBackoff`

**替换映射**:

| 现有位置 | 当前公式 | 替换为 |
|----------|---------|--------|
| core/src/util.rs backoff() | 200ms × 2^(n-1) × jitter(0.9~1.1) | `ExponentialBackoff::aggressive()` |
| core/src/tools/retry_config.rs backoff() | base × 2^n × jitter(±10%) | `ExponentialBackoff::aggressive()` |
| codex-client/src/retry.rs backoff() | base × 2^(n-1) × jitter(0.9~1.1) | `ExponentialBackoff::standard()` |
| patent-agents/llm.rs inline | 1000ms × 2^n 无jitter | `ExponentialBackoff::standard()` |
| google_patents.rs inline | 2^n seconds 无jitter | `ExponentialBackoff::conservative()` |

**改动文件**: 上述5个文件
**注意**: 需逐个替换，每次替换后跑测试确认行为不变

#### G8: HITL超时（P3）

**目标**: 人工审批挂起超时后自动处理

**实现方案**:

1. **配置**: `HumanApprovalConfig { timeout: Option<Duration>, timeout_action: TimeoutAction }`
2. **TimeoutAction**: `Fail`（标记失败）/ `AutoApprove`（自动通过）/ `Escalate`（升级通知）
3. **实现**: 在 FlowStep::HumanApproval 的 wait 逻辑中添加超时

**改动文件**:
- `codex-patent-workflow/src/flow.rs` — 添加 timeout_action 配置
- `codex-patent-workflow/src/graph_executor.rs` — HumanApproval 添加超时

#### G9: 健康检查端点（P3）

**目标**: 提供系统健康状态查询

**实现方案**:

1. **HealthReport**: 汇总所有熔断器状态、Agent存活状态、DLQ大小
2. **实现**: 如果有HTTP服务，添加 `/health` 端点；否则提供 CLI 命令
3. **状态判定**: Healthy（所有熔断器Closed + 所有Agent Alive） / Degraded（有HalfOpen或部分Unresponsive） / Unhealthy（有Open或Unrecoverable）

**改动文件**:
- `core/src/resilience/health.rs` — 新模块
- 依赖 G1 的 CircuitBreakerRegistry 和 G3 的 Agent 存活状态

---

## 三、实施阶段

### Phase 1: 基础设施（预计 3-4 天）

**目标**: 建立共享原语，后续阶段依赖

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T1.1 | 创建 `core/src/resilience/mod.rs` 模块 | 低 | 1个新文件 | 无 |
| T1.2 | 实现 `ExponentialBackoff` | 中 | 1个新文件 + 测试 | T1.1 |
| T1.3 | 实现 `CircuitBreaker` trait + `StdCircuitBreaker` | 高 | 1个新文件 + 测试 | T1.1 |
| T1.4 | 实现 `CircuitBreakerRegistry` | 中 | 同T1.3 | T1.3 |
| T1.5 | 实现 `RecoveryPolicy` + `RecoveryContext` | 中 | 1个新文件 + 测试 | T1.2 |
| T1.6 | 将 `resilience` 模块注册到 `core/src/lib.rs` | 低 | 1个文件 | T1.2-T1.5 |
| T1.7 | 为所有新类型编写单元测试 | 中 | 测试文件 | T1.2-T1.5 |

**检查清单**:
- [ ] `cargo check -p codex-core` 通过
- [ ] `cargo clippy -p codex-core` 0警告
- [ ] `cargo nextest run -p codex-core` 所有新测试通过
- [ ] `ExponentialBackoff` 各预设配置的单测覆盖
- [ ] `CircuitBreaker` Closed→Open→HalfOpen→Closed 全生命周期测试
- [ ] `CircuitBreakerRegistry` 并发注册/获取测试
- [ ] `RecoveryContext` 状态转换测试

### Phase 2: P0 关键修复（预计 5-7 天）

**目标**: 解决3个P0缺口（G1, G2, G3）

#### Phase 2A: 全局熔断器 (G1)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T2A.1 | SessionServices 添加 CircuitBreakerRegistry | 低 | core/src/state/service.rs | T1.4 |
| T2A.2 | 创建 `ResilientClient` wrapper | 中 | core/src/resilience/client.rs | T1.3 |
| T2A.3 | 包装 LLM Responses API 调用 | 中 | core/src/client.rs | T2A.2 |
| T2A.4 | 包装 Agent LLM blocking 调用 | 中 | patent-agents/llm.rs | T2A.2 |
| T2A.5 | 包装 Google Patents 调用 | 中 | patent-tools/google_patents.rs | T2A.2 |
| T2A.6 | 包装 Embedding Service 调用 | 中 | embedding_client.rs | T2A.2 |
| T2A.7 | 集成测试: 熔断器阻止级联故障 | 高 | 新测试文件 | T2A.3-T2A.6 |

**检查清单**:
- [ ] `cargo check` 全部通过
- [ ] LLM调用在熔断器Open时快速失败（不发出网络请求）
- [ ] Google Patents连续5次失败后熔断器Open
- [ ] Embedding服务熔断器Open后不影响其他工具
- [ ] HalfOpen状态允许探测请求并自动恢复
- [ ] 熔断器统计信息可查询

#### Phase 2B: Agent自动重启 (G2)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T2B.1 | AgentMetadata 添加 recovery_context 字段 | 低 | core/src/agent/registry.rs | T1.5 |
| T2B.2 | AgentStatus 添加 Restarting 变体 | 低 | core/src/agent/control.rs | 无 |
| T2B.3 | spawn_agent_thread 添加重启循环 | 高 | patent-agents/spawn.rs | T2B.1, T2B.2 |
| T2B.4 | RecoveryPolicy 配置传递 | 中 | patent-agents/spawn.rs + config | T1.5 |
| T2B.5 | Agent重启事件广播 | 中 | core/src/agent/bus.rs | T2B.3 |
| T2B.6 | 集成测试: Agent panic后自动重启 | 高 | 新测试文件 | T2B.3-T2B.5 |

**检查清单**:
- [ ] Agent panic后自动重启（最多 max_restarts 次）
- [ ] 重启间隔符合指数退避
- [ ] 超过 max_restarts 后标记 Unrecoverable
- [ ] 重启期间 agent_path/nickname 不释放
- [ ] AgentEvent::Restarting 事件正确广播
- [ ] CancellationToken 在重启时正确传播

#### Phase 2C: 心跳接入恢复 (G3)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T2C.1 | check_liveness 添加 on_unresponsive 回调 | 中 | core/src/agent/bus.rs | 无 |
| T2C.2 | 注册回调：Unresponsive → 触发重启流程 | 中 | core/src/agent/control.rs | T2B.3, T2C.1 |
| T2C.3 | 添加 AgentEvent::UnresponsiveDetected | 低 | core/src/agent/bus.rs | 无 |
| T2C.4 | 熔断器联动：外部服务Open时跳过重启 | 中 | core/src/agent/control.rs | T2A.2, T2C.2 |
| T2C.5 | 集成测试: Unresponsive检测触发恢复 | 高 | 新测试文件 | T2C.1-T2C.4 |

**检查清单**:
- [ ] Unresponsive 检测后触发恢复回调
- [ ] 回调检查 restart_count < max_restarts
- [ ] AgentEvent::UnresponsiveDetected 正确发送
- [ ] 外部服务熔断器Open时不盲目重启
- [ ] Unrecoverable Agent 的 pending tasks 进入 DLQ

### Phase 3: P1 重要改进（预计 3-5 天）

**目标**: 解决2个P1缺口（G4, G5）

#### Phase 3A: 任务转移 (G4)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T3A.1 | 定义 TaskDescriptor 类型 | 低 | core/src/agent/bus.rs | 无 |
| T3A.2 | AgentBus 添加 pending_tasks 追踪 | 中 | core/src/agent/bus.rs | T3A.1 |
| T3A.3 | 实现 reassign_orphaned_tasks() | 高 | core/src/agent/bus.rs | T3A.2, T2C.2 |
| T3A.4 | 按 role 查询可用Agent方法 | 低 | core/src/agent/registry.rs | 无 |
| T3A.5 | 集成测试: Agent死亡后任务转移到其他Agent | 高 | 新测试文件 | T3A.3 |

**检查清单**:
- [ ] Agent标记Unrecoverable时pending_tasks自动进入重分配队列
- [ ] 任务转移到具有相同role的可用Agent
- [ ] 无可用Agent时任务进入DLQ
- [ ] 转移事件通过AgentEvent广播
- [ ] 任务转移不影响正在执行的工作

#### Phase 3B: Turn中间检查点 (G5)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T3B.1 | 移除GraphExecutor checkpoint_store dead_code标记 | 低 | patent-workflow/graph_executor.rs | 无 |
| T3B.2 | execute_step完成后写入检查点 | 中 | patent-workflow/graph_executor.rs | T3B.1 |
| T3B.3 | CheckpointStore支持turn-level检查点 | 中 | patent-workflow/checkpoint.rs | 无 |
| T3B.4 | run_turn() sampling request循环保存中间状态 | 高 | core/src/session/turn.rs | T3B.3 |
| T3B.5 | 恢复流程：从检查点恢复Turn | 高 | core/src/session/turn.rs | T3B.4 |
| T3B.6 | 集成测试: Turn崩溃后从检查点恢复 | 高 | 新测试文件 | T3B.5 |

**检查清单**:
- [ ] GraphExecutor每个step完成后写入检查点
- [ ] Turn每个sampling request完成后保存中间状态
- [ ] 检查点增量保存（不全量序列化）
- [ ] 崩溃后能正确从最后检查点恢复
- [ ] 恢复后跳过已完成的步骤
- [ ] 检查点写入不影响正常执行性能（异步写入）

### Phase 4: P2-P3 优化打磨（预计 3-4 天）

**目标**: 解决4个低优先级缺口（G6, G7, G8, G9）

#### Phase 4A: 舱壁隔离 (G6)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T4A.1 | GraphExecutor并行执行添加Semaphore | 中 | patent-workflow/graph_executor.rs | 无 |
| T4A.2 | 高资源工具添加Semaphore许可 | 中 | core/src/tools/retry_dispatch.rs | 无 |

#### Phase 4B: 退避统一 (G7)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T4B.1 | 替换 core/src/util.rs backoff() | 低 | core/src/util.rs | T1.2 |
| T4B.2 | 替换 core/src/tools/retry_config.rs backoff() | 低 | core/src/tools/retry_config.rs | T1.2 |
| T4B.3 | 替换 codex-client/src/retry.rs backoff() | 低 | codex-client/src/retry.rs | T1.2 |
| T4B.4 | 替换 patent-agents/llm.rs inline backoff | 低 | patent-agents/llm.rs | T1.2 |
| T4B.5 | 替换 google_patents.rs inline backoff | 低 | patent-tools/google_patents.rs | T1.2 |

**检查清单**:
- [ ] 5处退避实现全部替换为 ExponentialBackoff
- [ ] 退避行为向后兼容（延迟范围一致）
- [ ] 所有现有测试仍然通过
- [ ] 无新增编译警告

#### Phase 4C: HITL超时 (G8)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T4C.1 | 定义 HumanApprovalConfig + TimeoutAction | 低 | patent-workflow/flow.rs | 无 |
| T4C.2 | HumanApproval wait添加超时处理 | 中 | patent-workflow/graph_executor.rs | T4C.1 |

#### Phase 4D: 健康检查 (G9)

| 任务ID | 任务 | 复杂度 | 改动文件 | 依赖 |
|--------|------|--------|----------|------|
| T4D.1 | 实现 HealthReport 类型 | 低 | core/src/resilience/health.rs | T2A.1 |
| T4D.2 | 实现健康状态聚合逻辑 | 中 | core/src/resilience/health.rs | T4D.1 |
| T4D.3 | CLI命令或端点暴露健康状态 | 中 | 待定 | T4D.2 |

---

## 四、风险评估

### 高风险项

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Agent重启循环中再次panic（双重故障） | 无限循环 | 严格限制 max_restarts，Unrecoverable后停止 |
| Mutex在恢复路径中poison | 恢复逻辑本身失败 | 使用 `lock().unwrap_or_else(|e| e.into_inner())` 处理poison |
| 熔断器误判正常服务为故障 | 正常请求被拒绝 | 合理设置阈值 + HalfOpen探测机制 |
| Turn检查点写入性能开销 | 降低吞吐量 | 异步写入 + 增量保存 |

### 中风险项

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 退避替换后行为变化 | 部分场景重试延迟不同 | 替换前测量原延迟，匹配预设配置 |
| 任务转移导致重复执行 | 幂等性问题 | 转移前记录已完成步骤，恢复时跳过 |
| CancellationToken在重启时泄露 | 资源泄露 | Restarting状态创建新child_token |

### Rust特定风险

| 风险 | 缓解 |
|------|------|
| std::thread 与 tokio 混合 | Agent重启在std::thread中执行，不依赖tokio runtime |
| catch_unwind 不能捕获所有panic | async panic无法捕获 — 确保关键路径是同步的 |
| Arc引用循环导致内存泄露 | RecoveryContext 不持有 AgentControl 的 Arc |
| Send + Sync 约束 | 所有新类型必须满足线程安全约束 |

---

## 五、测试策略

### 单元测试（每个Phase必须）

- **Phase 1**: ExponentialBackoff计算验证、CircuitBreaker状态机转换、RecoveryPolicy逻辑
- **Phase 2A**: 熔断器在连续失败后Open、HalfOpen探测成功后Closed
- **Phase 2B**: Agent重启计数、退避间隔、超过max_restarts后停止
- **Phase 2C**: Unresponsive检测触发回调、与熔断器联动
- **Phase 3A**: 任务转移匹配、无可用Agent时DLQ
- **Phase 3B**: 检查点写入/读取、崩溃后恢复正确性

### 集成测试（Phase完成后）

- **Phase 2完成后**: Agent panic → 自动重启 → 继续处理消息 → 熔断器阻止级联故障
- **Phase 3完成后**: Agent死亡 → 任务转移 → 其他Agent接手 → 工作流检查点恢复
- **全量完成后**: 模拟外部API故障 → 熔断器Open → Agent重启 → 任务转移 → 最终恢复

### 回归测试

每个Phase完成后运行：
```bash
cargo check -p codex-core -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools -p codex-patent-workflow
cargo clippy -p codex-core -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools -p codex-patent-workflow
cargo nextest run -p codex-core -p codex-patent-domain -p codex-patent-agents -p codex-patent-workflow
```

---

## 六、文件改动总览

### 新增文件（7个）

| 文件 | 模块 | Phase |
|------|------|-------|
| `core/src/resilience/mod.rs` | 模块入口 | Phase 1 |
| `core/src/resilience/backoff.rs` | 统一退避 | Phase 1 |
| `core/src/resilience/circuit_breaker.rs` | 熔断器 | Phase 1 |
| `core/src/resilience/recovery.rs` | 恢复策略 | Phase 1 |
| `core/src/resilience/client.rs` | 弹性客户端 | Phase 2A |
| `core/src/resilience/health.rs` | 健康检查 | Phase 4D |
| `codex-patent-workflow/src/checkpoint_turn.rs` | Turn检查点 | Phase 3B |

### 修改文件（18个）

| 文件 | 改动类型 | Phase |
|------|----------|-------|
| `core/src/lib.rs` | 注册resilience模块 | Phase 1 |
| `core/src/state/service.rs` | 添加CircuitBreakerRegistry | Phase 2A |
| `core/src/agent/registry.rs` | AgentMetadata添加recovery + 按role查询 | Phase 2B, 3A |
| `core/src/agent/control.rs` | AgentStatus添加Restarting + 回调注册 | Phase 2B, 2C |
| `core/src/agent/bus.rs` | 心跳回调 + pending_tasks + 事件 | Phase 2C, 3A |
| `core/src/client.rs` | 包装LLM调用 | Phase 2A |
| `core/src/util.rs` | 替换backoff() | Phase 4B |
| `core/src/tools/retry_config.rs` | 替换backoff() | Phase 4B |
| `core/src/tools/retry_dispatch.rs` | 添加Semaphore | Phase 4A |
| `codex-patent-agents/src/agent_runtime/spawn.rs` | 重启循环 | Phase 2B |
| `codex-patent-agents/src/agent_runtime/llm.rs` | 包装调用 + 替换backoff | Phase 2A, 4B |
| `codex-patent-tools/src/google_patents.rs` | 包装调用 + 替换backoff | Phase 2A, 4B |
| `codex-patent-tools/src/embedding_client.rs` | 包装调用 | Phase 2A |
| `codex-patent-workflow/src/graph_executor.rs` | 激活检查点 + Semaphore | Phase 3B, 4A |
| `codex-patent-workflow/src/checkpoint.rs` | Turn级检查点 | Phase 3B |
| `codex-patent-workflow/src/flow.rs` | HITL超时配置 | Phase 4C |
| `core/src/session/turn.rs` | 中间检查点保存 | Phase 3B |
| `codex-client/src/retry.rs` | 替换backoff() | Phase 4B |

---

## 七、验收标准

### 最终系统应满足

1. **单点故障不影响整体** (目标 9/10)
   - 任何Agent崩溃后60秒内自动恢复或任务转移
   - 外部API持续故障不耗尽系统资源（熔断器保护）

2. **Agent崩溃后可恢复** (目标 8/10)
   - panic后自动重启，最多3次
   - Unresponsive检测后自动恢复
   - 不可恢复时任务自动转移

3. **超时与重试策略** (保持 9/10)
   - 统一退避算法，所有路径行为一致
   - 熔断器在持续失败时快速失败

### 每Phase验收

- Phase 1: 所有新类型编译通过 + 单测覆盖率 > 90%
- Phase 2: 模拟Agent panic + 外部API故障，系统自动恢复
- Phase 3: 模拟Turn中途崩溃，从检查点恢复；Agent死亡后任务转移
- Phase 4: 退避统一、资源隔离、HITL超时、健康检查全部工作

# BCIP 专利 Crate 修复计划

> 基于 Karpathy 式全面代码审计 + 4 个独立验证 agent 的精确发现
> 生成时间: 2025-01-10
> 覆盖范围: 10 个专利 crate (core/domain/tools/agents/skills/constitutional/text/workflow/scheduler/knowledge)

---

## 执行优先级

| 优先级 | 说明 | 数量 |
|--------|------|------|
| **P0** | 编译错误风险 / 功能完全损坏 / 安全删除（零风险） | 4 项 |
| **P1** | 性能 bug / 死代码 / 架构债务 / 超大文件 | 4 项 |
| **P2** | 优化项 / 启用被忽略测试 / 体验改进 | 4 项 |

---

## Phase 2: P0 立即修复 — 清理未使用依赖和死代码

### 2.1 删除 codex-patent-skills crate（零消费者）

**验证状态**: ✅ 确认 — 没有任何外部 crate 依赖 codex-patent-skills
**风险**: 零 — 只有自身测试使用 `SkillLoader`

**步骤**:
1. **迁移 assets** — 将 `codex-patent-skills/assets/` → `codex-patent-agents/assets/skills/`
2. **更新路径** — 修改 `codex-patent-agents/src/roles.rs` 第 178-184 行:
   ```rust
   // 旧路径
   PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../codex-patent-skills/assets/_shared")
   // 新路径
   PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/skills/_shared")
   ```
3. **删除目录** — `rm -rf codex-patent-skills/`
4. **更新 workspace** — 从 `codex-rs/Cargo.toml` 的 `members` 和 `[workspace.dependencies]` 移除 `"codex-patent-skills"`

**预期结果**: 编译通过，功能无损，消除 195 LOC 死代码 + 299 LOC 死测试

---

### 2.2 清理未使用的 Cargo.toml 依赖

**验证状态**: ✅ 全部确认

| Crate | 未使用依赖 | 操作 |
|-------|-----------|------|
| `codex-patent-constitutional` | `regex`, `tracing` | 删除两行 |
| `codex-patent-knowledge` | `codex-patent-text`, `thiserror` | 删除两行 |
| `codex-patent-tools` | `thiserror`, `tracing` | 删除两行 |
| `codex-patent-workflow` | `thiserror` | 删除一行 |

**文件位置**:
- `codex-rs/codex-patent-constitutional/Cargo.toml` 第 12-13 行
- `codex-rs/codex-patent-knowledge/Cargo.toml` (需定位具体行)
- `codex-rs/codex-patent-tools/Cargo.toml` 第 18, 22 行
- `codex-rs/codex-patent-workflow/Cargo.toml` (需定位具体行)

---

### 2.3 删除未使用的 feature flags

**验证状态**: ✅ 确认 — 6 个 feature 在源码中无任何 `#[cfg]` 守卫

**文件**: `codex-rs/codex-patent-tools/Cargo.toml` 第 28-36 行

**操作**:
```toml
# 删除以下行:
search-tools = []
analysis-tools = []
review-tools = []
drafting-tools = []
management-tools = []
evaluation-tools = []

# 保留:
default = []
document-pdf = ["liteparse"]
```

---

### 2.4 删除旧格式 DEPRECATED TOML 文件

**验证状态**: ✅ 确认 — 9 个旧格式文件在 `assets/` 根目录，新版在 `assets/bcip/` 子目录

**文件列表** (位于 `codex-patent-agents/assets/`):
- `retriever.toml` → 由 `bcip/retriever.toml` 替代
- `analyzer.toml` → 由 `bcip/analyzer.toml` 替代
- `writer.toml` → 由 `bcip/writer.toml` 替代
- `novelty_checker.toml` → 由 `bcip/novelty_checker.toml` 替代
- `creativity_checker.toml` → 由 `bcip/creativity_checker.toml` 替代
- `infringement_checker.toml` → 由 `bcip/infringement_checker.toml` 替代
- `invalidity_checker.toml` → 由 `bcip/invalidity_checker.toml` 替代
- `reviewer.toml` → 由 `bcip/reviewer.toml` 替代
- `quality_checker.toml` → 由 `bcip/quality_checker.toml` 替代

**操作**: 删除上述 9 个文件，保留 `assets/bcip/*.toml`

**后续**: 检查 `roles.rs` 中的旧格式回退逻辑（第 346-369 行），确认是否需要保留或简化

---

## Phase 3: P0 立即修复 — Workflow 核心功能

### 3.1 SimplePlanGenerator 桩实现

**验证状态**: ✅ 确认 — `app-server/src/request_processors/workflow_processor.rs` 第 142-151 行返回 `Err("not implemented")`

**问题**: 这是 workflow 唯一生产消费者，始终失败意味着整个 workflow 系统不可用

**决策选项**:
- **选项 A**: 实现最小可用版本（基于 goal 字符串返回单步骤计划）
- **选项 B**: 如果 workflow 系统暂不启用，将整个 workflow_processor.rs 标记为 `#[cfg(feature = "workflow")]` 或移除
- **选项 C**: 如果短期内不实现，将错误信息改为更具描述性的 `"workflow planning is not yet available"`

**建议**: 先实施选项 C（最小改动），后续迭代中实现选项 A

---

### 3.2 resume_from_checkpoint 假恢复

**验证状态**: ✅ 确认 — `graph_executor.rs` 第 149 行 `self.execute(graph)` 忽略 checkpoint 状态

**文件**: `codex-rs/codex-patent-workflow/src/graph_executor.rs` 第 132-150 行

**问题代码**:
```rust
pub fn resume_from_checkpoint(&self, run_id: &str, graph: &FlowGraph) -> Result<GraphExecution, String> {
    let checkpoint = self.checkpoint_store.load_checkpoint(run_id)?
        .ok_or_else(|| format!("no checkpoint found for run {}", run_id))?;

    // 日志输出 checkpoint 信息但实际不使用
    tracing::info!(...);

    self.execute(graph)  // ← 问题: 从头执行，忽略 checkpoint.step_index
}
```

**修复方向**: 使用 `checkpoint.step_index` 跳过已完成的步骤，或从 `checkpoint.state` 恢复执行状态

**最小修复** (选项 A — 先跳过已完成的步骤):
```rust
pub fn resume_from_checkpoint(&self, run_id: &str, graph: &FlowGraph) -> Result<GraphExecution, String> {
    let checkpoint = self.checkpoint_store.load_checkpoint(run_id)?
        .ok_or_else(|| format!("no checkpoint found for run {}", run_id))?;

    let mut execution = self.execute(graph)?;
    // 恢复已完成步骤的结果
    for (step_id, result) in &checkpoint.state.step_results {
        execution.results.insert(step_id.clone(), result.clone());
    }
    execution.current_step = checkpoint.state.current_step;
    Ok(execution)
}
```

---

## Phase 4: P0 立即修复 — Domain 性能 Bug

### 4.1 interaction.rs Regex 重复编译

**验证状态**: ✅ 确认 — 第 79、94、108 行每次调用都 `regex::Regex::new(pattern)`

**文件**: `codex-rs/codex-patent-domain/src/interaction.rs`

**问题**: 3 个函数 (`is_frustrated`, `wants_continue`, `would_upgrade_effort`) 每次调用都重新编译 23 条正则表达式

**修复**: 使用 `regex_cache` (已存在但未被使用)

```rust
// 在文件顶部添加
use crate::rules::regex_cache::get_or_compile_regex;

// 修改 3 处:
// 旧: if let Ok(re) = regex::Regex::new(pattern) && re.is_match(text)
// 新: if get_or_compile_regex(pattern).map(|re| re.is_match(text)).unwrap_or(false)
```

**替代方案** (更简单): 如果 pattern 集合固定，使用 `lazy_static!` 或 `LazyLock` 预编译:
```rust
use regex::Regex;
use std::sync::LazyLock;

static NEGATIVE_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    NEGATIVE_PATTERNS.iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
});
```

---

### 4.2 quality_rules.rs YAML 重复解析

**验证状态**: ✅ 确认 — `load_config()` 每次调用都 `serde_yaml::from_str(RULES_YAML)`

**文件**: `codex-rs/codex-patent-domain/src/quality_rules.rs` 第 76-78 行

**问题**: 6 个函数各自调用 `load_config()`，同一请求中 YAML 被反复解析

**修复**:
```rust
use std::sync::OnceLock;

static CONFIG_CACHE: OnceLock<SpecQualityConfig> = OnceLock::new();

fn load_config() -> &'static SpecQualityConfig {
    CONFIG_CACHE.get_or_init(|| {
        serde_yaml::from_str(RULES_YAML).unwrap_or_default()
    })
}
```

---

## Phase 5: P1 重要重构 — 拆分超大文件

### 5.1 rule_engine.rs (1295 行 → 目标 <500 行)

**文件**: `codex-rs/codex-patent-domain/src/rule_engine.rs`

**拆分策略** (按功能域):
```
rule_engine/
  mod.rs          # 公共接口 (100 行)
  evaluator.rs    # 规则评估引擎 (300 行)
  scoring.rs      # 打分逻辑 (250 行)
  matching.rs     # 特征匹配 (250 行)
  context.rs      # 上下文构建 (200 行)
  utils.rs        # 辅助函数 (200 行)
```

---

### 5.2 graph_executor.rs (1200 行 → 目标 <500 行)

**文件**: `codex-rs/codex-patent-workflow/src/graph_executor.rs`

**拆分策略**:
```
graph_executor/
  mod.rs              # 公共接口 + 执行入口 (150 行)
  step_runner.rs      # 单步骤执行 (250 行)
  condition_router.rs # 条件分支路由 (200 行)
  checkpoint_mgr.rs   # 检查点管理 (150 行)
  parallel.rs         # 并行执行 (200 行)
  error_handler.rs    # 错误处理 (150 行)
```

---

### 5.3 legal_tools.rs (1220 行)

**文件**: `codex-rs/codex-patent-tools/src/legal_tools.rs`

**拆分策略** (按工具类型):
```
legal_tools/
  mod.rs           # 注册和分发 (100 行)
  patent_search.rs # 专利检索工具 (300 行)
  legal_analysis.rs # 法律分析工具 (300 行)
  document_gen.rs  # 文书生成工具 (250 行)
  utils.rs         # 辅助函数 (200 行)
```

---

## Phase 6: P1 重要重构 — 清理死代码

### 6.1 codex-patent-core 死类型

**文件**: `codex-rs/codex-patent-core/src/types.rs`

**可删除/迁移**:
- `LegalLayer` — 无任何消费者
- `LegalEntityType` — 无任何消费者
- `RelationCategory` — 无任何消费者
- `RuleViolation` — 仅在 constitutional 中使用，可迁出
- `PatentDocument` — 仅在 domain 中使用，可迁出
- `AnalysisResult` — 仅在 domain 中使用，可迁出

**操作**: 先标记 `#[deprecated]`，确认无外部引用后删除

---

### 6.2 codex-patent-text 死代码

**文件**: `codex-rs/codex-patent-text/src/similarity.rs`

**问题**: `cosine_similarity()` 无任何消费者

**操作**: 删除整个 `similarity.rs` 模块（140 LOC），从 `lib.rs` 移除导出

---

### 6.3 constitutional 假规则

**文件**: `codex-rs/codex-patent-constitutional/src/engine.rs`

**问题**: 22 个 `RuleCheck` 变体中 16 个 (73%) 走 fallback 返回 0.5 置信度

**决策选项**:
- 选项 A: 删除未实现的变体（激进但正确）
- 选项 B: 添加 `#[deprecated]` 标记未实现变体，提示使用者
- 选项 C: 保持现状但添加 `tracing::warn!` 在 fallback 时记录

**建议**: 选项 B + C — 标记未实现变体并添加警告日志

---

### 6.4 agents 死 API

**文件**: `codex-rs/codex-patent-agents/src/lib.rs`

**问题**: `ScenarioRegistry` (473 LOC)、`AgentRegistry`、`ReflectionEngine` 方法、`LearningStore` 方法被 `pub use` 导出但无外部使用

**操作**:
1. 将 `pub use` 改为 `pub(crate) use` 缩小可见性
2. 如果确认无内部使用，进一步删除

```rust
// 修改前
pub use scenario::ScenarioRegistry;
pub use roles::AgentRegistry;
pub use reflection::ReflectionEngine;
pub use learning::LearningStore;

// 修改后
pub(crate) use scenario::ScenarioRegistry;
pub(crate) use roles::AgentRegistry;
// 如果 reflection/learning 无内部引用，直接删除
```

---

### 6.5 零字段结构体 → 纯函数

**文件**: 多个 domain 文件

**列表** (13 个):
- `InventionClassifier` — `invention_classifier.rs:12`
- `ArgumentationLibrary` — `examiner_simulator/types.rs:61`
- `QualityAssessor` — `quality.rs:18`
- `InvalidityPipeline` — `invalidity.rs:64`
- `ClaimParser` — `claim_parser.rs:13`
- `FeatureMatcher` — `compare.rs:165`
- `ClaimGenerator` — `claim_generator.rs:27`
- `OaParser` — `oa.rs:13`
- `OaResponder` — `oa.rs:131`
- `ComprehensiveAnalyzer` — `analysis.rs:94`
- `DisclosureParser` — `disclosure.rs:15`
- `FeatureExtractor` — `disclosure.rs:140`
- `InfringementPipeline` — `infringement.rs:33`

**操作**: 将 `impl StructName { pub fn method(...) }` 改为 `pub fn method(...)` 纯函数

---

## Phase 7: P1 重要重构 — 类型系统清理

### 7.1 CaseContext God Struct

**文件**: `codex-rs/codex-patent-core/src/types.rs`

**问题**: 22 个 `Option<T>` 字段 — 任何调用者都不知道哪些字段是必需的

**重构方向**:
```rust
// 拆分为上下文特定的子结构体
pub struct CaseContext {
    pub id: String,
    pub case_type: CaseType,
    pub patent_info: Option<PatentInfo>,    // ← 提取子结构体
    pub applicant_info: Option<ApplicantInfo>,
    pub technical_field: Option<TechnicalField>,
    pub priority_claims: Vec<PriorityClaim>,
    // ... 其余字段按语义分组
}

struct PatentInfo { publication_number, application_number, filing_date, ... }
struct ApplicantInfo { applicant_name, applicant_address, ... }
struct TechnicalField { ipc_codes, keywords, ... }
```

---

### 7.2 统一质量评估类型

**问题**: `DraftQualityReport` + `QualityAssessment` 两套并行类型系统

**操作**: 选择一套作为主类型，另一套添加 `From`/`Into` 转换，逐步统一

---

## Phase 8: P2 架构改进

### 8.1 Scheduler 优化

**文件**: `codex-rs/codex-patent-scheduler/src/scheduler.rs`

**问题列表**:
1. `run_loop` 每秒轮询 → 计算 `next_run - now()` 精确 sleep
2. `next_run()` 线性扫描一整年 → 使用数学计算（模运算 + 查找表）
3. `uuid_simple()` 16 位随机 hex → 评估是否够用，或改用 `uuid` crate
4. `notify` 依赖未使用 → 删除或实现文件监听触发

---

### 8.2 CJK 分词器改进

**文件**: `codex-rs/codex-patent-text/src/tokenizer.rs`

**问题**: CJK 按字符分割（"专利申请"→[专,利,申,请]）

**建议**: 添加 jieba-rs 或类似的 CJK 分词依赖，或实现基于词典的前向最大匹配

**注意**: 这会引入新依赖，需权衡收益

---

### 8.3 启用被忽略的测试

**文件**: `codex-rs/codex-patent-knowledge/tests/` (所有测试标记 `#[ignore]`)

**操作**:
1. 移除 `#[ignore]` 属性
2. 如果测试需要外部服务，改为条件编译 `#[cfg(feature = "integration-tests")]`
3. 或者使用 `mockall` / `wiremock` 模拟外部依赖

---

### 8.4 缓存简化

**文件**: `codex-rs/codex-patent-knowledge/src/law_db.rs`

**问题**: 三重嵌套全局缓存 `OnceLock<Mutex<HashMap<..., Arc<Mutex<Connection>>>>>`

**简化方向**:
```rust
// 使用连接池替代手动管理
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;

static DB_POOL: OnceLock<Pool<SqliteConnectionManager>> = OnceLock::new();
```

---

## Phase 9: 验证清单

每完成一个 Phase 后运行:

```bash
# 1. 编译检查（所有专利 crate）
cargo check -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

# 2. Clippy 检查（0 警告）
cargo clippy -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

# 3. 格式化检查
cargo fmt --check

# 4. 测试
cargo nextest run --no-fail-fast -p codex-patent-domain -p codex-patent-agents -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills
```

---

## 附录: 快速修复速查表

| 文件 | 行号 | 问题 | 修复 |
|------|------|------|------|
| `app-server/src/request_processors/workflow_processor.rs` | 142-151 | SimplePlanGenerator 返回 Err | 实现最小计划生成或标记未实现 |
| `codex-patent-workflow/src/graph_executor.rs` | 132-150 | resume_from_checkpoint 假恢复 | 使用 checkpoint 状态恢复执行位置 |
| `codex-patent-domain/src/interaction.rs` | 79, 94, 108 | Regex 重复编译 | 使用 regex_cache 或 LazyLock |
| `codex-patent-domain/src/quality_rules.rs` | 76-78 | YAML 重复解析 | 使用 OnceLock 缓存 |
| `codex-patent-tools/Cargo.toml` | 30-35 | 6 个未使用 feature flags | 删除 |
| `codex-patent-agents/assets/*.toml` | — | 9 个 DEPRECATED 旧格式文件 | 删除 |
| `codex-patent-constitutional/Cargo.toml` | 12-13 | regex, tracing 未使用 | 删除 |
| `codex-patent-knowledge/Cargo.toml` | — | codex-patent-text, thiserror 未使用 | 删除 |
| `codex-patent-agents/src/lib.rs` | 54-57 | 死 API pub use | 改为 pub(crate) |
| `codex-patent-text/src/similarity.rs` | 全部 | cosine_similarity 无消费者 | 删除文件 |

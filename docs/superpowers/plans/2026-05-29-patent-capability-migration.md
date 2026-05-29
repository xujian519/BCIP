# 专利能力深度集成实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将 YunXi 的全部专利能力（50+ 工具 / 9 Agent / 知识图谱 / 法规库 / 知识卡片）深度集成为 BCIP 的 native crate。

**Architecture:** 在 codex-rs workspace 下新建 7 个 `codex-patent-*` crate。知识资产从 YunXi 拷贝。专利工具通过 BCIP tool handler trait 注册。Agent 角色用 TOML 运行时加载。不引入 BGE-M3 向量检索，CNIPA 复用已有 skill。

**Tech Stack:** Rust (rusqlite, serde, serde_json, serde_yaml, strsim, tokio, regex), SQLite (FTS5), TOML。

**设计文档:** `docs/superpowers/specs/2026-05-29-patent-capability-migration-design.md`

---

### 阶段0: 地基 — 类型、资产与 Cargo 配置

#### Task 0.1: 复制知识资产文件

**Files:**
- Copy: 从 YunXi 复制全部知识资产到 `codex-rs/codex-patent-assets/`

- [ ] **Step 1: 创建资产目录并复制 SQLite 数据库**

```
mkdir -p codex-rs/codex-patent-assets/{documents/books,cards}
cp /Users/xujian/projects/YunXi/assets/knowledge-base/patent_kg.db codex-rs/codex-patent-assets/
cp /Users/xujian/projects/YunXi/assets/knowledge/data/laws.db codex-rs/codex-patent-assets/
cp /Users/xujian/projects/YunXi/assets/knowledge/data/laws-full.db codex-rs/codex-patent-assets/
```

- [ ] **Step 2: 复制知识卡片和文档**

```
cp /Users/xujian/projects/YunXi/assets/knowledge-base/card-index.json codex-rs/codex-patent-assets/
cp -r /Users/xujian/projects/YunXi/assets/knowledge-base/cards/* codex-rs/codex-patent-assets/cards/
cp /Users/xujian/projects/YunXi/assets/knowledge-base/Concept-Hierarchy.md codex-rs/codex-patent-assets/
cp -r /Users/xujian/projects/YunXi/assets/knowledge-base/20260429-* codex-rs/codex-patent-assets/documents/
```

- [ ] **Step 3: 复制书籍骨架和专题文档**

```
for dir in 书籍 专利判决 专利侵权 专利实务 审查指南 复审无效 法律法规 商标 方法论; do
  src="/Users/xujian/projects/YunXi/assets/knowledge-base/$dir"
  [ -d "$src" ] && cp -r "$src" codex-rs/codex-patent-assets/documents/
done
cp -r /Users/xujian/projects/YunXi/assets/knowledge-base/书籍/* codex-rs/codex-patent-assets/documents/books/ 2>/dev/null || true
```

- [ ] **Step 4: 提交**

```
git add codex-rs/codex-patent-assets/
git commit -m "feat(patent): 迁移 YunXi 知识资产文件"
```

---

#### Task 0.2: 创建 codex-patent-core crate — 领域核心类型

**Files:**
- Create: `codex-rs/codex-patent-core/Cargo.toml`
- Create: `codex-rs/codex-patent-core/src/lib.rs`
- Create: `codex-rs/codex-patent-core/src/types.rs`
- Create: `codex-rs/codex-patent-core/src/error.rs`
- Modify: `codex-rs/Cargo.toml` (workspace members)

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "codex-patent-core"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = "2"
```

- [ ] **Step 2: 创建 `src/error.rs`**

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatentError {
    #[error("kg error: {0}")]
    KnowledgeGraph(String),
    #[error("law db error: {0}")]
    LawDb(String),
    #[error("search error: {0}")]
    Search(String),
    #[error("claim parse error: {0}")]
    ClaimParse(String),
    #[error("rule engine error: {0}")]
    RuleEngine(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
}
```

- [ ] **Step 3: 创建 `src/types.rs`** — 完整领域类型定义（详见设计文档第5节，具体类型参见 YunXi `patent-domain/src/models.rs`：`ClaimType`, `ParsedClaim`, `ParsedFeature`, `CaseContext`, `AnalysisResult`, `KgNode`, `KgEdge`, `LawDocument`, `KnowledgeCard`, `SearchResult`, `InvaltionDecision`, `DraftQualityReport`, `CompareFeature`, `FeatureMatchResult`, `PatentDocument`, `RuleViolation` 等 30+ 类型）

- [ ] **Step 4: 创建 `src/lib.rs`**

```rust
mod error;
mod types;
pub use error::PatentError;
pub use types::*;
```

- [ ] **Step 5: 加入 workspace** — 在 `codex-rs/Cargo.toml` members 数组中插入 `"crates/codex-patent-core",`

- [ ] **Step 6: 验证编译**

```
cargo check -p codex-patent-core
```

- [ ] **Step 7: 提交**

```
git add codex-rs/codex-patent-core/ codex-rs/Cargo.toml
git commit -m "feat(patent): 创建 codex-patent-core crate — 领域类型与错误定义"
```

---

### 阶段1: 知识底座 — 知识库引擎

#### Task 1.1: 创建 codex-patent-knowledge crate 骨架

**Files:**
- Create: `codex-rs/codex-patent-knowledge/Cargo.toml`
- Create: `codex-rs/codex-patent-knowledge/src/lib.rs`
- Modify: `codex-rs/Cargo.toml`

- [ ] **Step 1: 创建 Cargo.toml**

```toml
[package]
name = "codex-patent-knowledge"
version = "0.1.0"
edition = "2024"

[dependencies]
codex-patent-core = { path = "../codex-patent-core" }
rusqlite = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = "2"
```

- [ ] **Step 2: 创建 `src/lib.rs`**

```rust
pub mod cards;
pub mod graph;
pub mod law_db;
pub mod search;
pub mod synonym;

pub use cards::CardIndex;
pub use graph::SqliteKnowledgeGraph;
pub use law_db::LawDatabase;
pub use search::{SearchConfig, UnifiedSearch};
```

- [ ] **Step 3: 加入 workspace, 验证编译, 提交**

---

#### Task 1.2: 知识图谱引擎 (`graph.rs`)

**Files:** Create: `codex-rs/codex-patent-knowledge/src/graph.rs`

从 YunXi `rust/crates/patent-domain/src/sqlite_graph.rs` 逐行迁移核心代码。关键 struct 和方法：

```rust
use rusqlite::{Connection, OpenFlags, params};
use codex_patent_core::{KgNode, KgEdge};

pub struct SqliteKnowledgeGraph { conn: Connection }

impl SqliteKnowledgeGraph {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String>
    pub fn from_connection(conn: Connection) -> Self  // 用于内存 KG (kg_seed)
    pub fn stats(&self) -> Result<KgStats, String>
    pub fn search_nodes(&self, query: &str, node_type: Option<&str>, limit: usize) -> Result<Vec<KgNode>, String>
    pub fn get_edges(&self, node_id: &str) -> Result<Vec<KgEdge>, String>
    pub fn get_nodes_by_type(&self, node_type: &str, limit: usize) -> Result<Vec<KgNode>, String>
    pub fn node_type_distribution(&self) -> Result<Vec<NodeTypeCount>, String>
}
```

**SQL 查询模式**: `nodes_fts` FTS5 全文搜索 + 可选的 `node_type` 过滤。直接复制 YunXi 的 SQL 语句。

- [ ] 实现 → 验证编译 → 提交: `feat(patent): 实现知识图谱引擎 — SQLite + FTS5 节点/边查询`

---

#### Task 1.3: 法律法规数据库 (`law_db.rs`)

**Files:** Create: `codex-rs/codex-patent-knowledge/src/law_db.rs`

从 YunXi `rust/crates/knowledge/src/law_db.rs` 逐行迁移。关键方法：

```rust
pub struct LawDatabase { conn: Connection }

impl LawDatabase {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String>
    pub fn search_by_name(&self, keyword: &str, limit: usize) -> Result<Vec<LawDocument>, String>
    pub fn search_by_content(&self, keyword: &str, limit: usize) -> Result<Vec<LawDocument>, String>
    pub fn list_by_level(&self, level: &str, limit: usize) -> Result<Vec<LawDocument>, String>
    pub fn list_levels(&self) -> Result<Vec<String>, String>
    pub fn list_categories(&self) -> Result<Vec<LawCategory>, String>
    pub fn count(&self) -> Result<usize, String>
    pub fn list_all(&self, limit: usize, offset: usize) -> Result<Vec<LawDocument>, String>
}
```

**搜索策略**: LIKE `%keyword%` 模糊匹配（YunXi 实际实现，非 FTS5）。

- [ ] 实现 → 验证编译 → 提交: `feat(patent): 实现法律法规数据库 — LIKE 搜索`

---

#### Task 1.4: 知识卡片索引 (`cards.rs`)

**Files:** Create: `codex-rs/codex-patent-knowledge/src/cards.rs`

从 YunXi `rust/crates/knowledge/src/knowledge_cards.rs` 逐行迁移：

```rust
pub struct CardIndex { cards: Vec<KnowledgeCard>, base_dir: String }

impl CardIndex {
    pub fn load(index_path: &str) -> Result<Self, String>
    pub fn len(&self) -> usize
    pub fn all(&self) -> &[KnowledgeCard]
    pub fn search_by_keyword(&self, keyword: &str, limit: usize) -> Vec<&KnowledgeCard>
    pub fn filter_by_quality(&self, threshold: f64, limit: usize) -> Vec<&KnowledgeCard>
    pub fn load_content(&self, card: &KnowledgeCard) -> Result<String, String>
    pub fn search_with_content(&self, keyword: &str, limit: usize) -> Vec<&KnowledgeCard>
}
```

**搜索策略**: 标题权重×3 + 概念权重×5 + 领域权重×2 的加权关键词匹配。

- [ ] 实现 → 验证编译 → 提交: `feat(patent): 实现知识卡片索引 — 关键词/质量分检索`

---

#### Task 1.5: 同义词词典 (`synonym.rs`)

**Files:** Create: `codex-rs/codex-patent-knowledge/src/synonym.rs`

从 YunXi `rust/crates/tools/src/patent_search/synonym.rs` 迁移 70+ 专利术语同义词词典：

```rust
pub struct SynonymDict { entries: HashMap<String, Vec<String>> }

impl SynonymDict {
    pub fn new() -> Self  // 54+ 条核心同义词 (新颖性/创造性/侵权/无效/权利要求/说明书等)
    pub fn expand(&self, term: &str) -> Vec<&str>  // 正向+反向扩展
    pub fn search_synonyms(&self, keyword: &str) -> Vec<String>  // 模糊搜索
}
```

- [ ] 实现 → 验证编译 → 提交: `feat(patent): 实现同义词词典 — 70+ 专利术语扩展`

---

#### Task 1.6: 统一搜索接口 (`search.rs`)

**Files:** Create: `codex-rs/codex-patent-knowledge/src/search.rs`

从 YunXi `rust/crates/knowledge/src/search.rs` 迁移核心结构，简化语义搜索部分（不引入 BGE-M3）：

```rust
pub enum SearchMode { Text, Hybrid }

pub struct SearchConfig {
    pub query: String, pub limit: usize,
    pub search_kg: bool, pub search_law: bool, pub search_cards: bool,
    pub min_card_quality: f64, pub mode: SearchMode,
}

pub struct UnifiedSearch {
    kg: Option<SqliteKnowledgeGraph>,
    law_db: Option<LawDatabase>,
    card_index: Option<CardIndex>,
    synonym_dict: SynonymDict,
}

impl UnifiedSearch {
    pub fn new(kg_path: Option<&str>, law_db_path: Option<&str>, card_index_path: Option<&str>) -> Self
    pub fn search(&self, config: &SearchConfig) -> Vec<SearchResult>  // 跨源聚合 + 同义词扩展
    pub fn status(&self) -> serde_json::Value
}
```

**搜索逻辑**: 对 query 做同义词扩展 → 并行查图谱(含同义词) → 法规 → 卡片 → 按分数排序截断。

- [ ] 实现 → 验证编译 → 提交: `feat(patent): 实现统一搜索接口 — 跨图谱/法规/卡片搜索`

---

#### Task 1.7: 知识库引擎集成测试

**Files:** Create: `codex-rs/codex-patent-knowledge/tests/integration_test.rs`

- [ ] 编写 6 个测试用例: 打开 KG、搜索 KG、打开法规库、搜索法规、加载卡片索引、统一搜索
- [ ] 运行测试: `cargo test -p codex-patent-knowledge`
- [ ] 提交

---

### 阶段2: 检索系统 — 专利获取

#### Task 2.1: 创建 codex-patent-tools crate 骨架

**Files:**
- Create: `codex-rs/codex-patent-tools/Cargo.toml`
- Create: `codex-rs/codex-patent-tools/src/lib.rs`
- Modify: `codex-rs/Cargo.toml`

依赖: `codex-patent-core`, `codex-patent-knowledge`, `reqwest`, `tokio`, `serde`, `serde_json`

- [ ] 创建 → 加入 workspace → 验证编译 → 提交

---

#### Task 2.2: Google Patents 检索和批量下载

**Files:** Create: `codex-rs/codex-patent-tools/src/google_patents.rs`

从 YunXi `rust/crates/tools/src/patent_search/` 迁移，纯 Rust reqwest 实现：

```rust
pub async fn fetch_google_patents(input: GooglePatentsInput) -> Result<Vec<PatentResult>, String>
pub async fn download_patent(input: PatentDownloadInput) -> Result<String, String>  // PDF 下载到本地
```

**约束**: Google Patents 动态渲染页面，纯静态 HTTP 解析可能有限。基础实现返回结构化结果，后续可对接其他数据源。

- [ ] 实现 → 验证编译 → 提交

---

#### Task 2.3: 统一检索 + 检索式构建 + 迭代检索

**Files:**
- Create: `codex-rs/codex-patent-tools/src/patent_search.rs`
- Create: `codex-rs/codex-patent-tools/src/search_tools.rs`

从 YunXi `rust/crates/tools/src/patent_search/` 迁移核心逻辑：

```rust
// patent_search.rs
pub async fn patent_search(input: PatentSearchInput) -> Result<Value, String>
    // 同义词扩展 → Google Patents 检索
pub async fn search_query_builder(input: SearchQueryBuilderInput) -> Result<Value, String>
    // 3阶段: 精确 → 语义 → 变体
pub async fn iterative_search(input: IterativeSearchInput) -> Result<Value, String>
    // 多轮扩展检索

// search_tools.rs — 工具 handler 注册
pub fn register_search_tools() -> HashMap<String, ToolFn>
    // PatentSearch, GooglePatentsFetch, SearchQueryBuilder, IterativeSearch, PatentDownload
```

- [ ] 实现 → 验证编译 → 提交

---

### 阶段3: 解析与对比 — 专利领域服务

#### Task 3.1: 创建 codex-patent-domain crate 骨架

**Files:**
- Create: `codex-rs/codex-patent-domain/Cargo.toml`
- Create: `codex-rs/codex-patent-domain/src/lib.rs`
- Modify: `codex-rs/Cargo.toml`

依赖: `codex-patent-core`, `codex-patent-knowledge`, `rusqlite`, `serde`, `serde_json`, `serde_yaml`, `regex`, `strsim`

```rust
pub mod claim_parser;
pub mod compare;
pub mod drafting;
pub mod examiner_simulator;
pub mod guideline_graph;
pub mod invalid_decision;
pub mod kg_seed;
pub mod legal_reasoning;
pub mod retrieval;
pub mod rule_engine;
pub mod rules;
```

- [ ] 创建 → 加入 workspace → 验证编译 → 提交

---

#### Task 3.2: 权利要求解析器 (`claim_parser.rs`)

**Files:** Create: `codex-rs/codex-patent-domain/src/claim_parser.rs`

从 YunXi `patent-domain/src/claim_parser.rs` 逐行迁移。核心功能：解析独立/从属权利要求，识别前序/过渡词/特征，提取「所述XX」组件，计算 Jaccard 特征相似度，分类对应关系(精确/等同/差异/缺失)。

包含单元测试(独立权利要求解析、从属权利要求解析、特征相似度)。

- [ ] 实现 → `cargo test -p codex-patent-domain -- claim_parser` → 提交

---

#### Task 3.3: 规则引擎 (`rule_engine.rs`)

**Files:** Create: `codex-rs/codex-patent-domain/src/rule_engine.rs`

从 YunXi `patent-domain/src/rule_engine.rs` 迁移 9 条纯 Rust 规则：
- NR-01: 单独对比原则 / NR-02: 区别技术特征 / NR-03: 实质相同判断
- IR-01: 技术效果显著性 / IR-02: 性能提升幅度 / IR-03: 显而易见性判断
- OA-01: 新颖性驳回应对 / OA-02: 创造性驳回应对 / OA-03: 跨领域组合应对

包含单元测试(新颖性有区别特征、创造性显而易见)。

- [ ] 实现 → `cargo test -p codex-patent-domain -- rule_engine` → 提交

---

#### Task 3.4: 专利对比矩阵 (`compare.rs`)

**Files:** Create: `codex-rs/codex-patent-domain/src/compare.rs`

从 YunXi `patent-domain/src/compare/mod.rs` 迁移：`FeatureMatcher::compare()` (精确匹配≥0.95 / 等同匹配≥0.7)、`lexical_similarity()` (bigram Jaccard)、`ipc_alignment()` (IPC 前缀匹配)。

- [ ] 实现 → `cargo test -p codex-patent-domain -- compare` → 提交

---

#### Task 3.5: 审查员模拟器 (`examiner_simulator.rs`)

**Files:** Create: `codex-rs/codex-patent-domain/src/examiner_simulator.rs`

从 YunXi 迁移：5 种驳回类型检测、4 种论证策略、答复评分(completeness 25% + persuasiveness 30% + technical_depth 25% + logic_consistency 20%)。

- [ ] 实现 → 验证编译 → 提交

---

#### Task 3.6: 法律推理引擎 (`legal_reasoning.rs`)

**Files:** Create: `codex-rs/codex-patent-domain/src/legal_reasoning.rs`

从 YunXi 迁移：基于知识图谱的结构化推理 — 三步法新颖性分析、侵权分析方法(全面覆盖+等同原则)、`perform_novelty_analysis()` 纯文本对比分析。

- [ ] 实现 → 验证编译 → 提交

---

#### Task 3.7: 其余领域模块批量实现

**Files:** 创建 7 个文件:

| 文件 | YunXi 来源 | 核心功能 |
|------|-----------|---------|
| `drafting.rs` | `patent-domain/src/drafting.rs` | 7 维度撰写质量评估 |
| `invalid_decision.rs` | `patent-domain/src/invalid_decision.rs` | 无效决定存储与检索 |
| `guideline_graph.rs` | `patent-domain/src/guideline_graph.rs` | 法律实体关系图加载 |
| `kg_seed.rs` | `patent-domain/src/kg_seed.rs` | 内存 KG 种子数据(测试用) |
| `rules/schema.rs` | `patent-domain/src/rules/schema.rs` | YAML 规则 schema |
| `rules/engine.rs` | `patent-domain/src/rules/engine.rs` | 规则求值引擎 |
| `rules/checks.rs` | `patent-domain/src/rules/checks.rs` | 检查函数(required/pattern/min/max/enum) |

需要在 `SqliteKnowledgeGraph` 中添加 `from_connection()` 方法以支持内存 KG。

- [ ] 实现所有文件 → `cargo check -p codex-patent-domain` → 提交

---

### 阶段4: 核心分析引擎 — 分析工具注册

#### Task 4.1: 分析工具注册到 codex-patent-tools

**Files:**
- Create: `codex-rs/codex-patent-tools/src/analysis_tools.rs`
- Modify: `codex-rs/codex-patent-tools/src/lib.rs`

实现 7 个分析工具的 handler，封装 `codex-patent-domain` 和 `codex-patent-knowledge` 的能力：

| 工具 | Input struct | 核心调用 |
|------|-------------|---------|
| `ClaimParse` | `{claim_text, claim_number}` | `ClaimParser::parse()` |
| `ClaimCompare` | `{claim_a, claim_b}` | `ClaimParser::feature_similarity()` + `classify_correspondence()` |
| `NoveltyAnalysis` | `{invention_description, prior_art_descriptions, differences}` | `QualitativeRuleEngine::analyze_novelty()` + 文本对比 |
| `InventivenessAnalysis` | `{invention_description, technical_effect, performance_improvement, obviousness}` | `QualitativeRuleEngine::analyze_inventiveness()` |
| `InfringementAnalysis` | `{claim_text, accused_product_description}` | `FeatureMatcher::compare()` |
| `LegalQA` | `{question}` | 知识库搜索基础回答 |
| `KnowledgeSearch` | `{query, limit}` | `UnifiedSearch::search()` |

添加 `sync_tool!` 宏简化所有 handler 注册。在 `lib.rs` 中提供 `register_all_tools()` 返回 `HashMap<String, ToolFn>`。

- [ ] 实现 → `cargo check -p codex-patent-tools` → 提交

---

### 阶段5: 审查与撰写工具

#### Task 5.1: 审查与撰写工具注册

**Files:**
- Create: `codex-rs/codex-patent-tools/src/review_tools.rs`
- Create: `codex-rs/codex-patent-tools/src/drafting_tools.rs`
- Modify: `codex-rs/codex-patent-tools/src/lib.rs`

实现 10 个审查与撰写工具的 handler：

**审查工具**:
| 工具 | 核心功能 |
|------|---------|
| `FormalCheck` | 权利要求编号连续性 + 引用有效性 + 章节完整性检查 |
| `QualityAssess` | 调用 `evaluate_draft()` 7 维度评分 |
| `SubjectMatterCheck` | 5 类排除客体检测(智力活动/医疗/核/发现/游戏) |
| `UnityCheck` | 权利要求间共同术语 Jaccard 分析 |
| `OaStrategy` | 调用 `QualitativeRuleEngine::suggest_oa_strategy()` |
| `ResponseTemplate` | 6 个内置模板(新颖性争辩/创造性争辩/修改方案/充分公开/证据不足/延期) |

**撰写工具**:
| 工具 | 核心功能 |
|------|---------|
| `SpecificationDrafter` | 组装技术领域+背景+发明内容+实施例为说明书 |
| `ClaimGenerator` | 生成独立权利要求 + 从属权利要求 |
| `AbstractDrafter` | 生成摘要(技术问题+方案+效果) |
| `InnovationEvaluator` | 调用规则引擎评估创新度(高/中/低) |

在 `lib.rs` 中更新 `register_all_tools()` 添加所有 10 个工具。

- [ ] 实现 → `cargo check -p codex-patent-tools` → 提交

---

### 阶段6: Agent 与 Skill 融合

#### Task 6.1: 创建 codex-patent-agents crate — 9 个 Agent 角色

**Files:**
- Create: `codex-rs/codex-patent-agents/Cargo.toml`
- Create: `codex-rs/codex-patent-agents/src/lib.rs`
- Create: `codex-rs/codex-patent-agents/src/roles.rs`
- Create: 9 个 TOML 角色定义文件: `codex-rs/codex-patent-agents/assets/{role_id}.toml`
- Modify: `codex-rs/Cargo.toml`

**Cargo.toml 依赖**: `serde`, `toml`, `thiserror`

**`roles.rs` 核心设计**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRoleConfig {
    pub role_id: String,
    pub name: String,
    pub identity: String,
    pub methodology: Vec<MethodologyStep>,
    pub output_format: String,
    pub primary_tools: Vec<String>,
    pub secondary_tools: Vec<String>,
    pub constraints: Vec<String>,
}

pub enum PatentAgentRole {
    Retriever, Analyzer, Writer, NoveltyChecker, CreativityChecker,
    InfringementChecker, InvalidityChecker, Reviewer, QualityChecker,
}

impl PatentAgentRole {
    pub fn load_config(role_dir: &str) -> Result<HashMap<String, AgentRoleConfig>, String>
    pub fn system_prompt(&self, config: &AgentRoleConfig) -> String
    pub fn allowed_tools(&self, config: &AgentRoleConfig) -> Vec<String>
    pub fn name(&self) -> &'static str
    pub fn from_str(s: &str) -> Option<Self>
}
```

**TOML 角色文件示例** (`retriever.toml`):

```toml
role_id = "retriever"
name = "检索专家"

[[methodology]]
step_number = 1
step_name = "构建检索式"
description = "基于发明信息构建多层级检索式"

[[methodology]]
step_number = 2
step_name = "多源检索"
description = "专利全文、复审决定、知识图谱跨源并行检索"

[[methodology]]
step_number = 3
step_name = "结果筛选"
description = "按相关度排序并筛选最相关的对比文件"

output_format = "检索报告: 对比文件列表 + 相关性说明 + 引用分析"
primary_tools = ["PatentSearch", "GooglePatentsFetch", "IterativeSearch", "KnowledgeSearch"]
secondary_tools = ["SynonymSearch", "SearchQueryBuilder", "WebSearch", "WebFetch"]
constraints = ["至少检索3种不同来源", "优先使用结构化检索式", "每个对比文件需标注相关度"]
```

其他 8 个角色文件同理。从 YunXi `assets/agents/*.xml` 转为 TOML 格式。

- [ ] 实现所有文件 → `cargo check -p codex-patent-agents` → 提交

---

#### Task 6.2: 创建 codex-patent-skills crate — 26 个 Skill 定义

**Files:**
- Create: `codex-rs/codex-patent-skills/Cargo.toml`
- Create: `codex-rs/codex-patent-skills/src/lib.rs`
- Create: `codex-rs/codex-patent-skills/assets/` 下所有 skill 定义文件
- Modify: `codex-rs/Cargo.toml`

**Skill 结构**:

```
codex-patent-skills/assets/
├── _shared/
│   ├── legal_reasoning.toml
│   ├── hitl_protocol.toml
│   ├── output_standards.toml
│   ├── quality_checklist.toml
│   └── patent_glossary.toml
├── cap-retrieval.toml
├── cap-analysis.toml
├── cap-writing.toml
├── cap-disclosure-exam.toml
├── cap-inventive.toml
├── cap-clarity-exam.toml
├── cap-invalid.toml
├── cap-prior-art-ident.toml
├── cap-response.toml
├── cap-formal-exam.toml
├── foundation-hitl.toml
└── stop-slop.toml
```

**Skill 加载器** (`src/lib.rs`):

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SkillDefinition {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub includes: Vec<String>,  // _shared 模块引用
    pub required_tools: Vec<String>,
}

pub struct SkillLoader {
    skills: HashMap<String, SkillDefinition>,
    shared: HashMap<String, SkillDefinition>,
}

impl SkillLoader {
    pub fn load(skills_dir: &str) -> Result<Self, String>
    pub fn resolve(&self, skill_id: &str) -> Result<String, String>  // 展开 include 引用
    pub fn list(&self) -> Vec<&str>
}
```

**TOML Skill 示例** (`cap-retrieval.toml`):

```toml
skill_id = "cap-retrieval"
name = "专利检索能力"
description = "智能化专利文献搜索与筛选能力"
includes = ["_shared/patent_glossary", "_shared/output_standards"]
required_tools = ["PatentSearch", "GooglePatentsFetch", "IterativeSearch", "KnowledgeSearch", "SynonymSearch"]
instructions = """
## 专利检索能力

### 检索策略
1. 首先使用 SearchQueryBuilder 构建多层级检索式
2. 使用 SynonymSearch 扩展同义词
3. 通过 PatentSearch 执行统一检索
4. 对高相关结果使用 IterativeSearch 深度扩展

### 输出标准
参照 _shared/output_standards 的格式要求
"""
```

- [ ] 实现 → `cargo check -p codex-patent-skills` → 提交

---

### 阶段7: 管理与交付工具

#### Task 7.1: 管理与评估工具

**Files:**
- Create: `codex-rs/codex-patent-tools/src/management_tools.rs`
- Create: `codex-rs/codex-patent-tools/src/evaluation_tools.rs`
- Modify: `codex-rs/codex-patent-tools/src/lib.rs`

**管理工具**:
| 工具 | 核心功能 |
|------|---------|
| `PatentManager` | 专利生命周期状态机 (draft→filed→published→examined→granted→maintained→expired)，CRUD + 状态转换 |
| `TemplateLibrary` | 5 个内置文档模板(OA答复/专利申请/无效宣告/复审请求/审查意见) |
| `TrademarkAnalysis` | 商标可注册性规则评分(显著性检查+禁注条款检查) |
| `ProcessChart` | 专利处理流程图生成(基本 Mermaid 格式) |

**评估工具**:
| 工具 | 核心功能 |
|------|---------|
| `ActionReview` | 行动回顾 — 检查结果与预期的一致性 |
| `LLMReflection` | LLM 自我反思评估 |
| `FaithfulnessEval` | 忠实度评估 — 检查输出与输入的一致性 |
| `SelfConsistencyEval` | 自一致性评估 — 多次执行结果一致性 |
| `GEval` | G-Eval 评估框架 |

在 `lib.rs` 中更新 `register_all_tools()` 添加所有工具。

- [ ] 实现 → `cargo check -p codex-patent-tools` → 提交

---

### 阶段8: 集成测试与验证

#### Task 8.1: 端到端专利工作流测试

**Files:** Create: `codex-rs/codex-patent-tools/tests/e2e_tests.rs`

- [ ] 编写以下关键路径测试:

1. **检索工作流**: SearchQueryBuilder → PatentSearch → GooglePatentsFetch → IterativeSearch
2. **分析工作流**: ClaimParse → NoveltyAnalysis → ClaimCompare
3. **审查工作流**: FormalCheck → QualityAssess → SubjectMatterCheck → UnityCheck
4. **撰写工作流**: ClaimGenerator → SpecificationDrafter → AbstractDrafter → InnovationEvaluator
5. **OA 工作流**: 驳回类型检测 → OaStrategy → ResponseTemplate → SuccessPredictor
6. **知识搜索工作流**: KnowledgeSearch → KnowledgeGraphQuery → LawDatabaseQuery
7. **侵权分析工作流**: ClaimParse → FeatureMatcher → InfringementAnalysis
8. **Agent 加载**: 运行时加载 9 个 TOML 角色定义
9. **Skill 加载**: 运行时加载所有 skill 并验证 include 展开

- [ ] 运行: `cargo test -p codex-patent-tools`
- [ ] 提交

---

#### Task 8.2: 知识库引擎回归测试

**Files:** Create: `codex-rs/codex-patent-knowledge/tests/regression_tests.rs`

- [ ] 对知识图谱、法规库、卡片索引的核心接口做回归测试
- [ ] 运行: `cargo test -p codex-patent-knowledge`
- [ ] 提交

---

#### Task 8.3: 运行完整编译和 lint

- [ ] 运行 `cargo check --workspace` 确保所有 crate 编译通过
- [ ] 运行 `just fmt` 进行代码格式化
- [ ] 每个 crate 运行 `just test -p <crate>` 确保测试通过

---

### 附录: 文件清单总览

| Crate | 文件数 | 工具数 |
|-------|--------|--------|
| codex-patent-core | 3 (lib, types, error) + Cargo.toml | 0 |
| codex-patent-knowledge | 6 (lib, graph, law_db, cards, synonym, search) + Cargo.toml | 0 |
| codex-patent-domain | 14 (lib + 10 modules + 3 rules) + Cargo.toml | 0 |
| codex-patent-tools | 8 (lib, search_tools, patent_search, google_patents, analysis_tools, review_tools, drafting_tools, management_tools, evaluation_tools) + Cargo.toml | ~45 |
| codex-patent-agents | 12 (lib, roles + 9 TOML) + Cargo.toml | 0 |
| codex-patent-skills | 18 (lib + 5 shared + 10 cap + 2 foundation + stop-slop TOML) + Cargo.toml | 0 |
| codex-patent-assets | ~170 (SQLite ×3 + JSON + 150 md + 10+ books) | 0 |
| **合计** | **~240 文件** | **~45 工具** |

# BCIP 专利能力全面提升计划

> 版本: 1.0 | 日期: 2026-06-09
> 基于 CodeGraph 全仓库深度分析（95,716 符号 / 3,831 文件）
> 目标: 将专利分析 + 撰写 + 知识库 + 工作流 + 合规全栈提升到生产级

---

## 诊断摘要

| 模块 | 当前评分 | 关键瓶颈 |
|------|---------|---------|
| 专利域模型 (domain) | 8/10 | 规则引擎覆盖面有限，interaction.rs 性能 bug |
| 专利工具链 (tools) | 7.5/10 | 撰写工具模板化，13 域工具深度不均 |
| 专利工作流 (workflow) | 8/10 | NoopPlanGenerator 过于简单，resume 假恢复 |
| 专利知识库 (knowledge) | 8/10 | 五引擎融合好，但数据资产管理缺自动化 |
| 专利文本 (text) | 5/10 | 仅 3 模块，CJK 单字分词，缺少术语标准化 |
| 宪法约束 (constitutional) | 7.5/10 | 73% 规则变体走 fallback，缺 LLM 语义合规 |
| 智能体 (agents) | 7/10 | 9 角色定义好，但 runtime 缺真实 LLM 集成 |

**核心矛盾**: 架构设计优秀（9/10），但实现层深度不足（5-7/10）。

---

## 计划总览

```
Phase 1: 工程健康（修复 + 清理）          ← 消除技术债务
Phase 2: 撰写能力跃升                     ← 补齐最短板
Phase 3: 分析能力深化                     ← 建立差异化竞争力
Phase 4: 知识库增强                       ← 数据驱动升级
Phase 5: 工作流智能化                     ← 从编排到自主规划
Phase 6: 合规引擎升级                     ← 从关键词到语义合规
Phase 7: Agent Runtime 完善               ← 真实 LLM 集成
Phase 8: 端到端集成测试                   ← 验证闭环
```

---

## Phase 1: 工程健康（修复 + 清理）

> **目标**: 消除现有 PLAN.md 中已识别的所有技术债务
> **预估**: 2-3 天 | **风险**: 低

### 1.1 修复 resume_from_checkpoint 假恢复

**文件**: `codex-rs/codex-patent-workflow/src/graph_executor/checkpoint.rs:52-70`

**当前问题**: 加载 checkpoint 后直接调用 `self.execute(graph)` 从头执行，忽略已完成的步骤。

**修复方案**: 基于 `checkpoint.step_index` 跳过已完成层级，从断点恢复。

```rust
// checkpoint.rs — resume_from_checkpoint
pub fn resume_from_checkpoint(
    &self,
    run_id: &str,
    graph: &FlowGraph,
) -> Result<GraphExecution, String> {
    let checkpoint = self.checkpoint_store
        .load_checkpoint(run_id)?
        .ok_or_else(|| format!("no checkpoint found for run {run_id}"))?;

    graph.validate().map_err(|errs| errs.join("; "))?;
    let entry = graph.resolve_entry_node()
        .ok_or_else(|| "无法确定入口节点".to_string())?;
    let run_id = generate_run_id();
    let levels = graph.topological_levels()?;

    // 从 checkpoint.step_index 对应的层级开始
    let start_level = checkpoint.step_index.min(levels.len());
    let mut state = scheduler::ExecutionState::new(entry);
    let mut node_results: Vec<GraphNodeResult> = Vec::new();

    // 恢复已完成步骤的结果
    for (step_id, result) in &checkpoint.state.step_results {
        node_results.push(GraphNodeResult {
            node_id: step_id.clone(),
            step_result: result.clone(),
        });
        state.mark_completed(step_id);
    }

    // 从断点层级继续执行
    for level in levels.iter().skip(start_level) {
        if !state.should_continue() { break; }
        scheduler::execute_level(
            self, graph, &run_id, level,
            &mut state, &mut node_results,
            graph.retry_on_failure.unwrap_or(self.max_retries),
        )?;
    }

    let final_status = status::determine_final_status(state.suspended, state.failed);
    Ok(status::build_execution_result(graph, run_id, final_status, node_results))
}
```

**验证**: 新增测试 `test_resume_skips_completed_levels`，构造 3 步图，在 step 1 后保存 checkpoint，resume 后验证 step 0/1 不重新执行。

### 1.2 修复 interaction.rs Regex 重复编译

**文件**: `codex-rs/codex-patent-domain/src/interaction.rs`

**修复**: 使用 `std::sync::LazyLock` 预编译所有正则。

```rust
use std::sync::LazyLock;
use regex::Regex;

static NEGATIVE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    ["不行", "不对", "错了", "糟糕", "失败", "不满意", "不满意",
     "没意义", "没帮助", "毫无", "浪费时间"]
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
});

static CONTINUE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    ["继续", "请继续", "还有", "接下来", "然后呢", "深入分析", "详细说明"]
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
});

// is_frustrated 改为：
fn is_frustrated(text: &str) -> bool {
    NEGATIVE_PATTERNS.iter().any(|re| re.is_match(text))
}
```

### 1.3 修复 quality_rules.rs YAML 重复解析

**文件**: `codex-rs/codex-patent-domain/src/quality_rules.rs`

```rust
use std::sync::OnceLock;

static CONFIG_CACHE: OnceLock<SpecQualityConfig> = OnceLock::new();

fn load_config() -> &'static SpecQualityConfig {
    CONFIG_CACHE.get_or_init(|| {
        serde_yaml::from_str(RULES_YAML).unwrap_or_default()
    })
}
```

### 1.4 清理未使用依赖和 feature flags

已完成项（确认状态）:
- ✅ `codex-patent-skills` crate 已迁移合并
- ✅ `search-tools` 等 6 个未使用 feature flags 已删除（当前只剩 `document-pdf`）

待执行:

| 文件 | 操作 |
|------|------|
| `codex-patent-constitutional/Cargo.toml` | 删除未使用的 `regex`, `tracing` |
| `codex-patent-knowledge/Cargo.toml` | 删除未使用的 `codex-patent-text`, `thiserror` |
| `codex-patent-workflow/Cargo.toml` | 删除未使用的 `thiserror` |

### 1.5 删除旧格式 TOML 文件

**文件**: `codex-rs/codex-patent-agents/assets/` 下 9 个根目录 TOML（已有 `bcip/` 替代）

```bash
# 删除列表
codex-patent-agents/assets/retriever.toml
codex-patent-agents/assets/analyzer.toml
codex-patent-agents/assets/writer.toml
codex-patent-agents/assets/novelty_checker.toml
codex-patent-agents/assets/creativity_checker.toml
codex-patent-agents/assets/infringement_checker.toml
codex-patent-agents/assets/invalidity_checker.toml
codex-patent-agents/assets/reviewer.toml
codex-patent-agents/assets/quality_checker.toml
```

### 1.6 零字段结构体 → 纯函数（13 处）

**文件**: 多个 domain 文件

**示例转换**:
```rust
// Before
pub struct ClaimParser;
impl ClaimParser {
    pub fn new() -> Self { Self }
    pub fn parse(&self, claim_number: u32, text: &str) -> ParsedClaim { ... }
}

// After
pub fn parse_claim(claim_number: u32, text: &str) -> ParsedClaim { ... }
```

**完整列表**: `ClaimParser`, `FeatureMatcher`, `ClaimGenerator`, `OaParser`, `OaResponder`, `InventionClassifier`, `ArgumentationLibrary`, `QualityAssessor`, `InvalidityPipeline`, `ComprehensiveAnalyzer`, `DisclosureParser`, `FeatureExtractor`, `InfringementPipeline`

### 1.7 Agent 死 API 收缩

**文件**: `codex-rs/codex-patent-agents/src/lib.rs`

```rust
// 改为 pub(crate) 或删除
pub(crate) use scenario::ScenarioRegistry;
pub(crate) use reflection::ReflectionEngine;
pub(crate) use learning::LearningStore;
```

---

## Phase 2: 撰写能力跃升

> **目标**: 从模板化撰写 → 结构化智能撰写
> **预估**: 5-7 天 | **风险**: 中

### 2.1 说明书撰写引擎重写

**当前**: `DraftingTools::specification_draft` 纯字符串拼接。

**目标**: 结构化说明书生成器，支持 CNIPA 五段式 + 实施例展开。

**新建文件**: `codex-rs/codex-patent-domain/src/spec_writer.rs`

```rust
/// 说明书结构化生成器
pub struct SpecOutline {
    pub title: String,
    pub tech_field: Section,
    pub background: Section,
    pub summary: Section,         // 发明内容（含技术问题/方案/效果）
    pub brief_description: Section, // 附图说明（可选）
    pub embodiments: Vec<Embodiment>,
    pub abstract_text: String,
    pub sequence_listing: Option<String>, // 序列表（生物类）
}

pub struct Section {
    pub heading: String,
    pub content: String,
    pub subsections: Vec<Subsection>,
}

pub struct Embodiment {
    pub number: u32,
    pub title: String,
    pub description: String,
    pub examples: Vec<TechExample>, // 实施例
    pub reference_signs: Vec<(String, String)>, // 附图标记
}

pub struct TechExample {
    pub title: String,
    pub conditions: Vec<String>,  // 实验条件
    pub results: Vec<String>,     // 实验结果
    pub comparative: Option<String>, // 对比例
}

/// 从技术交底书生成说明书大纲
pub fn generate_spec_outline(disclosure: &DisclosureDoc) -> Result<SpecOutline, PatentError> {
    // 1. 解析交底书各节
    // 2. 提取技术问题/方案/效果三元组
    // 3. 识别关键实施方式
    // 4. 生成结构化大纲
    // 5. 自动分配附图标记
    todo!("实现结构化大纲生成")
}

/// 从大纲展开为完整说明书文本
pub fn expand_outline_to_text(outline: &SpecOutline) -> String {
    // 按模板展开各节，保持 CNIPA 格式
    todo!("实现说明书展开")
}
```

**修改文件**: `codex-rs/codex-patent-tools/src/drafting_tools.rs`

```rust
// 重写 specification_draft，调用新的结构化引擎
pub fn specification_draft(input: SpecificationInput) -> Result<serde_json::Value, String> {
    let disclosure = DisclosureDoc {
        raw_text: format!(
            "技术领域：{}\n背景技术：{}\n发明内容：{}\n实施方式：{}",
            input.technical_field, input.background,
            input.invention_content, input.embodiments
        ),
        sections: HashMap::from([
            ("technical_field".into(), input.technical_field.clone()),
            ("background".into(), input.background.clone()),
            ("invention_content".into(), input.invention_content.clone()),
            ("embodiments".into(), input.embodiments.clone()),
        ]),
        confidence: 1.0,
    };

    let outline = codex_patent_domain::spec_writer::generate_spec_outline(&disclosure)
        .map_err(|e| format!("{e}"))?;
    let text = codex_patent_domain::spec_writer::expand_outline_to_text(&outline);

    Ok(serde_json::json!({
        "title": input.title,
        "specification": text,
        "outline": outline,
        "word_count": text.len(),
        "section_count": outline.embodiments.len() + 4,
    }))
}
```

### 2.2 权利要求撰写引擎增强

**当前**: `claim_generator` 仅生成模板化独权 + 从权。

**目标**: 支持多种权利要求类型、多层从属、方法+产品双报。

**修改文件**: `codex-rs/codex-patent-domain/src/claim_generator.rs`

```rust
/// 权利要求布局策略
pub enum ClaimLayout {
    /// 标准布局：1独权 + N从权
    Standard { dependent_count: usize },
    /// 双报布局：方法独权 + 产品独权 + 各自从权
    DualClaim {
        method_dependents: usize,
        product_dependents: usize,
    },
    /// 多项从属：从权引用多项前权
    MultipleDependency {
        primary_deps: usize,
        secondary_deps: usize,
    },
}

/// 高级权利要求生成输入
pub struct AdvancedClaimInput {
    pub invention_name: String,
    pub essential_features: Vec<String>,
    pub optional_features: Vec<Vec<String>>,
    pub layout: ClaimLayout,
    pub claim_type: ClaimCategory,  // Method, Product, Use
    pub preamble_text: Option<String>, // 前序部分自定义
}

/// 生成权利要求书
pub fn generate_claims(input: &AdvancedClaimInput) -> Vec<String> {
    match &input.layout {
        ClaimLayout::Standard { dependent_count } => {
            generate_standard_layout(input, *dependent_count)
        }
        ClaimLayout::DualClaim { method_dependents, product_dependents } => {
            generate_dual_layout(input, *method_dependents, *product_dependents)
        }
        ClaimLayout::MultipleDependency { primary_deps, secondary_deps } => {
            generate_multi_dep_layout(input, *primary_deps, *secondary_deps)
        }
    }
}
```

### 2.3 审查意见答复模板库

**新建文件**: `codex-rs/codex-patent-domain/src/oa_templates.rs`

```rust
/// OA 答复模板（按驳回类型分类）
pub enum OaTemplate {
    /// 新颖性驳回 — 区别特征论证
    NoveltyRejection {
        distinguishing_features: Vec<String>,
        technical_effects: Vec<String>,
    },
    /// 创造性驳回 — 三步法答复
    InventivenessRejection {
        closest_prior_art: String,
        distinguishing_features: Vec<String>,
        actual_problem: String,
        non_obviousness_args: Vec<String>,
    },
    /// 不清楚驳回 — 修改 + 说明
    ClarityRejection {
        unclear_terms: Vec<String>,
        clarifications: Vec<String>,
    },
    /// 不支持驳回 — 补充说明
    SupportRejection {
        unsupported_claims: Vec<u32>,
        support_evidence: Vec<String>,
    },
}

/// 生成答复模板文本
pub fn render_oa_template(template: &OaTemplate, case: &CaseContext) -> String {
    match template {
        OaTemplate::NoveltyRejection { distinguishing_features, technical_effects } => {
            format!(
                "一、关于新颖性\n\n\
                 申请人认为，权利要求{}与对比文件{}相比存在以下区别技术特征：\n\n\
                 {}\n\n\
                 上述区别技术特征使得本发明具备以下技术效果：\n\n\
                 {}\n\n\
                 因此，权利要求{}相对于对比文件{}具备新颖性，符合专利法第22条第2款的规定。",
                /* claim_number */, /* prior_art */,
                distinguishing_features.iter()
                    .enumerate()
                    .map(|(i, f)| format!("({}) {}", i + 1, f))
                    .collect::<Vec<_>>()
                    .join("\n"),
                technical_effects.iter()
                    .enumerate()
                    .map(|(i, e)| format!("({}) {}", i + 1, e))
                    .collect::<Vec<_>>()
                    .join("\n"),
                /* claim_number */, /* prior_art */
            )
        }
        // ... 其他模板
        _ => String::new(),
    }
}
```

### 2.4 撰写质量自动评审

**扩展文件**: `codex-rs/codex-patent-domain/src/drafting.rs`

新增审查维度：

```rust
/// 撰写质量多维度审查报告
pub struct DraftReviewReport {
    pub overall_score: f64,
    // 基础合规
    pub sufficiency_of_disclosure: DimensionScore,  // 充分公开（A26.3）
    pub clarity: DimensionScore,                     // 清楚完整（A26.4）
    pub claim_support: DimensionScore,               // 以说明书为依据（A26.4）
    // 结构质量
    pub logical_flow: DimensionScore,                // 逻辑连贯性
    pub terminology_consistency: DimensionScore,     // 术语一致性
    pub reference_sign_consistency: DimensionScore,  // 附图标记一致性
    // 深度质量
    pub embodiment_depth: DimensionScore,            // 实施例深度
    pub experimental_data: DimensionScore,           // 实验数据充分性
    pub scope_reasonableness: DimensionScore,        // 保护范围合理性
    // 风险
    pub risks: Vec<DraftRisk>,
}

pub struct DimensionScore {
    pub score: f64,        // 0-100
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

pub struct DraftRisk {
    pub severity: RiskSeverity,  // High, Medium, Low
    pub category: String,
    pub description: String,
    pub suggestion: String,
}

/// 审查说明书草稿
pub fn review_draft(spec: &str, claims: &[String]) -> DraftReviewReport {
    let mut report = DraftReviewReport::default();

    // 1. 充分公开检查：是否有"能够实现"的完整描述
    report.sufficiency_of_disclosure = check_sufficiency(spec);

    // 2. 清楚性检查：模糊用语检测
    report.clarity = check_clarity(claims);

    // 3. 支持性检查：权利要求特征是否在说明书中有依据
    report.claim_support = check_claim_support(spec, claims);

    // 4. 术语一致性：同一概念是否使用了不同术语
    report.terminology_consistency = check_terminology(spec);

    // 5. 实施例深度：是否有具体参数/条件
    report.embodiment_depth = check_embodiment_depth(spec);

    // 6. 计算总分
    report.calculate_overall();

    report
}
```

---

## Phase 3: 分析能力深化

> **目标**: 从启发式规则 → 多层分析（规则 + 统计 + LLM 语义）
> **预估**: 5-7 天 | **风险**: 中

### 3.1 规则引擎扩展 — 领域差异化

**修改文件**: `codex-rs/codex-patent-domain/src/rule_engine/`

```rust
/// 技术领域分类（影响分析策略）
pub enum TechDomain {
    Mechanical,    // 机械 — 侧重结构+功能
    Electrical,    // 电子 — 侧重电路+信号
    Chemical,      // 化学 — 侧重参数+效果+实施例
    Biotechnology, // 生物 — 侧重序列+实验数据
    Software,      // 软件 — 侧重算法+技术效果
    Communication, // 通信 — 侧重协议+性能指标
}

/// 领域感知的分析上下文
pub struct DomainAwareContext {
    pub base: CaseContext,
    pub tech_domain: TechDomain,
    /// 化学领域：参数范围、实施例数量
    pub chemical_params: Option<ChemicalAnalysisParams>,
    /// 软件领域：算法步骤数、技术效果类型
    pub software_params: Option<SoftwareAnalysisParams>,
}

/// 化学领域专有分析参数
pub struct ChemicalAnalysisParams {
    pub parameter_ranges: Vec<(String, String, String)>, // (参数名, 下限, 上限)
    pub embodiment_count: usize,
    pub has_comparative_data: bool,
    pub has_unexpected_effect: bool,
}

/// 软件领域专有分析参数
pub struct SoftwareAnalysisParams {
    pub algorithm_steps: usize,
    pub has_technical_effect: bool,    // 是否有"技术"效果（非商业效果）
    pub solves_technical_problem: bool, // 是否解决技术问题
    pub hardware_involved: bool,       // 是否涉及硬件
}

impl QualitativeRuleEngine {
    /// 领域感知的创造性分析
    pub fn analyze_inventiveness_domain_aware(
        &mut self,
        ctx: &DomainAwareContext,
    ) -> Result<AnalysisResult, PatentError> {
        match ctx.tech_domain {
            TechDomain::Chemical => self.analyze_chemical_inventiveness(ctx),
            TechDomain::Software => self.analyze_software_inventiveness(ctx),
            _ => self.analyze_inventiveness(&ctx.base),
        }
    }

    fn analyze_chemical_inventiveness(&mut self, ctx: &DomainAwareContext) -> Result<AnalysisResult, PatentError> {
        // 化学领域创造性规则（更侧重：
        // 1. 参数范围的非常规性
        // 2. 预料不到的技术效果
        // 3. 协同效果
        // 4. 实施例充分性
        let mut applied = Vec::new();
        let mut total_score = 0.0;
        let mut count = 0;

        if let Some(ref params) = ctx.chemical_params {
            // 检查实施例充分性（化学领域至少 3 个实施例）
            if params.embodiment_count >= 3 {
                applied.push(AppliedRule {
                    rule_name: "chemical_embodiment_sufficiency".into(),
                    conclusion: "化学领域实施例充分".into(),
                    applies: true,
                    score: 0.8,
                });
                total_score += 0.8;
                count += 1;
            }

            // 检查预料不到效果
            if params.has_unexpected_effect {
                applied.push(AppliedRule {
                    rule_name: "unexpected_effect".into(),
                    conclusion: "存在预料不到的技术效果，强有力支持创造性".into(),
                    applies: true,
                    score: 0.95,
                });
                total_score += 0.95;
                count += 1;
            }
        }

        // 继续通用规则...
        let base_result = self.analyze_inventiveness(&ctx.base)?;
        applied.extend(base_result.applied_rules);

        let all_scores: Vec<f64> = applied.iter().map(|r| r.score).collect();
        let avg = if count > 0 {
            (total_score + base_result.net_score * base_result.applied_rules.len() as f64)
                / (count + base_result.applied_rules.len()) as f64
        } else {
            base_result.net_score
        };

        Ok(AnalysisResult {
            conclusion: if avg > 0.5 { "具备创造性".into() } else { "可能缺乏创造性".into() },
            net_score: avg,
            confidence: base_result.confidence,
            applied_rules: applied,
        })
    }
}
```

### 3.2 权利要求对比引擎 — 等同侵权分析

**修改文件**: `codex-rs/codex-patent-domain/src/infringement.rs`

```rust
/// 等同侵权分析（全面覆盖原则 + 等同替换）
pub struct EquivalenceAnalysis {
    pub literal_infringement: LiteralResult,
    pub doctrine_of_equivalents: EquivalenceResult,
    pub prosecution_history_estoppel: Option<EstoppelResult>,
    pub overall: InfringementConclusion,
}

pub struct LiteralResult {
    pub all_features_covered: bool,
    pub missing_features: Vec<String>,
    pub matching_features: Vec<FeatureMatch>,
}

pub struct EquivalenceResult {
    pub equivalent_features: Vec<EquivalentFeature>,
    pub non_equivalent_features: Vec<String>,
}

pub struct EquivalentFeature {
    pub claim_feature: String,
    pub accused_feature: String,
    pub equivalence_type: EquivalenceType,  // KnownSubstitution, SameFunction, etc.
    pub reasoning: String,
    pub confidence: f64,
}

pub enum EquivalenceType {
    KnownSubstitution,   // 已知替换手段
    SameFunctionWayResult, // 相同功能/方式/结果
    InsubstantialDifference, // 非实质性差异
}

/// 执行全面侵权分析
pub fn analyze_infringement_comprehensive(
    claim_text: &str,
    accused_description: &str,
    prosecution_history: Option<&str>,
) -> EquivalenceAnalysis {
    let parser = crate::claim_parser::parse_claim;
    let claim = parser(1, claim_text);
    let accused = parser(2, accused_description);

    // 第一层：字面侵权（全面覆盖）
    let literal = analyze_literal_infringement(&claim, &accused);

    // 第二层：等同侵权
    let equivalence = analyze_equivalence(&claim, &accused);

    // 第三层：禁止反悔（如果有审查历史）
    let estoppel = prosecution_history.map(|h| analyze_estoppel(h, &equivalence));

    // 综合判断
    let overall = determine_infringement_conclusion(&literal, &equivalence, &estoppel);

    EquivalenceAnalysis {
        literal_infringement: literal,
        doctrine_of_equivalents: equivalence,
        prosecution_history_estoppel: estoppel,
        overall,
    }
}
```

### 3.3 无效宣告分析管线增强

**修改文件**: `codex-rs/codex-patent-domain/src/invalidity.rs`

```rust
/// 无效宣告理由（A45 全覆盖）
pub enum InvalidityGround {
    /// 不符合授予条件（三性）
    LackOfNovelty { prior_art: String, claim_comparison: String },
    LackOfInventiveness { three_step_analysis: String },
    LackOfUtility { reason: String },
    /// 不符合形式要求
    InsufficientDisclosure { missing_elements: Vec<String> },
    ClaimsNotSupported { unsupported_claims: Vec<u32> },
    ClaimsUnclear { unclear_claims: Vec<u32> },
    /// 程序问题
    BeyondOriginalScope { additions: Vec<String> },
    DoublePatenting { related_patent: String },
    /// 主题不合格
    NonPatentableSubjectMatter { subject_matter: String },
}

/// 无效分析结果
pub struct InvalidityAnalysis {
    pub grounds: Vec<InvalidityGround>,
    pub strongest_ground: Option<InvalidityGround>,
    pub evidence_requirements: Vec<EvidenceRequirement>,
    pub success_probability: f64,
    pub recommended_strategy: String,
}

pub struct EvidenceRequirement {
    pub evidence_type: String,  // "对比文件", "公知常识证据", "实验数据"
    pub description: String,
    pub importance: String,     // "必需", "重要", "辅助"
}

/// 全面无效分析
pub fn analyze_invalidity(
    patent_claims: &[String],
    patent_spec: &str,
    prior_art: &[String],
) -> InvalidityAnalysis {
    let mut grounds = Vec::new();

    // 1. 新颖性无效
    for pa in prior_art {
        if let Some(novelty_ground) = check_novelty_ground(patent_claims, pa) {
            grounds.push(novelty_ground);
        }
    }

    // 2. 创造性无效
    if let Some(inv_ground) = check_inventiveness_ground(patent_claims, prior_art) {
        grounds.push(inv_ground);
    }

    // 3. 充分公开无效
    if let Some(disc_ground) = check_disclosure_ground(patent_claims, patent_spec) {
        grounds.push(disc_ground);
    }

    // 4. 清楚性/支持性无效
    if let Some(clarity_ground) = check_clarity_ground(patent_claims) {
        grounds.push(clarity_ground);
    }

    // 排序：选择最强理由
    let strongest = grounds.iter().max_by(|a, b| {
        ground_strength(a).partial_cmp(&ground_strength(b)).unwrap_or(std::cmp::Ordering::Equal)
    }).cloned();

    InvalidityAnalysis {
        strongest_ground: strongest,
        evidence_requirements: collect_evidence_requirements(&grounds),
        success_probability: estimate_success_probability(&grounds),
        recommended_strategy: recommend_strategy(&grounds),
        grounds,
    }
}
```

### 3.4 审查员模拟器增强 — 多轮模拟

**修改文件**: `codex-rs/codex-patent-domain/src/examiner_simulator/`

```rust
/// 多轮审查模拟
pub struct MultiRoundSimulation {
    pub rounds: Vec<SimulatedRound>,
    pub final_prediction: GrantPrediction,
}

pub struct SimulatedRound {
    pub round_number: u32,
    pub examiner_action: ExaminerAction,
    pub suggested_response: String,
    pub quality_score: f64,
    pub remaining_issues: Vec<String>,
}

pub enum ExaminerAction {
    FirstOfficeAction { rejections: Vec<SimulatedRejection> },
    SubsequentAction {
        rejections: Vec<SimulatedRejection>,
        allowances: Vec<u32>,
    },
    NoticeOfAllowance,
    FinalRejection { grounds: Vec<String> },
}

pub struct SimulatedRejection {
    pub claim_numbers: Vec<u32>,
    pub rejection_type: RejectionType,
    pub cited_art: Vec<String>,
    pub reasoning: String,
    pub difficulty: RejectionDifficulty, // Easy, Moderate, Hard
}

/// 执行多轮模拟
pub fn simulate_multi_round(
    claims: &[String],
    spec: &str,
    prior_art: &[String],
    max_rounds: u32,
) -> MultiRoundSimulation {
    let mut rounds = Vec::new();
    let mut current_claims = claims.to_vec();

    for round in 1..=max_rounds {
        // 模拟审查员行为
        let examiner = simulate_examiner_round(&current_claims, spec, prior_art, round);

        // 生成建议答复
        let response = suggest_response(&examiner, &current_claims, spec);

        // 评估答复质量
        let quality = evaluate_response_quality(&response);

        // 预测下一轮
        let remaining = identify_remaining_issues(&examiner, &response);

        rounds.push(SimulatedRound {
            round_number: round,
            examiner_action: examiner,
            suggested_response: response,
            quality_score: quality,
            remaining_issues: remaining,
        });

        // 如果审查员允许或最终驳回，停止
        if matches!(rounds.last().unwrap().examiner_action,
            ExaminerAction::NoticeOfAllowance | ExaminerAction::FinalRejection { .. })
        {
            break;
        }
    }

    let prediction = predict_grant_outcome(&rounds);
    MultiRoundSimulation { rounds, final_prediction: prediction }
}
```

---

## Phase 4: 知识库增强

> **目标**: 数据资产管理自动化 + 搜索质量可观测
> **预估**: 4-5 天 | **风险**: 中

### 4.1 知识库数据管线自动化

**新建文件**: `codex-rs/codex-patent-knowledge/src/pipeline/`

```rust
/// 知识库数据刷新管线
pub struct KnowledgePipeline {
    kg_path: PathBuf,
    law_db_path: PathBuf,
    card_index_path: PathBuf,
    vector_index_path: PathBuf,
}

impl KnowledgePipeline {
    /// 完整刷新：从 CNIPA + Google Patents 拉取 → 解析 → 索引 → 向量化
    pub async fn full_refresh(&self) -> Result<PipelineReport, String> {
        let mut report = PipelineReport::default();

        // 1. 拉取最新专利数据
        let patents = self.fetch_recent_patents().await?;
        report.patents_fetched = patents.len();

        // 2. 更新知识图谱
        let kg_updated = self.update_knowledge_graph(&patents)?;
        report.kg_nodes_added = kg_updated;

        // 3. 更新法规库（检查新法规）
        let laws_updated = self.update_law_database()?;
        report.laws_updated = laws_updated;

        // 4. 更新知识卡片
        let cards_updated = self.update_card_index(&patents)?;
        report.cards_updated = cards_updated;

        // 5. 向量化新文档
        let vectors_added = self.update_vector_index(&patents).await?;
        report.vectors_added = vectors_added;

        // 6. 验证搜索质量
        report.search_quality = self.run_search_eval()?;

        Ok(report)
    }

    /// 增量更新：仅处理新增/变更数据
    pub async fn incremental_update(&self, since: &chrono::DateTime<Utc>) -> Result<PipelineReport, String> {
        // 基于时间戳的增量拉取
        todo!()
    }
}

#[derive(Default)]
pub struct PipelineReport {
    pub patents_fetched: usize,
    pub kg_nodes_added: usize,
    pub laws_updated: usize,
    pub cards_updated: usize,
    pub vectors_added: usize,
    pub search_quality: SearchQualityMetrics,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

pub struct SearchQualityMetrics {
    pub precision_at_5: f64,
    pub precision_at_10: f64,
    pub recall: f64,
    pub mrr: f64,
    pub regression_detected: bool,
}
```

### 4.2 同义词库扩展 — 专利术语标准化

**修改文件**: `codex-rs/codex-patent-knowledge/src/synonym.rs`

```rust
/// 专利术语标准化器
pub struct PatentTerminologyNormalizer {
    /// 技术领域 → 术语映射
    domain_terms: HashMap<String, Vec<TermEntry>>,
    /// 通用缩写扩展
    abbreviations: HashMap<String, Vec<String>>,
    /// 中英对照
    cn_en_map: HashMap<String, String>,
}

pub struct TermEntry {
    pub canonical: String,           // 标准术语
    pub variants: Vec<String>,       // 同义变体
    pub domain: String,              // 所属技术领域
    pub ipc_codes: Vec<String>,      // 相关 IPC 分类
    pub definition: Option<String>,  // 定义
}

impl PatentTerminologyNormalizer {
    /// 标准化一个术语到规范形式
    pub fn normalize(&self, term: &str) -> NormalizedTerm {
        // 1. 查找精确匹配
        // 2. 查找变体匹配
        // 3. 查找缩写扩展
        // 4. 查找中英对照
        // 5. 返回标准术语 + 置信度
        todo!()
    }

    /// 对一段专利文本进行术语标准化
    pub fn normalize_text(&self, text: &str) -> Vec<NormalizedTermOccurrence> {
        todo!()
    }
}

pub struct NormalizedTerm {
    pub original: String,
    pub canonical: String,
    pub confidence: f64,
    pub domain: Option<String>,
}
```

### 4.3 向量搜索优化 — 分领域索引

**修改文件**: `codex-rs/codex-patent-knowledge/src/vector_index.rs`

```rust
impl VectorIndex {
    /// 分领域搜索（优先返回同领域结果）
    pub fn search_by_domain(
        &self,
        query_embedding: &[f32],
        domain: &str,
        top_k: usize,
    ) -> Vec<ScoredChunk> {
        let all = self.search(query_embedding, top_k * 3);
        // 对同领域结果加权
        let mut scored: Vec<_> = all.into_iter().map(|chunk| {
            let domain_boost = if chunk.metadata.get("domain").map_or(false, |d| d == domain) {
                1.3 // 同领域加权
            } else {
                1.0
            };
            (chunk, domain_boost)
        }).collect();
        scored.sort_by(|a, b| {
            (b.0.score * b.1).partial_cmp(&(a.0.score * a.1)).unwrap_or(Ordering::Equal)
        });
        scored.into_iter().take(top_k).map(|(c, _)| c).collect()
    }
}
```

---

## Phase 5: 工作流智能化

> **目标**: 从 NoopPlanGenerator → LLM 驱动的智能规划
> **预估**: 5-7 天 | **风险**: 高（依赖 LLM API）

### 5.1 LLM 驱动的计划生成器

**新建文件**: `codex-rs/codex-patent-workflow/src/llm_plan_generator.rs`

```rust
use super::plan::{ExecutionPlan, PlanGenerator, PlanStep, PlanStepStatus, RoutingHint};
use super::flow::FlowStep;
use crate::agent_bridge::AgentExecutor;

/// LLM 驱动的计划生成器
///
/// 将用户目标 + 路由提示发送给 LLM，生成结构化的执行计划。
pub struct LlmPlanGenerator {
    agent_executor: Box<dyn AgentExecutor>,
    /// 预定义的工作流模板（常见任务可直接匹配）
    templates: Vec<WorkflowTemplate>,
}

/// 预定义工作流模板
pub struct WorkflowTemplate {
    pub name: String,
    pub trigger_keywords: Vec<String>,
    pub domains: Vec<String>,
    pub steps: Vec<TemplateStep>,
}

pub struct TemplateStep {
    pub description: String,
    pub step: FlowStep,
    pub depends_on: Vec<String>,
}

impl LlmPlanGenerator {
    pub fn new(agent_executor: Box<dyn AgentExecutor>) -> Self {
        Self {
            agent_executor,
            templates: Self::builtin_templates(),
        }
    }

    /// 预定义的专利工作流模板
    fn builtin_templates() -> Vec<WorkflowTemplate> {
        vec![
            // 模板 1: 新颖性检索 + 分析
            WorkflowTemplate {
                name: "novelty_search_analysis".into(),
                trigger_keywords: vec!["新颖性".into(), "查新".into(), "prior art search".into()],
                domains: vec!["patent".into()],
                steps: vec![
                    TemplateStep {
                        description: "检索现有技术".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "PatentSearch".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec![],
                    },
                    TemplateStep {
                        description: "分析新颖性".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "NoveltyAnalysis".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec!["step_0".into()],
                    },
                    TemplateStep {
                        description: "生成新颖性报告".into(),
                        step: FlowStep::AgentCall {
                            agent_name: "analyst".into(),
                            prompt: "基于检索和分析结果，生成新颖性分析报告".into(),
                        },
                        depends_on: vec!["step_1".into()],
                    },
                ],
            },
            // 模板 2: OA 答复
            WorkflowTemplate {
                name: "oa_response".into(),
                trigger_keywords: vec!["审查意见".into(), "OA答复".into(), "office action".into()],
                domains: vec!["patent".into()],
                steps: vec![
                    TemplateStep {
                        description: "解析审查意见".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "OaParser".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec![],
                    },
                    TemplateStep {
                        description: "制定答复策略".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "OaStrategist".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec!["step_0".into()],
                    },
                    TemplateStep {
                        description: "审查员模拟评估".into(),
                        step: FlowStep::QualityCheck {
                            criteria: vec!["persuasiveness".into(), "technical_depth".into()],
                        },
                        depends_on: vec!["step_1".into()],
                    },
                    TemplateStep {
                        description: "人工审核".into(),
                        step: FlowStep::HumanApproval {
                            title: "审查意见答复".into(),
                            description: "请审核生成的答复".into(),
                            timeout_secs: Some(3600),
                            timeout_action: Default::default(),
                        },
                        depends_on: vec!["step_2".into()],
                    },
                ],
            },
        ]
    }
}

impl PlanGenerator for LlmPlanGenerator {
    fn generate(&self, goal: &str) -> Result<ExecutionPlan, String> {
        // 1. 先匹配预定义模板
        if let Some(template) = self.match_template(goal) {
            return self.instantiate_template(goal, &template);
        }

        // 2. 无匹配模板，委托 LLM 生成计划
        // 将 goal 发送给 planner agent，解析返回的结构化步骤
        self.llm_generate_plan(goal)
    }

    fn name(&self) -> &str { "llm_plan_generator" }
}
```

### 5.2 智能路由增强

**修改文件**: `codex-rs/codex-patent-workflow/src/plan.rs`

```rust
/// 增强的路由提示
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingHint {
    pub domain: String,
    pub complexity: String,
    pub workflow: WorkflowType,
    pub suggested_tools: Vec<String>,
    pub suggested_agents: Vec<String>,
    pub reasoning: String,
    // --- 新增字段 ---
    /// 目标语言（影响检索和撰写策略）
    pub target_jurisdiction: Option<Jurisdiction>,
    /// 预估步骤数
    pub estimated_steps: Option<usize>,
    /// 是否需要人工审核
    pub requires_human_review: bool,
    /// 优先级（影响执行顺序和并行策略）
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Jurisdiction {
    CN,   // 中国
    US,   // 美国
    EP,   // 欧洲
    JP,   // 日本
    KR,   // 韩国
    PCT,  // PCT 国际申请
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Urgent,
}
```

### 5.3 条件路由增强

**修改文件**: `codex-rs/codex-patent-workflow/src/flow.rs`

```rust
/// 增强的流程步骤（支持条件分支）
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FlowStep {
    // ... 现有变体 ...
    AgentCall { agent_name: String, prompt: String },
    AgentTool { agent_name: String, input: serde_json::Value },
    QualityCheck { criteria: Vec<String> },
    HumanApproval { title: String, description: String, timeout_secs: Option<u64>, timeout_action: HumanApprovalTimeoutAction },
    ToolCall { tool_name: String, input: serde_json::Value },
    CodeBlock { language: String, code: String },
    // --- 新增变体 ---
    /// 条件分支：根据上游结果选择路径
    Conditional {
        condition: BranchCondition,
        then_steps: Vec<String>,
        else_steps: Vec<String>,
    },
    /// 并行扇出：同时执行多个步骤
    ParallelFanOut {
        branches: Vec<String>,
        merge_strategy: MergeStrategy,
    },
}

#[serde(tag = "type", rename_all = "snake_case")]
pub enum BranchCondition {
    ScoreAbove { step_id: String, threshold: f64 },
    ScoreBelow { step_id: String, threshold: f64 },
    ContainsText { step_id: String, pattern: String },
    ToolSuccess { step_id: String },
    ToolFailure { step_id: String },
}

#[serde(tag = "type", rename_all = "snake_case")]
pub enum MergeStrategy {
    WaitAll,        // 等待所有分支完成
    WaitAny,        // 任一分支完成即继续
    WaitN { n: usize }, // 等待 N 个分支完成
}
```

---

## Phase 6: 合规引擎升级

> **目标**: 从关键词匹配 → 多层合规（关键词 + 模式 + LLM 语义）
> **预估**: 4-5 天 | **风险**: 中

### 6.1 实现 fallback 规则变体

**修改文件**: `codex-rs/codex-patent-constitutional/src/checkers.rs`

当前 16/22 个 `RuleCheck` 变体走 fallback（返回 0.5 置信度）。需要逐一实现：

```rust
/// 规则检查器 — 逐一实现每种检查类型
pub fn evaluate_rule(
    rule: &ConstitutionalRule,
    tool_name: &str,
    input_text: &str,
    output_text: Option<&str>,
) -> RuleCheckResult {
    match &rule.check {
        RuleCheck::KeywordBlocklist { keywords, absolute_ban, .. } => {
            check_keyword_blocklist(keywords, absolute_ban, input_text, output_text)
        }
        RuleCheck::PatternMatch { patterns, .. } => {
            check_pattern_match(patterns, input_text, output_text)
        }
        RuleCheck::LengthConstraint { min, max, .. } => {
            check_length_constraint(*min, *max, output_text)
        }
        RuleCheck::RequiredSection { sections, .. } => {
            check_required_sections(sections, output_text)
        }
        RuleCheck::ForbiddenPhrase { phrases, .. } => {
            check_forbidden_phrases(phrases, input_text, output_text)
        }
        RuleCheck::ClaimStructure { max_independent, max_dependent_per_independent, .. } => {
            check_claim_structure(*max_independent, *max_dependent_per_independent, output_text)
        }
        RuleCheck::TerminologyConsistency { terms, .. } => {
            check_terminology(terms, input_text, output_text)
        }
        RuleCheck::ScopeLimitation { scope_keywords, .. } => {
            check_scope_limitation(scope_keywords, input_text, output_text)
        }
        // 未实现的变体：添加 warn 日志
        other => {
            tracing::warn!(rule_id = %rule.id, check_type = %other.type_name(), "规则检查未实现，使用 fallback");
            RuleCheckResult {
                rule_id: rule.id.clone(),
                passed: true,  // 不阻断
                confidence: 0.5,
                details: "规则检查未实现，自动放行".into(),
                action: crate::types::RuleAction::parse(&rule.action),
            }
        }
    }
}
```

### 6.2 LLM 语义合规层（可选）

**新建文件**: `codex-rs/codex-patent-constitutional/src/semantic_checker.rs`

```rust
/// LLM 语义合规检查器
///
/// 对于规则引擎无法处理的复杂合规场景，
/// 将文本和规则发送给 LLM 进行语义判断。
pub struct SemanticChecker {
    /// 需要 LLM 判断的规则
    semantic_rules: Vec<ConstitutionalRule>,
}

impl SemanticChecker {
    /// 对输出进行语义合规检查
    ///
    /// 仅对标记了 `requires_semantic: true` 的规则触发。
    pub async fn check_semantic(
        &self,
        tool_name: &str,
        input_text: &str,
        output_text: &str,
        phase: &str,
    ) -> Vec<RuleCheckResult> {
        let mut results = Vec::new();

        for rule in &self.semantic_rules {
            if !rule.phase.is_empty() && rule.phase != phase {
                continue;
            }

            // 构造 LLM prompt
            let prompt = format!(
                "你是一位专利合规审查专家。请检查以下输出是否符合规则。\n\n\
                 规则: {} ({})\n\
                 描述: {}\n\
                 法律依据: {}\n\n\
                 工具: {}\n\
                 输入: {}\n\
                 输出: {}\n\n\
                 请判断输出是否违反该规则。回答JSON格式：\n\
                 {{\"passed\": bool, \"confidence\": f64, \"reasoning\": string}}",
                rule.name, rule.id, rule.description, rule.legal_basis,
                tool_name, input_text, output_text,
            );

            // 发送给 LLM 并解析结果
            // （实际实现通过 AgentExecutor）
            let _ = prompt; // TODO: 接入 LLM
        }

        results
    }
}
```

---

## Phase 7: Agent Runtime 完善

> **目标**: 从 NoopAgentExecutor → 真实 LLM Agent 集成
> **预估**: 5-7 天 | **风险**: 高（核心架构变更）

### 7.1 真实 Agent 执行器实现

**修改文件**: `codex-rs/codex-patent-workflow/src/agent_bridge.rs`

```rust
/// 基于 Codex core 的真实 Agent 执行器
pub struct CodexAgentExecutor {
    /// agent 名称 → 角色 TOML 配置路径
    role_configs: HashMap<String, PathBuf>,
    /// LLM provider
    provider: String,
    /// model 路由（不同任务使用不同模型）
    model_router: ModelRouter,
}

/// 模型路由策略
pub struct ModelRouter {
    /// 简单任务（格式化、提取）→ 快速模型
    fast_model: String,
    /// 标准任务（分析、检索）→ 默认模型
    default_model: String,
    /// 复杂任务（撰写、答复）→ 强推理模型
    reasoning_model: String,
}

impl AgentExecutor for CodexAgentExecutor {
    fn delegate_to(
        &mut self,
        agent_name: &str,
        input: &str,
    ) -> Result<AgentExecutionResult, String> {
        // 1. 加载角色配置
        let config = self.load_role_config(agent_name)?;

        // 2. 构建 prompt（系统指令 + 角色 prompt + 用户输入）
        let system_prompt = self.build_system_prompt(&config)?;
        let model = self.model_router.select_model(agent_name, input);

        // 3. 调用 LLM
        // （实际实现通过 codex-core 的 session 机制）
        Ok(AgentExecutionResult {
            output: String::new(), // LLM 响应
            success: true,
            token_usage: TokenUsage::default(),
            model_used: model,
        })
    }

    fn agent_names(&self) -> &[String] {
        &self.agent_names
    }
}
```

### 7.2 角色配置增强 — 上下文注入

**修改文件**: `codex-rs/codex-patent-agents/src/roles.rs`

```rust
/// 增强的角色加载：支持动态上下文注入
pub fn load_role_with_context(
    role_name: &str,
    context: &RoleContext,
) -> Result<RoleConfig, String> {
    let mut config = load_base_role(role_name)?;

    // 注入宪法规则上下文
    if let Some(constitutional_ctx) = &context.constitutional_context {
        config.system_prompt = format!(
            "{}\n\n{}",
            config.system_prompt, constitutional_ctx
        );
    }

    // 注入知识库检索结果
    if let Some(knowledge) = &context.knowledge_context {
        config.system_prompt = format!(
            "{}\n\n## 相关知识\n{}",
            config.system_prompt, knowledge
        );
    }

    // 注入对话历史摘要
    if let Some(history) = &context.conversation_summary {
        config.system_prompt = format!(
            "{}\n\n## 对话历史摘要\n{}",
            config.system_prompt, history
        );
    }

    Ok(config)
}

pub struct RoleContext {
    pub constitutional_context: Option<String>,
    pub knowledge_context: Option<String>,
    pub conversation_summary: Option<String>,
    pub target_jurisdiction: Option<String>,
}
```

---

## Phase 8: 端到端集成测试

> **目标**: 验证全链路闭环
> **预估**: 3-4 天 | **风险**: 低

### 8.1 关键场景测试矩阵

**新建文件**: `codex-rs/codex-patent-tools/tests/e2e_scenarios.rs`

```rust
/// 端到端场景测试

#[test]
fn e2e_novelty_analysis_pipeline() {
    // 1. 输入技术交底书
    // 2. 自动检索对比文件
    // 3. 权利要求解析
    // 4. 新颖性分析
    // 5. 生成报告
    // 验证：报告包含结论、评分、引用的对比文件
}

#[test]
fn e2e_oa_response_workflow() {
    // 1. 输入审查意见文本
    // 2. OA 解析
    // 3. 策略生成
    // 4. 答复草稿
    // 5. 审查员模拟评估
    // 6. 质量检查
    // 验证：答复质量评分 > 70
}

#[test]
fn e2e_claim_drafting_workflow() {
    // 1. 输入技术交底书
    // 2. 提取发明内容
    // 3. 生成权利要求
    // 4. 说明书撰写
    // 5. 撰写质量评审
    // 6. 宪法合规检查
    // 验证：质量评分 > 70，无 block 级合规违规
}

#[test]
fn e2e_infringement_analysis() {
    // 1. 输入专利权利要求 + 被控侵权产品描述
    // 2. 权利要求解析
    // 3. 字面侵权分析
    // 4. 等同侵权分析
    // 5. 综合判断
    // 验证：分析结果包含特征对比矩阵
}

#[test]
fn e2e_workflow_with_checkpoint() {
    // 1. 启动 5 步工作流
    // 2. 执行到 step 2 后保存 checkpoint
    // 3. resume_from_checkpoint
    // 4. 验证 step 0-2 不重新执行
    // 5. 验证 step 3-4 正常执行
}

#[test]
fn e2e_constitutional_compliance() {
    // 1. 构造包含违规内容的输出
    // 2. 运行宪法合规检查
    // 3. 验证 block 级规则正确阻断
    // 4. 验证 warn 级规则正确告警
}
```

### 8.2 搜索质量基准

**新建文件**: `codex-rs/codex-patent-knowledge/tests/search_benchmark.rs`

```rust
/// 搜索质量基准测试
/// 确保每次知识库更新后搜索质量不退化

#[test]
fn benchmark_keyword_search() {
    // 使用 eval_queries.json 测试集
    // 验证 P@5 >= 0.6, P@10 >= 0.5, MRR >= 0.7
}

#[test]
fn benchmark_semantic_search() {
    // 需要向量索引可用
    // 验证语义搜索优于关键词搜索
}

#[test]
fn benchmark_hybrid_search() {
    // 验证混合搜索优于单一搜索
}
```

---

## 实施路线图

```
Week 1-2:  Phase 1 (工程健康) → 所有 PR 合入
Week 2-3:  Phase 2 (撰写能力) + Phase 4 (知识库) 并行
Week 3-4:  Phase 3 (分析深化) + Phase 6 (合规) 并行
Week 4-5:  Phase 5 (工作流) + Phase 7 (Agent) 并行
Week 5-6:  Phase 8 (集成测试) + 全链路验证
```

## 验证清单

每个 Phase 完成后执行：

```bash
# 编译
cargo check -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools \
  --no-default-features -p codex-patent-workflow -p codex-patent-constitutional

# Clippy（0 警告）
cargo clippy -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools \
  --no-default-features -p codex-patent-workflow -p codex-patent-constitutional

# 格式化
cargo fmt --check

# 测试
cargo nextest run --no-fail-fast \
  -p codex-patent-domain -p codex-patent-agents -p codex-patent-workflow \
  -p codex-patent-constitutional
```

## 预期提升

| 维度 | 当前 | 目标 | 关键改进 |
|------|------|------|---------|
| **撰写能力** | 5/10 | 8/10 | 结构化说明书生成 + 高级权利要求布局 + 质量评审 |
| **分析能力** | 8/10 | 9/10 | 领域差异化 + 等同侵权 + 无效分析管线 + 多轮模拟 |
| **知识库** | 8/10 | 9/10 | 数据管线自动化 + 术语标准化 + 分领域搜索 |
| **工作流** | 8/10 | 9/10 | LLM 规划 + 模板匹配 + 条件路由 + 真实恢复 |
| **合规** | 7.5/10 | 9/10 | 全变体实现 + 语义合规（可选）|
| **Agent** | 7/10 | 8.5/10 | 真实 LLM 集成 + 上下文注入 + 模型路由 |
| **文本处理** | 5/10 | 7/10 | 术语标准化（Phase 4.2）覆盖 |
| **整体** | 7.5/10 | **9/10** | 全栈均衡提升 |

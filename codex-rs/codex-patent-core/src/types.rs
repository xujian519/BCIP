use serde::{Deserialize, Serialize};

/// 权利要求类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimType {
    Independent,
    Dependent,
}

/// 特征类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FeatureType {
    Element,
    Action,
    Parameter,
    Condition,
    Result,
}

/// 特征对应关系类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrespondenceType {
    Exact,
    Equivalent,
    Different,
    Missing,
}

/// 解析后的特征
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ParsedFeature {
    pub id: String,
    pub description: String,
    pub feature_type: FeatureType,
    pub component: Option<String>,
    pub parameters: Vec<String>,
}

/// 解析后的权利要求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedClaim {
    pub claim_number: u32,
    pub claim_type: ClaimType,
    pub preamble: String,
    pub transition_word: String,
    pub body: String,
    pub features: Vec<ParsedFeature>,
    pub dependent_from: Option<u32>,
}

/// 知识卡片
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeCard {
    pub id: String,
    pub title: String,
    pub concept: String,
    pub quality: f64,
    pub domain: String,
    pub file_path: String,
    pub related_concepts: Vec<String>,
    pub generated_at: i64,
    pub version: String,
}

/// 法律文档
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LawDocument {
    pub id: String,
    pub level: String,
    pub name: String,
    pub subtitle: Option<String>,
    pub filename: String,
    pub publish: bool,
    pub expired: bool,
    pub category_id: String,
    pub content: String,
}

/// 法律分类
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LawCategory {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub is_sub_folder: bool,
    pub group: String,
    pub order: i32,
}

/// 检索来源
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSource {
    KnowledgeGraph,
    LawDatabase,
    KnowledgeCard,
}

/// 检索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub source: SearchSource,
    pub title: String,
    pub content: String,
    pub score: f64,
    pub id: String,
    pub item_type: String,
}

/// 知识卡片索引元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CardIndexMeta {
    pub total_cards: usize,
    pub cards: Vec<KnowledgeCard>,
}

/// 知识图谱节点
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KgNode {
    pub id: String,
    pub node_type: String,
    pub name: String,
    pub title: String,
    pub content: Option<String>,
    pub law_refs_count: Option<i64>,
    pub source: Option<String>,
    pub full_ref: Option<String>,
    pub chapter: Option<String>,
    pub article_number: Option<String>,
}

/// 知识图谱边
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KgEdge {
    pub id: i64,
    pub source: String,
    pub target: String,
    pub relation: String,
}

/// 推理步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningStep {
    pub step_name: String,
    pub description: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
}

/// 推理结论
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningConclusion {
    pub method: String,
    pub steps: Vec<ReasoningStep>,
    pub conclusion: String,
    pub evidence_ids: Vec<String>,
}

/// 案例上下文
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseContext {
    pub invention: Option<String>,
    pub prior_art_contains_all: Option<bool>,
    pub differences: Option<Vec<String>>,
    pub technical_effect: Option<String>,
    pub performance_improvement: Option<f64>,
    pub obviousness: Option<bool>,
    pub rejection_type: Option<String>,
    pub technical_effects: Option<Vec<String>>,
    pub prior_art_different_field: Option<bool>,
}

/// 应用的规则
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedRule {
    pub rule_name: String,
    pub conclusion: String,
    pub applies: bool,
    pub score: f64,
}

/// 分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub conclusion: String,
    pub net_score: f64,
    pub confidence: f64,
    pub applied_rules: Vec<AppliedRule>,
}

/// 驳回类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RejectionType {
    Inventiveness,
    Obviousness,
    LackOfNovelty,
    InsufficientDisclosure,
    UnpatentableSubject,
}

/// 决定类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    Invalid,
    PartialInvalid,
    Maintain,
}

/// 无效决定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvalidDecision {
    pub id: String,
    pub patent_number: String,
    pub decision_number: String,
    pub decision_date: String,
    pub decision_type: DecisionType,
    pub grounds: Vec<String>,
    pub conclusion: String,
}

/// 撰写质量报告
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftQualityReport {
    pub overall_score: f64,
    pub is_acceptable: bool,
    pub dimensions: Vec<QualityDimension>,
    pub critical_issues: Vec<String>,
    pub warnings: Vec<String>,
}

/// 质量维度
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityDimension {
    pub name: String,
    pub score: f64,
    pub max_score: f64,
    pub description: String,
}

/// 对比特征
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompareFeature {
    pub id: String,
    pub description: String,
}

/// 对比文档
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompareDocument {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub abstract_text: String,
    #[serde(default)]
    pub claims: Vec<String>,
    #[serde(default)]
    pub ipc_codes: Vec<String>,
    #[serde(default)]
    pub features: Vec<CompareFeature>,
}

/// 特征匹配结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMatchResult {
    pub exact_matches: Vec<FeatureMatch>,
    pub equivalent_matches: Vec<FeatureMatch>,
    pub different_features: Vec<String>,
    pub missing_features: Vec<String>,
    pub coverage_ratio: f64,
    pub infringement_type: Option<InfringementType>,
}

/// 特征匹配
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureMatch {
    pub target_feature: String,
    pub prior_feature: String,
    pub similarity_score: f64,
    pub match_type: CorrespondenceType,
}

/// 侵权类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InfringementType {
    Literal,
    DoctrineOfEquivalents,
    NoInfringement,
}

/// 规则违反
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleViolation {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
    pub location: String,
}

/// 严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 专利文档
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatentDocument {
    pub title: Option<String>,
    pub abstract_text: Option<String>,
    pub claims: Vec<String>,
    pub specification: Option<String>,
    pub drawings: Vec<String>,
}
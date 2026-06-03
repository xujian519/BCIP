use serde::Deserialize;
use serde::Serialize;

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
pub struct KnowledgeCard {
    pub id: String,
    pub title: String,
    pub concept: String,
    pub quality: f64,
    pub domain: String,
    #[serde(alias = "filePath")]
    pub file_path: String,
    pub related_concepts: Vec<String>,
    pub generated_at: String,
    pub version: i64,
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
    #[serde(default)]
    pub source_path: String,
    #[serde(default)]
    pub source_db: String,
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

/// 发明类型（创造性三步法分类）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum InventionType {
    Pioneering,
    Combination,
    Selection,
    Transconversion,
    NewUse,
    ElementModification,
    #[default]
    Unknown,
}

/// 组合发明类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombinationType {
    SimpleStack,
    Synergistic,
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
    // ── 三步法扩展字段 ──
    pub closest_prior_art: Option<String>,
    pub claim_features: Option<Vec<ParsedFeature>>,
    pub prior_art_features: Option<Vec<ParsedFeature>>,
    pub distinguishing_features: Option<Vec<String>>,
    pub actual_problem_solved: Option<String>,
    pub invention_type: Option<InventionType>,
    pub has_teaching_away: Option<bool>,
    pub has_technical_prejudice: Option<bool>,
    pub has_unexpected_effect: Option<bool>,
    pub has_long_felt_need: Option<bool>,
    pub is_combination: Option<CombinationType>,
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

// ── OA 相关类型 ──

/// 审查意见类型。
/// 审查意见类型。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OaType {
    /// 缺乏新颖性。
    Novelty,
    /// 缺乏创造性（非显而易见性）。
    InventiveStep,
    /// 不清楚/不明确。
    Clarity,
    /// 权利要求得不到说明书支持。
    Support,
    /// 超范围/修改超范围。
    Scope,
    /// 形式缺陷。
    Formal,
    /// 其他类型的审查意见。
    Other(String),
}

/// 引用的对比文件（审查意见通知书中引用的现有技术文献）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CitedReference {
    pub document_number: String,
    pub relevancy: String,
    pub claims_affected: Vec<usize>,
}

/// 审查意见通知书。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OfficeAction {
    pub oa_type: OaType,
    pub citations: Vec<CitedReference>,
    pub examiner_arguments: String,
    pub affected_claims: Vec<usize>,
}

/// 答复策略类型。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ResponseStrategyType {
    AmendClaims,
    Argue,
    Hybrid,
    Withdraw,
}

/// 审查意见答复策略。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResponseStrategy {
    pub strategy_type: ResponseStrategyType,
    pub reasoning: String,
    pub confidence: f32,
}

// ── 质量评估相关类型 ──

/// 单个质量问题的描述。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityIssue {
    pub dimension: String,
    pub severity: String,
    pub description: String,
    pub suggestion: String,
}

/// 综合质量评估结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QualityAssessment {
    pub clarity_score: f32,
    pub support_score: f32,
    pub scope_score: f32,
    pub enablement_score: f32,
    pub overall_score: f32,
    pub issues: Vec<QualityIssue>,
}

// ── 技术特征相关类型 ──

/// 技术特征类别。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FeatureCategory {
    Structural,
    Functional,
    Method,
    Material,
    Other,
}

/// 技术特征描述。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TechnicalFeature {
    pub id: String,
    pub description: String,
    pub feature_type: super::FeatureType,
    pub category: FeatureCategory,
    pub component: Option<String>,
    pub function: Option<String>,
}

/// 问题-特征-效果（PFE）三元组，用于创造性三步法分析。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProblemFeatureEffect {
    pub id: String,
    pub technical_problem: String,
    pub technical_features: Vec<TechnicalFeature>,
    pub technical_effects: Vec<String>,
}

/// 技术交底书文档。
pub struct DisclosureDoc {
    pub raw_text: String,
    pub sections: std::collections::HashMap<String, String>,
    pub confidence: f32,
}

/// 权利要求草稿（撰写过程中使用）。
pub struct ClaimDraft {
    pub id: String,
    pub claim_type: ClaimType,
    pub preamble: String,
    pub transitional_phrase: String,
    pub elements: Vec<String>,
    pub dependent_on: Option<String>,
}

// ── 法律世界模型三层架构 ──
// 参考: Athena constitution.py 三层架构（基础法→专利专业法→司法案例）

/// 法律层级类型（三层架构）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalLayer {
    /// 第一层：基础法律层（民法典、民诉法等通用法律 + 司法解释）
    FoundationLaw,
    /// 第二层：专利专业层（专利法、审查指南、复审无效决定书）
    PatentProfessional,
    /// 第三层：司法案例层（法院判决文书）
    JudicialCase,
}

impl LegalLayer {
    pub fn weight(&self) -> u8 {
        match self {
            Self::FoundationLaw => 1,
            Self::PatentProfessional => 2,
            Self::JudicialCase => 3,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FoundationLaw => "foundation_law",
            Self::PatentProfessional => "patent_professional",
            Self::JudicialCase => "judicial_case",
        }
    }
}

/// 法律实体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalEntityType {
    Law,
    Regulation,
    Guideline,
    Interpretation,
    InvalidationDecision,
    Judgment,
    CourtVerdict,
}

/// 知识图谱节点关系类别（跨层/同层/跨域）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationCategory {
    LayerReference,
    PeerReference,
    CrossDomainMapping,
}

/// 工具所属业务域，用于角色感知的工具裁剪。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolDomain {
    Search,
    WebSearch,
    Drafting,
    Oa,
    Quality,
    Analysis,
    Document,
    Legal,
    Management,
    Review,
    Evaluation,
    Simulator,
    Council,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legal_layer_weight_order() {
        assert!(LegalLayer::FoundationLaw.weight() < LegalLayer::PatentProfessional.weight());
        assert!(LegalLayer::PatentProfessional.weight() < LegalLayer::JudicialCase.weight());
    }

    #[test]
    fn legal_layer_serde_roundtrip() {
        let layer = LegalLayer::PatentProfessional;
        let json = serde_json::to_string(&layer).unwrap();
        assert_eq!(json, "\"patent_professional\"");
        let back: LegalLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(back, LegalLayer::PatentProfessional);
    }

    #[test]
    fn legal_entity_type_discrimination() {
        let law = serde_json::to_string(&LegalEntityType::Law).unwrap();
        let guideline = serde_json::to_string(&LegalEntityType::Guideline).unwrap();
        assert_ne!(law, guideline);
    }
}

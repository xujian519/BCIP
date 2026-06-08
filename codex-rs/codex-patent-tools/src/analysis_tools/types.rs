use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClaimParseInput {
    /// 权利要求文本（简体中文）。
    pub claim_text: String,
    /// 权利要求编号（在原文中的序号）。
    pub claim_number: u32,
}
/// 权利要求对比输入。
///
/// 包含两份待比较的权利要求文本。
#[derive(Debug, Deserialize)]
pub struct ClaimCompareInput {
    /// 第一份权利要求文本。
    pub claim_a: String,
    /// 第二份权利要求文本。
    pub claim_b: String,
}
/// 新颖性分析输入。
///
/// 包含发明技术方案描述及其与现有技术的差异。
#[derive(Debug, Deserialize)]
pub struct NoveltyAnalysisInput {
    /// 发明的技术方案描述。
    pub invention_description: String,
    /// 现有技术的方案描述列表。
    pub prior_art_descriptions: Option<Vec<String>>,
    /// 发明与现有技术的区别点描述。
    pub differences: Option<Vec<String>>,
}
/// 创造性分析输入。
///
/// 支持传统的创造性判断和"三步法"评估模型。
#[derive(Debug, Deserialize)]
pub struct InventivenessAnalysisInput {
    /// 发明的技术方案描述。
    pub invention_description: Option<String>,
    /// 技术效果描述。
    pub technical_effect: Option<String>,
    /// 性能提升幅度。
    pub performance_improvement: Option<f64>,
    /// 是否显而易见。
    pub obviousness: Option<bool>,
    // ── 三步法增强字段 ──
    /// 权利要求文本（用于三步法分析）。
    pub claim_text: Option<String>,
    /// 最接近的现有技术。
    pub closest_prior_art: Option<String>,
    /// 是否存在相反教导。
    pub has_teaching_away: Option<bool>,
    /// 是否存在技术偏见。
    pub has_technical_prejudice: Option<bool>,
    /// 是否存在预料不到的技术效果。
    pub has_unexpected_effect: Option<bool>,
    /// 是否存在长期需要但未解决的需求。
    pub has_long_felt_need: Option<bool>,
}
/// 创新性评估输入。
///
/// 适用于实用新型或非传统发明的创新性评估。
#[derive(Debug, Deserialize)]
pub struct InnovationEvaluatorInput {
    /// 发明的技术方案描述。
    pub invention_description: String,
    /// 技术效果描述。
    pub technical_effect: Option<String>,
    /// 性能提升幅度。
    pub performance_improvement: Option<f64>,
    /// 是否显而易见。
    pub obviousness: Option<bool>,
}
/// 侵权分析输入。
///
/// 包含被侵权权利要求和被控侵权产品的描述。
#[derive(Debug, Deserialize)]
pub struct InfringementAnalysisInput {
    /// 权利要求文本。
    pub claim_text: String,
    /// 被控侵权产品的技术方案描述。
    pub accused_product_description: String,
}
/// 法律问答输入。
///
/// 基于专利法律知识库的问答请求。
#[derive(Debug, Deserialize)]
pub struct LegalQAInput {
    /// 用户提出的法律问题。
    pub question: String,
}
/// 知识检索输入。
///
/// 用于从专利知识库中检索相关信息。
#[derive(Debug, Deserialize)]
pub struct KnowledgeSearchInput {
    /// 检索查询文本。
    pub query: String,
    /// 返回结果数量上限。
    pub limit: Option<usize>,
    /// 是否启用语义检索（而非关键词匹配）。
    pub semantic: Option<bool>,
}

/// 技术三元组提取输入。
///
/// 从技术文本中提取（技术问题、技术手段、技术效果）三元组。
#[derive(Debug, Deserialize)]
pub struct TechTripleExtractorInput {
    /// 待分析的技术文本。
    pub text: String,
}
/// 技术特征提取输入。
///
/// 从专利权利要求中提取技术特征。
#[derive(Debug, Deserialize)]
pub struct FeatureExtractorInput {
    /// 包含技术特征的文本。
    pub text: String,
}
/// 专利对比输入。
///
/// 用于两份专利文件之间的对比分析。
#[derive(Debug, Deserialize)]
pub struct PatentCompareInput {
    /// 目标专利（待评估）。
    pub target: String,
    /// 对比文件（现有技术）。
    pub prior_art: String,
}
/// 发明理解输入。
///
/// 包含发明的基本信息供 AI 理解和分析。
#[derive(Debug, Deserialize)]
pub struct InventionUnderstandingInput {
    /// 发明名称。
    pub invention_title: String,
    /// 发明所属技术领域。
    pub technical_field: String,
    /// 发明的技术方案公开内容。
    pub technical_disclosure: String,
}
/// 技术单元输入。
///
/// 从权利要求中提取技术单元的输入。
#[derive(Debug, Deserialize)]
pub struct TechUnitInput {
    /// 权利要求文本。
    pub claim_text: String,
}
/// 保护范围分析输入。
///
/// 用于分析权利要求的保护范围和解释空间。
#[derive(Debug, Deserialize)]
pub struct ClaimScopeInput {
    /// 权利要求文本。
    pub claim_text: String,
    /// 说明书描述（用于解释权利要求）。
    pub description: Option<String>,
}
fn default_researcher_depth() -> u64 {
    2
}
/// 深度研究输入。
///
/// 控制 AI 研究员的检索深度和范围。
#[derive(Debug, Deserialize)]
pub struct ResearcherInput {
    /// 研究查询文本。
    pub query: String,
    #[serde(default = "default_researcher_depth")]
    /// 研究的递归深度（默认 2）。
    pub depth: u64,
}

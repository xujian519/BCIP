use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QualityCheckInput {
    /// 待检查的权利要求列表。
    pub claims: Vec<ClaimDraftInput>,
    /// 专利类型（可选，如 "invention" / "utility_model"）。
    pub patent_type: Option<String>,
}

/// 单条权利要求草稿输入。
#[derive(Debug, Deserialize)]
pub struct ClaimDraftInput {
    /// 权利要求序号。
    pub id: Option<String>,
    /// 权利要求类型："independent" 或 "dependent"。
    pub claim_type: String,
    /// 前序部分。
    pub preamble: String,
    /// 过渡语（如 "包括"、"由……组成"）。
    pub transitional_phrase: Option<String>,
    /// 技术特征列表。
    pub elements: Vec<String>,
    /// 引用的权利要求序号（从属权利要求时）。
    pub dependent_on: Option<String>,
}

/// 客体审查输入参数。
#[derive(Debug, Deserialize)]
pub struct SubjectMatterInput {
    /// 发明名称。
    pub invention_title: String,
    /// 权利要求全文列表。
    pub claims: Vec<String>,
    /// 专利类型（可选）。
    pub patent_type: Option<String>,
}

/// 单一性检查输入参数。
#[derive(Debug, Deserialize)]
pub struct UnityInput {
    /// 权利要求全文列表。
    pub claims: Vec<String>,
    /// 专利类型（可选）。
    pub patent_type: Option<String>,
    /// 发明名称（可选）。
    pub invention_title: Option<String>,
}

/// 说明书形式审查输入参数。
#[derive(Debug, Deserialize)]
pub struct SpecFormalityInput {
    /// 说明书各部分内容。
    pub specification: SpecInput,
    /// 权利要求列表。
    pub claims: Vec<String>,
    /// 专利类型（可选）。
    pub patent_type: Option<String>,
}

/// 说明书各部分输入。
#[derive(Debug, Deserialize)]
pub struct SpecInput {
    /// 技术领域。
    pub technical_field: Option<String>,
    /// 背景技术。
    pub background_art: Option<String>,
    /// 发明内容。
    pub invention_content: Option<String>,
    /// 具体实施方式。
    pub embodiment: Option<String>,
    /// 附图说明（可选，有图则提供）。
    pub drawings_description: Option<String>,
}

/// 法律用语合规检查输入参数。
#[derive(Debug, Deserialize)]
pub struct LegalLanguageInput {
    /// 权利要求全文列表。
    pub claims: Vec<String>,
    /// 检查严格程度（1-3，默认 1）。
    pub check_level: Option<u32>,
}

/// 权利要求依赖关系检查输入参数。
#[derive(Debug, Deserialize)]
pub struct ClaimDependencyInput {
    /// 权利要求全文列表。
    pub claims: Vec<String>,
}

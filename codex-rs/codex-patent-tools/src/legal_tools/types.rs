use serde::Deserialize;

/// 法律问答输入。
///
/// 基于知识库的法律问题查询。
#[derive(Debug, Deserialize)]
pub struct LegalQAInput {
    /// 用户提出的法律问题。
    pub question: String,
    /// 法律领域（如 "patent" / "trademark" / "copyright"）。
    pub domain: Option<String>,
}

/// 法律知识检索输入。
///
/// 从法律知识库中检索相关内容。
#[derive(Debug, Deserialize)]
pub struct LegalKnowledgeInput {
    /// 检索查询文本。
    pub query: String,
    /// 返回结果数量上限。
    pub limit: Option<usize>,
    /// 检索类别过滤。
    pub category: Option<String>,
}

/// 法律依据检索输入。
///
/// 根据法律问题查找对应的法律条文和依据。
#[derive(Debug, Deserialize)]
pub struct LegalBasisInput {
    /// 待查询的法律问题描述。
    pub legal_issue: String,
    /// 专利类型（发明/实用新型/外观设计）。
    pub patent_type: Option<String>,
}

/// IPC 分类检索输入。
///
/// 根据关键词搜索 IPC 分类号及其含义。
#[derive(Debug, Deserialize)]
pub struct IpcSearchInput {
    /// 检索查询文本（关键词或分类号）。
    pub query: String,
    /// 返回结果数量上限。
    pub limit: Option<usize>,
}

/// 三角化查询输入。
///
/// 同时从 IPC 分类、概念图、法条三个维度交叉查询。
#[derive(Debug, Deserialize)]
pub struct TriangleQueryInput {
    /// IPC 分类号过滤。
    pub ipc: Option<String>,
    /// 技术概念过滤。
    pub concept: Option<String>,
    /// 法条过滤。
    pub clause: Option<String>,
    /// 返回结果数量上限。
    pub limit: Option<usize>,
}

/// 决定/判例检索输入。
///
/// 根据法律条文和理由检索专利审查决定或司法判例。
#[derive(Debug, Deserialize)]
pub struct DecisionSearchInput {
    /// 相关法律条文。
    pub law_article: Option<String>,
    /// 决定理由关键词。
    pub reason: Option<String>,
    /// 决定结论（如 "驳回" / "授权" / "无效"）。
    pub conclusion: Option<String>,
    /// IPC 分类号过滤。
    pub ipc: Option<String>,
    /// 返回结果数量上限。
    pub limit: Option<usize>,
}

/// 知识库原始检索输入。
///
/// 直接对知识库执行原始查询，返回未加工的匹配结果。
#[derive(Debug, Deserialize)]
pub struct KnowledgeSearchRawInput {
    /// 检索查询文本。
    pub query: String,
    #[serde(default)]
    /// 返回结果数量上限。
    pub limit: u64,
    #[serde(default)]
    /// 是否启用语义检索。
    pub semantic: bool,
}

/// 图谱查询原始输入。
///
/// 从知识图谱中按节点遍历检索关联数据。
#[derive(Debug, Deserialize)]
pub struct GraphQueryRawInput {
    /// 起始节点 ID。
    pub start_id: String,
    #[serde(default = "default_max_depth")]
    /// 最大遍历深度。
    pub max_depth: u64,
    /// 关系类型过滤（仅返回指定关系）。
    pub relation_filter: Option<Vec<String>>,
}

fn default_max_depth() -> u64 {
    2
}

/// 图邻居查询输入。
///
/// 获取指定节点的直接邻居。
#[derive(Debug, Deserialize)]
pub struct GraphNeighborsRawInput {
    /// 目标节点 ID。
    pub node_id: String,
}

/// 图谱链接查询输入。
///
/// 按关键词搜索图谱中的链接关系。
#[derive(Debug, Deserialize)]
pub struct LinkGraphRawInput {
    #[serde(default)]
    /// 关联关键词。
    pub keyword: String,
    /// 知识库根路径过滤。
    pub kb_root: Option<String>,
}

/// 知识卡片检索输入。
///
/// 从知识卡片索引中检索结构化知识条目。
#[derive(Debug, Deserialize)]
pub struct CardSearchRawInput {
    /// 检索查询文本。
    pub query: String,
    #[serde(default = "default_card_limit")]
    /// 返回结果数量上限（默认 10）。
    pub limit: u64,
}

fn default_card_limit() -> u64 {
    10
}

/// 图谱路径查询输入。
///
/// 查找两个节点之间的关联路径。
#[derive(Debug, Deserialize)]
pub struct FindPathRawInput {
    /// 起始节点 ID。
    pub from_id: String,
    /// 目标节点 ID。
    pub to_id: String,
    #[serde(default = "default_max_depth")]
    /// 最大搜索深度。
    pub max_depth: u64,
}

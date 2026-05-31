use codex_patent_knowledge::SearchConfig;
use codex_patent_knowledge::SearchMode;
use codex_patent_knowledge::SqliteKnowledgeGraph;
use codex_patent_knowledge::UnifiedSearch;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LegalQAInput {
    pub question: String,
    pub domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LegalKnowledgeInput {
    pub query: String,
    pub limit: Option<usize>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LegalBasisInput {
    pub legal_issue: String,
    pub patent_type: Option<String>,
}

pub struct LegalTools;

impl LegalTools {
    pub fn legal_qa(input: LegalQAInput) -> Result<serde_json::Value, String> {
        let domain = input.domain.as_deref().unwrap_or("patent");
        let templates = [
            (
                "新颖性",
                "《专利法》第22条第2款规定：新颖性是指该发明或者实用新型不属于现有技术。判断标准：单独对比、全部技术特征逐一比较。",
            ),
            (
                "创造性",
                "《专利法》第22条第3款规定：创造性是指与现有技术相比，该发明具有突出的实质性特点和显著的进步。判断方法：三步法。",
            ),
            (
                "实用性",
                "《专利法》第22条第4款规定：实用性是指该发明或者实用新型能够制造或者使用，并能够产生积极效果。",
            ),
            (
                "充分公开",
                "《专利法》第26条第3款：说明书应当对发明作出清楚、完整的说明，以所属技术领域的技术人员能够实现为准。",
            ),
            (
                "修改超范围",
                "《专利法》第33条：不得超出原说明书和权利要求书记载的范围。",
            ),
            (
                "优先权",
                "《专利法》第29条：申请人自首次申请日起12个月内可要求优先权。",
            ),
            ("单一性", "《专利法》第31条：一件专利申请应当限于一项发明。"),
            (
                "保护范围",
                "《专利法》第59条：发明或者实用新型专利权的保护范围以其权利要求的内容为准。",
            ),
        ];
        let answer = templates
            .iter()
            .find(|(k, _)| input.question.contains(k))
            .map(|(_, v)| *v)
            .unwrap_or("该问题涉及专利法领域，建议查阅《专利法》及《专利法实施细则》相关规定。");
        let related = templates
            .iter()
            .filter(|(k, _)| input.question.contains(k) || answer.contains(k))
            .map(|(k, _)| *k)
            .collect::<Vec<_>>();
        Ok(
            serde_json::json!({"question": input.question, "domain": domain, "answer": answer, "related_articles": related}),
        )
    }

    pub fn legal_knowledge_search(input: LegalKnowledgeInput) -> Result<serde_json::Value, String> {
        let limit = input.limit.unwrap_or(5);
        let laws = match input.category.as_deref() {
            Some("novelty") => vec![
                ("专利法第22条第2款", "新颖性定义"),
                ("审查指南第二部分第三章", "新颖性审查"),
            ],
            Some("inventive") => vec![
                ("专利法第22条第3款", "创造性定义"),
                ("审查指南第二部分第四章", "创造性审查"),
            ],
            Some("specification") => vec![
                ("专利法第26条第3款", "充分公开"),
                ("实施细则第17条", "说明书撰写顺序"),
            ],
            _ => vec![
                ("专利法", "全文"),
                ("专利法实施细则", "全文"),
                ("审查指南", "全文"),
            ],
        };
        let results: Vec<serde_json::Value> = laws.iter().take(limit).map(|(title, desc)| {
            serde_json::json!({"title": title, "description": desc, "relevance": if title.contains(&input.query) { 0.9 } else { 0.5 }})
        }).collect();
        Ok(serde_json::json!({"query": input.query, "results": results, "total": results.len()}))
    }

    pub fn legal_basis_refs(input: LegalBasisInput) -> Result<serde_json::Value, String> {
        let refs = [
            (
                "专利法第22条第2款",
                "新颖性",
                "该发明或实用新型不属于现有技术",
            ),
            (
                "专利法第22条第3款",
                "创造性",
                "突出的实质性特点和显著的进步",
            ),
            (
                "专利法第22条第4款",
                "实用性",
                "能够制造或者使用，并产生积极效果",
            ),
            ("专利法第26条第3款", "充分公开", "清楚、完整地说明发明"),
            (
                "专利法第26条第4款",
                "权利要求",
                "以说明书为依据，清楚、简要",
            ),
            ("专利法第29条", "优先权", "12个月内可要求优先权"),
            ("专利法第31条", "单一性", "属于一个总的发明构思"),
            ("专利法第33条", "修改", "不得超出原范围"),
            ("专利法第45条", "无效宣告", "自授权公告日起可请求宣告无效"),
            ("专利法第59条", "保护范围", "以权利要求的内容为准"),
            ("实施细则第17条", "说明书", "说明书撰写顺序"),
            ("实施细则第19条", "独立权利要求", "记载必要技术特征"),
            ("实施细则第22条", "从属权利要求", "引用在前的权利要求"),
            (
                "实施细则第68条",
                "无效修改",
                "修改方式仅限于删除、合并、进一步限定",
            ),
        ];
        let related: Vec<serde_json::Value> = refs
            .iter()
            .filter(|(_, cat, _)| {
                input.legal_issue.contains(cat) || cat.contains(&input.legal_issue)
            })
            .map(|(art, _, desc)| serde_json::json!({"article": art, "description": desc}))
            .collect();
        Ok(
            serde_json::json!({"issue": input.legal_issue, "related_articles": related, "total": related.len()}),
        )
    }

    pub fn knowledge_search(
        query: &str,
        limit: usize,
        semantic: bool,
    ) -> Result<serde_json::Value, String> {
        let mlx_url =
            std::env::var("BCIP_MLX_URL").unwrap_or_else(|_| "http://localhost:8009".into());
        let mlx_key = std::env::var("BCIP_MLX_API_KEY").unwrap_or_default();
        let search = UnifiedSearch::with_vector(
            Some(&codex_patent_knowledge::paths::kg_db_path()),
            Some(&codex_patent_knowledge::paths::law_db_path()),
            Some(&codex_patent_knowledge::paths::card_index_path()),
            Some(&codex_patent_knowledge::paths::semantic_index_path()),
            Some(&mlx_url),
            if mlx_key.is_empty() {
                None
            } else {
                Some(&mlx_key)
            },
            Some("bge-m3-mlx-8bit"),
        );
        let config = SearchConfig {
            query: query.to_string(),
            limit,
            mode: if semantic {
                SearchMode::Hybrid
            } else {
                SearchMode::KeywordEnhanced
            },
            ..Default::default()
        };
        let results = search.search(&config);
        serde_json::to_value(&results).map_err(|e| format!("{e}"))
    }

    /// 图遍历查询
    pub fn graph_query(
        start_id: &str,
        relation_filter: Option<Vec<String>>,
        max_depth: usize,
    ) -> Result<serde_json::Value, String> {
        let kg = SqliteKnowledgeGraph::open(&codex_patent_knowledge::paths::kg_db_path())
            .map_err(|e| format!("无法打开知识图谱: {e}"))?;
        let filter: Option<Vec<&str>> = relation_filter
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());
        let filter_ref = filter.as_deref();
        let edges = kg
            .traverse(start_id, filter_ref, max_depth)
            .map_err(|e| format!("图遍历失败: {e}"))?;
        serde_json::to_value(&edges).map_err(|e| format!("{e}"))
    }

    pub fn graph_neighbors(node_id: &str) -> Result<serde_json::Value, String> {
        let kg = SqliteKnowledgeGraph::open(&codex_patent_knowledge::paths::kg_db_path())
            .map_err(|e| format!("无法打开知识图谱: {e}"))?;
        let edges = kg.get_edges(node_id).map_err(|e| format!("{e}"))?;
        serde_json::to_value(&edges).map_err(|e| format!("{e}"))
    }
}

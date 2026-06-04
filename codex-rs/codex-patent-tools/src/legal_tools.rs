//! 法律知识检索与图谱查询工具集。
//!
//! 提供专利法领域相关的知识检索能力，包括：
//! - 法律问答与知识检索 (`LegalQAInput`, `LegalKnowledgeInput`, `LegalBasisInput`)
//! - IPC 分类检索 (`IpcSearchInput`)
//! - 三角化查询（IPC + 概念 + 法条） (`TriangleQueryInput`)
//! - 决定/判例检索 (`DecisionSearchInput`)
//! - 知识图谱原始查询（遍历/邻居/链接/卡片/路径） (`GraphQueryRawInput`, `GraphNeighborsRawInput`, `LinkGraphRawInput`, `CardSearchRawInput`, `FindPathRawInput`)
//! - 原始知识库检索 (`KnowledgeSearchRawInput`)

use codex_patent_knowledge::CardIndex;
use codex_patent_knowledge::SearchConfig;
use codex_patent_knowledge::SearchMode;
use codex_patent_knowledge::UnifiedSearch;
use serde::Deserialize;
use std::sync::Mutex;

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

/// 法律知识查询工具集。
///
/// 提供法律问答、知识检索、图谱查询等法律知识服务能力。
pub struct LegalTools;

impl LegalTools {
    pub fn legal_qa(input: LegalQAInput) -> Result<serde_json::Value, String> {
        let domain = input.domain.as_deref().unwrap_or("patent");
        let question = &input.question;

        // 第一步: 尝试从知识库检索
        let kb_answer = Self::try_knowledge_search(question);

        // 第二步: 尝试从知识卡片检索
        let card_answer = Self::try_card_search(question);

        // 第三步: 如果知识源有结果，优先返回
        if let Some(ref answer) = kb_answer
            && !answer.is_empty()
        {
            return Ok(serde_json::json!({
                "question": question,
                "domain": domain,
                "answer": answer,
                "source": "knowledge_graph",
                "fallback": false,
            }));
        }

        if let Some(ref answer) = card_answer
            && !answer.is_empty()
        {
            return Ok(serde_json::json!({
                "question": question,
                "domain": domain,
                "answer": answer,
                "source": "knowledge_card",
                "fallback": false,
            }));
        }

        // Fallback: 硬编码模板匹配
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
            .find(|(k, _)| question.contains(k))
            .map(|(_, v)| *v)
            .unwrap_or("该问题涉及专利法领域，建议查阅《专利法》及《专利法实施细则》相关规定。");
        let related = templates
            .iter()
            .filter(|(k, _)| question.contains(k) || answer.contains(k))
            .map(|(k, _)| *k)
            .collect::<Vec<_>>();

        Ok(serde_json::json!({
            "question": question,
            "domain": domain,
            "answer": answer,
            "source": "template",
            "fallback": true,
            "related_articles": related,
        }))
    }

    fn try_knowledge_search(query: &str) -> Option<String> {
        let search = UnifiedSearch::global();
        let config = SearchConfig {
            query: query.to_string(),
            limit: 3,
            mode: SearchMode::KeywordEnhanced,
            ..Default::default()
        };
        let results = search.search(&config);
        if results.is_empty() {
            return None;
        }
        let top = &results[0];
        let title = &top.title;
        let content = &top.content;
        let preview: String = content.chars().take(500).collect();
        if preview.is_empty() {
            return None;
        }
        Some(format!("{}：{}", title, preview))
    }

    fn try_card_search(query: &str) -> Option<String> {
        let card_mutex: &Mutex<CardIndex> = UnifiedSearch::global().card_index()?;
        let index = card_mutex.lock().ok()?;
        let cards = index.search_by_concept(query, 3);
        if cards.is_empty() {
            // Try keyword search
            let kw_cards = index.search_by_keyword(query, 3);
            if kw_cards.is_empty() {
                return None;
            }
            let card = &kw_cards[0];
            let content = index.load_content(card).ok()?;
            let preview: String = content.chars().take(500).collect();
            if preview.is_empty() {
                return None;
            }
            return Some(format!("[{}] {}", card.title, preview));
        }
        let card = &cards[0];
        let content = index.load_content(card).ok()?;
        let preview: String = content.chars().take(500).collect();
        if preview.is_empty() {
            return None;
        }
        Some(format!("[{}] {}", card.title, preview))
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
        let search = UnifiedSearch::global();
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
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;
        let filter: Option<Vec<&str>> = relation_filter
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());
        let filter_ref = filter.as_deref();
        let edges = kg
            .traverse(start_id, filter_ref, max_depth)
            .map_err(|e| format!("图遍历失败: {e}"))?;
        serde_json::to_value(&edges).map_err(|e| e.to_string())
    }

    pub fn graph_neighbors(node_id: &str) -> Result<serde_json::Value, String> {
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;
        let edges = kg.get_edges(node_id).map_err(|e| e.to_string())?;
        serde_json::to_value(&edges).map_err(|e| e.to_string())
    }

    /// IPC 技术领域搜索
    pub fn ipc_search(input: IpcSearchInput) -> Result<serde_json::Value, String> {
        let limit = input.limit.unwrap_or(10);
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;
        let results = kg
            .search_ipc(&input.query, limit)
            .map_err(|e| format!("IPC 搜索失败: {e}"))?;
        let total = results.len();
        Ok(serde_json::json!({
            "query": input.query,
            "results": results,
            "total": total
        }))
    }

    /// 三角关联查询：通过 IPC/法条/概念任意组合查找关联节点
    pub fn triangle_query(input: TriangleQueryInput) -> Result<serde_json::Value, String> {
        let limit = input.limit.unwrap_or(20);
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;
        let results = kg
            .search_by_triangle(
                input.ipc.as_deref(),
                input.concept.as_deref(),
                input.clause.as_deref(),
                limit,
            )
            .map_err(|e| format!("三角查询失败: {e}"))?;
        let total = results.len();
        Ok(serde_json::json!({
            "query": {
                "ipc": input.ipc,
                "concept": input.concept,
                "clause": input.clause,
            },
            "results": results.iter().map(|n| {
                let content_preview = n.content.as_deref().map(|c| &c[..c.len().min(200)]).unwrap_or("");
                serde_json::json!({
                    "id": n.id,
                    "node_type": n.node_type,
                    "title": n.title,
                    "content": content_preview,
                })
            }).collect::<Vec<_>>(),
            "total": total
        }))
    }

    /// 复审决定搜索：按法条、无效理由、结论、IPC 搜索
    pub fn decision_search(input: DecisionSearchInput) -> Result<serde_json::Value, String> {
        let limit = input.limit.unwrap_or(20);
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;

        // 构建搜索查询
        let mut query_parts = Vec::new();
        if let Some(ref reason) = input.reason {
            query_parts.push(reason.clone());
        }
        if let Some(ref conclusion) = input.conclusion {
            query_parts.push(conclusion.clone());
        }
        let query = if query_parts.is_empty() {
            "无效".to_string()
        } else {
            query_parts.join(" ")
        };

        let nodes = kg
            .search_nodes(&query, Some("InvalidDecision"), limit)
            .map_err(|e| format!("复审决定搜索失败: {e}"))?;

        // 按 IPC/法条三角查询补充
        let triangle_nodes = if input.ipc.is_some() || input.law_article.is_some() {
            let clause = input.law_article.as_deref();
            kg.search_by_triangle(input.ipc.as_deref(), None, clause, limit)
                .unwrap_or_default()
                .into_iter()
                .filter(|n| n.node_type == "InvalidDecision")
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // 合并去重
        let mut seen = std::collections::HashSet::new();
        let mut results = Vec::new();
        for n in nodes.iter().chain(triangle_nodes.iter()) {
            if seen.insert(n.id.clone()) {
                let matches_reason = input
                    .reason
                    .as_ref()
                    .is_none_or(|r| n.chapter.as_deref().is_some_and(|ch| ch.contains(r)));
                let matches_conclusion = input
                    .conclusion
                    .as_ref()
                    .is_none_or(|c| n.title.contains(c));
                if matches_reason && matches_conclusion {
                    let content_preview = n
                        .content
                        .as_deref()
                        .map(|c| &c[..c.len().min(200)])
                        .unwrap_or("");
                    results.push(serde_json::json!({
                        "id": n.id,
                        "title": n.title,
                        "content": content_preview,
                        "chapter": n.chapter,
                        "article_number": n.article_number,
                    }));
                }
            }
        }
        results.truncate(limit);
        let total = results.len();
        Ok(serde_json::json!({
            "query": {
                "law_article": input.law_article,
                "reason": input.reason,
                "conclusion": input.conclusion,
                "ipc": input.ipc,
            },
            "results": results,
            "total": total
        }))
    }

    /// 知识卡片搜索：按概念/关键词检索 Wiki 知识卡片
    pub fn card_search(query: &str, limit: usize) -> Result<serde_json::Value, String> {
        let card_mutex = UnifiedSearch::global()
            .card_index()
            .ok_or("知识卡片索引未初始化")?;
        let index = card_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;

        let by_keyword = index.search_by_keyword(query, limit);
        let by_concept = index.search_by_concept(query, limit);

        // 合并去重，概念搜索优先
        let mut seen_ids = std::collections::HashSet::new();
        let mut results = Vec::new();
        for card in by_concept.iter().chain(by_keyword.iter()) {
            if seen_ids.insert(card.id.clone()) {
                let content = index.load_content(card).unwrap_or_default();
                let preview = if content.len() > 300 {
                    &content[..300]
                } else {
                    &content
                };
                results.push(serde_json::json!({
                    "id": card.id,
                    "title": card.title,
                    "concept": card.concept,
                    "domain": card.domain,
                    "quality": card.quality,
                    "related_concepts": card.related_concepts,
                    "content_preview": preview,
                }));
                if results.len() >= limit {
                    break;
                }
            }
        }
        Ok(serde_json::json!({
            "query": query,
            "total": results.len(),
            "results": results,
        }))
    }

    /// 路径查找：在知识图谱中查找两个节点之间的最短路径
    pub fn find_path(
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> Result<serde_json::Value, String> {
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;
        let paths = kg
            .find_path(from_id, to_id, max_depth)
            .map_err(|e| format!("路径查找失败: {e}"))?;
        Ok(serde_json::json!({
            "from": from_id,
            "to": to_id,
            "max_depth": max_depth,
            "paths": paths,
            "path_count": paths.len(),
        }))
    }
}

pub fn register_legal_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("LegalQA".into(), |input| {
        Box::pin(async move {
            let parsed: LegalQAInput = serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::legal_qa(parsed)
        })
    });
    t.insert("LegalKnowledgeSearch".into(), |input| {
        Box::pin(async move {
            let parsed: LegalKnowledgeInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::legal_knowledge_search(parsed)
        })
    });
    t.insert("LegalBasisRefs".into(), |input| {
        Box::pin(async move {
            let parsed: LegalBasisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::legal_basis_refs(parsed)
        })
    });
    t.insert("KnowledgeSearch".into(), |input| {
        Box::pin(async move {
            let parsed: KnowledgeSearchRawInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::knowledge_search(&parsed.query, parsed.limit as usize, parsed.semantic)
        })
    });
    t.insert("GraphQuery".into(), |input| {
        Box::pin(async move {
            let parsed: GraphQueryRawInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::graph_query(
                &parsed.start_id,
                parsed.relation_filter,
                parsed.max_depth as usize,
            )
        })
    });
    t.insert("GraphNeighbors".into(), |input| {
        Box::pin(async move {
            let parsed: GraphNeighborsRawInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::graph_neighbors(&parsed.node_id)
        })
    });
    t.insert("IpcSearch".into(), |input| {
        Box::pin(async move {
            let parsed: IpcSearchInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::ipc_search(parsed)
        })
    });
    t.insert("TriangleQuery".into(), |input| {
        Box::pin(async move {
            let parsed: TriangleQueryInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::triangle_query(parsed)
        })
    });
    t.insert("DecisionSearch".into(), |input| {
        Box::pin(async move {
            let parsed: DecisionSearchInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::decision_search(parsed)
        })
    });
    t.insert("LinkGraph".into(), |input| {
        Box::pin(async move {
            let parsed: LinkGraphRawInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let link_root = parsed
                .kb_root
                .unwrap_or_else(codex_patent_knowledge::paths::kb_root);
            let graph =
                codex_patent_knowledge::LinkGraph::build(&link_root).map_err(|e| e.to_string())?;
            let links: Vec<serde_json::Value> = if parsed.keyword.is_empty() {
                graph
                    .all_links()
                    .iter()
                    .take(100)
                    .map(|l| {
                        serde_json::json!({
                            "source": l.source_file,
                            "target": l.target_file,
                            "anchor": l.anchor,
                        })
                    })
                    .collect()
            } else {
                graph
                    .search_by_concept(&parsed.keyword)
                    .iter()
                    .take(50)
                    .map(|l| {
                        serde_json::json!({
                            "source": l.source_file,
                            "target": l.target_file,
                            "anchor": l.anchor,
                        })
                    })
                    .collect()
            };
            Ok(serde_json::json!({
                "total": graph.total_links(),
                "links": links,
            }))
        })
    });
    t.insert("RefreshKnowledge".into(), |_input| {
        Box::pin(async move {
            let pipeline = codex_patent_knowledge::RefreshPipeline::new(
                &codex_patent_knowledge::paths::kb_root(),
                &format!(
                    "{}/.kb-version.json",
                    codex_patent_knowledge::paths::kb_root()
                ),
            );
            pipeline.status_json()
        })
    });
    t.insert("SearchEval".into(), |input| {
        Box::pin(async move {
            let semantic = input
                .get("semantic")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let mode = if semantic {
                codex_patent_knowledge::SearchMode::Hybrid
            } else {
                codex_patent_knowledge::SearchMode::KeywordEnhanced
            };
            let evaluator = codex_patent_knowledge::SearchEval::new(
                Some(&codex_patent_knowledge::paths::kg_db_path()),
                Some(&codex_patent_knowledge::paths::law_db_path()),
                Some(&codex_patent_knowledge::paths::card_index_path()),
                &codex_patent_knowledge::paths::eval_queries_path(),
            )
            .map_err(|e| e.to_string())?;
            let results = evaluator.run(mode);
            let summary = codex_patent_knowledge::SearchEval::summary(&results);
            serde_json::to_value(&summary).map_err(|e| e.to_string())
        })
    });
    t.insert("CardSearch".into(), |input| {
        Box::pin(async move {
            let parsed: CardSearchRawInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::card_search(&parsed.query, parsed.limit as usize)
        })
    });
    t.insert("FindPath".into(), |input| {
        Box::pin(async move {
            let parsed: FindPathRawInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            LegalTools::find_path(&parsed.from_id, &parsed.to_id, parsed.max_depth as usize)
        })
    });
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Input struct deserialization tests ---

    #[test]
    fn deserialize_legal_qa_input() {
        let json = serde_json::json!({
            "question": "什么是新颖性？",
            "domain": "patent"
        });
        let input: LegalQAInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.question, "什么是新颖性？");
        assert_eq!(input.domain.as_deref(), Some("patent"));
    }

    #[test]
    fn deserialize_legal_qa_input_optional_domain() {
        let json = serde_json::json!({
            "question": "测试问题"
        });
        let input: LegalQAInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.domain.is_none());
    }

    #[test]
    fn deserialize_legal_knowledge_input() {
        let json = serde_json::json!({
            "query": "新颖性",
            "limit": 3,
            "category": "novelty"
        });
        let input: LegalKnowledgeInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.query, "新颖性");
        assert_eq!(input.limit, Some(3));
        assert_eq!(input.category.as_deref(), Some("novelty"));
    }

    #[test]
    fn deserialize_legal_basis_input() {
        let json = serde_json::json!({
            "legal_issue": "新颖性",
            "patent_type": "invention"
        });
        let input: LegalBasisInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.legal_issue, "新颖性");
    }

    #[test]
    fn deserialize_ipc_search_input() {
        let json = serde_json::json!({
            "query": "G06F",
            "limit": 5
        });
        let input: IpcSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.query, "G06F");
        assert_eq!(input.limit, Some(5));
    }

    #[test]
    fn deserialize_triangle_query_input() {
        let json = serde_json::json!({
            "ipc": "G06F",
            "concept": "数据处理",
            "clause": "第22条",
            "limit": 10
        });
        let input: TriangleQueryInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.ipc.as_deref(), Some("G06F"));
        assert_eq!(input.concept.as_deref(), Some("数据处理"));
        assert_eq!(input.clause.as_deref(), Some("第22条"));
    }

    #[test]
    fn deserialize_decision_search_input() {
        let json = serde_json::json!({
            "law_article": "第22条第3款",
            "reason": "创造性",
            "conclusion": "维持有效",
            "ipc": "H04N",
            "limit": 10
        });
        let input: DecisionSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.reason.as_deref(), Some("创造性"));
        assert_eq!(input.conclusion.as_deref(), Some("维持有效"));
    }

    #[test]
    fn deserialize_decision_search_input_minimal() {
        let json = serde_json::json!({});
        let input: DecisionSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.law_article.is_none());
        assert!(input.reason.is_none());
        assert!(input.conclusion.is_none());
        assert!(input.ipc.is_none());
        assert!(input.limit.is_none());
    }

    // --- legal_qa template matching tests ---

    #[test]
    fn legal_qa_novelty_match() {
        let input = LegalQAInput {
            question: "什么是新颖性？".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        assert_eq!(result["domain"], "patent");
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第22条第2款"));
            let related = result["related_articles"].as_array().unwrap();
            assert!(related.contains(&serde_json::json!("新颖性")));
        }
    }

    #[test]
    fn legal_qa_inventiveness_match() {
        let input = LegalQAInput {
            question: "创造性的判断标准是什么？".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第22条第3款"));
            assert!(answer.contains("三步法"));
        }
    }

    #[test]
    fn legal_qa_practical_utility_match() {
        let input = LegalQAInput {
            question: "实用性要求是什么".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第22条第4款"));
        }
    }

    #[test]
    fn legal_qa_sufficient_disclosure_match() {
        let input = LegalQAInput {
            question: "如何判断充分公开？".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第26条第3款"));
        }
    }

    #[test]
    fn legal_qa_modification_match() {
        let input = LegalQAInput {
            question: "修改超范围的限制".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第33条"));
        }
    }

    #[test]
    fn legal_qa_priority_match() {
        let input = LegalQAInput {
            question: "优先权如何行使？".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第29条"));
            assert!(answer.contains("12个月"));
        }
    }

    #[test]
    fn legal_qa_unity_match() {
        let input = LegalQAInput {
            question: "单一性要求".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第31条"));
        }
    }

    #[test]
    fn legal_qa_protection_scope_match() {
        let input = LegalQAInput {
            question: "保护范围怎么确定".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第59条"));
        }
    }

    #[test]
    fn legal_qa_unknown_question_fallback() {
        let input = LegalQAInput {
            question: "专利年费缴纳".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).unwrap();
        let answer = result["answer"].as_str().unwrap();
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("查阅《专利法》"));
        }
    }

    #[test]
    fn legal_qa_custom_domain() {
        let input = LegalQAInput {
            question: "新颖性".into(),
            domain: Some("trademark".into()),
        };
        let result = LegalTools::legal_qa(input).unwrap();
        assert_eq!(result["domain"], "trademark");
    }

    // --- legal_knowledge_search tests ---

    #[test]
    fn legal_knowledge_search_novelty_category() {
        let input = LegalKnowledgeInput {
            query: "新颖性".into(),
            limit: Some(5),
            category: Some("novelty".into()),
        };
        let result = LegalTools::legal_knowledge_search(input).unwrap();
        let results = result["results"].as_array().unwrap();
        assert!(!results.is_empty());
        assert!(results[0]["title"].as_str().unwrap().contains("22条"));
    }

    #[test]
    fn legal_knowledge_search_inventive_category() {
        let input = LegalKnowledgeInput {
            query: "创造性".into(),
            limit: Some(5),
            category: Some("inventive".into()),
        };
        let result = LegalTools::legal_knowledge_search(input).unwrap();
        let results = result["results"].as_array().unwrap();
        assert!(results[0]["title"].as_str().unwrap().contains("22条第3款"));
    }

    #[test]
    fn legal_knowledge_search_default_category() {
        let input = LegalKnowledgeInput {
            query: "专利法".into(),
            limit: Some(2),
            category: None,
        };
        let result = LegalTools::legal_knowledge_search(input).unwrap();
        assert_eq!(result["total"], 2);
    }

    #[test]
    fn legal_knowledge_search_respects_limit() {
        let input = LegalKnowledgeInput {
            query: "测试".into(),
            limit: Some(1),
            category: None,
        };
        let result = LegalTools::legal_knowledge_search(input).unwrap();
        assert_eq!(result["total"], 1);
    }

    #[test]
    fn legal_knowledge_search_default_limit() {
        let input = LegalKnowledgeInput {
            query: "测试".into(),
            limit: None,
            category: Some("specification".into()),
        };
        let result = LegalTools::legal_knowledge_search(input).unwrap();
        assert!(result["total"].as_u64().unwrap() <= 5);
    }

    // --- legal_basis_refs tests ---

    #[test]
    fn legal_basis_refs_novelty() {
        let input = LegalBasisInput {
            legal_issue: "新颖性".into(),
            patent_type: None,
        };
        let result = LegalTools::legal_basis_refs(input).unwrap();
        let articles = result["related_articles"].as_array().unwrap();
        assert!(!articles.is_empty());
        assert!(
            articles
                .iter()
                .any(|a| a["article"].as_str().unwrap().contains("22条第2款"))
        );
    }

    #[test]
    fn legal_basis_refs_inventiveness() {
        let input = LegalBasisInput {
            legal_issue: "创造性".into(),
            patent_type: Some("invention".into()),
        };
        let result = LegalTools::legal_basis_refs(input).unwrap();
        let articles = result["related_articles"].as_array().unwrap();
        assert!(
            articles
                .iter()
                .any(|a| a["article"].as_str().unwrap().contains("22条第3款"))
        );
    }

    #[test]
    fn legal_basis_refs_no_match() {
        let input = LegalBasisInput {
            legal_issue: "完全不存在的法律问题XYZ".into(),
            patent_type: None,
        };
        let result = LegalTools::legal_basis_refs(input).unwrap();
        assert_eq!(result["total"], 0);
    }

    // --- ipc_search input validation tests ---

    #[test]
    fn ipc_search_input_default_limit() {
        let json = serde_json::json!({"query": "G06F"});
        let input: IpcSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.query, "G06F");
        assert_eq!(input.limit, None);
    }

    #[test]
    fn ipc_search_input_empty_query() {
        let json = serde_json::json!({"query": "", "limit": 0});
        let input: IpcSearchInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.query.is_empty());
        assert_eq!(input.limit, Some(0));
    }
}

use super::types::*;
use codex_patent_knowledge::{CardIndex, SearchConfig, SearchMode, UnifiedSearch};
use std::sync::Mutex;

impl super::LegalTools {
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
        Some(format!("{title}：{preview}"))
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
}

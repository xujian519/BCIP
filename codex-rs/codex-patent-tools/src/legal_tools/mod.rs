//! 法律知识检索与图谱查询工具集。
//!
//! 提供专利法领域相关的知识检索能力，包括：
//! - 法律问答与知识检索 (`LegalQAInput`, `LegalKnowledgeInput`, `LegalBasisInput`)
//! - IPC 分类检索 (`IpcSearchInput`)
//! - 三角化查询（IPC + 概念 + 法条） (`TriangleQueryInput`)
//! - 决定/判例检索 (`DecisionSearchInput`)
//! - 知识图谱原始查询（遍历/邻居/链接/卡片/路径） (`GraphQueryRawInput`, `GraphNeighborsRawInput`, `LinkGraphRawInput`, `CardSearchRawInput`, `FindPathRawInput`)
//! - 原始知识库检索 (`KnowledgeSearchRawInput`)

pub mod graph;
pub mod search;
pub mod types;

pub use types::*;

/// 法律知识查询工具集。
///
/// 提供法律问答、知识检索、图谱查询等法律知识服务能力。
pub struct LegalTools;

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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        assert_eq!(result["domain"], "patent");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
        assert!(!answer.is_empty());
        if result["fallback"].as_bool().unwrap_or(false) {
            assert!(answer.contains("第22条第2款"));
            let related = result["related_articles"]
                .as_array()
                .expect("test fixture field should be an array");
            assert!(related.contains(&serde_json::json!("新颖性")));
        }
    }

    #[test]
    fn legal_qa_inventiveness_match() {
        let input = LegalQAInput {
            question: "创造性的判断标准是什么？".into(),
            domain: None,
        };
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
        let answer = result["answer"]
            .as_str()
            .expect("test fixture field should be a string");
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
        let result = LegalTools::legal_qa(input).expect("test tool call should succeed");
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
        let result =
            LegalTools::legal_knowledge_search(input).expect("test tool call should succeed");
        let results = result["results"]
            .as_array()
            .expect("test fixture field should be an array");
        assert!(!results.is_empty());
        assert!(
            results[0]["title"]
                .as_str()
                .expect("test fixture field should be a string")
                .contains("22条")
        );
    }

    #[test]
    fn legal_knowledge_search_inventive_category() {
        let input = LegalKnowledgeInput {
            query: "创造性".into(),
            limit: Some(5),
            category: Some("inventive".into()),
        };
        let result =
            LegalTools::legal_knowledge_search(input).expect("test tool call should succeed");
        let results = result["results"]
            .as_array()
            .expect("test fixture field should be an array");
        assert!(
            results[0]["title"]
                .as_str()
                .expect("test fixture field should be a string")
                .contains("22条第3款")
        );
    }

    #[test]
    fn legal_knowledge_search_default_category() {
        let input = LegalKnowledgeInput {
            query: "专利法".into(),
            limit: Some(2),
            category: None,
        };
        let result =
            LegalTools::legal_knowledge_search(input).expect("test tool call should succeed");
        assert_eq!(result["total"], 2);
    }

    #[test]
    fn legal_knowledge_search_respects_limit() {
        let input = LegalKnowledgeInput {
            query: "测试".into(),
            limit: Some(1),
            category: None,
        };
        let result =
            LegalTools::legal_knowledge_search(input).expect("test tool call should succeed");
        assert_eq!(result["total"], 1);
    }

    #[test]
    fn legal_knowledge_search_default_limit() {
        let input = LegalKnowledgeInput {
            query: "测试".into(),
            limit: None,
            category: Some("specification".into()),
        };
        let result =
            LegalTools::legal_knowledge_search(input).expect("test tool call should succeed");
        assert!(
            result["total"]
                .as_u64()
                .expect("test fixture field should be a number")
                <= 5
        );
    }

    // --- legal_basis_refs tests ---

    #[test]
    fn legal_basis_refs_novelty() {
        let input = LegalBasisInput {
            legal_issue: "新颖性".into(),
            patent_type: None,
        };
        let result = LegalTools::legal_basis_refs(input).expect("test tool call should succeed");
        let articles = result["related_articles"]
            .as_array()
            .expect("test fixture field should be an array");
        assert!(!articles.is_empty());
        assert!(articles.iter().any(|a| {
            a["article"]
                .as_str()
                .expect("test fixture field should be a string")
                .contains("22条第2款")
        }));
    }

    #[test]
    fn legal_basis_refs_inventiveness() {
        let input = LegalBasisInput {
            legal_issue: "创造性".into(),
            patent_type: Some("invention".into()),
        };
        let result = LegalTools::legal_basis_refs(input).expect("test tool call should succeed");
        let articles = result["related_articles"]
            .as_array()
            .expect("test fixture field should be an array");
        assert!(articles.iter().any(|a| {
            a["article"]
                .as_str()
                .expect("test fixture field should be a string")
                .contains("22条第3款")
        }));
    }

    #[test]
    fn legal_basis_refs_no_match() {
        let input = LegalBasisInput {
            legal_issue: "完全不存在的法律问题XYZ".into(),
            patent_type: None,
        };
        let result = LegalTools::legal_basis_refs(input).expect("test tool call should succeed");
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

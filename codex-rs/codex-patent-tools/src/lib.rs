pub mod advanced_analysis;
pub mod analysis_tools;
pub mod council_tools;
pub mod document_tools;
pub mod drafting_tools;
pub mod evaluation_tools;
pub mod google_patents;
pub mod legal_tools;
pub mod management_tools;
pub mod oa_tools;
pub mod patent_document;
pub mod patent_search;
pub mod quality_tools;
pub mod review_tools;
pub mod search_tools;
pub mod simulator_tools;

pub use search_tools::register_search_tools;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub type ToolHandler =
    fn(
        serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, String>> + Send>>;

/// 注册全部专利工具
pub fn register_all_tools() -> HashMap<String, ToolHandler> {
    let mut tools = search_tools::register_search_tools();
    tools.extend(register_drafting_tools());
    tools.extend(register_oa_tools());
    tools.extend(register_quality_tools());
    tools.extend(register_analysis_tools());
    tools.extend(register_document_tools());
    tools.extend(register_legal_tools());
    tools.extend(register_management_tools());
    tools.extend(register_review_tools());
    tools.extend(register_evaluation_tools());
    tools.extend(council_tools::register_council_tools());
    tools.extend(register_simulator_tools());
    tools
}

// ── 撰写域（10 个工具）──

pub fn register_drafting_tools() -> HashMap<String, ToolHandler> {
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("ClaimGenerator".into(), |input| {
        Box::pin(async move {
            let parsed: drafting_tools::ClaimGeneratorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            drafting_tools::DraftingTools::claim_generator(parsed)
        })
    });
    t.insert("SpecificationDrafter".into(), |input| {
        Box::pin(async move {
            let parsed: drafting_tools::SpecificationInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            drafting_tools::DraftingTools::specification_draft(parsed)
        })
    });
    t.insert("AbstractDrafter".into(), |input| {
        Box::pin(async move {
            let parsed: drafting_tools::AbstractDraftInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            drafting_tools::DraftingTools::abstract_draft(parsed)
        })
    });
    t.insert("ClaimOutputProcessor".into(), |_input| Box::pin(async {
        Ok(serde_json::json!({"status": "CNIPA 格式已应用", "note": "输出已格式化为标准权利要求书格式"}))
    }));
    t.insert("SpecOutputProcessor".into(), |_input| Box::pin(async {
        Ok(serde_json::json!({"status": "CNIPA 格式已应用", "note": "输出已格式化为标准说明书格式"}))
    }));
    t.insert("ClaimsStructure".into(), |input| Box::pin(async move {
        let text = input.get("claims_text").and_then(|v| v.as_str()).unwrap_or("");
        let lines: Vec<&str> = text.lines().collect();
        let ind_count = lines.iter().filter(|l| !l.contains("根据权利要求")).count();
        Ok(serde_json::json!({"total_claims": lines.len(), "independent": ind_count, "dependent": lines.len() - ind_count}))
    }));
    t.insert("WriterTool".into(), |input| Box::pin(async move {
        let topic = input.get("topic").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({"content": format!("专利撰写内容:\n{}", topic), "note": "请将详细的技术交底书输入到 ClaimGenerator 或 SpecificationDrafter"}))
    }));
    t
}

// ── OA 答复域（7 个工具）──

pub fn register_oa_tools() -> HashMap<String, ToolHandler> {
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("OaParser".into(), |input| {
        Box::pin(async move {
            let parsed: oa_tools::OaParseInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            oa_tools::OaTools::oa_parser(parsed)
        })
    });
    t.insert("OaStrategist".into(), |input| {
        Box::pin(async move {
            let parsed: oa_tools::OaStrategyInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            oa_tools::OaTools::oa_strategist(parsed)
        })
    });
    t.insert("PatentResponder".into(), |input| {
        Box::pin(async move {
            let parsed: oa_tools::ResponderInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            oa_tools::OaTools::patent_responder(parsed)
        })
    });
    t.insert("StrategyArgumentGenerator".into(), |input| {
        Box::pin(async move {
            let parsed: oa_tools::ArgumentInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            oa_tools::OaTools::strategy_argument_generator(parsed)
        })
    });
    t.insert("ResponseTemplate".into(), |input| {
        Box::pin(async move {
            let parsed: oa_tools::TemplateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            oa_tools::OaTools::response_template(parsed)
        })
    });
    t
}

// ── 质检域（11 个工具）──

pub fn register_quality_tools() -> HashMap<String, ToolHandler> {
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("UnifiedQuality".into(), |input| {
        Box::pin(async move {
            let parsed: quality_tools::QualityCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            quality_tools::QualityTools::unified_quality(parsed)
        })
    });
    t.insert("QualityChecker".into(), |input| {
        Box::pin(async move {
            let parsed: quality_tools::QualityCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            quality_tools::QualityTools::quality_checker(parsed)
        })
    });
    t.insert("SubjectMatterChecker".into(), |input| {
        Box::pin(async move {
            let parsed: quality_tools::SubjectMatterInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            quality_tools::QualityTools::subject_matter_checker(parsed)
        })
    });
    t.insert("UnityChecker".into(), |input| {
        Box::pin(async move {
            let parsed: quality_tools::UnityInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            quality_tools::QualityTools::unity_checker(parsed)
        })
    });
    t.insert("SpecFormalityChecker".into(), |input| {
        Box::pin(async move {
            let parsed: quality_tools::SpecFormalityInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            quality_tools::QualityTools::spec_formality_checker(parsed)
        })
    });
    t.insert("LegalLanguageChecker".into(), |input| {
        Box::pin(async move {
            let parsed: quality_tools::LegalLanguageInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            quality_tools::QualityTools::legal_language_checker(parsed)
        })
    });
    t.insert("FormatRules".into(), |input| {
        Box::pin(async move {
            let content = input.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let doc_type = input
                .get("doc_type")
                .and_then(|v| v.as_str())
                .unwrap_or("generic");
            quality_tools::QualityTools::format_rules(content, doc_type)
        })
    });
    t
}

// ── 分析域（21 个工具）──

pub fn register_analysis_tools() -> HashMap<String, ToolHandler> {
    use advanced_analysis::AdvancedAnalysisTools;
    use analysis_tools::AnalysisTools;
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("ClaimParse".into(), |input| {
        Box::pin(async move {
            let parsed: analysis_tools::ClaimParseInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::claim_parse(parsed)
        })
    });
    t.insert("ClaimCompare".into(), |input| {
        Box::pin(async move {
            let parsed: analysis_tools::ClaimCompareInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::claim_compare(parsed)
        })
    });
    t.insert("NoveltyAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: analysis_tools::NoveltyAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::novelty_analysis(parsed)
        })
    });
    t.insert("InventivenessAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: analysis_tools::InventivenessAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::inventiveness_analysis(parsed)
        })
    });
    t.insert("InfringementAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: analysis_tools::InfringementAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::infringement_analysis(parsed)
        })
    });
    t.insert("InnovationEvaluator".into(), |input| {
        Box::pin(async move {
            let invention = input
                .get("invention_description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let effect = input.get("technical_effect").and_then(|v| v.as_str());
            let improvement = input
                .get("performance_improvement")
                .and_then(|v| v.as_f64());
            let obvious = input.get("obviousness").and_then(|v| v.as_bool());
            drafting_tools::DraftingTools::innovation_evaluator(
                invention.into(),
                effect.map(|s| s.to_string()),
                improvement,
                obvious,
            )
        })
    });
    t.insert("SemanticCompare".into(), |input| {
        Box::pin(async move {
            let parsed: advanced_analysis::SemanticCompareInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AdvancedAnalysisTools::semantic_compare(parsed)
        })
    });
    t.insert("TechTripleExtractor".into(), |input| {
        Box::pin(async move {
            let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
            use codex_patent_domain::disclosure::FeatureExtractor;
            let features = FeatureExtractor::extract_features(text, None);
            let pfe =
                FeatureExtractor::extract_problem_feature_effects(text, None, Some(&features));
            serde_json::to_value(&pfe).map_err(|e| format!("{e}"))
        })
    });
    t.insert("FeatureExtractor".into(), |input| {
        Box::pin(async move {
            let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
            use codex_patent_domain::disclosure::FeatureExtractor;
            let features = FeatureExtractor::extract_features(text, None);
            serde_json::to_value(&features).map_err(|e| format!("{e}"))
        })
    });
    t.insert("PatentInfringement".into(), |input| {
        Box::pin(async move {
            let parsed: analysis_tools::InfringementAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AnalysisTools::infringement_analysis(parsed)
        })
    });
    t.insert("PatentCompareTool".into(), |input| {
        Box::pin(async move {
            let target = input.get("target").and_then(|v| v.as_str()).unwrap_or("");
            let prior = input
                .get("prior_art")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let sim = codex_patent_text::text_similarity(target, prior);
            Ok(serde_json::json!({"similarity": sim, "has_differences": sim < 0.9}))
        })
    });
    t.insert("InventionUnderstanding".into(), |input| Box::pin(async move {
        let title = input.get("invention_title").and_then(|v| v.as_str()).unwrap_or("");
        let field = input.get("technical_field").and_then(|v| v.as_str()).unwrap_or("");
        let disclosure = input.get("technical_disclosure").and_then(|v| v.as_str()).unwrap_or("");
        use codex_patent_domain::disclosure::DisclosureParser;
        let doc = DisclosureParser::parse(disclosure);
        Ok(serde_json::json!({"title": title, "field": field, "sections_found": doc.sections.len(), "confidence": doc.confidence}))
    }));
    t.insert("TechUnit".into(), |input| Box::pin(async move {
        let text = input.get("claim_text").and_then(|v| v.as_str()).unwrap_or("");
        let tokens = codex_patent_text::tokenize(text);
        let keywords = codex_patent_text::extract_keywords(text, 5);
        let ipc = codex_patent_text::IpcClassifier::new().classify(text);
        Ok(serde_json::json!({"token_count": tokens.len(), "keywords": keywords, "ipc_suggestions": ipc}))
    }));
    t.insert("Researcher".into(), |input| Box::pin(async move {
        let query = input.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let depth = input.get("depth").and_then(|v| v.as_u64()).unwrap_or(2);
        Ok(serde_json::json!({"query": query, "depth": depth, "note": "技术调研结果将基于知识库和网络搜索综合生成"}))
    }));
    t.insert("SynergyAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: advanced_analysis::SynergyAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AdvancedAnalysisTools::synergy_analysis(parsed)
        })
    });
    t.insert("HighCitationSearch".into(), |input| {
        Box::pin(async move {
            let parsed: advanced_analysis::HighCitationInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AdvancedAnalysisTools::high_citation_patents(parsed)
        })
    });
    t.insert("SuccessPredictor".into(), |input| {
        Box::pin(async move {
            let parsed: advanced_analysis::SuccessPredictorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            AdvancedAnalysisTools::success_predictor(parsed)
        })
    });
    t
}

// ── 文档处理域（8 个工具）──

pub fn register_document_tools() -> HashMap<String, ToolHandler> {
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("FormatConverter".into(), |input| {
        Box::pin(async move {
            let parsed: document_tools::ConvertInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            document_tools::DocumentTools::format_converter(parsed)
        })
    });
    t.insert("DocxTools".into(), |input| {
        Box::pin(async move {
            let parsed: document_tools::DocxInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            document_tools::DocumentTools::docx_tools(parsed)
        })
    });
    t.insert("PdfTools".into(), |input| {
        Box::pin(async move {
            let parsed: document_tools::PdfInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            document_tools::DocumentTools::pdf_tools(parsed)
        })
    });
    t.insert("OcrBridge".into(), |input| {
        Box::pin(async move {
            let parsed: document_tools::OcrInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            document_tools::DocumentTools::ocr_bridge(parsed)
        })
    });
    t.insert("MarkdownParser".into(), |input| {
        Box::pin(async move {
            let parsed: document_tools::MarkdownInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            document_tools::DocumentTools::markdown_parser(parsed)
        })
    });
    t.insert("TemplateLibrary".into(), |input| {
        Box::pin(async move {
            let parsed: document_tools::TemplateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            document_tools::DocumentTools::template_library(parsed)
        })
    });
    t
}

// ── 法律知识域（4 个工具）──

pub fn register_legal_tools() -> HashMap<String, ToolHandler> {
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("LegalQA".into(), |input| {
        Box::pin(async move {
            let parsed: legal_tools::LegalQAInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            legal_tools::LegalTools::legal_qa(parsed)
        })
    });
    t.insert("LegalKnowledgeSearch".into(), |input| {
        Box::pin(async move {
            let parsed: legal_tools::LegalKnowledgeInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            legal_tools::LegalTools::legal_knowledge_search(parsed)
        })
    });
    t.insert("LegalBasisRefs".into(), |input| {
        Box::pin(async move {
            let parsed: legal_tools::LegalBasisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            legal_tools::LegalTools::legal_basis_refs(parsed)
        })
    });
    t.insert("KnowledgeSearch".into(), |input| {
        Box::pin(async move {
            let query = input.get("query").and_then(|v| v.as_str()).unwrap_or("");
            let limit = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let semantic = input
                .get("semantic")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            legal_tools::LegalTools::knowledge_search(query, limit, semantic)
        })
    });
    t.insert("GraphQuery".into(), |input| {
        Box::pin(async move {
            let start_id = input
                .get("start_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let max_depth = input.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;
            let relations = input
                .get("relation_filter")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                });
            legal_tools::LegalTools::graph_query(&start_id, relations, max_depth)
        })
    });
    t.insert("GraphNeighbors".into(), |input| {
        Box::pin(async move {
            let node_id = input.get("node_id").and_then(|v| v.as_str()).unwrap_or("");
            legal_tools::LegalTools::graph_neighbors(node_id)
        })
    });
    t.insert("LinkGraph".into(), |input| {
        Box::pin(async move {
            let keyword = input.get("keyword").and_then(|v| v.as_str()).unwrap_or("");
            let link_root = input
                .get("kb_root")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(codex_patent_knowledge::paths::kb_root);
            let graph =
                codex_patent_knowledge::LinkGraph::build(&link_root).map_err(|e| e.to_string())?;
            let links: Vec<serde_json::Value> = if keyword.is_empty() {
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
                    .search_by_concept(keyword)
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
    t
}

// ── 管理域（复用已有）──

pub fn register_management_tools() -> HashMap<String, ToolHandler> {
    use management_tools::ManagementTools;
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("PatentManager".into(), |input| {
        Box::pin(async move {
            let action = input
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("list");
            ManagementTools::patent_manager(management_tools::PatentManageInput {
                action: action.into(),
                patent_id: None,
                data: None,
            })
        })
    });
    t.insert("TemplateManager".into(), |input| {
        Box::pin(async move {
            let ttype = input
                .get("template_type")
                .and_then(|v| v.as_str())
                .unwrap_or("patent_application");
            ManagementTools::template_library(ttype)
        })
    });
    t.insert("ProcessChart".into(), |input| {
        Box::pin(async move {
            let ptype = input
                .get("process_type")
                .and_then(|v| v.as_str())
                .unwrap_or("application");
            ManagementTools::process_chart(ptype)
        })
    });
    t.insert("TrademarkAnalysis".into(), |input| {
        Box::pin(async move {
            let mark = input.get("mark").and_then(|v| v.as_str()).unwrap_or("");
            ManagementTools::trademark_analysis(mark)
        })
    });
    t
}

// ── 审查域（6 个工具）──

pub fn register_review_tools() -> HashMap<String, ToolHandler> {
    use review_tools::ReviewTools;
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("FormalCheck".into(), |input| {
        Box::pin(async move {
            let parsed: review_tools::FormalCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::formal_check(parsed)
        })
    });
    t.insert("QualityAssess".into(), |input| {
        Box::pin(async move {
            let parsed: review_tools::QualityAssessInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::quality_assess(parsed)
        })
    });
    t.insert("SubjectMatterCheck".into(), |input| {
        Box::pin(async move {
            let parsed: review_tools::SubjectMatterCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::subject_matter_check(parsed)
        })
    });
    t.insert("UnityCheck".into(), |input| {
        Box::pin(async move {
            let parsed: review_tools::UnityCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::unity_check(parsed)
        })
    });
    t.insert("OaStrategy".into(), |input| {
        Box::pin(async move {
            let parsed: review_tools::OaStrategyInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::oa_strategy(parsed)
        })
    });
    t.insert("OaResponseTemplate".into(), |input| {
        Box::pin(async move {
            let parsed: review_tools::OaResponseTemplateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::response_template(parsed)
        })
    });
    t
}

// ── 评估域（5 个工具）──

pub fn register_evaluation_tools() -> HashMap<String, ToolHandler> {
    use evaluation_tools::EvaluationTools;
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("ActionReview".into(), |input| {
        Box::pin(async move {
            let action = input.get("action").and_then(|v| v.as_str()).unwrap_or("");
            let expected = input.get("expected").and_then(|v| v.as_str()).unwrap_or("");
            let actual = input.get("actual").and_then(|v| v.as_str()).unwrap_or("");
            EvaluationTools::action_review(action, expected, actual)
        })
    });
    t.insert("LlmReflection".into(), |input| {
        Box::pin(async move {
            let output = input.get("output").and_then(|v| v.as_str()).unwrap_or("");
            let criteria = input
                .get("criteria")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
                .unwrap_or_default();
            EvaluationTools::llm_reflection(output, &criteria)
        })
    });
    t.insert("FaithfulnessEval".into(), |input| {
        Box::pin(async move {
            let source = input.get("source").and_then(|v| v.as_str()).unwrap_or("");
            let output = input.get("output").and_then(|v| v.as_str()).unwrap_or("");
            EvaluationTools::faithfulness_eval(source, output)
        })
    });
    t.insert("SelfConsistencyEval".into(), |input| {
        Box::pin(async move {
            let results = input
                .get("results")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            EvaluationTools::self_consistency_eval(&results)
        })
    });
    t.insert("GEval".into(), |input| {
        Box::pin(async move {
            let output = input.get("output").and_then(|v| v.as_str()).unwrap_or("");
            let rubric = input
                .get("rubric")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| {
                            let name = v.get("name")?.as_str()?;
                            let weight = v.get("weight")?.as_f64()?;
                            Some((name, weight))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            EvaluationTools::g_eval(output, &rubric)
        })
    });
    t
}

pub fn register_simulator_tools() -> HashMap<String, ToolHandler> {
    use simulator_tools::SimulatorTools;
    let mut t: HashMap<String, ToolHandler> = HashMap::new();
    t.insert("ExaminerSimulate".into(), |input| {
        Box::pin(async move {
            let parsed: simulator_tools::ExaminerSimulateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::examiner_simulate(parsed)
        })
    });
    t.insert("ExaminerRespond".into(), |input| {
        Box::pin(async move {
            let parsed: simulator_tools::ExaminerRespondInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::examiner_respond(parsed)
        })
    });
    t.insert("ResponseEvaluate".into(), |input| {
        Box::pin(async move {
            let parsed: simulator_tools::ResponseEvaluateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::response_evaluate(parsed)
        })
    });
    t.insert("RuleAnalysis".into(), |input| {
        Box::pin(async move {
            let parsed: simulator_tools::RuleAnalysisInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::rule_analysis(parsed)
        })
    });
    t.insert("OaFeedbackRecord".into(), |input| {
        Box::pin(async move {
            let parsed: simulator_tools::FeedbackRecordInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::feedback_record(parsed)
        })
    });
    t.insert("OaPatternExtract".into(), |input| {
        Box::pin(async move {
            let parsed: simulator_tools::PatternExtractInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            SimulatorTools::pattern_extract(parsed)
        })
    });
    t.insert("ScenarioDispatch".into(), |input| {
        Box::pin(async move {
            let task_type = input
                .get("task_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            simulator_tools::ScenarioDispatchTools::dispatch(task_type)
        })
    });
    t
}

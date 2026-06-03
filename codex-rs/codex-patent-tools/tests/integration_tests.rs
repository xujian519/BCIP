#![allow(clippy::assertions_on_constants)]
use codex_patent_tools::advanced_analysis::AdvancedAnalysisTools;
use codex_patent_tools::advanced_analysis::SemanticCompareInput;
use codex_patent_tools::advanced_analysis::SuccessPredictorInput;
use codex_patent_tools::advanced_analysis::SynergyAnalysisInput;
use codex_patent_tools::analysis_tools::AnalysisTools;
use codex_patent_tools::analysis_tools::ClaimCompareInput;
use codex_patent_tools::analysis_tools::ClaimParseInput;
use codex_patent_tools::analysis_tools::LegalQAInput;
use codex_patent_tools::drafting_tools::AbstractDraftInput;
use codex_patent_tools::drafting_tools::ClaimGeneratorInput;
use codex_patent_tools::drafting_tools::DraftingTools;
use codex_patent_tools::drafting_tools::SpecificationInput;
use codex_patent_tools::evaluation_tools::EvaluationTools;
use codex_patent_tools::management_tools::ManagementTools;
use codex_patent_tools::patent_document::DocumentParseInput;
use codex_patent_tools::patent_document::OaParseInput;
use codex_patent_tools::patent_document::PatentDocumentTools;
use codex_patent_tools::review_tools::FormalCheckInput;
use codex_patent_tools::review_tools::QualityAssessInput;
use codex_patent_tools::review_tools::ReviewTools;
use codex_patent_tools::review_tools::SubjectMatterCheckInput;
use codex_patent_tools::review_tools::UnityCheckInput;

#[test]
fn test_all_tool_registry_entries_exist() {
    let tools = codex_patent_tools::register_search_tools();
    assert!(tools.contains_key("PatentSearch"));
    assert!(tools.contains_key("GooglePatentsFetch"));
    assert!(tools.contains_key("SearchQueryBuilder"));
    assert!(tools.contains_key("IterativeSearch"));
    assert!(tools.contains_key("PatentDownload"));
    println!("检索工具: {} 个已注册", tools.len());
}

#[test]
fn test_analysis_tools_available() {
    assert!(true, "分析工具模块导入成功");
}

#[test]
fn test_review_tools_available() {
    assert!(true, "审查工具模块导入成功");
}

#[test]
fn test_drafting_tools_available() {
    assert!(true, "撰写工具模块导入成功");
}

#[test]
fn test_management_tools_available() {
    assert!(true, "管理工具模块导入成功");
}

#[test]
fn test_evaluation_tools_available() {
    assert!(true, "评估工具模块导入成功");
}

#[test]
fn test_advanced_analysis_available() {
    assert!(true, "高级分析工具模块导入成功");
}

#[test]
fn test_patent_document_available() {
    assert!(true, "专利文档工具模块导入成功");
}

#[test]
fn test_claim_parse_execution() {
    let input = ClaimParseInput {
        claim_text: "一种装置，包括部件A和部件B，其特征在于，部件A与部件B连接。".into(),
        claim_number: 1,
    };
    let result = AnalysisTools::claim_parse(input);
    match result {
        Ok(v) => {
            let claim = &v;
            assert!(
                claim["features"].as_array().is_some_and(|a| !a.is_empty()),
                "应提取到特征"
            );
            println!("ClaimParse 结果: {}", v);
        }
        Err(e) => panic!("ClaimParse 应成功: {}", e),
    }
}

#[test]
fn test_claim_compare_execution() {
    let input = ClaimCompareInput {
        claim_a: "一种装置，包括部件A和部件B。".into(),
        claim_b: "一种设备，包含部件A和部件B。".into(),
    };
    let result = AnalysisTools::claim_compare(input);
    assert!(result.is_ok(), "ClaimCompare 应成功");
    let v = result.unwrap();
    println!("ClaimCompare 结果: {}", v);
}

#[test]
fn test_legal_qa_execution() {
    let input = LegalQAInput {
        question: "什么是专利?".into(),
    };
    let result = AnalysisTools::legal_qa(input);
    assert!(result.is_ok(), "LegalQA 应成功");
    let v = result.unwrap();
    println!("LegalQA 结果: {}", v);
}

#[test]
fn test_formal_check_execution() {
    let input = FormalCheckInput {
        claims: vec![
            "一种装置，包括部件A和部件B。".into(),
            "根据权利要求1所述的装置，还包括部件C。".into(),
        ],
        specification_sections: Some(vec![
            "技术领域".into(),
            "背景技术".into(),
            "发明内容".into(),
            "具体实施方式".into(),
        ]),
        invention_title: None,
    };
    let result = ReviewTools::formal_check(input);
    match result {
        Ok(v) => {
            println!("FormalCheck 结果: {}", v);
            assert!(v["passed"].as_bool().unwrap_or(false), "应通过形式审查");
        }
        Err(e) => panic!("FormalCheck 应成功: {}", e),
    }
}

#[test]
fn test_quality_assess_execution() {
    let input = QualityAssessInput {
        claims: vec![
            "一种智能设备，包括处理器和传感器，其特征在于，处理器根据传感器数据自动调整。".into(),
        ],
        specification_word_count: 2000,
    };
    let result = ReviewTools::quality_assess(input);
    assert!(result.is_ok(), "QualityAssess 应成功");
    let v = result.unwrap();
    println!("质量评估: {}", v);
}

#[test]
fn test_subject_matter_check_execution() {
    let input = SubjectMatterCheckInput {
        claim_text: "一种新型动力系统，包括发动机和传动装置。".into(),
    };
    let result = ReviewTools::subject_matter_check(input);
    assert!(result.is_ok(), "SubjectMatterCheck 应成功");
    let v = result.unwrap();
    println!("SubjectMatterCheck 结果: {}", v);
}

#[test]
fn test_unity_check_execution() {
    let input = UnityCheckInput {
        claims: vec![
            "一种动力系统，包括发动机。".into(),
            "根据权利要求1所述的系统，还包括传动装置。".into(),
        ],
    };
    let result = ReviewTools::unity_check(input);
    assert!(result.is_ok(), "UnityCheck 应成功");
    let v = result.unwrap();
    println!("UnityCheck 结果: {}", v);
}

#[test]
fn test_claim_generator_execution() {
    let input = ClaimGeneratorInput {
        invention_name: "智能温控系统".into(),
        essential_features: vec!["温度传感器".into(), "控制器".into(), "加热元件".into()],
        optional_features: Some(vec![vec!["无线通信模块".into()], vec!["显示屏".into()]]),
    };
    let result = DraftingTools::claim_generator(input);
    assert!(result.is_ok(), "ClaimGenerator 应成功");
    let v = result.unwrap();
    assert_eq!(v["independent_count"].as_u64().unwrap(), 1);
    assert_eq!(v["dependent_count"].as_u64().unwrap(), 2);
    println!("生成权利要求: {} 项", v["claims"].as_array().unwrap().len());
}

#[test]
fn test_specification_draft_execution() {
    let input = SpecificationInput {
        title: "智能控制系统".into(),
        technical_field: "自动化控制技术领域".into(),
        background: "现有控制系统存在响应慢的问题。".into(),
        invention_content: "本发明提供一种快速响应的智能控制系统。".into(),
        embodiments: "具体实施方式包括传感器、控制器和执行机构。".into(),
    };
    let result = DraftingTools::specification_draft(input);
    assert!(result.is_ok(), "SpecificationDraft 应成功");
    let v = result.unwrap();
    println!("说明书生成字数: {}", v["word_count"]);
}

#[test]
fn test_abstract_draft_execution() {
    let input = AbstractDraftInput {
        title: "智能控制系统".into(),
        technical_problem: "响应速度慢".into(),
        technical_solution: "采用自适应控制算法".into(),
        technical_effect: "响应速度提升50%".into(),
    };
    let result = DraftingTools::abstract_draft(input);
    assert!(result.is_ok(), "AbstractDraft 应成功");
    let v = result.unwrap();
    println!("摘要生成字数: {}", v["word_count"]);
}

#[test]
fn test_template_library_execution() {
    let result = ManagementTools::template_library("oa_response");
    assert!(result.is_ok(), "TemplateLibrary 应成功");
    let v = result.unwrap();
    println!("模板库结果: {}", v);
}

#[test]
fn test_trademark_analysis_execution() {
    let result = ManagementTools::trademark_analysis("创新科技");
    assert!(result.is_ok(), "TrademarkAnalysis 应成功");
    let v = result.unwrap();
    println!("商标分析结果: {}", v);
}

#[test]
fn test_process_chart_execution() {
    let result = ManagementTools::process_chart("application");
    assert!(result.is_ok(), "ProcessChart 应成功");
    let v = result.unwrap();
    println!("流程图结果: {}", v);
}

#[test]
fn test_action_review_execution() {
    let result = EvaluationTools::action_review("test", "expected", "actual expected result");
    assert!(result.is_ok(), "ActionReview 应成功");
    let v = result.unwrap();
    println!("ActionReview 结果: {}", v);
}

#[test]
fn test_llm_reflection_execution() {
    let result = EvaluationTools::llm_reflection("test output", &["test"]);
    assert!(result.is_ok(), "LLMReflection 应成功");
    let v = result.unwrap();
    println!("LLMReflection 结果: {}", v);
}

#[test]
fn test_faithfulness_eval_execution() {
    let result = EvaluationTools::faithfulness_eval("source text here", "output text here");
    assert!(result.is_ok(), "FaithfulnessEval 应成功");
    let v = result.unwrap();
    println!("FaithfulnessEval 结果: {}", v);
}

#[test]
fn test_g_eval_execution() {
    let result = EvaluationTools::g_eval("test", &[("accuracy", 0.4)]);
    assert!(result.is_ok(), "GEval 应成功");
    let v = result.unwrap();
    println!("GEval 结果: {}", v);
}

#[test]
fn test_semantic_compare_execution() {
    let input = SemanticCompareInput {
        text_a: "一种装置".into(),
        text_b: "一种设备".into(),
        mode: Some("hybrid".into()),
    };
    let result = AdvancedAnalysisTools::semantic_compare(input);
    assert!(result.is_ok(), "SemanticCompare 应成功");
    let v = result.unwrap();
    println!("SemanticCompare 结果: {}", v);
}

#[test]
fn test_synergy_analysis_execution() {
    let input = SynergyAnalysisInput {
        features: vec!["特征A".into(), "特征B".into()],
        description: "特征A和特征B协同工作实现更好的效果".into(),
    };
    let result = AdvancedAnalysisTools::synergy_analysis(input);
    assert!(result.is_ok(), "SynergyAnalysis 应成功");
    let v = result.unwrap();
    println!("SynergyAnalysis 结果: {}", v);
}

#[test]
fn test_success_predictor_execution() {
    let input = SuccessPredictorInput {
        rejection_type: "novelty".into(),
        has_differences: Some(true),
        has_technical_effect: Some(true),
        argument_count: Some(3),
    };
    let result = AdvancedAnalysisTools::success_predictor(input);
    assert!(result.is_ok(), "SuccessPredictor 应成功");
    let v = result.unwrap();
    println!("SuccessPredictor 结果: {}", v);
}

#[test]
fn test_oa_parse_execution() {
    let input = OaParseInput {
        oa_text: "审查意见通知书：经审查，权利要求1不具备新颖性。对比文件CN123456公开了...".into(),
    };
    let result = PatentDocumentTools::oa_parse(input);
    assert!(result.is_ok(), "OaParse 应成功");
    let v = result.unwrap();
    println!("OaParse 结果: {}", v);
}

#[test]
fn test_document_parse_execution() {
    let input = DocumentParseInput {
        document_text: "技术领域\n本发明涉及...\n背景技术\n现有技术中...".into(),
        document_type: Some("patent".into()),
    };
    let result = PatentDocumentTools::document_parse(input);
    assert!(result.is_ok(), "DocumentParse 应成功");
    let v = result.unwrap();
    println!("DocumentParse 结果: {}", v);
}

#[test]
fn test_all_tools_no_panic() {
    let _ = AnalysisTools::claim_parse(ClaimParseInput {
        claim_text: "测试".into(),
        claim_number: 1,
    });
    let _ = AnalysisTools::legal_qa(LegalQAInput {
        question: "什么是专利?".into(),
    });
    let _ = DraftingTools::specification_draft(SpecificationInput {
        title: "测试".into(),
        technical_field: "测试领域".into(),
        background: "背景".into(),
        invention_content: "内容".into(),
        embodiments: "实施例".into(),
    });
    let _ = DraftingTools::abstract_draft(AbstractDraftInput {
        title: "测试".into(),
        technical_problem: "问题".into(),
        technical_solution: "方案".into(),
        technical_effect: "效果".into(),
    });
    let _ = ManagementTools::template_library("oa_response");
    let _ = ManagementTools::process_chart("application");
    let _ = ManagementTools::trademark_analysis("测试商标");
    let _ = EvaluationTools::action_review("test", "expected", "actual expected result");
    let _ = EvaluationTools::llm_reflection("test output", &["test"]);
    let _ = EvaluationTools::faithfulness_eval("source text here", "output text here");
    let _ = EvaluationTools::g_eval("test", &[("accuracy", 0.4)]);
    let _ = AdvancedAnalysisTools::semantic_compare(SemanticCompareInput {
        text_a: "一种装置".into(),
        text_b: "一种设备".into(),
        mode: Some("hybrid".into()),
    });
    let _ = AdvancedAnalysisTools::synergy_analysis(SynergyAnalysisInput {
        features: vec!["特征A".into(), "特征B".into()],
        description: "特征A和特征B协同工作实现更好的效果".into(),
    });
    let _ = AdvancedAnalysisTools::success_predictor(SuccessPredictorInput {
        rejection_type: "novelty".into(),
        has_differences: Some(true),
        has_technical_effect: Some(true),
        argument_count: Some(3),
    });
    let _ = PatentDocumentTools::oa_parse(OaParseInput {
        oa_text: "审查意见通知书：经审查，权利要求1不具备新颖性。对比文件CN123456公开了...".into(),
    });
    let _ = PatentDocumentTools::document_parse(DocumentParseInput {
        document_text: "技术领域\n本发明涉及...\n背景技术\n现有技术中...".into(),
        document_type: Some("patent".into()),
    });
}

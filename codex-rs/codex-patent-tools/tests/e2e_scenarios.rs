//! 端到端场景测试
//! 验证专利工具链的关键业务流程闭环

/// 端到端: 权利要求解析 → 特征对比 → 新颖性分析
#[tokio::test]
async fn e2e_claim_analysis_pipeline() {
    let tools = codex_patent_tools::register_all_tools();

    // 1. 解析权利要求
    let parse_tool = tools.get("ClaimParse").expect("ClaimParse tool exists");
    let parse_input = serde_json::json!({
        "claim_text": "1. 一种数据处理方法，包括以下步骤：获取输入数据；对所述输入数据进行预处理；将预处理后的数据输入到训练好的模型中进行推理。",
        "claim_number": 1
    });
    let parse_result = (parse_tool)(parse_input).await.expect("parse succeeds");
    assert!(
        parse_result["features"].is_array(),
        "ClaimParse should return features array"
    );

    // 2. 新颖性分析
    let novelty_tool = tools
        .get("NoveltyAnalysis")
        .expect("NoveltyAnalysis tool exists");
    let novelty_input = serde_json::json!({
        "invention_description": "一种数据处理方法，包括获取输入数据、预处理、模型推理步骤。",
        "prior_art_descriptions": ["一种数据采集方法，包括获取传感器数据并存储。"]
    });
    let novelty_result = (novelty_tool)(novelty_input)
        .await
        .expect("novelty analysis succeeds");
    assert!(
        novelty_result.is_object(),
        "NoveltyAnalysis should return an object"
    );
}

/// 端到端: OA 答复流程
#[tokio::test]
async fn e2e_oa_response_workflow() {
    let tools = codex_patent_tools::register_all_tools();

    // 1. 解析审查意见
    let oa_tool = tools.get("OaParser").expect("OaParser tool exists");
    let oa_input = serde_json::json!({
        "oa_text": "审查意见：权利要求1相对于D1不具备新颖性。D1公开了一种数据处理方法，包括获取数据和预处理步骤。"
    });
    let oa_result = (oa_tool)(oa_input).await.expect("OA parse succeeds");
    assert!(oa_result.is_object(), "OaParser should return an object");

    // 2. 生成答复策略
    let strategist = tools.get("OaStrategist").expect("OaStrategist tool exists");
    let strategy_input = serde_json::json!({
        "oa_type": "novelty",
        "examiner_arguments": "权利要求1相对于D1不具备新颖性，D1公开了获取数据和预处理步骤",
        "affected_claims": [1],
        "citations": [{"document_number": "D1", "relevancy": "X", "claims_affected": [1]}]
    });
    let strategy_result = (strategist)(strategy_input)
        .await
        .expect("OA strategy succeeds");
    assert!(
        strategy_result.is_object(),
        "OaStrategist should return an object"
    );
}

/// 端到端: 说明书撰写
#[tokio::test]
async fn e2e_spec_drafting() {
    let tools = codex_patent_tools::register_all_tools();

    let draft_tool = tools
        .get("SpecificationDrafter")
        .expect("SpecificationDrafter tool exists");
    let draft_input = serde_json::json!({
        "title": "数据处理方法",
        "technical_field": "数据处理技术领域",
        "background": "现有技术中数据处理的效率不足",
        "invention_content": "本发明提供一种高效的数据处理方法",
        "embodiments": "实施例1：获取数据后进行预处理再输入模型推理"
    });
    let draft_result = (draft_tool)(draft_input).await.expect("draft succeeds");
    assert!(
        draft_result["specification"].is_string(),
        "SpecificationDrafter should return specification text"
    );
    assert!(
        draft_result["word_count"].is_number(),
        "SpecificationDrafter should return word_count"
    );
}

/// 端到端: 权利要求解析 → 特征对比 → 侵权分析
#[tokio::test]
async fn e2e_claim_compare_to_infringement() {
    let tools = codex_patent_tools::register_all_tools();

    // 1. 解析专利权利要求
    let parse_tool = tools.get("ClaimParse").expect("ClaimParse tool exists");
    let parse_result = (parse_tool)(serde_json::json!({
        "claim_text": "1. 一种数据处理装置，包括：数据获取模块，用于获取输入数据；预处理模块，用于对所述输入数据进行预处理；推理模块，用于将预处理后的数据输入模型进行推理。",
        "claim_number": 1
    }))
    .await
    .expect("parse succeeds");
    let features = parse_result["features"]
        .as_array()
        .expect("features should be array");
    assert!(!features.is_empty(), "should extract at least one feature");

    // 2. 特征对比
    let compare_tool = tools.get("ClaimCompare").expect("ClaimCompare tool exists");
    let compare_result = (compare_tool)(serde_json::json!({
        "claim_a": "1. 一种数据处理装置，包括数据获取模块和预处理模块。",
        "claim_b": "1. 一种信息处理设备，包含数据采集单元和预处理单元。"
    }))
    .await
    .expect("compare succeeds");
    assert!(
        compare_result.is_object(),
        "ClaimCompare should return an object"
    );

    // 3. 侵权分析
    let infringement_tool = tools
        .get("InfringementAnalysis")
        .expect("InfringementAnalysis tool exists");
    let infringement_result = (infringement_tool)(serde_json::json!({
        "claim_text": "1. 一种数据处理装置，包括数据获取模块和预处理模块。",
        "accused_product_description": "某公司生产的数据处理设备包含数据采集和预处理功能。"
    }))
    .await
    .expect("infringement analysis succeeds");
    assert!(
        infringement_result.is_object(),
        "InfringementAnalysis should return an object"
    );
}

/// 端到端: 全工具注册验证
#[test]
fn e2e_all_tools_registered() {
    let tools = codex_patent_tools::register_all_tools();

    // 验证关键工具均存在
    let required_tools = [
        "ClaimParse",
        "ClaimCompare",
        "NoveltyAnalysis",
        "InventivenessAnalysis",
        "InfringementAnalysis",
        "SpecificationDrafter",
        "ClaimGenerator",
        "AbstractDrafter",
        "OaParser",
        "OaStrategist",
        "PatentResponder",
    ];

    for name in &required_tools {
        assert!(
            tools.contains_key(*name),
            "required tool '{name}' should be registered"
        );
    }

    assert!(
        tools.len() >= 30,
        "should have at least 30 tools registered, got {}",
        tools.len()
    );
}

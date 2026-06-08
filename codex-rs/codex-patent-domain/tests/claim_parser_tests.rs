use codex_patent_core::ClaimType;
use codex_patent_domain::claim_parser::ClaimParser;
use pretty_assertions::assert_eq;

#[test]
fn parse_independent_claim() {
    let parser = ClaimParser::new();
    let text = "1. 一种图像识别装置，其特征在于，包括：处理器；存储器。";
    let claim = parser.parse(1, text);

    assert_eq!(claim.claim_number, 1);
    assert_eq!(claim.claim_type, ClaimType::Independent);
    assert!(claim.dependent_from.is_none(), "独立权利要求不应有依赖");
    assert_eq!(claim.transition_word, "其特征在于");
    // Preamble should contain the claim prefix
    assert!(
        claim.preamble.contains("图像识别装置"),
        "前序部分应包含主题名称: {}",
        claim.preamble
    );
    assert!(!claim.features.is_empty(), "应提取到特征");
}

#[test]
fn parse_dependent_claim() {
    let parser = ClaimParser::new();
    let text = "2. 根据权利要求1所述的装置，其特征在于，所述处理器为GPU。";
    let claim = parser.parse(2, text);

    assert_eq!(claim.claim_number, 2);
    assert_eq!(claim.claim_type, ClaimType::Dependent);
    assert_eq!(claim.dependent_from, Some(1));
    assert_eq!(claim.transition_word, "其特征在于");
}

#[test]
fn parse_multiple_dependent_claim() {
    let parser = ClaimParser::new();
    let text = "3. 根据权利要求1或2所述的装置，其特征在于，还包括散热模块。";
    let claim = parser.parse(3, text);

    assert_eq!(claim.claim_type, ClaimType::Dependent);
    // extract_reference_cn takes the first digit found
    assert_eq!(
        claim.dependent_from,
        Some(1),
        "多项从属权利要求应引用第一个编号"
    );
}

#[test]
fn parse_claim_no_period() {
    let parser = ClaimParser::new();
    // Claim text without ending period — should still parse features
    let text = "一种数据处理方法，其特征在于，包括：数据采集模块；分析模块";
    let claim = parser.parse(4, text);

    assert_eq!(claim.claim_type, ClaimType::Independent);
    assert_eq!(claim.transition_word, "其特征在于");
    assert!(
        !claim.features.is_empty(),
        "无结尾句号时仍应提取特征: body={}",
        claim.body
    );
}

#[test]
fn parse_claim_no_feature_phrase() {
    let parser = ClaimParser::new();
    // No "其特征在于" — entire text becomes the body
    let text = "一种图像识别装置，包括处理器和存储器";
    let claim = parser.parse(5, text);

    assert_eq!(
        claim.transition_word, "",
        "无过渡词时 transition_word 应为空"
    );
    assert_eq!(claim.preamble, "", "无过渡词时 preamble 应为空");
    assert_eq!(claim.body, text, "无过渡词时整个文本为 body");
    assert_eq!(claim.claim_type, ClaimType::Independent);
}

#[test]
fn parse_empty_claim() {
    let parser = ClaimParser::new();
    let claim = parser.parse(99, "");

    assert_eq!(claim.claim_number, 99);
    assert_eq!(claim.claim_type, ClaimType::Independent);
    assert!(claim.dependent_from.is_none());
    assert!(claim.features.is_empty(), "空字符串不应产生特征");
    assert_eq!(claim.preamble, "");
    assert_eq!(claim.transition_word, "");
    assert_eq!(claim.body, "");
}

#[test]
fn parse_claim_extracts_features() {
    let parser = ClaimParser::new();
    // Two segments separated by Chinese semicolons
    let text = "一种装置，其特征在于，包括：处理器；存储器，所述存储器为非易失性存储器。";
    let claim = parser.parse(1, text);

    assert!(
        !claim.features.is_empty(),
        "应提取至少一个特征: got {} features",
        claim.features.len()
    );

    // Verify feature IDs follow "F1", "F2", ... pattern
    for (i, feat) in claim.features.iter().enumerate() {
        let expected_id = format!("F{}", i + 1);
        assert_eq!(
            feat.id,
            expected_id,
            "第 {} 个特征 ID 应为 {}，实际为 {}",
            i + 1,
            expected_id,
            feat.id
        );
    }

    // Each feature should have a non-empty description
    for feat in &claim.features {
        assert!(
            !feat.description.trim().is_empty(),
            "特征 {} 的描述不应为空",
            feat.id
        );
    }

    // At least one feature should describe "处理器" or "存储器"
    let descriptions: Vec<&str> = claim
        .features
        .iter()
        .map(|f| f.description.as_str())
        .collect();
    let has_processor_or_memory = descriptions
        .iter()
        .any(|d| d.contains("处理器") || d.contains("存储器"));
    assert!(
        has_processor_or_memory,
        "特征中应包含处理器或存储器相关描述: {:?}",
        descriptions
    );
}

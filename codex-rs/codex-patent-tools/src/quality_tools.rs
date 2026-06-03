use codex_patent_core::ClaimDraft;
use codex_patent_core::ClaimType;
use codex_patent_domain::quality::QualityAssessor;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QualityCheckInput {
    pub claims: Vec<ClaimDraftInput>,
    pub patent_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClaimDraftInput {
    pub id: Option<String>,
    pub claim_type: String,
    pub preamble: String,
    pub transitional_phrase: Option<String>,
    pub elements: Vec<String>,
    pub dependent_on: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubjectMatterInput {
    pub invention_title: String,
    pub claims: Vec<String>,
    pub patent_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UnityInput {
    pub claims: Vec<String>,
    pub patent_type: Option<String>,
    pub invention_title: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpecFormalityInput {
    pub specification: SpecInput,
    pub claims: Vec<String>,
    pub patent_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpecInput {
    pub technical_field: Option<String>,
    pub background_art: Option<String>,
    pub invention_content: Option<String>,
    pub embodiment: Option<String>,
    pub drawings_description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LegalLanguageInput {
    pub claims: Vec<String>,
    pub check_level: Option<u32>,
}

pub struct QualityTools;

impl QualityTools {
    fn to_claim_draft(input: &ClaimDraftInput) -> ClaimDraft {
        ClaimDraft {
            id: input.id.clone().unwrap_or_else(|| "1".into()),
            claim_type: if input.claim_type.contains("independent") {
                ClaimType::Independent
            } else {
                ClaimType::Dependent
            },
            preamble: input.preamble.clone(),
            transitional_phrase: input.transitional_phrase.clone().unwrap_or_default(),
            elements: input.elements.clone(),
            dependent_on: input.dependent_on.clone(),
        }
    }

    pub fn unified_quality(input: QualityCheckInput) -> Result<serde_json::Value, String> {
        let drafts: Vec<ClaimDraft> = input.claims.iter().map(Self::to_claim_draft).collect();
        let assessment = QualityAssessor::assess_claims(&drafts);
        serde_json::to_value(assessment).map_err(|e| format!("{e}"))
    }

    pub fn quality_checker(input: QualityCheckInput) -> Result<serde_json::Value, String> {
        let text: Vec<String> = input
            .claims
            .iter()
            .map(|c| format!("{} {}", c.preamble, c.elements.join("；")))
            .collect();
        let mut issues = Vec::new();
        let vague_words = [
            "大约",
            "左右",
            "基本上",
            "适当",
            "一定",
            "某种",
            "优选地",
            "最好",
        ];
        for (i, t) in text.iter().enumerate() {
            for w in &vague_words {
                if t.contains(w) {
                    issues.push(format!("权利要求{}含模糊词: {}", i + 1, w));
                }
            }
        }
        Ok(
            serde_json::json!({"passed": issues.is_empty(), "issues": issues, "count": issues.len()}),
        )
    }

    pub fn subject_matter_checker(input: SubjectMatterInput) -> Result<serde_json::Value, String> {
        let text = input.claims.join(" ");
        let t = text.to_lowercase();
        let (mut blocks, mut warns) = (Vec::new(), Vec::new());
        let art25 = [
            (
                "智力活动",
                r"(?:方法|步骤).*(?:计算|运算|统计|分析|推理|判断|决策)",
            ),
            ("医疗诊断", r"(?:疾病|病症|病情).*(?:诊断|检查|筛查|检测)"),
            ("科学发现", r"(?:发现|找到).*(?:新|未知)"),
            ("游戏规则", r"(?:游戏|竞赛|比赛).*(?:规则|方法)"),
            ("动物植物", r"(?:动物|植物).*(?:品种|变种)"),
        ];
        for (name, pat) in &art25 {
            if let Ok(re) = regex::Regex::new(pat)
                && re.is_match(&t)
            {
                warns.push(format!("可能涉及第25条排除客体: {}", name));
            }
        }
        let tech_markers = ["装置", "设备", "系统", "方法", "模块", "电路"];
        let has_tech = tech_markers.iter().any(|m| t.contains(m));
        if !has_tech {
            blocks.push("缺少技术手段");
        }
        Ok(
            serde_json::json!({"is_patentable": blocks.is_empty(), "blocking_issues": blocks, "warnings": warns}),
        )
    }

    pub fn unity_checker(input: UnityInput) -> Result<serde_json::Value, String> {
        if input.claims.len() <= 1 {
            return Ok(serde_json::json!({"has_unity": true, "reason": "单一权利要求"}));
        }
        let sets: Vec<std::collections::HashSet<&str>> = input
            .claims
            .iter()
            .map(|c| c.split_whitespace().collect())
            .collect();
        let common: std::collections::HashSet<&str> =
            sets[0].intersection(&sets[1]).cloned().collect();
        if common.len() >= 2 {
            Ok(serde_json::json!({"has_unity": true, "common_terms": common.len()}))
        } else {
            Ok(
                serde_json::json!({"has_unity": false, "note": "权利要求间缺少相同或相应特定技术特征"}),
            )
        }
    }

    pub fn spec_formality_checker(input: SpecFormalityInput) -> Result<serde_json::Value, String> {
        let s = &input.specification;
        let mut missing = Vec::new();
        if s.technical_field.as_ref().is_none_or(|t| t.is_empty()) {
            missing.push("技术领域");
        }
        if s.background_art.as_ref().is_none_or(|b| b.is_empty()) {
            missing.push("背景技术");
        }
        if s.invention_content.as_ref().is_none_or(|i| i.is_empty()) {
            missing.push("发明内容");
        }
        if s.embodiment.as_ref().is_none_or(|e| e.is_empty()) {
            missing.push("具体实施方式");
        }
        if s.drawings_description
            .as_ref()
            .is_some_and(|d| !d.is_empty())
            && input.claims.is_empty()
        {
            missing.push("有附图说明但无附图文件说明");
        }
        Ok(serde_json::json!({"passed": missing.is_empty(), "missing_sections": missing}))
    }

    pub fn legal_language_checker(input: LegalLanguageInput) -> Result<serde_json::Value, String> {
        let mut issues = Vec::new();
        let forbidden = ["最好", "最佳", "最先进", "世界领先", "国际领先", "独一无二"];
        for (i, c) in input.claims.iter().enumerate() {
            for f in &forbidden {
                if c.contains(*f) {
                    issues.push(format!("权利要求{}含禁用词: {}", i + 1, f));
                }
            }
        }
        Ok(serde_json::json!({"passed": issues.is_empty(), "issues": issues}))
    }

    pub fn format_rules(content: &str, doc_type: &str) -> Result<serde_json::Value, String> {
        let report = if doc_type == "claims" {
            let lines: Vec<&str> = content.lines().collect();
            serde_json::json!({"total_claims": lines.len(), "independent": lines.iter().filter(|l| !l.contains("根据权利要求")).count()})
        } else {
            serde_json::json!({"word_count": content.len(), "doc_type": doc_type})
        };
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Input struct deserialization tests ---

    #[test]
    fn deserialize_quality_check_input() {
        let json = serde_json::json!({
            "claims": [{
                "claim_type": "independent",
                "preamble": "一种装置",
                "elements": ["特征A", "特征B"]
            }],
            "patent_type": "invention"
        });
        let input: QualityCheckInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.claims.len(), 1);
        assert_eq!(input.claims[0].claim_type, "independent");
        assert_eq!(input.claims[0].elements.len(), 2);
        assert_eq!(input.patent_type.as_deref(), Some("invention"));
    }

    #[test]
    fn deserialize_claim_draft_input_defaults() {
        let json = serde_json::json!({
            "claim_type": "dependent",
            "preamble": "根据权利要求1所述的方法",
            "elements": []
        });
        let input: ClaimDraftInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.id.is_none());
        assert!(input.transitional_phrase.is_none());
        assert!(input.dependent_on.is_none());
    }

    #[test]
    fn deserialize_subject_matter_input() {
        let json = serde_json::json!({
            "invention_title": "数据压缩方法",
            "claims": ["一种数据压缩方法，包括步骤A"],
            "patent_type": "invention"
        });
        let input: SubjectMatterInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.invention_title, "数据压缩方法");
        assert_eq!(input.claims.len(), 1);
    }

    #[test]
    fn deserialize_unity_input() {
        let json = serde_json::json!({
            "claims": ["一种装置，包括特征A", "一种方法，包括特征A"],
            "patent_type": "invention",
            "invention_title": "测试发明"
        });
        let input: UnityInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.claims.len(), 2);
        assert_eq!(input.invention_title.as_deref(), Some("测试发明"));
    }

    #[test]
    fn deserialize_spec_formality_input() {
        let json = serde_json::json!({
            "specification": {
                "technical_field": "电子通信",
                "background_art": "现有技术描述",
                "invention_content": "发明内容描述",
                "embodiment": "具体实施方式描述",
                "drawings_description": null
            },
            "claims": ["一种装置"],
            "patent_type": "invention"
        });
        let input: SpecFormalityInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert!(input.specification.drawings_description.is_none());
    }

    #[test]
    fn deserialize_legal_language_input() {
        let json = serde_json::json!({
            "claims": ["一种装置"],
            "check_level": 2
        });
        let input: LegalLanguageInput =
            serde_json::from_value(json).expect("deserialization should succeed");
        assert_eq!(input.check_level, Some(2));
    }

    // --- format_rules tests ---

    #[test]
    fn format_rules_claims_counts_independent() {
        let content = "一种装置，包括特征A\n根据权利要求1所述的装置，还包括特征B\n根据权利要求1所述的装置，还包括特征C";
        let result = QualityTools::format_rules(content, "claims").unwrap();
        assert_eq!(result["total_claims"], 3);
        assert_eq!(result["independent"], 1);
    }

    #[test]
    fn format_rules_claims_all_independent() {
        let content = "一种方法\n一种装置\n一种系统";
        let result = QualityTools::format_rules(content, "claims").unwrap();
        assert_eq!(result["total_claims"], 3);
        assert_eq!(result["independent"], 3);
    }

    #[test]
    fn format_rules_non_claims_doc_type() {
        let content = "这是一段描述性文字";
        let result = QualityTools::format_rules(content, "specification").unwrap();
        assert_eq!(result["word_count"], content.len());
        assert_eq!(result["doc_type"], "specification");
    }

    #[test]
    fn format_rules_empty_claims() {
        let content = "";
        let result = QualityTools::format_rules(content, "claims").unwrap();
        // Empty string yields 0 lines from .lines()
        assert_eq!(result["total_claims"], 0);
        assert_eq!(result["independent"], 0);
    }

    // --- subject_matter_checker tests ---

    #[test]
    fn subject_matter_pure_software_no_tech_markers() {
        let input = SubjectMatterInput {
            invention_title: "数据分析方法".into(),
            claims: vec!["一种数据分析的方法，包括统计和推理的步骤".into()],
            patent_type: Some("invention".into()),
        };
        let result = QualityTools::subject_matter_checker(input).unwrap();
        // Should detect "智力活动" regex and lack of tech markers
        assert!(!result["blocking_issues"].as_array().unwrap().is_empty() || !result["warnings"].as_array().unwrap().is_empty());
    }

    #[test]
    fn subject_matter_with_tech_markers_patentable() {
        let input = SubjectMatterInput {
            invention_title: "一种传感器装置".into(),
            claims: vec!["一种传感器装置，包括检测模块和信号处理电路".into()],
            patent_type: Some("utility_model".into()),
        };
        let result = QualityTools::subject_matter_checker(input).unwrap();
        assert_eq!(result["is_patentable"], true);
        assert!(result["blocking_issues"].as_array().unwrap().is_empty());
    }

    #[test]
    fn subject_matter_medical_diagnosis_warning() {
        let input = SubjectMatterInput {
            invention_title: "疾病诊断方法".into(),
            claims: vec!["一种疾病诊断的方法，包括对病情的检测步骤".into()],
            patent_type: None,
        };
        let result = QualityTools::subject_matter_checker(input).unwrap();
        let warns = result["warnings"].as_array().unwrap();
        assert!(warns.iter().any(|w| w.as_str().unwrap().contains("医疗诊断")));
    }

    // --- unity_checker tests ---

    #[test]
    fn unity_single_claim_passes() {
        let input = UnityInput {
            claims: vec!["一种装置，包括特征A".into()],
            patent_type: None,
            invention_title: None,
        };
        let result = QualityTools::unity_checker(input).unwrap();
        assert_eq!(result["has_unity"], true);
        assert_eq!(result["reason"], "单一权利要求");
    }

    #[test]
    fn unity_claims_with_common_terms() {
        let input = UnityInput {
            claims: vec![
                "一种 数据 处理 装置 包括 特征A".into(),
                "一种 数据 处理 方法 包括 特征A".into(),
            ],
            patent_type: None,
            invention_title: None,
        };
        let result = QualityTools::unity_checker(input).unwrap();
        assert_eq!(result["has_unity"], true);
        assert!(result["common_terms"].as_u64().unwrap() >= 2);
    }

    #[test]
    fn unity_claims_without_common_terms() {
        let input = UnityInput {
            claims: vec![
                "完全不同的XYZ描述".into(),
                "毫无关联的ABC内容".into(),
            ],
            patent_type: None,
            invention_title: None,
        };
        let result = QualityTools::unity_checker(input).unwrap();
        assert_eq!(result["has_unity"], false);
    }

    // --- quality_checker tests ---

    #[test]
    fn quality_checker_no_issues() {
        let input = QualityCheckInput {
            claims: vec![ClaimDraftInput {
                id: Some("1".into()),
                claim_type: "independent".into(),
                preamble: "一种装置，包括特征A".into(),
                transitional_phrase: Some("包括".into()),
                elements: vec!["特征A".into()],
                dependent_on: None,
            }],
            patent_type: None,
        };
        let result = QualityTools::quality_checker(input).unwrap();
        assert_eq!(result["passed"], true);
        assert_eq!(result["count"], 0);
    }

    #[test]
    fn quality_checker_detects_vague_words() {
        let input = QualityCheckInput {
            claims: vec![ClaimDraftInput {
                id: Some("1".into()),
                claim_type: "independent".into(),
                preamble: "一种装置".into(),
                transitional_phrase: None,
                elements: vec!["大约50%的成分".into(), "适当的温度".into()],
                dependent_on: None,
            }],
            patent_type: None,
        };
        let result = QualityTools::quality_checker(input).unwrap();
        assert_eq!(result["passed"], false);
        let issues = result["issues"].as_array().unwrap();
        assert!(issues.len() >= 2);
    }

    // --- spec_formality_checker tests ---

    #[test]
    fn spec_formality_checker_all_present() {
        let input = SpecFormalityInput {
            specification: SpecInput {
                technical_field: Some("电子通信".into()),
                background_art: Some("现有技术描述".into()),
                invention_content: Some("发明内容描述".into()),
                embodiment: Some("具体实施方式描述".into()),
                drawings_description: None,
            },
            claims: vec!["一种装置".into()],
            patent_type: None,
        };
        let result = QualityTools::spec_formality_checker(input).unwrap();
        assert_eq!(result["passed"], true);
    }

    #[test]
    fn spec_formality_checker_missing_sections() {
        let input = SpecFormalityInput {
            specification: SpecInput {
                technical_field: None,
                background_art: Some("".into()),
                invention_content: None,
                embodiment: None,
                drawings_description: None,
            },
            claims: vec![],
            patent_type: None,
        };
        let result = QualityTools::spec_formality_checker(input).unwrap();
        assert_eq!(result["passed"], false);
        let missing = result["missing_sections"].as_array().unwrap();
        assert!(missing.contains(&serde_json::json!("技术领域")));
        assert!(missing.contains(&serde_json::json!("背景技术")));
        assert!(missing.contains(&serde_json::json!("发明内容")));
        assert!(missing.contains(&serde_json::json!("具体实施方式")));
    }

    // --- legal_language_checker tests ---

    #[test]
    fn legal_language_checker_clean() {
        let input = LegalLanguageInput {
            claims: vec!["一种装置，包括特征A".into()],
            check_level: None,
        };
        let result = QualityTools::legal_language_checker(input).unwrap();
        assert_eq!(result["passed"], true);
    }

    #[test]
    fn legal_language_checker_detects_forbidden_words() {
        let input = LegalLanguageInput {
            claims: vec!["一种世界领先的装置".into(), "独一无二的方法".into()],
            check_level: None,
        };
        let result = QualityTools::legal_language_checker(input).unwrap();
        assert_eq!(result["passed"], false);
        let issues = result["issues"].as_array().unwrap();
        assert!(issues.iter().any(|i| i.as_str().unwrap().contains("世界领先")));
        assert!(issues.iter().any(|i| i.as_str().unwrap().contains("独一无二")));
    }

    // --- to_claim_draft test ---

    #[test]
    fn to_claim_draft_independent() {
        let input = ClaimDraftInput {
            id: Some("3".into()),
            claim_type: "independent".into(),
            preamble: "一种传感器".into(),
            transitional_phrase: Some("包括".into()),
            elements: vec!["模块A".into()],
            dependent_on: None,
        };
        let draft = QualityTools::to_claim_draft(&input);
        assert_eq!(draft.id, "3");
        assert!(matches!(draft.claim_type, ClaimType::Independent));
    }

    #[test]
    fn to_claim_draft_dependent() {
        let input = ClaimDraftInput {
            id: None,
            claim_type: "dependent".into(),
            preamble: "根据权利要求1所述的传感器".into(),
            transitional_phrase: None,
            elements: vec![],
            dependent_on: Some("1".into()),
        };
        let draft = QualityTools::to_claim_draft(&input);
        assert_eq!(draft.id, "1"); // default
        assert!(matches!(draft.claim_type, ClaimType::Dependent));
        assert_eq!(draft.transitional_phrase, "");
    }
}

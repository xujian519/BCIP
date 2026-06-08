//! 专利质量检测工具。
//!
//! 提供权利要求格式检查、客体审查、单一性检查、形式审查、法律用语合规检查等质量相关功能。
//! 所有检测器通过 [`QualityTools`] 结构体的静态方法暴露，
//! 并通过 [`register_quality_tools`] 注册到统一的工具注册表。

pub mod types;

use codex_patent_core::ClaimDraft;
use codex_patent_core::ClaimType;
use codex_patent_core::QualityAssessment;
use codex_patent_core::QualityIssue;
use codex_patent_domain::quality::QualityAssessor;
use codex_patent_domain::quality_rules;
pub use types::*;

/// 专利质量检测工具集合。
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

    fn detect_vague_issues(texts: &[String]) -> Vec<String> {
        let mut issues = Vec::new();
        let all_terms = quality_rules::all_quality_terms();
        for (i, t) in texts.iter().enumerate() {
            for w in &all_terms {
                if t.contains(w.as_str()) {
                    issues.push(format!("权利要求{}含问题词汇: {}", i + 1, w));
                }
            }
        }
        issues
    }

    /// 统一质量检查入口。
    ///
    /// 结合规则质检和语义分析，返回综合质量评估结果。
    /// 包含权利要求格式验证、模糊用语检测等维度。
    pub fn unified_quality(input: QualityCheckInput) -> Result<serde_json::Value, String> {
        let drafts: Vec<ClaimDraft> = input.claims.iter().map(Self::to_claim_draft).collect();
        let assessment = QualityAssessor::assess_claims(&drafts);

        let texts: Vec<String> = input
            .claims
            .iter()
            .map(|c| format!("{} {}", c.preamble, c.elements.join("；")))
            .collect();
        let vague_issues = Self::detect_vague_issues(&texts);

        let merged: QualityAssessment = QualityAssessment {
            issues: [
                assessment.issues,
                vague_issues
                    .into_iter()
                    .map(|desc| QualityIssue {
                        dimension: "规则质检".into(),
                        severity: "中".into(),
                        description: desc,
                        suggestion: "替换为具体明确的表述".into(),
                    })
                    .collect(),
            ]
            .concat(),
            ..assessment
        };

        serde_json::to_value(merged).map_err(|e| format!("序列化质量评估结果失败: {e}"))
    }

    /// 基础质量检查器。
    ///
    /// 检测权利要求中是否包含模糊用语（如"约"、"大致"、"优选"），
    /// 返回检查通过状态和问题列表。
    pub fn quality_checker(input: QualityCheckInput) -> Result<serde_json::Value, String> {
        let texts: Vec<String> = input
            .claims
            .iter()
            .map(|c| format!("{} {}", c.preamble, c.elements.join("；")))
            .collect();
        let issues = Self::detect_vague_issues(&texts);
        Ok(
            serde_json::json!({"passed": issues.is_empty(), "issues": issues, "count": issues.len()}),
        )
    }

    /// 专利客体审查。
    ///
    /// 根据专利法第5条（违反法律/社会公德）和第25条（不授予专利权的客体）
    /// 排除规则，判断发明主题是否属于授权客体。通过关键词和正则模式匹配
    /// 检测排除客体。
    pub fn subject_matter_checker(input: SubjectMatterInput) -> Result<serde_json::Value, String> {
        let text = input.claims.join(" ");
        let t = text.to_lowercase();
        let (mut blocks, mut warns) = (Vec::new(), Vec::new());

        let art5 = [
            ("违法内容", r"(?:赌博|吸毒|毒品|走私|伪造|假币)"),
            ("违反社会公德", r"(?:歧视|侮辱|诽谤)"),
        ];
        for (name, pat) in &art5 {
            if let Ok(re) = regex::Regex::new(pat)
                && re.is_match(&t)
            {
                blocks.push(format!("违反第5条排除客体: {}", name));
            }
        }

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
            blocks.push("缺少技术手段".to_string());
        }
        Ok(
            serde_json::json!({"is_patentable": blocks.is_empty() && warns.is_empty(), "blocking_issues": blocks, "warnings": warns}),
        )
    }

    /// 单一性检查。
    ///
    /// 判断多项权利要求之间是否具备单一性（属于一个总的发明构思）。
    /// 通过共享技术特征的数量判断是否满足单一性要求。
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

    /// 说明书形式审查。
    ///
    /// 检查说明书是否包含必要部分：技术领域、背景技术、发明内容、
    /// 具体实施方式。如果存在附图说明但缺少附图文件说明也会提示。
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

    /// 法律用语合规检查。
    ///
    /// 检测权利要求中是否包含禁用词（绝对性用语、广告用语）和
    /// 商业宣传用语（如"世界领先"、"独一无二"），确保法律用语严谨。
    pub fn legal_language_checker(input: LegalLanguageInput) -> Result<serde_json::Value, String> {
        let mut issues = Vec::new();
        let forbidden = quality_rules::forbidden_terms();
        let all_rules = quality_rules::commercial_terms();
        for (i, c) in input.claims.iter().enumerate() {
            for f in &forbidden {
                if c.contains(*f) {
                    issues.push(format!("权利要求{}含禁用词: {}", i + 1, f));
                }
            }
            for r in &all_rules {
                if c.contains(r.as_str()) {
                    issues.push(format!("权利要求{}含商业用语: {}", i + 1, r));
                }
            }
        }
        Ok(serde_json::json!({"passed": issues.is_empty(), "issues": issues}))
    }

    /// 格式规则检查。
    ///
    /// 检查文档格式合规性。对权利要求书统计独立/从属权利要求数量，
    /// 对其他文档类型返回字数和类型信息。
    pub fn format_rules(content: &str, doc_type: &str) -> Result<serde_json::Value, String> {
        let report = if doc_type == "claims" {
            let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
            let ind_count = lines
                .iter()
                .filter(|l| {
                    !l.contains("根据权利要求")
                        && regex::Regex::new(r"^\d+\.")
                            .map(|re| re.is_match(l))
                            .unwrap_or(false)
                })
                .count();
            serde_json::json!({"total_claims": lines.len(), "independent": ind_count})
        } else {
            serde_json::json!({"word_count": content.len(), "doc_type": doc_type})
        };
        Ok(report)
    }

    /// 权利要求依赖关系验证。
    ///
    /// 验证权利要求引用关系的合法性：
    /// - 引用的权利要求必须存在
    /// - 不能引用自身或后续权利要求
    /// - 引用链不能存在循环引用
    /// - 至少包含一条独立权利要求
    pub fn claim_dependency_validator(
        input: ClaimDependencyInput,
    ) -> Result<serde_json::Value, String> {
        let claims = &input.claims;
        if claims.is_empty() {
            return Err("权利要求列表不能为空".to_string());
        }

        let mut issues = Vec::new();
        let re = regex::Regex::new(r"根据权利要求(\d+)").expect("test tool call should succeed");

        for (i, claim) in claims.iter().enumerate() {
            let claim_num = i + 1; // 1-indexed

            if !claim.contains("根据权利要求") {
                // 独立权利要求，检查是否为第一条或编号合法
                continue;
            }

            // 提取所有引用编号
            let refs: Vec<usize> = re
                .captures_iter(claim)
                .filter_map(|cap| cap.get(1)?.as_str().parse().ok())
                .collect();

            if refs.is_empty() {
                issues.push(format!("权利要求{}: 包含引用但无法解析编号", claim_num));
                continue;
            }

            for &ref_num in &refs {
                // 检查引用编号在有效范围内
                if ref_num == 0 || ref_num > claims.len() {
                    issues.push(format!(
                        "权利要求{}: 引用了不存在的权利要求{}",
                        claim_num, ref_num
                    ));
                }
                // 检查不能引用自身或后续权利要求
                else if ref_num >= claim_num {
                    issues.push(format!(
                        "权利要求{}: 只能引用在前的权利要求（引用了{}）",
                        claim_num, ref_num
                    ));
                }
            }
        }

        // 检查引用链是否有环（传递依赖）
        // 构建引用图
        let mut graph: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();
        for (i, claim) in claims.iter().enumerate() {
            let refs: Vec<usize> = re
                .captures_iter(claim)
                .filter_map(|cap| cap.get(1)?.as_str().parse().ok())
                .filter(|&r| r >= 1 && r <= claims.len() && r <= i)
                .collect();
            graph.insert(i + 1, refs);
        }

        // DFS 检测环
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();

        fn has_cycle(
            node: usize,
            graph: &std::collections::HashMap<usize, Vec<usize>>,
            visited: &mut std::collections::HashSet<usize>,
            in_stack: &mut std::collections::HashSet<usize>,
        ) -> bool {
            if in_stack.contains(&node) {
                return true;
            }
            if visited.contains(&node) {
                return false;
            }
            visited.insert(node);
            in_stack.insert(node);
            if let Some(neighbors) = graph.get(&node) {
                for &next in neighbors {
                    if has_cycle(next, graph, visited, in_stack) {
                        return true;
                    }
                }
            }
            in_stack.remove(&node);
            false
        }

        for node in 1..=claims.len() {
            if has_cycle(node, &graph, &mut visited, &mut in_stack) {
                issues.push("检测到循环引用".to_string());
                break;
            }
        }

        // 检查独立权利要求存在性
        let independent_count = claims
            .iter()
            .filter(|c| !c.contains("根据权利要求"))
            .count();
        if independent_count == 0 {
            issues.push("缺少独立权利要求".to_string());
        }

        Ok(serde_json::json!({
            "passed": issues.is_empty(),
            "issues": issues,
            "total_claims": claims.len(),
            "independent_claims": independent_count,
            "dependent_claims": claims.len() - independent_count,
        }))
    }
}

pub fn register_quality_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("UnifiedQuality".into(), |input| {
        Box::pin(async move {
            let parsed: QualityCheckInput = serde_json::from_value(input)
                .map_err(|e| format!("解析 UnifiedQuality 输入失败: {e}"))?;
            QualityTools::unified_quality(parsed)
        })
    });
    t.insert("SubjectMatterChecker".into(), |input| {
        Box::pin(async move {
            let parsed: SubjectMatterInput = serde_json::from_value(input)
                .map_err(|e| format!("解析 SubjectMatterChecker 输入失败: {e}"))?;
            QualityTools::subject_matter_checker(parsed)
        })
    });
    t.insert("UnityChecker".into(), |input| {
        Box::pin(async move {
            let parsed: UnityInput = serde_json::from_value(input)
                .map_err(|e| format!("解析 UnityChecker 输入失败: {e}"))?;
            QualityTools::unity_checker(parsed)
        })
    });
    t.insert("SpecFormalityChecker".into(), |input| {
        Box::pin(async move {
            let parsed: SpecFormalityInput = serde_json::from_value(input)
                .map_err(|e| format!("解析 SpecFormalityChecker 输入失败: {e}"))?;
            QualityTools::spec_formality_checker(parsed)
        })
    });
    t.insert("LegalLanguageChecker".into(), |input| {
        Box::pin(async move {
            let parsed: LegalLanguageInput = serde_json::from_value(input)
                .map_err(|e| format!("解析 LegalLanguageChecker 输入失败: {e}"))?;
            QualityTools::legal_language_checker(parsed)
        })
    });
    t.insert("FormatRules".into(), |input| {
        Box::pin(async move {
            let content = input.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let doc_type = input
                .get("doc_type")
                .and_then(|v| v.as_str())
                .unwrap_or("generic");
            QualityTools::format_rules(content, doc_type)
        })
    });
    t.insert("ClaimDependencyValidator".into(), |input| {
        Box::pin(async move {
            let parsed: ClaimDependencyInput = serde_json::from_value(input)
                .map_err(|e| format!("解析 ClaimDependencyValidator 输入失败: {e}"))?;
            QualityTools::claim_dependency_validator(parsed)
        })
    });
    t
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
        let content = "1. 一种装置，包括特征A\n2. 根据权利要求1所述的装置，还包括特征B\n3. 根据权利要求1所述的装置，还包括特征C";
        let result =
            QualityTools::format_rules(content, "claims").expect("test tool call should succeed");
        assert_eq!(result["total_claims"], 3);
        assert_eq!(result["independent"], 1);
    }

    #[test]
    fn format_rules_claims_all_independent() {
        let content = "1. 一种方法\n2. 一种装置\n3. 一种系统";
        let result =
            QualityTools::format_rules(content, "claims").expect("test tool call should succeed");
        assert_eq!(result["total_claims"], 3);
        assert_eq!(result["independent"], 3);
    }

    #[test]
    fn format_rules_non_claims_doc_type() {
        let content = "这是一段描述性文字";
        let result = QualityTools::format_rules(content, "specification")
            .expect("test tool call should succeed");
        assert_eq!(result["word_count"], content.len());
        assert_eq!(result["doc_type"], "specification");
    }

    #[test]
    fn format_rules_empty_claims() {
        let content = "";
        let result =
            QualityTools::format_rules(content, "claims").expect("test tool call should succeed");
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
        let result =
            QualityTools::subject_matter_checker(input).expect("test tool call should succeed");
        assert!(
            !result["blocking_issues"]
                .as_array()
                .expect("test fixture field should be an array")
                .is_empty()
                || !result["warnings"]
                    .as_array()
                    .expect("test fixture field should be an array")
                    .is_empty()
        );
    }

    #[test]
    fn subject_matter_with_tech_markers_patentable() {
        let input = SubjectMatterInput {
            invention_title: "一种传感器装置".into(),
            claims: vec!["一种传感器装置，包括检测模块和信号处理电路".into()],
            patent_type: Some("utility_model".into()),
        };
        let result =
            QualityTools::subject_matter_checker(input).expect("test tool call should succeed");
        assert_eq!(result["is_patentable"], true);
        assert!(
            result["blocking_issues"]
                .as_array()
                .expect("test fixture field should be an array")
                .is_empty()
        );
    }

    #[test]
    fn subject_matter_medical_diagnosis_warning() {
        let input = SubjectMatterInput {
            invention_title: "疾病诊断方法".into(),
            claims: vec!["一种疾病诊断的方法，包括对病情的检测步骤".into()],
            patent_type: None,
        };
        let result =
            QualityTools::subject_matter_checker(input).expect("test tool call should succeed");
        let warns = result["warnings"]
            .as_array()
            .expect("test fixture field should be an array");
        assert!(warns.iter().any(|w| {
            w.as_str()
                .expect("test fixture field should be a string")
                .contains("医疗诊断")
        }));
    }

    #[test]
    fn subject_matter_article5_blocked() {
        let input = SubjectMatterInput {
            invention_title: "赌博装置".into(),
            claims: vec!["一种赌博装置".into()],
            patent_type: Some("invention".into()),
        };
        let result =
            QualityTools::subject_matter_checker(input).expect("test tool call should succeed");
        let blocks = result["blocking_issues"]
            .as_array()
            .expect("test fixture field should be an array");
        assert!(blocks.iter().any(|b| {
            b.as_str()
                .expect("test fixture field should be a string")
                .contains("第5条")
        }));
    }

    // --- unity_checker tests ---

    #[test]
    fn unity_single_claim_passes() {
        let input = UnityInput {
            claims: vec!["一种装置，包括特征A".into()],
            patent_type: None,
            invention_title: None,
        };
        let result = QualityTools::unity_checker(input).expect("test tool call should succeed");
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
        let result = QualityTools::unity_checker(input).expect("test tool call should succeed");
        assert_eq!(result["has_unity"], true);
        assert!(
            result["common_terms"]
                .as_u64()
                .expect("test fixture field should be a number")
                >= 2
        );
    }

    #[test]
    fn unity_claims_without_common_terms() {
        let input = UnityInput {
            claims: vec!["完全不同的XYZ描述".into(), "毫无关联的ABC内容".into()],
            patent_type: None,
            invention_title: None,
        };
        let result = QualityTools::unity_checker(input).expect("test tool call should succeed");
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
        let result = QualityTools::quality_checker(input).expect("test tool call should succeed");
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
                elements: vec!["约50%的成分".into(), "适当调整的温度".into()],
                dependent_on: None,
            }],
            patent_type: None,
        };
        let result = QualityTools::quality_checker(input).expect("test tool call should succeed");
        assert_eq!(result["passed"], false);
        let issues = result["issues"]
            .as_array()
            .expect("test fixture field should be an array");
        assert!(issues.len() >= 2);
    }

    #[test]
    fn unified_quality_includes_vague_issues() {
        let input = QualityCheckInput {
            claims: vec![ClaimDraftInput {
                id: Some("1".into()),
                claim_type: "independent".into(),
                preamble: "一种装置".into(),
                transitional_phrase: None,
                elements: vec!["大约50%的成分".into(), "最佳参数".into()],
                dependent_on: None,
            }],
            patent_type: None,
        };
        let result = QualityTools::unified_quality(input).expect("test tool call should succeed");
        let issues = result["issues"]
            .as_array()
            .expect("test fixture field should be an array");
        let all_text: String = issues
            .iter()
            .map(|i| i["description"].as_str().unwrap_or(""))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(all_text.contains("大约") || all_text.contains("最佳"));
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
        let result =
            QualityTools::spec_formality_checker(input).expect("test tool call should succeed");
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
        let result =
            QualityTools::spec_formality_checker(input).expect("test tool call should succeed");
        assert_eq!(result["passed"], false);
        let missing = result["missing_sections"]
            .as_array()
            .expect("test fixture field should be an array");
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
        let result =
            QualityTools::legal_language_checker(input).expect("test tool call should succeed");
        assert_eq!(result["passed"], true);
    }

    #[test]
    fn legal_language_checker_detects_forbidden_words() {
        let input = LegalLanguageInput {
            claims: vec!["一种世界领先的装置".into(), "独一无二的方法".into()],
            check_level: None,
        };
        let result =
            QualityTools::legal_language_checker(input).expect("test tool call should succeed");
        assert_eq!(result["passed"], false);
        let issues = result["issues"]
            .as_array()
            .expect("test fixture field should be an array");
        assert!(issues.iter().any(|i| {
            i.as_str()
                .expect("test fixture field should be a string")
                .contains("世界领先")
        }));
        assert!(issues.iter().any(|i| {
            i.as_str()
                .expect("test fixture field should be a string")
                .contains("独一无二")
        }));
    }

    #[test]
    fn legal_language_checker_also_detects_commercial_terms() {
        let input = LegalLanguageInput {
            claims: vec!["一种革命性的装置".into()],
            check_level: None,
        };
        let result =
            QualityTools::legal_language_checker(input).expect("test tool call should succeed");
        assert_eq!(result["passed"], false);
    }
}

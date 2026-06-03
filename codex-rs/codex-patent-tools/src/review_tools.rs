use codex_patent_core::CaseContext;
use codex_patent_domain::rule_engine::QualitativeRuleEngine;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FormalCheckInput {
    pub claims: Vec<String>,
    pub specification_sections: Option<Vec<String>>,
}
#[derive(Debug, Deserialize)]
pub struct QualityAssessInput {
    pub claims: Vec<String>,
    pub specification_word_count: usize,
}
#[derive(Debug, Deserialize)]
pub struct SubjectMatterCheckInput {
    pub claim_text: String,
}
#[derive(Debug, Deserialize)]
pub struct UnityCheckInput {
    pub claims: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub struct OaStrategyInput {
    pub rejection_type: String,
    pub differences: Option<Vec<String>>,
    pub technical_effects: Option<Vec<String>>,
    pub prior_art_different_field: Option<bool>,
}
#[derive(Debug, Deserialize)]
pub struct OaResponseTemplateInput {
    pub template_type: String,
    pub arguments: Option<Vec<String>>,
}

pub struct ReviewTools;

impl ReviewTools {
    pub fn formal_check(input: FormalCheckInput) -> Result<serde_json::Value, String> {
        let mut issues = Vec::new();
        // Claim numbering check
        for (i, claim) in input.claims.iter().enumerate() {
            if !claim.contains("根据权利要求") && !claim.contains(&format!("权利要求{}", i + 1))
            {
                if i == 0 {
                    continue;
                }
                issues.push(format!("权利要求{} 可能缺少正确编号", i + 1));
            }
        }
        // Reference validity check
        use regex::Regex;
        let re = Regex::new(r"根据权利要求(\d+)").unwrap();
        for claim in &input.claims {
            for cap in re.captures_iter(claim) {
                let ref_num: usize = cap.get(1).unwrap().as_str().parse().unwrap_or(0);
                if ref_num == 0 || ref_num > input.claims.len() {
                    issues.push(format!("无效引用: 引用不存在的权利要求{}", ref_num));
                }
            }
        }
        // Section completeness
        if let Some(ref sections) = input.specification_sections {
            for req in &["技术领域", "背景技术", "发明内容", "具体实施方式"] {
                if !sections.iter().any(|s| s.contains(req)) {
                    issues.push(format!("缺少必要章节: {}", req));
                }
            }
        }
        Ok(serde_json::json!({"passed": issues.is_empty(), "issues": issues}))
    }

    pub fn quality_assess(input: QualityAssessInput) -> Result<serde_json::Value, String> {
        let score = if input.claims.is_empty() { 0.0 } else { 70.0 };
        let word_count = input.specification_word_count;
        let length_score = if word_count > 1000 {
            1.0
        } else {
            word_count as f64 / 1000.0
        };
        Ok(
            serde_json::json!({"overall_score": score * length_score, "claim_count": input.claims.len(), "word_count": word_count}),
        )
    }

    pub fn subject_matter_check(
        input: SubjectMatterCheckInput,
    ) -> Result<serde_json::Value, String> {
        let lower = input.claim_text.to_lowercase();
        let excluded = ["智力活动", "医疗诊断", "原子核", "科学发现", "游戏规则"];
        let violations: Vec<&&str> = excluded
            .iter()
            .filter(|kw| lower.contains(&kw.to_lowercase()))
            .collect();
        Ok(serde_json::json!({"is_patentable": violations.is_empty(), "violations": violations}))
    }

    pub fn unity_check(input: UnityCheckInput) -> Result<serde_json::Value, String> {
        if input.claims.len() <= 1 {
            return Ok(serde_json::json!({"has_unity": true}));
        }
        let mut common: std::collections::HashSet<&str> =
            input.claims[0].split_whitespace().collect();
        for c in &input.claims[1..] {
            let terms: std::collections::HashSet<&str> = c.split_whitespace().collect();
            common = common.intersection(&terms).cloned().collect();
        }
        Ok(serde_json::json!({"has_unity": common.len() >= 2, "common_terms_count": common.len()}))
    }

    pub fn oa_strategy(input: OaStrategyInput) -> Result<serde_json::Value, String> {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            differences: input.differences,
            rejection_type: Some(input.rejection_type),
            technical_effects: input.technical_effects,
            prior_art_different_field: input.prior_art_different_field,
            ..Default::default()
        };
        let r = engine
            .suggest_oa_strategy(&ctx)
            .map_err(|e| format!("{e}"))?;
        serde_json::to_value(r).map_err(|e| format!("{e}"))
    }

    pub fn response_template(input: OaResponseTemplateInput) -> Result<serde_json::Value, String> {
        let templates = [
            (
                "新颖性争辩",
                "申请人认为，对比文件未公开权利要求的技术特征，本申请具备新颖性，符合专利法第22条第2款。",
            ),
            (
                "创造性争辩",
                "区别技术特征具有非显而易见的技术效果，现有技术未给出技术启示，本申请具备创造性。",
            ),
            (
                "修改方案",
                "申请人将权利要求合并，形成新的独立权利要求，修改未超出原申请范围。",
            ),
            ("充分公开", "说明书已清楚完整公开，本领域技术人员能够实现。"),
            (
                "证据不足",
                "审查员关于公知常识的认定缺乏证据支持，请提供证据。",
            ),
            ("延期请求", "申请人请求延长答复期限以便充分准备意见。"),
        ];
        let found = templates
            .iter()
            .find(|(n, _)| n.contains(&input.template_type))
            .map(|(_, t)| t.to_string())
            .unwrap_or_default();
        Ok(serde_json::json!({"template_type": input.template_type, "content": found}))
    }
}

pub fn register_review_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("FormalCheck".into(), |input| {
        Box::pin(async move {
            let parsed: FormalCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::formal_check(parsed)
        })
    });
    t.insert("QualityAssess".into(), |input| {
        Box::pin(async move {
            let parsed: QualityAssessInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::quality_assess(parsed)
        })
    });
    t.insert("SubjectMatterCheck".into(), |input| {
        Box::pin(async move {
            let parsed: SubjectMatterCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::subject_matter_check(parsed)
        })
    });
    t.insert("UnityCheck".into(), |input| {
        Box::pin(async move {
            let parsed: UnityCheckInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::unity_check(parsed)
        })
    });
    t.insert("OaStrategy".into(), |input| {
        Box::pin(async move {
            let parsed: OaStrategyInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::oa_strategy(parsed)
        })
    });
    t.insert("OaResponseTemplate".into(), |input| {
        Box::pin(async move {
            let parsed: OaResponseTemplateInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            ReviewTools::response_template(parsed)
        })
    });
    t
}

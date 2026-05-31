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

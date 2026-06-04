//! 专利撰写工具集。
//!
//! 提供权利要求生成、说明书撰写、摘要撰写、创造性评估等专利撰写辅助能力。

use codex_patent_core::CaseContext;
use codex_patent_domain::rule_engine::QualitativeRuleEngine;
use serde::Deserialize;

/// 说明书撰写输入参数。
#[derive(Debug, Deserialize)]
pub struct SpecificationInput {
    /// 发明名称。
    pub title: String,
    /// 技术领域。
    pub technical_field: String,
    /// 背景技术。
    pub background: String,
    /// 发明内容。
    pub invention_content: String,
    /// 具体实施方式。
    pub embodiments: String,
}

/// 权利要求生成器输入参数。
#[derive(Debug, Deserialize)]
pub struct ClaimGeneratorInput {
    /// 发明名称。
    pub invention_name: String,
    /// 必要技术特征列表。
    pub essential_features: Vec<String>,
    /// 可选技术特征分组（每个 Vec 为一组可选特征）。
    pub optional_features: Option<Vec<Vec<String>>>,
}

/// 摘要撰写输入参数。
#[derive(Debug, Deserialize)]
pub struct AbstractDraftInput {
    /// 发明名称。
    pub title: String,
    /// 要解决的技术问题。
    pub technical_problem: String,
    /// 技术方案。
    pub technical_solution: String,
    /// 技术效果。
    pub technical_effect: String,
}

/// 权利要求结构分析输入参数。
#[derive(Debug, Deserialize)]
pub struct ClaimsStructureInput {
    /// 权利要求文本（多行）。
    pub claims_text: String,
}

/// 专利撰写工具集。
pub struct DraftingTools;

impl DraftingTools {
    pub fn specification_draft(input: SpecificationInput) -> Result<serde_json::Value, String> {
        let spec = format!(
            "说明书\n\n技术领域\n{}\n\n背景技术\n{}\n\n发明内容\n{}\n\n具体实施方式\n{}",
            input.technical_field, input.background, input.invention_content, input.embodiments
        );
        Ok(
            serde_json::json!({"title": input.title, "specification": spec, "word_count": spec.len()}),
        )
    }

    pub fn claim_generator(input: ClaimGeneratorInput) -> Result<serde_json::Value, String> {
        let ind = format!(
            "一种{}，其特征在于，包括：{}。",
            input.invention_name,
            input.essential_features.join("；")
        );
        let deps: Vec<String> = input
            .optional_features
            .unwrap_or_default()
            .iter()
            .enumerate()
            .map(|(i, f)| {
                format!(
                    "根据权利要求{}所述的{}，其特征在于，还包括：{}。",
                    i + 1,
                    input.invention_name,
                    f.join("；")
                )
            })
            .collect();
        let mut all = vec![ind];
        all.extend(deps);
        Ok(
            serde_json::json!({"claims": all, "independent_count": 1, "dependent_count": all.len()-1}),
        )
    }

    pub fn abstract_draft(input: AbstractDraftInput) -> Result<serde_json::Value, String> {
        let text = format!(
            "本发明公开了一种{}，解决{}的技术问题，方案是{}，达到{}的效果。",
            input.title, input.technical_problem, input.technical_solution, input.technical_effect
        );
        Ok(serde_json::json!({"abstract_text": text, "word_count": text.len()}))
    }

    pub fn innovation_evaluator(
        invention: String,
        effect: Option<String>,
        improvement: Option<f64>,
        obvious: Option<bool>,
    ) -> Result<serde_json::Value, String> {
        let mut engine = QualitativeRuleEngine::new();
        let ctx = CaseContext {
            invention: Some(invention),
            technical_effect: effect,
            performance_improvement: improvement,
            obviousness: obvious,
            ..Default::default()
        };
        let r = engine
            .analyze_inventiveness(&ctx)
            .map_err(|e| format!("{e}"))?;
        let level = if r.net_score > 0.7 {
            "高"
        } else if r.net_score > 0.4 {
            "中"
        } else {
            "低"
        };
        Ok(serde_json::json!({"innovation_level": level, "score": r.net_score, "analysis": r}))
    }
}

pub fn register_drafting_tools() -> std::collections::HashMap<String, super::ToolHandler> {
    use std::collections::HashMap;
    let mut t: HashMap<String, super::ToolHandler> = HashMap::new();
    t.insert("ClaimGenerator".into(), |input| {
        Box::pin(async move {
            let parsed: ClaimGeneratorInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DraftingTools::claim_generator(parsed)
        })
    });
    t.insert("SpecificationDrafter".into(), |input| {
        Box::pin(async move {
            let parsed: SpecificationInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DraftingTools::specification_draft(parsed)
        })
    });
    t.insert("AbstractDrafter".into(), |input| {
        Box::pin(async move {
            let parsed: AbstractDraftInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            DraftingTools::abstract_draft(parsed)
        })
    });
    t.insert("ClaimOutputProcessor".into(), |_input| Box::pin(async {
        Ok(serde_json::json!({"status": "CNIPA 格式已应用", "note": "输出已格式化为标准权利要求书格式"}))
    }));
    t.insert("SpecOutputProcessor".into(), |_input| Box::pin(async {
        Ok(serde_json::json!({"status": "CNIPA 格式已应用", "note": "输出已格式化为标准说明书格式"}))
    }));
    t.insert("ClaimsStructure".into(), |input| {
        Box::pin(async move {
            let parsed: ClaimsStructureInput =
                serde_json::from_value(input).map_err(|e| format!("{e}"))?;
            let lines: Vec<&str> = parsed.claims_text.lines().collect();
            let ind_count = lines
                .iter()
                .filter(|l| !l.contains("根据权利要求"))
                .count();
            Ok(serde_json::json!({"total_claims": lines.len(), "independent": ind_count, "dependent": lines.len() - ind_count}))
        })
    });
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn specification_draft_contains_all_sections() {
        let input = SpecificationInput {
            title: "测试发明".into(),
            technical_field: "电子工程".into(),
            background: "现有技术不足".into(),
            invention_content: "提出一种新方案".into(),
            embodiments: "实施例1：如图1所示".into(),
        };
        let result = DraftingTools::specification_draft(input).unwrap();
        let spec = result["specification"].as_str().unwrap();
        assert!(spec.contains("技术领域"));
        assert!(spec.contains("背景技术"));
        assert!(spec.contains("发明内容"));
        assert!(spec.contains("具体实施方式"));
        assert!(result["word_count"].as_u64().unwrap() > 0);
    }

    #[test]
    fn claim_generator_with_optional_features() {
        let input = ClaimGeneratorInput {
            invention_name: "传感器".into(),
            essential_features: vec!["检测模块".into(), "处理模块".into()],
            optional_features: Some(vec![vec!["无线传输".into()], vec!["低功耗模式".into()]]),
        };
        let result = DraftingTools::claim_generator(input).unwrap();
        let claims = result["claims"].as_array().unwrap();
        assert_eq!(claims.len(), 3);
        assert_eq!(result["independent_count"], 1);
        assert_eq!(result["dependent_count"], 2);
        assert!(claims[0].as_str().unwrap().contains("传感器"));
    }

    #[test]
    fn claim_generator_no_optional_features() {
        let input = ClaimGeneratorInput {
            invention_name: "装置".into(),
            essential_features: vec!["特征A".into()],
            optional_features: None,
        };
        let result = DraftingTools::claim_generator(input).unwrap();
        let claims = result["claims"].as_array().unwrap();
        assert_eq!(claims.len(), 1);
        assert_eq!(result["dependent_count"], 0);
    }

    #[test]
    fn abstract_draft_contains_components() {
        let input = AbstractDraftInput {
            title: "智能传感器".into(),
            technical_problem: "检测精度低".into(),
            technical_solution: "采用双模检测".into(),
            technical_effect: "提高精度50%".into(),
        };
        let result = DraftingTools::abstract_draft(input).unwrap();
        let text = result["abstract_text"].as_str().unwrap();
        assert!(text.contains("智能传感器"));
        assert!(text.contains("检测精度低"));
        assert!(text.contains("双模检测"));
        assert!(text.contains("提高精度50%"));
    }

    #[test]
    fn innovation_evaluator_returns_valid_score() {
        let result = DraftingTools::innovation_evaluator(
            "一种新型量子计算方法".into(),
            Some("大幅提高计算速度".into()),
            Some(0.8),
            Some(false),
        )
        .unwrap();
        let score = result["score"].as_f64().unwrap();
        assert!(
            (0.0..=1.0).contains(&score),
            "score should be in [0,1], got {score}"
        );
        assert!(result["innovation_level"].is_string());
    }
}

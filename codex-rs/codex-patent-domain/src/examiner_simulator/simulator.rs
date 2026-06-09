//! ExaminerSimulator 核心实现：初次审查、权利要求异议生成

use serde_json::Map;
use serde_json::Value;
use strsim::jaro;

use codex_patent_core::RejectionType;

use super::types::*;

impl ExaminerSimulator {
    pub fn new() -> Self {
        Self {
            rejection_type: RejectionType::Inventiveness,
            current_strategy: ArgumentationStrategy::StrictLiteral,
        }
    }

    /// 从审查意见文本检测驳回类型
    pub fn detect_rejection_type(oa_text: &str) -> RejectionType {
        let checks: &[(RejectionType, &[&str])] = &[
            (
                RejectionType::Inventiveness,
                &[
                    "创造性",
                    "专利法第22条第3款",
                    "突出的实质性特点",
                    "显著进步",
                ],
            ),
            (
                RejectionType::Obviousness,
                &["显而易见", "显而易见性", "本领域技术人员容易想到"],
            ),
            (
                RejectionType::LackOfNovelty,
                &["新颖性", "专利法第22条第2款", "相同", "完全公开"],
            ),
            (
                RejectionType::InsufficientDisclosure,
                &["公开不充分", "无法实现", "说明书未清楚记载"],
            ),
            (
                RejectionType::UnpatentableSubject,
                &["智力活动规则", "疾病诊断方法", "不属于专利保护客体"],
            ),
        ];

        for (ty, patterns) in checks {
            if patterns.iter().any(|p| oa_text.contains(p)) {
                return *ty;
            }
        }
        RejectionType::Inventiveness
    }

    /// 模拟初次审查意见(规则层)
    pub fn simulate_initial_review(
        &mut self,
        oa_text: &str,
        claims: &[String],
        prior_art_analysis: &Value,
    ) -> Value {
        self.rejection_type = Self::detect_rejection_type(oa_text);
        self.current_strategy = Self::select_strategy(prior_art_analysis);

        let objections: Vec<ClaimObjection> = claims
            .iter()
            .enumerate()
            .map(|(i, claim)| self.generate_claim_objection(i + 1, claim, prior_art_analysis))
            .collect();

        let output = SimulateReviewOutput {
            rejection_type: rejection_type_as_str(&self.rejection_type),
            strategy: self.current_strategy.as_str(),
            objections,
            overall_conclusion: Self::overall_conclusion(self.rejection_type),
            integration_mode: "rust_rule_layer",
        };
        serde_json::to_value(output)
            .expect("serializing ExaminerSimulator output should never fail")
    }

    pub(crate) fn select_strategy(prior_art_analysis: &Value) -> ArgumentationStrategy {
        let prior_art_count = prior_art_analysis
            .as_object()
            .map(|m| {
                m.keys()
                    .filter(|k| k.starts_with('d') || k.starts_with('D'))
                    .count()
            })
            .unwrap_or(0);

        match prior_art_count {
            0 | 1 => ArgumentationStrategy::StrictLiteral,
            n if n >= 3 => ArgumentationStrategy::CombinationAnalysis,
            _ => ArgumentationStrategy::BroadInterpretation,
        }
    }

    fn generate_claim_objection(
        &self,
        claim_number: usize,
        claim_text: &str,
        prior_art_analysis: &Value,
    ) -> ClaimObjection {
        let features = Self::extract_features_from_claim(claim_text);
        let feature_objections: Vec<String> = features
            .iter()
            .map(|f| {
                let (disclosed, info) = Self::check_disclosure(f, prior_art_analysis);
                if disclosed {
                    Self::disclosure_objection(f, info.as_ref())
                } else {
                    Self::obviousness_objection(f, prior_art_analysis)
                }
            })
            .collect();

        let conclusion = if feature_objections.len() >= 3 {
            "因此,权利要求的技术方案不具备突出的实质性特点和显著的进步,不具备创造性。"
        } else {
            "权利要求的上述技术特征被对比文件公开或属于本领域的常规技术手段。"
        };

        let preview = if claim_text.chars().count() > 100 {
            format!("{}...", claim_text.chars().take(100).collect::<String>())
        } else {
            claim_text.to_string()
        };

        ClaimObjection {
            claim_number,
            claim_text: preview,
            feature_objections,
            conclusion,
        }
    }

    fn disclosure_objection(feature: &str, info: Option<&Map<String, Value>>) -> String {
        let prior_art = info
            .and_then(|m| m.get("priorArt"))
            .and_then(|v| v.as_str())
            .unwrap_or("D1");
        format!(
            "对比文件{prior_art}已经公开了{feature},本领域技术人员根据其教导,容易想到将其应用于本案。"
        )
    }

    fn obviousness_objection(feature: &str, prior_art_analysis: &Value) -> String {
        if let Some(similar) = Self::find_most_similar_feature(feature, prior_art_analysis) {
            format!(
                "对于{feature},本领域技术人员基于对比文件公开的{similar},结合本领域的常规技术手段,无需创造性劳动即可得到。"
            )
        } else {
            format!("{feature}属于本领域的公知常识或常规技术手段。")
        }
    }

    fn extract_features_from_claim(claim_text: &str) -> Vec<String> {
        let parts: Vec<&str> = claim_text
            .split(['，', '。', '；', ',', ';', '\n'])
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();

        parts
            .into_iter()
            .filter_map(|mut part| {
                for prefix in ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'] {
                    if let Some(rest) = part
                        .strip_prefix(prefix)
                        .and_then(|s| s.strip_prefix(['.', '、', '．']))
                    {
                        part = rest;
                        break;
                    }
                }
                let len = part.chars().count();
                if (10..100).contains(&len) {
                    Some(part.to_string())
                } else {
                    None
                }
            })
            .take(5)
            .collect()
    }

    fn check_disclosure(
        feature: &str,
        prior_art_analysis: &Value,
    ) -> (bool, Option<Map<String, Value>>) {
        let Some(obj) = prior_art_analysis.as_object() else {
            return (false, None);
        };

        let prefix = feature.chars().take(30).collect::<String>();

        for (key, value) in obj {
            if !key.to_ascii_lowercase().starts_with('d') {
                continue;
            }
            let undisclosed = value
                .get("undisclosed_features")
                .or_else(|| value.get("undisclosedFeatures"))
                .and_then(|v| v.as_array());

            let hidden = undisclosed.is_some_and(|arr| {
                arr.iter().filter_map(|u| u.as_str()).any(|u| {
                    let u30: String = u.chars().take(30).collect();
                    prefix.contains(&u30) || u30.contains(&prefix)
                })
            });

            if !hidden {
                let mut info = Map::new();
                info.insert("priorArt".into(), Value::String(key.to_uppercase()));
                info.insert("disclosed".into(), Value::Bool(true));
                return (true, Some(info));
            }
        }
        (false, None)
    }

    fn find_most_similar_feature(feature: &str, prior_art_analysis: &Value) -> Option<String> {
        let obj = prior_art_analysis.as_object()?;
        let mut best: Option<(f64, String)> = None;

        for (key, value) in obj {
            if !key.to_ascii_lowercase().starts_with('d') {
                continue;
            }
            let implementation = value
                .get("implementation")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let sim = jaro(feature, implementation);
            if sim > 0.3 && best.as_ref().is_none_or(|(s, _)| sim > *s) {
                let snippet: String = implementation.chars().take(50).collect();
                best = Some((sim, snippet));
            }
        }
        best.map(|(_, s)| s)
    }

    pub(crate) fn overall_conclusion(rejection_type: RejectionType) -> &'static str {
        match rejection_type {
            RejectionType::Inventiveness => {
                "综上所述,本申请权利要求不具备专利法第22条第3款规定的创造性。"
            }
            RejectionType::Obviousness => {
                "综上所述,本申请权利要求的技术方案对本领域技术人员来说是显而易见的。"
            }
            RejectionType::LackOfNovelty => {
                "综上所述,本申请权利要求不具备专利法第22条第2款规定的新颖性。"
            }
            _ => "综上所述,本申请存在上述驳回问题。",
        }
    }
}

#[allow(dead_code)]
pub fn get_objection_templates(rejection: RejectionType) -> Vec<ObjectionTemplate> {
    let mut templates = Vec::new();

    match rejection {
        RejectionType::Inventiveness => {
            templates.push(ObjectionTemplate {
                rejection_type: rejection,
                strategy: ArgumentationStrategy::StrictLiteral,
                templates: vec![
                    "对比文件{d}已经公开了{feature}，本领域技术人员根据其教导，容易想到将其应用于本案。".into(),
                    "{feature}属于本领域的常规技术手段，无需创造性劳动即可获得。".into(),
                ],
                description: "严格字面对比".into(),
            });
            templates.push(ObjectionTemplate {
                rejection_type: rejection,
                strategy: ArgumentationStrategy::CombinationAnalysis,
                templates: vec![
                    "对比文件{d1}给出了{feature1}的技术启示，结合对比文件{d2}的{feature2}，得到本申请权利要求的技术方案是显而易见的。".into(),
                ],
                description: "组合分析".into(),
            });
            templates.push(ObjectionTemplate {
                rejection_type: rejection,
                strategy: ArgumentationStrategy::HindsightBias,
                templates: vec![
                    "在对比文件{d}的基础上，本领域技术人员结合其掌握的公知常识，无需创造性劳动就能得到权利要求的技术方案。".into(),
                    "区别技术特征{feature}仅仅是常用技术手段的简单替换，其效果是本领域技术人员可以预料的。".into(),
                ],
                description: "事后诸葛亮/公知常识".into(),
            });
        }
        RejectionType::Obviousness => {
            templates.push(ObjectionTemplate {
                rejection_type: rejection,
                strategy: ArgumentationStrategy::StrictLiteral,
                templates: vec![
                    "权利要求{claim}的技术方案是对比文件{d}与公知常识的简单组合。".into(),
                ],
                description: "显而易见组合".into(),
            });
        }
        RejectionType::LackOfNovelty => {
            templates.push(ObjectionTemplate {
                rejection_type: rejection,
                strategy: ArgumentationStrategy::StrictLiteral,
                templates: vec![
                    "对比文件{d}已经公开了权利要求{claim}的全部技术特征，因此该权利要求不具备新颖性。".into(),
                ],
                description: "新颖性对比".into(),
            });
        }
        RejectionType::InsufficientDisclosure => {
            templates.push(ObjectionTemplate {
                rejection_type: rejection,
                strategy: ArgumentationStrategy::BroadInterpretation,
                templates: vec![
                    "说明书中未清楚、完整地公开{feature}的具体实施方式，本领域技术人员无法实现该技术方案。".into(),
                ],
                description: "公开不充分".into(),
            });
        }
        RejectionType::UnpatentableSubject => {
            templates.push(ObjectionTemplate {
                rejection_type: rejection,
                strategy: ArgumentationStrategy::StrictLiteral,
                templates: vec![
                    "本申请涉及{feature}，属于智力活动的规则和方法，不属于专利法保护的客体。"
                        .into(),
                ],
                description: "不属于保护客体".into(),
            });
        }
    }

    templates
}

/// 生成特征逐一对比论证
#[allow(dead_code)]
pub fn generate_feature_matching(
    document: &str,
    feature: &str,
    page: Option<&str>,
    line: Option<&str>,
) -> String {
    if let (Some(p), Some(l)) = (page, line) {
        format!("对比文件{document}明确公开了{feature}（参见第{p}页第{l}行）")
    } else {
        format!("对比文件{document}明确公开了{feature}")
    }
}

/// 生成组合对比论证
#[allow(dead_code)]
pub fn generate_combination_analysis(
    doc1: &str,
    feature1: &str,
    doc2: &str,
    feature2: &str,
) -> String {
    format!(
        "对比文件{doc1}公开了{feature1}，对比文件{doc2}公开了{feature2}，本领域技术人员有动机将两者结合。"
    )
}

/// 生成显而易见变型论证
#[allow(dead_code)]
pub fn generate_obvious_variation(feature: &str, prior_art: &str) -> String {
    format!(
        "特征{feature}与对比文件{prior_art}公开的技术方案相比，仅是本领域技术人员的常规设计选择。"
    )
}

/// 生成公知常识追加论证
#[allow(dead_code)]
pub fn generate_common_knowledge_addition(
    doc: &str,
    feature: &str,
    common_knowledge: &str,
) -> String {
    format!(
        "在对比文件{doc}已公开方案的基础上，引入{common_knowledge}中的{feature}是本领域技术人员的常规做法。"
    )
}

impl ArgumentationDialog {
    pub fn new(rejection_type: RejectionType) -> Self {
        Self {
            rounds: Vec::new(),
            rejection_type,
            current_round: 0,
        }
    }

    /// 添加一轮论证
    pub fn add_round(&mut self, strategy: ArgumentationStrategy, template: String) {
        self.current_round += 1;
        self.rounds.push(ArgumentationRound {
            round_number: self.current_round,
            examiner_objection: template.clone(),
            reasoning_template: template,
            strategy,
        });
    }

    /// 获取最后一轮的论证内容
    pub fn last_objection(&self) -> Option<&str> {
        self.rounds.last().map(|r| r.examiner_objection.as_str())
    }

    /// 总轮数
    pub fn len(&self) -> usize {
        self.rounds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rounds.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detect_inventiveness_rejection() {
        let ty = ExaminerSimulator::detect_rejection_type(
            "根据专利法第22条第3款,权利要求1不具备创造性。",
        );
        assert_eq!(ty, RejectionType::Inventiveness);
    }

    #[test]
    fn simulate_initial_review_produces_objections() {
        let mut sim = ExaminerSimulator::new();
        let prior = json!({
            "d1": {
                "undisclosed_features": ["盐水处理", "活性炭"],
                "implementation": "对比文件使用清水处理"
            }
        });
        let claims =
            vec!["1. 一种吊水净化处理罗非鱼泥腥味的方法,包括盐水处理步骤,水温15-25℃。".into()];
        let result = sim.simulate_initial_review(
            "根据专利法第22条第3款的规定,权利要求1不具备创造性。",
            &claims,
            &prior,
        );
        assert_eq!(result["rejectionType"], "inventiveness");
        assert!(result["objections"].as_array().unwrap().len() == 1);
    }

    #[test]
    fn inventiveness_has_multiple_templates() {
        let templates = get_objection_templates(RejectionType::Inventiveness);
        assert!(templates.len() >= 3);
    }

    #[test]
    fn feature_matching_includes_document() {
        let result = generate_feature_matching("CN123", "特征A", Some("5"), Some("10"));
        assert!(result.contains("CN123"));
        assert!(result.contains("特征A"));
        assert!(result.contains("第5页第10行"));
    }

    #[test]
    fn combination_analysis_refs_both_docs() {
        let result = generate_combination_analysis("CN123", "特征A", "CN456", "特征B");
        assert!(result.contains("CN123"));
        assert!(result.contains("CN456"));
    }

    #[test]
    fn argumentation_dialog_tracks_rounds() {
        let mut dialog = ArgumentationDialog::new(RejectionType::Inventiveness);
        assert!(dialog.is_empty());
        dialog.add_round(ArgumentationStrategy::StrictLiteral, "特征A已被公开".into());
        assert_eq!(dialog.len(), 1);
        assert_eq!(dialog.last_objection(), Some("特征A已被公开"));
    }

    #[test]
    fn novelty_template_includes_legal_basis_refs() {
        let templates = get_objection_templates(RejectionType::LackOfNovelty);
        assert_eq!(templates.len(), 1);
        assert!(templates[0].templates[0].contains("不具备新颖性"));
    }
}

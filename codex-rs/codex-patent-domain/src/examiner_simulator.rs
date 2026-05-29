//! 审查员模拟器(规则层)
//!
//! 用于 OA 答复预演与答复质量评估。
//! 不调用 LLM,纯规则引擎实现。

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use strsim::jaro;

use codex_patent_core::RejectionType;

fn rejection_type_as_str(ty: &RejectionType) -> &'static str {
    match ty {
        RejectionType::Inventiveness => "inventiveness",
        RejectionType::Obviousness => "obviousness",
        RejectionType::LackOfNovelty => "lack_of_novelty",
        RejectionType::InsufficientDisclosure => "insufficient_disclosure",
        RejectionType::UnpatentableSubject => "unpatentable_subject",
    }
}

/// 论证策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgumentationStrategy {
    StrictLiteral,
    BroadInterpretation,
    CombinationAnalysis,
    HindsightBias,
}

impl ArgumentationStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StrictLiteral => "strict_literal",
            Self::BroadInterpretation => "broad_interpretation",
            Self::CombinationAnalysis => "combination_analysis",
            Self::HindsightBias => "hindsight_bias",
        }
    }
}

/// 审查员模拟器(规则层)
#[derive(Debug)]
pub struct ExaminerSimulator {
    rejection_type: RejectionType,
    current_strategy: ArgumentationStrategy,
}

impl Default for ExaminerSimulator {
    fn default() -> Self {
        Self::new()
    }
}

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

        let objections: Vec<Value> = claims
            .iter()
            .enumerate()
            .map(|(i, claim)| self.generate_claim_objection(i + 1, claim, prior_art_analysis))
            .collect();

        json!({
            "rejectionType": rejection_type_as_str(&self.rejection_type),
            "strategy": self.current_strategy.as_str(),
            "objections": objections,
            "overallConclusion": Self::overall_conclusion(self.rejection_type),
            "integrationMode": "rust_rule_layer"
        })
    }

    /// 模拟审查员对申请人答复的回应(规则层)
    pub fn respond_to_applicant_argument(
        &self,
        applicant_argument: &str,
        prior_art_analysis: &Value,
        round_number: u32,
    ) -> Value {
        let argument_analysis = Self::analyze_applicant_argument(applicant_argument);
        let response_strategy = Self::determine_response_strategy(&argument_analysis, round_number);
        let rebuttal =
            Self::generate_rebuttal(&argument_analysis, prior_art_analysis, response_strategy);

        json!({
            "roundNumber": round_number,
            "responseStrategy": response_strategy,
            "rebuttal": rebuttal,
            "applicantPointsAddressed": argument_analysis.get("keyPoints").cloned(),
            "integrationMode": "rust_rule_layer"
        })
    }

    /// 评估申请人最终答复质量(规则层,0–100)
    pub fn evaluate_final_response(applicant_response: &str) -> Value {
        let completeness = Self::score_completeness(applicant_response);
        let persuasiveness = Self::score_persuasiveness(applicant_response);
        let technical_depth = Self::score_technical_depth(applicant_response);
        let logic_consistency = Self::score_logic_consistency(applicant_response);

        let overall = completeness * 0.25
            + persuasiveness * 0.30
            + technical_depth * 0.25
            + logic_consistency * 0.20;

        json!({
            "overallScore": overall,
            "scores": {
                "completeness": completeness,
                "persuasiveness": persuasiveness,
                "technicalDepth": technical_depth,
                "logicConsistency": logic_consistency
            },
            "strengths": Self::identify_strengths(applicant_response),
            "weaknesses": Self::identify_weaknesses(applicant_response),
            "recommendations": Self::recommendations(overall),
            "predictedOutcome": Self::predict_outcome(overall),
            "integrationMode": "rust_rule_layer"
        })
    }

    fn select_strategy(prior_art_analysis: &Value) -> ArgumentationStrategy {
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
    ) -> Value {
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

        json!({
            "claimNumber": claim_number,
            "claimText": preview,
            "featureObjections": feature_objections,
            "conclusion": conclusion
        })
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

    fn overall_conclusion(rejection_type: RejectionType) -> &'static str {
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

    fn analyze_applicant_argument(argument: &str) -> Value {
        let mut key_points = Vec::new();
        if argument.contains("四要素") || argument.contains("协同") {
            key_points.push("四要素协同效应");
        }
        if argument.contains("预料不到") || argument.contains("意想不到") {
            key_points.push("预料不到的技术效果");
        }
        if argument.contains("对比文件") && argument.contains("未公开") {
            key_points.push("对比文件未公开");
        }
        if argument.contains("商业成功") {
            key_points.push("商业成功");
        }

        let technical_keywords = ["参数", "工艺", "方法", "机理", "原理"];
        let technical_depth = technical_keywords
            .iter()
            .filter(|kw| argument.contains(*kw))
            .count();

        json!({
            "keyPoints": key_points,
            "technicalDepth": technical_depth,
            "argumentLength": argument.len(),
            "citationCount": argument.matches("参见").count() + argument.matches("如").count()
        })
    }

    fn determine_response_strategy(argument_analysis: &Value, round_number: u32) -> &'static str {
        let depth = argument_analysis
            .get("technicalDepth")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);

        match round_number {
            1 => "strict",
            2 if depth >= 3 => "moderate",
            2 => "strict",
            _ => "flexible",
        }
    }

    fn generate_rebuttal(
        argument_analysis: &Value,
        _prior_art_analysis: &Value,
        strategy: &str,
    ) -> Value {
        let mut rebuttal_points = Vec::new();
        if let Some(points) = argument_analysis
            .get("keyPoints")
            .and_then(|v| v.as_array())
        {
            for point in points {
                let p = point.as_str().unwrap_or("");
                let text = match p {
                    "四要素协同效应" => {
                        "关于四要素协同效应:对比文件已公开各要素的单独作用,本领域技术人员有动机组合使用,协同效果无需创造性劳动。"
                    }
                    "预料不到的技术效果" => {
                        "关于预料不到的技术效果:申请人未提供充分实验数据证明效果预料不到,且效果可通过对比文件教导的常规优化得到。"
                    }
                    "对比文件未公开" => {
                        "关于对比文件未公开:申请人声称的未公开特征,实际上在对比文件中已有明确教导或属于公知常识。"
                    }
                    _ => continue,
                };
                rebuttal_points.push(text);
            }
        }

        let remaining_concerns: Vec<&str> = match strategy {
            "strict" => vec![
                "权利要求的技术方案与对比文件相比差异不明显",
                "技术效果的论述缺乏充分的实验数据支持",
                "未充分说明为何所述技术方案是非显而易见的",
            ],
            "moderate" => vec!["需要进一步补充实验数据证明技术效果的显著性"],
            _ => vec![],
        };

        let suggestions: Vec<&str> = if matches!(strategy, "moderate" | "flexible") {
            vec![
                "建议补充对比实验数据,证明技术效果的显著性",
                "建议详细说明各要素之间的协同机理",
            ]
        } else {
            vec![]
        };

        json!({
            "rebuttalPoints": rebuttal_points,
            "remainingConcerns": remaining_concerns,
            "suggestions": suggestions,
            "tone": strategy
        })
    }

    fn score_completeness(response: &str) -> f64 {
        let elements = ["权利要求", "对比文件", "技术效果", "法律依据"];
        let mut score: f64 = 0.0;
        for el in elements {
            if response.contains(el) {
                score += 25.0;
            }
        }
        score.min(100.0)
    }

    fn score_persuasiveness(response: &str) -> f64 {
        let mut score: f64 = 0.0;
        if ["实验数据", "对比试验", "参数", "效果显著"]
            .iter()
            .any(|kw| response.contains(kw))
        {
            score += 25.0;
        }
        if response.matches("因此").count() + response.matches("综上").count() >= 2 {
            score += 25.0;
        }
        if response.contains("对比文件") && response.contains("参见") {
            score += 20.0;
        }
        if response.contains("专利法") && (response.contains("技术") || response.contains("效果"))
        {
            score += 30.0;
        }
        score.min(100.0)
    }

    fn score_technical_depth(response: &str) -> f64 {
        let kws = [
            "机理", "原理", "参数", "工艺", "方法", "协同", "优化", "效果", "性能", "实验",
        ];
        let mut score =
            (kws.iter().filter(|kw| response.contains(*kw)).count() as f64 * 10.0).min(70.0);
        if ["℃", "%", "g/mL", "h", "min"]
            .iter()
            .any(|u| response.contains(u))
        {
            score += 15.0;
        }
        if response.contains("机理") || response.contains("原理") {
            score += 15.0;
        }
        score.min(100.0)
    }

    fn score_logic_consistency(response: &str) -> f64 {
        let mut score: f64 = 0.0;
        if response.contains("首先") || response.contains("其一") {
            score += 20.0;
        }
        if response.contains("其次") || response.contains("其二") {
            score += 20.0;
        }
        if response.contains("最后") || response.contains("综上") || response.contains("因此")
        {
            score += 20.0;
        }
        if response.contains("因此") && (response.contains("所以") || response.contains("从而"))
        {
            score += 20.0;
        }
        if response.contains("参见") || response.contains("如") {
            score += 20.0;
        }
        score.min(100.0)
    }

    fn identify_strengths(response: &str) -> Vec<&'static str> {
        let mut s = Vec::new();
        if response.contains("实验数据") || response.contains("对比试验") {
            s.push("提供了充分的实验数据支撑");
        }
        if response.contains("机理") || response.contains("原理") {
            s.push("深入分析了技术机理");
        }
        if response.contains("专利法") {
            s.push("正确引用了法律条款");
        }
        if s.is_empty() {
            s.push("答复结构完整");
        }
        s
    }

    fn identify_weaknesses(response: &str) -> Vec<&'static str> {
        let mut w = Vec::new();
        if !response.contains("实验数据") && !response.contains("对比试验") {
            w.push("缺乏充分的实验数据支撑");
        }
        if !response.contains("机理") && !response.contains("原理") {
            w.push("技术机理分析不够深入");
        }
        if response.matches("对比文件").count() < 2 {
            w.push("与对比文件的对比不够详细");
        }
        if w.is_empty() {
            w.push("无明显不足");
        }
        w
    }

    fn recommendations(score: f64) -> Vec<&'static str> {
        if score >= 85.0 {
            vec!["答复质量优秀,可以提交", "建议保持当前论证深度"]
        } else if score >= 70.0 {
            vec![
                "答复质量良好,可考虑进一步优化",
                "建议补充更多实验数据",
                "建议加强技术机理分析",
            ]
        } else {
            vec![
                "答复质量需要改进",
                "必须补充充分的实验数据",
                "必须详细对比与对比文件的差异",
                "必须引用相关法律条款",
            ]
        }
    }

    fn predict_outcome(score: f64) -> &'static str {
        if score >= 85.0 {
            "很有可能获得授权(成功率85%+)"
        } else if score >= 70.0 {
            "有望获得授权(成功率60-85%)"
        } else if score >= 50.0 {
            "存在授权可能(成功率40-60%)"
        } else {
            "授权可能性较低(成功率<40%)"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn respond_to_applicant_argument_round1() {
        let sim = ExaminerSimulator::new();
        let prior = json!({ "d1": { "undisclosed_features": [], "implementation": "清水" } });
        let arg = "四要素产生了协同效果,对比文件未公开活性炭组合。";
        let resp = sim.respond_to_applicant_argument(arg, &prior, 1);
        assert_eq!(resp["responseStrategy"], "strict");
        assert!(
            !resp["rebuttal"]["rebuttalPoints"]
                .as_array()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn evaluate_final_response_scores() {
        let resp = ExaminerSimulator::evaluate_final_response(
            "因此权利要求具备创造性。参见对比文件D1。实验数据显示效果显著。专利法第22条。",
        );
        assert!(resp["overallScore"].as_f64().unwrap() > 0.0);
    }
}

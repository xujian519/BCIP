//! 审查员对申请人答复的回应与论证分析

use serde_json::Value;
use serde_json::json;

use super::types::*;

impl ExaminerSimulator {
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

    pub(crate) fn analyze_applicant_argument(argument: &str) -> Value {
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

    pub(crate) fn determine_response_strategy(
        argument_analysis: &Value,
        round_number: u32,
    ) -> &'static str {
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

    pub(crate) fn generate_rebuttal(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
}

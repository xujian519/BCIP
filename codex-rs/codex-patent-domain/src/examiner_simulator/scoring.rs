//! 评分与策略函数：答复质量评估、强弱项识别、结果预测

use super::types::*;

impl ExaminerSimulator {
    /// 评估申请人最终答复质量(规则层,0–100)
    pub fn evaluate_final_response(applicant_response: &str) -> serde_json::Value {
        let completeness = Self::score_completeness(applicant_response);
        let persuasiveness = Self::score_persuasiveness(applicant_response);
        let technical_depth = Self::score_technical_depth(applicant_response);
        let logic_consistency = Self::score_logic_consistency(applicant_response);

        let overall = completeness * 0.25
            + persuasiveness * 0.30
            + technical_depth * 0.25
            + logic_consistency * 0.20;

        let output = EvaluationOutput {
            overall_score: overall,
            scores: EvaluationScores {
                completeness,
                persuasiveness,
                technical_depth,
                logic_consistency,
            },
            strengths: Self::identify_strengths(applicant_response),
            weaknesses: Self::identify_weaknesses(applicant_response),
            recommendations: Self::recommendations(overall),
            predicted_outcome: Self::predict_outcome(overall),
            integration_mode: "rust_rule_layer",
        };
        serde_json::to_value(output)
            .expect("serializing ExaminerSimulator output should never fail")
    }

    pub(crate) fn score_completeness(response: &str) -> f64 {
        let elements = ["权利要求", "对比文件", "技术效果", "法律依据"];
        let mut score: f64 = 0.0;
        for el in elements {
            if response.contains(el) {
                score += 25.0;
            }
        }
        score.min(100.0)
    }

    pub(crate) fn score_persuasiveness(response: &str) -> f64 {
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

    pub(crate) fn score_technical_depth(response: &str) -> f64 {
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

    pub(crate) fn score_logic_consistency(response: &str) -> f64 {
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

    pub(crate) fn identify_strengths(response: &str) -> Vec<&'static str> {
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

    pub(crate) fn identify_weaknesses(response: &str) -> Vec<&'static str> {
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

    pub(crate) fn recommendations(score: f64) -> Vec<&'static str> {
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

    pub(crate) fn predict_outcome(score: f64) -> &'static str {
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
    fn evaluate_final_response_scores() {
        let resp = ExaminerSimulator::evaluate_final_response(
            "因此权利要求具备创造性。参见对比文件D1。实验数据显示效果显著。专利法第22条。",
        );
        assert!(resp["overallScore"].as_f64().unwrap() > 0.0);
    }
}

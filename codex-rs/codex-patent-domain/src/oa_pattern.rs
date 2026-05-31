//! OA 答复模式提取
//!
//! 从历史 OA 答复轨迹中提取可复用的工作流模式。

use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_order: usize,
    pub step_name: String,
    pub step_type: String,
    pub description: String,
    pub expected_output: String,
}

/// 可复用的工作流模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPattern {
    pub pattern_id: String,
    pub pattern_name: String,
    pub domain: String,
    /// 触发条件（OA 类型等）
    pub trigger_conditions: Vec<String>,
    /// 工作流步骤列表
    pub steps: Vec<WorkflowStep>,
    /// 成功信号
    pub success_signals: Vec<String>,
    /// 复用模板
    pub reusable_templates: Vec<String>,
    /// 提取来源的轨迹数
    pub source_trajectory_count: usize,
}

/// OA 答复轨迹（简化的输入）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OaResponseTrajectory {
    pub trajectory_id: String,
    pub oa_type: String,
    pub strategy: String,
    pub steps: Vec<String>,
    pub outcome: String,
    pub quality_score: f64,
}

/// 模式提取器
pub struct PatternExtractor {
    /// 最小支持度：同样模式至少出现几次才提取
    min_support: usize,
    /// 最小成功率阈值
    min_success_rate: f64,
    stored_trajectories: Vec<OaResponseTrajectory>,
}

impl PatternExtractor {
    pub fn new(min_support: usize, min_success_rate: f64) -> Self {
        Self {
            min_support,
            min_success_rate,
            stored_trajectories: Vec::new(),
        }
    }

    /// 添加一条轨迹
    pub fn add_trajectory(&mut self, trajectory: OaResponseTrajectory) {
        self.stored_trajectories.push(trajectory);
    }

    /// 提取与指定 OA 类型相关的成功模式
    pub fn extract_patterns_for(&self, oa_type: &str) -> Vec<WorkflowPattern> {
        let relevant: Vec<&OaResponseTrajectory> = self
            .stored_trajectories
            .iter()
            .filter(|t| t.oa_type == oa_type && t.outcome == "success")
            .collect();

        if relevant.len() < self.min_support {
            return vec![];
        }

        let mut strategy_groups: HashMap<String, Vec<&&OaResponseTrajectory>> = HashMap::new();
        for t in &relevant {
            strategy_groups
                .entry(t.strategy.clone())
                .or_default()
                .push(t);
        }

        let mut patterns = Vec::new();
        for (strategy, group) in strategy_groups {
            if group.len() < self.min_support {
                continue;
            }

            let avg_quality: f64 =
                group.iter().map(|t| t.quality_score).sum::<f64>() / group.len() as f64;
            if avg_quality < self.min_success_rate {
                continue;
            }

            let common_steps = Self::extract_common_steps(&group);
            let steps: Vec<WorkflowStep> = common_steps
                .iter()
                .enumerate()
                .map(|(i, name)| WorkflowStep {
                    step_order: i + 1,
                    step_name: name.clone(),
                    step_type: "analysis".into(),
                    description: format!("执行 {}", name),
                    expected_output: format!("输出 {}", name),
                })
                .collect();

            patterns.push(WorkflowPattern {
                pattern_id: format!("pattern-{oa_type}-{strategy}"),
                pattern_name: format!("{oa_type}-{strategy} 模式"),
                domain: "patent".into(),
                trigger_conditions: vec![format!("OA 类型={oa_type}")],
                steps,
                success_signals: vec!["答复被审查员接受".into()],
                reusable_templates: vec!["待补充模板内容".into()],
                source_trajectory_count: group.len(),
            });
        }

        patterns
    }

    fn extract_common_steps(trajectories: &[&&OaResponseTrajectory]) -> Vec<String> {
        if trajectories.is_empty() {
            return vec![];
        }
        trajectories[0].steps.clone()
    }

    /// 总轨迹数
    pub fn total_count(&self) -> usize {
        self.stored_trajectories.len()
    }

    /// 导出所有模式为 Markdown
    pub fn to_markdown(&self, oa_type: &str) -> String {
        let patterns = self.extract_patterns_for(oa_type);
        if patterns.is_empty() {
            return format!("# {oa_type}\n\n无提取到的模式。");
        }

        let mut md = format!("# {oa_type} 答复模式\n\n");
        if let Some(first) = patterns.first() {
            md.push_str(&format!(
                "> 来源轨迹数: {}\n\n",
                first.source_trajectory_count
            ));
        }

        for p in &patterns {
            md.push_str(&format!("## {}\n\n", p.pattern_name));
            md.push_str(&format!(
                "**触发条件:** {}\n\n",
                p.trigger_conditions.join(", ")
            ));
            md.push_str("### 工作流步骤\n\n");
            for step in &p.steps {
                md.push_str(&format!(
                    "{}. **{}**: {} → {}\n",
                    step.step_order, step.step_name, step.description, step.expected_output
                ));
            }
            md.push('\n');
        }

        md
    }
}

impl Default for PatternExtractor {
    fn default() -> Self {
        Self::new(3, 0.6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_trajectory(
        id: &str,
        oa_type: &str,
        strategy: &str,
        outcome: &str,
        quality: f64,
    ) -> OaResponseTrajectory {
        OaResponseTrajectory {
            trajectory_id: id.into(),
            oa_type: oa_type.into(),
            strategy: strategy.into(),
            steps: vec![
                "分析审查意见".into(),
                "查找对比文件".into(),
                "撰写答复".into(),
            ],
            outcome: outcome.into(),
            quality_score: quality,
        }
    }

    #[test]
    fn empty_extractor_returns_none() {
        let extractor = PatternExtractor::default();
        assert!(extractor.extract_patterns_for("novelty").is_empty());
    }

    #[test]
    fn insufficient_support_returns_none() {
        let mut extractor = PatternExtractor::new(3, 0.6);
        extractor.add_trajectory(make_trajectory("t1", "novelty", "argue", "success", 0.9));
        assert!(extractor.extract_patterns_for("novelty").is_empty());
    }

    #[test]
    fn sufficient_successful_trajectories_extract_pattern() {
        let mut extractor = PatternExtractor::new(3, 0.6);
        for i in 0..5 {
            extractor.add_trajectory(make_trajectory(
                &format!("t{i}"),
                "novelty",
                "argue",
                "success",
                0.85,
            ));
        }
        let patterns = extractor.extract_patterns_for("novelty");
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].steps.len(), 3);
        assert_eq!(patterns[0].source_trajectory_count, 5);
    }

    #[test]
    fn markdown_output_contains_pattern_name() {
        let mut extractor = PatternExtractor::new(3, 0.6);
        for i in 0..5 {
            extractor.add_trajectory(make_trajectory(
                &format!("t{i}"),
                "creativity",
                "hybrid",
                "success",
                0.9,
            ));
        }
        let md = extractor.to_markdown("creativity");
        assert!(md.contains("creativity-hybrid 模式"));
        assert!(md.contains("分析审查意见"));
    }

    #[test]
    fn failed_trajectories_are_ignored() {
        let mut extractor = PatternExtractor::new(3, 0.6);
        for i in 0..5 {
            extractor.add_trajectory(make_trajectory(
                &format!("t{i}"),
                "novelty",
                "argue",
                "failure",
                0.3,
            ));
        }
        let patterns = extractor.extract_patterns_for("novelty");
        assert!(patterns.is_empty());
    }
}

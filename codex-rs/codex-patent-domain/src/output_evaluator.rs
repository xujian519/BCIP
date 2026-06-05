//! 通用输出评估框架
//!
//! 多维度评估 agent 输出质量，支持规则评估和 LLM-as-Judge：
//! - 6 个评估维度：相关性、准确性、完整性、连贯性、专业性、实用性
//! - 规则引擎：基于模式的快速评估
//! - LLM-as-Judge：语义级别的深度评估（可选）
//! - 评估结果聚合和趋势追踪

use serde::Deserialize;
use serde::Serialize;

/// 单个评估维度的评分
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub dimension: EvaluationDimension,
    pub score: f64,
    pub weight: f64,
    pub issues: Vec<String>,
    pub passed: bool,
}

/// 评估维度枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationDimension {
    /// 相关性：输出是否与任务要求相关
    Relevance,
    /// 准确性：事实和技术信息是否正确
    Accuracy,
    /// 完整性：是否涵盖了任务的所有方面
    Completeness,
    /// 连贯性：逻辑和结构是否清晰
    Coherence,
    /// 专业性：领域术语和知识使用是否恰当
    DomainExpertise,
    /// 实用性：输出是否具有实际参考价值
    Practicality,
}

impl EvaluationDimension {
    pub fn label(&self) -> &str {
        match self {
            Self::Relevance => "相关性",
            Self::Accuracy => "准确性",
            Self::Completeness => "完整性",
            Self::Coherence => "连贯性",
            Self::DomainExpertise => "专业性",
            Self::Practicality => "实用性",
        }
    }

    pub fn default_weight(&self) -> f64 {
        match self {
            Self::Relevance => 0.20,
            Self::Accuracy => 0.25,
            Self::Completeness => 0.20,
            Self::Coherence => 0.10,
            Self::DomainExpertise => 0.15,
            Self::Practicality => 0.10,
        }
    }

    /// 所有维度列表
    pub fn all() -> &'static [EvaluationDimension] {
        &[
            Self::Relevance,
            Self::Accuracy,
            Self::Completeness,
            Self::Coherence,
            Self::DomainExpertise,
            Self::Practicality,
        ]
    }
}

/// 完整的评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// 各维度评分
    pub dimensions: Vec<DimensionScore>,
    /// 加权总分
    pub overall_score: f64,
    /// 是否通过（总分 >= threshold）
    pub passed: bool,
    /// 评估方法
    pub method: EvaluationMethod,
    /// 综合改进建议
    pub suggestions: Vec<String>,
    /// 时间戳
    pub timestamp: String,
}

/// 评估方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationMethod {
    /// 基于规则的快速评估
    RuleBased,
    /// LLM-as-Judge 深度评估
    LlmAsJudge,
    /// 规则 + LLM 混合评估
    Hybrid,
}

/// 评估器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatorConfig {
    /// 通过阈值
    pub pass_threshold: f64,
    /// 是否启用 LLM-as-Judge
    pub enable_llm_judge: bool,
    /// 自定义维度权重（dimension → weight）
    pub custom_weights: std::collections::HashMap<String, f64>,
}

impl Default for EvaluatorConfig {
    fn default() -> Self {
        Self {
            pass_threshold: 0.6,
            enable_llm_judge: false,
            custom_weights: std::collections::HashMap::new(),
        }
    }
}

/// 通用输出评估器
pub struct OutputEvaluator {
    config: EvaluatorConfig,
}

impl OutputEvaluator {
    pub fn new(config: EvaluatorConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建评估器
    pub fn default_evaluator() -> Self {
        Self::new(EvaluatorConfig::default())
    }

    /// 对输出进行全面评估
    pub fn evaluate(
        &self,
        task_description: &str,
        output: &str,
        role: &str,
        domain_keywords: &[&str],
    ) -> EvaluationResult {
        let mut dimensions = Vec::new();

        for &dim in EvaluationDimension::all() {
            let weight = self
                .config
                .custom_weights
                .get(dim.label())
                .copied()
                .unwrap_or_else(|| dim.default_weight());

            let (score, issues) =
                self.evaluate_dimension(dim, task_description, output, role, domain_keywords);
            let passed = score >= self.config.pass_threshold;

            dimensions.push(DimensionScore {
                dimension: dim,
                score,
                weight,
                issues,
                passed,
            });
        }

        let overall_score = dimensions.iter().map(|d| d.score * d.weight).sum::<f64>()
            / dimensions.iter().map(|d| d.weight).sum::<f64>();

        let suggestions = generate_suggestions(&dimensions);

        EvaluationResult {
            dimensions,
            overall_score,
            passed: overall_score >= self.config.pass_threshold,
            method: EvaluationMethod::RuleBased,
            suggestions,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// 生成 LLM-as-Judge 的 prompt（供外部调用）
    pub fn llm_judge_prompt(&self, task_description: &str, output: &str, role: &str) -> String {
        format!(
            "请作为专业评审员评估以下 AI 输出质量。\n\n\
             ## 任务描述\n{task_description}\n\n\
             ## Agent 角色\n{role}\n\n\
             ## 待评估输出\n{output}\n\n\
             ## 评估要求\n\
             请按以下 6 个维度分别打分（0.0-1.0），并指出具体问题：\n\
             1. 相关性 (relevance)：输出是否与任务要求直接相关\n\
             2. 准确性 (accuracy)：事实和技术信息是否正确\n\
             3. 完整性 (completeness)：是否涵盖了任务的所有方面\n\
             4. 连贯性 (coherence)：逻辑和结构是否清晰连贯\n\
             5. 专业性 (domain_expertise)：领域术语和知识使用是否恰当\n\
             6. 实用性 (practicality)：输出是否具有实际参考价值\n\n\
             请以 JSON 格式返回：\n\
             ```json\n\
             {{\n  \
               \"relevance\": {{\"score\": 0.0-1.0, \"issues\": []}},\n  \
               \"accuracy\": {{\"score\": 0.0-1.0, \"issues\": []}},\n  \
               \"completeness\": {{\"score\": 0.0-1.0, \"issues\": []}},\n  \
               \"coherence\": {{\"score\": 0.0-1.0, \"issues\": []}},\n  \
               \"domain_expertise\": {{\"score\": 0.0-1.0, \"issues\": []}},\n  \
               \"practicality\": {{\"score\": 0.0-1.0, \"issues\": []}},\n  \
               \"overall_suggestions\": []\n\
             }}\n\
             ```"
        )
    }

    fn evaluate_dimension(
        &self,
        dim: EvaluationDimension,
        task_description: &str,
        output: &str,
        role: &str,
        domain_keywords: &[&str],
    ) -> (f64, Vec<String>) {
        let mut issues = Vec::new();
        let score = match dim {
            EvaluationDimension::Relevance => {
                evaluate_relevance(task_description, output, &mut issues)
            }
            EvaluationDimension::Accuracy => evaluate_accuracy(output, &mut issues),
            EvaluationDimension::Completeness => {
                evaluate_completeness(task_description, output, &mut issues)
            }
            EvaluationDimension::Coherence => evaluate_coherence(output, &mut issues),
            EvaluationDimension::DomainExpertise => {
                evaluate_domain_expertise(output, role, domain_keywords, &mut issues)
            }
            EvaluationDimension::Practicality => evaluate_practicality(output, &mut issues),
        };
        (score, issues)
    }
}

// ---- 维度评估函数 ----

fn evaluate_relevance(task: &str, output: &str, issues: &mut Vec<String>) -> f64 {
    let task_terms: Vec<&str> = task.split_whitespace().collect();
    if task_terms.is_empty() {
        return 0.5;
    }

    let output_lower = output.to_lowercase();
    let matched = task_terms
        .iter()
        .filter(|t| output_lower.contains(&t.to_lowercase()))
        .count();
    let ratio = matched as f64 / task_terms.len() as f64;

    if ratio < 0.2 {
        issues.push("输出与任务要求关联度很低".to_string());
    } else if ratio < 0.5 {
        issues.push("输出仅部分覆盖任务要求".to_string());
    }

    0.3 + ratio * 0.7
}

fn evaluate_accuracy(output: &str, issues: &mut Vec<String>) -> f64 {
    let mut score: f64 = 0.85;

    // 检测常见的幻觉模式
    let hallucination_patterns = [
        ("根据我的了解", 0.1),
        ("据我所知", 0.05),
        ("一般来说", 0.03),
        ("可能存在", 0.02),
    ];

    for (pattern, penalty) in &hallucination_patterns {
        if output.contains(pattern) {
            score -= penalty;
        }
    }

    // 检测矛盾标记
    let contradiction_markers = ["然而实际上", "但事实上", "实际上并非如此"];
    for marker in &contradiction_markers {
        if output.contains(marker) {
            issues.push(format!("输出中包含矛盾标记: '{marker}'"));
            score -= 0.1;
        }
    }

    score.clamp(0.0, 1.0)
}

fn evaluate_completeness(task: &str, output: &str, issues: &mut Vec<String>) -> f64 {
    let mut score: f64 = 0.7;

    // 输出长度检查
    let output_len = output.chars().count();
    let _task_len = task.chars().count();

    if output_len < 50 {
        score -= 0.3;
        issues.push("输出过短，很可能未完整回答".to_string());
    } else if output_len < 100 {
        score -= 0.1;
    }

    // 任务中的问句/要求标记检查
    let question_marks = task.matches('？').count() + task.matches('?').count();
    if question_marks > 0 {
        // 检查输出是否分段回答了多个问题
        let sections = output.matches("##").count() + output.matches("\n\n").count();
        if sections < question_marks {
            score -= 0.1 * (question_marks - sections) as f64;
            issues.push(format!(
                "任务包含 {question_marks} 个问题，输出可能未全部回答"
            ));
        }
    }

    // 检查是否有"后续"类未完成标记
    let incomplete_markers = ["待续", "未完", "请继续", "继续生成"];
    for marker in &incomplete_markers {
        if output.contains(marker) {
            score -= 0.2;
            issues.push(format!("输出包含未完成标记: '{marker}'"));
        }
    }

    score.clamp(0.0, 1.0)
}

fn evaluate_coherence(output: &str, issues: &mut Vec<String>) -> f64 {
    let mut score: f64 = 0.8;

    let has_headers = output.contains("##") || output.contains("# ");
    let has_lists = output.contains("- ") || output.contains("* ") || output.contains("1. ");
    let has_paragraphs = output.contains("\n\n");
    let len = output.chars().count();

    if len > 300 && !has_headers {
        score -= 0.1;
        issues.push("长输出缺少标题结构".to_string());
    }
    if len > 500 && !has_lists {
        score -= 0.05;
    }
    if len > 200 && !has_paragraphs {
        score -= 0.1;
        issues.push("输出缺少段落分隔".to_string());
    }

    score.clamp(0.0, 1.0)
}

fn evaluate_domain_expertise(
    output: &str,
    role: &str,
    domain_keywords: &[&str],
    issues: &mut Vec<String>,
) -> f64 {
    if domain_keywords.is_empty() {
        return 0.7;
    }

    let output_lower = output.to_lowercase();
    let matched = domain_keywords
        .iter()
        .filter(|k| output_lower.contains(&k.to_lowercase()))
        .count();
    let ratio = matched as f64 / domain_keywords.len() as f64;

    if ratio < 0.2 && output.chars().count() > 200 {
        issues.push(format!(
            "角色 '{role}' 输出中领域关键词覆盖率仅 {:.0}%",
            ratio * 100.0
        ));
    }

    0.3 + ratio * 0.7
}

fn evaluate_practicality(output: &str, issues: &mut Vec<String>) -> f64 {
    let mut score: f64 = 0.7;

    // 实用性标记
    let practicality_markers = [
        "建议", "方案", "步骤", "实施", "具体", "例如", "如下", "first", "second", "步骤", "方法",
    ];
    let has_practical = practicality_markers.iter().any(|m| output.contains(m));
    if has_practical {
        score += 0.15;
    }

    // 空洞标记
    let vague_markers = ["总的来说", "总之", "需要进一步", "有待研究"];
    let vague_count = vague_markers.iter().filter(|m| output.contains(*m)).count();
    if vague_count >= 2 {
        score -= 0.1;
        issues.push("输出包含较多空洞结论".to_string());
    }

    score.clamp(0.0, 1.0)
}

fn generate_suggestions(dimensions: &[DimensionScore]) -> Vec<String> {
    let mut suggestions = Vec::new();

    for dim in dimensions {
        if !dim.passed {
            suggestions.push(format!(
                "【{}】得分 {:.0}%，建议：{}",
                dim.dimension.label(),
                dim.score * 100.0,
                dim.issues.join("；")
            ));
        }
    }

    if suggestions.is_empty() {
        suggestions.push("输出质量良好，各维度均通过评估".to_string());
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_high_quality() {
        let evaluator = OutputEvaluator::default_evaluator();
        let result = evaluator.evaluate(
            "分析专利权利要求的新颖性和创造性",
            "## 新颖性分析\n\n本发明的权利要求1涉及一种图像识别方法，其特征在于使用了深度学习模型。\n\n### 现有技术对比\n\n- 对比文件1（CN1234567A）公开了一种传统的图像识别方法\n- 本发明与对比文件1的区别在于使用了卷积神经网络\n\n## 创造性分析\n\n基于区别特征，本发明具有以下技术效果：提高了识别准确率\n\n建议的答复策略：强调深度学习模型带来的意外技术效果",
            "analyzer",
            &["权利要求", "新颖性", "创造性", "现有技术", "区别特征"],
        );

        assert!(result.overall_score > 0.5, "score={}", result.overall_score);
        assert!(!result.dimensions.is_empty());
    }

    #[test]
    fn test_evaluate_low_quality() {
        let evaluator = OutputEvaluator::default_evaluator();
        let result = evaluator.evaluate(
            "详细分析专利侵权的全部要素",
            "总的来说这个专利可能存在一些问题，需要进一步研究。",
            "analyzer",
            &["侵权", "权利要求", "要素"],
        );

        assert!(result.overall_score < 0.7);
        assert!(!result.suggestions.is_empty());
    }

    #[test]
    fn test_llm_judge_prompt() {
        let evaluator = OutputEvaluator::default_evaluator();
        let prompt = evaluator.llm_judge_prompt("分析新颖性", "输出内容", "analyzer");
        assert!(prompt.contains("relevance"));
        assert!(prompt.contains("accuracy"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_dimension_labels() {
        assert_eq!(EvaluationDimension::Relevance.label(), "相关性");
        assert_eq!(EvaluationDimension::Accuracy.label(), "准确性");
        assert_eq!(EvaluationDimension::Completeness.label(), "完整性");
    }

    #[test]
    fn test_custom_weights() {
        let mut config = EvaluatorConfig::default();
        config.custom_weights.insert("相关性".to_string(), 0.5);
        let evaluator = OutputEvaluator::new(config);
        let result = evaluator.evaluate(
            "分析专利",
            "这是一个关于专利分析的内容，包含了具体的分析步骤和建议方案",
            "analyzer",
            &[],
        );

        let relevance_dim = result
            .dimensions
            .iter()
            .find(|d| d.dimension == EvaluationDimension::Relevance)
            .unwrap();
        assert!((relevance_dim.weight - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_completeness_incomplete_marker() {
        let evaluator = OutputEvaluator::default_evaluator();
        let result = evaluator.evaluate("分析三个问题？如何？为什么？", "待续", "writer", &[]);
        // "待续" 极短输出 + 未完成标记 → 分数应显著低于通过阈值
        assert!(
            result.overall_score < 0.65,
            "incomplete output should score low, got {}",
            result.overall_score
        );
        // 完整性维度应明确不通过
        let completeness = result
            .dimensions
            .iter()
            .find(|d| d.dimension == EvaluationDimension::Completeness)
            .unwrap();
        assert!(!completeness.passed);
    }
}

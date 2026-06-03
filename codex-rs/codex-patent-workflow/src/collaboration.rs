//! Patent Agent 角色间协作 — 预定义工作流模板与角色编排。
//!
//! 基于 9 个 PatentAgentRole 定义常见协作模式：
//! 1. **检索分析链** Retriever → Analyzer → Writer
//! 2. **新颖性检查** Retriever → NoveltyChecker → Reviewer
//! 3. **创造性评估** Retriever → CreativityChecker → Reviewer
//! 4. **侵权分析** Retriever → InfringementChecker → Writer
//! 5. **无效宣告** Retriever → InvalidityChecker → Writer
//! 6. **全面审查** Retriever → (NoveltyChecker ∥ CreativityChecker) → Reviewer → QualityChecker
//!
//! 每个模板生成 `ExecutionPlan`，可由 `Orchestrator` 执行，
//! 同时通过 topic 发布协作事件供 AgentBus 订阅者消费。

use serde::Deserialize;
use serde::Serialize;

use super::flow::FlowStep;
use super::plan::ExecutionPlan;
use super::plan::PlanStep;
use super::plan::PlanStepStatus;
use super::plan::RoutingHint;
use super::plan::WorkflowType;

/// 协作模板标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborationTemplate {
    /// 检索 → 分析 → 撰写
    SearchAnalyzeDraft,
    /// 检索 → 新颖性检查 → 审核
    NoveltyCheck,
    /// 检索 → 创造性评估 → 审核
    CreativityCheck,
    /// 检索 → 侵权分析 → 撰写报告
    InfringementAnalysis,
    /// 检索 → 无效分析 → 撰写无效宣告
    InvalidityAnalysis,
    /// 检索 → (新颖性 ∥ 创造性) → 审核 → 质量检查
    FullReview,
}

impl CollaborationTemplate {
    /// 所有可用模板
    pub fn all() -> &'static [CollaborationTemplate] {
        &[
            CollaborationTemplate::SearchAnalyzeDraft,
            CollaborationTemplate::NoveltyCheck,
            CollaborationTemplate::CreativityCheck,
            CollaborationTemplate::InfringementAnalysis,
            CollaborationTemplate::InvalidityAnalysis,
            CollaborationTemplate::FullReview,
        ]
    }

    /// 模板显示名
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SearchAnalyzeDraft => "检索分析撰写",
            Self::NoveltyCheck => "新颖性检查",
            Self::CreativityCheck => "创造性评估",
            Self::InfringementAnalysis => "侵权分析",
            Self::InvalidityAnalysis => "无效宣告分析",
            Self::FullReview => "全面审查",
        }
    }

    /// 模板描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::SearchAnalyzeDraft => "Retriever 检索 → Analyzer 分析 → Writer 撰写",
            Self::NoveltyCheck => "Retriever 检索 → NoveltyChecker 检查 → Reviewer 审核",
            Self::CreativityCheck => "Retriever 检索 → CreativityChecker 评估 → Reviewer 审核",
            Self::InfringementAnalysis => "Retriever 检索 → InfringementChecker 分析 → Writer 撰写报告",
            Self::InvalidityAnalysis => "Retriever 检索 → InvalidityChecker 分析 → Writer 撰写无效宣告",
            Self::FullReview => "Retriever → (NoveltyChecker ∥ CreativityChecker) → Reviewer → QualityChecker",
        }
    }

    /// 涉及的角色列表（按执行顺序）
    pub fn roles(&self) -> Vec<&'static str> {
        match self {
            Self::SearchAnalyzeDraft => vec!["retriever", "analyzer", "writer"],
            Self::NoveltyCheck => vec!["retriever", "novelty_checker", "reviewer"],
            Self::CreativityCheck => vec!["retriever", "creativity_checker", "reviewer"],
            Self::InfringementAnalysis => vec!["retriever", "infringement_checker", "writer"],
            Self::InvalidityAnalysis => vec!["retriever", "invalidity_checker", "writer"],
            Self::FullReview => vec![
                "retriever",
                "novelty_checker",
                "creativity_checker",
                "reviewer",
                "quality_checker",
            ],
        }
    }

    /// AgentBus 主题名
    pub fn topic(&self) -> String {
        format!(
            "patent.collaboration.{}",
            serde_json::to_value(self)
                .unwrap()
                .as_str()
                .unwrap_or("unknown")
        )
    }

    /// 生成执行计划
    pub fn to_plan(&self, goal: &str) -> ExecutionPlan {
        match self {
            Self::SearchAnalyzeDraft => linear_plan(
                goal,
                "检索分析撰写",
                &[
                    ("retrieve", "retriever", "检索与目标相关的专利文献和现有技术"),
                    ("analyze", "analyzer", "分析检索结果，提取关键技术特征"),
                    ("draft", "writer", "基于分析结果撰写文档"),
                ],
            ),
            Self::NoveltyCheck => linear_plan(
                goal,
                "新颖性检查",
                &[
                    ("retrieve", "retriever", "检索对比文件"),
                    ("check_novelty", "novelty_checker", "评估新颖性"),
                    ("review", "reviewer", "审核新颖性检查结论"),
                ],
            ),
            Self::CreativityCheck => linear_plan(
                goal,
                "创造性评估",
                &[
                    ("retrieve", "retriever", "检索相关现有技术"),
                    ("check_creativity", "creativity_checker", "评估创造性"),
                    ("review", "reviewer", "审核创造性评估结论"),
                ],
            ),
            Self::InfringementAnalysis => linear_plan(
                goal,
                "侵权分析",
                &[
                    ("retrieve", "retriever", "检索涉嫌侵权产品/专利"),
                    ("check_infringement", "infringement_checker", "比对分析侵权要素"),
                    ("draft_report", "writer", "撰写侵权分析报告"),
                ],
            ),
            Self::InvalidityAnalysis => linear_plan(
                goal,
                "无效宣告分析",
                &[
                    ("retrieve", "retriever", "检索用于无效的对比文件"),
                    ("check_invalidity", "invalidity_checker", "分析无效理由"),
                    ("draft_declaration", "writer", "撰写无效宣告请求"),
                ],
            ),
            Self::FullReview => full_review_plan(goal),
        }
    }
}

impl std::fmt::Display for CollaborationTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

/// 构建线性 3 步协作计划
fn linear_plan(
    goal: &str,
    plan_name: &str,
    steps: &[(&str, &str, &str)],
) -> ExecutionPlan {
    let plan_steps: Vec<PlanStep> = steps
        .iter()
        .enumerate()
        .map(|(i, (id, agent, desc))| PlanStep {
            id: id.to_string(),
            description: desc.to_string(),
            step: FlowStep::AgentCall {
                agent_name: agent.to_string(),
                prompt: if i == 0 {
                    format!("{goal}\n\n请基于以上目标执行{desc}。")
                } else {
                    format!(
                        "上游步骤已完成。原始目标: {goal}\n\n请基于上游结果执行{desc}。"
                    )
                },
            },
            depends_on: if i > 0 {
                vec![steps[i - 1].0.to_string()]
            } else {
                vec![]
            },
            assigned_agent: Some(agent.to_string()),
            status: PlanStepStatus::Pending,
        })
        .collect();

    ExecutionPlan {
        id: uuid::Uuid::new_v4().to_string(),
        goal: goal.to_string(),
        steps: plan_steps,
        reasoning: format!("[{plan_name}] 自动生成的协作计划"),
        routing_hint: RoutingHint {
            domain: "patent".into(),
            complexity: "medium".into(),
            workflow: WorkflowType::PlanPlusHitl,
            suggested_tools: vec![],
            suggested_agents: steps.iter().map(|(_, agent, _)| agent.to_string()).collect(),
            reasoning: format!("{plan_name}协作工作流"),
        },
        retry_on_failure: Some(2),
    }
}

/// 全面审查计划 — 带并行分支
///
/// ```text
/// retrieve ─┬─ check_novelty ─┬─ review ── quality_check
///           └─ check_creativity┘
/// ```
fn full_review_plan(goal: &str) -> ExecutionPlan {
    ExecutionPlan {
        id: uuid::Uuid::new_v4().to_string(),
        goal: goal.to_string(),
        steps: vec![
            PlanStep {
                id: "retrieve".into(),
                description: "检索相关专利和现有技术".into(),
                step: FlowStep::AgentCall {
                    agent_name: "retriever".into(),
                    prompt: format!("{goal}\n\n请全面检索相关专利文献和现有技术。"),
                },
                depends_on: vec![],
                assigned_agent: Some("retriever".into()),
                status: PlanStepStatus::Pending,
            },
            PlanStep {
                id: "check_novelty".into(),
                description: "新颖性检查".into(),
                step: FlowStep::AgentCall {
                    agent_name: "novelty_checker".into(),
                    prompt: format!(
                        "上游检索已完成。原始目标: {goal}\n\n请基于检索结果评估新颖性。"
                    ),
                },
                depends_on: vec!["retrieve".into()],
                assigned_agent: Some("novelty_checker".into()),
                status: PlanStepStatus::Pending,
            },
            PlanStep {
                id: "check_creativity".into(),
                description: "创造性评估".into(),
                step: FlowStep::AgentCall {
                    agent_name: "creativity_checker".into(),
                    prompt: format!(
                        "上游检索已完成。原始目标: {goal}\n\n请基于检索结果评估创造性。"
                    ),
                },
                depends_on: vec!["retrieve".into()],
                assigned_agent: Some("creativity_checker".into()),
                status: PlanStepStatus::Pending,
            },
            PlanStep {
                id: "review".into(),
                description: "综合审核新颖性和创造性检查结果".into(),
                step: FlowStep::AgentCall {
                    agent_name: "reviewer".into(),
                    prompt: format!(
                        "新颖性和创造性检查均已完成。原始目标: {goal}\n\n请综合审核所有检查结果。"
                    ),
                },
                depends_on: vec!["check_novelty".into(), "check_creativity".into()],
                assigned_agent: Some("reviewer".into()),
                status: PlanStepStatus::Pending,
            },
            PlanStep {
                id: "quality_check".into(),
                description: "最终质量检查".into(),
                step: FlowStep::AgentCall {
                    agent_name: "quality_checker".into(),
                    prompt: format!(
                        "审核已完成。原始目标: {goal}\n\n请对整个分析过程进行最终质量检查。"
                    ),
                },
                depends_on: vec!["review".into()],
                assigned_agent: Some("quality_checker".into()),
                status: PlanStepStatus::Pending,
            },
        ],
        reasoning: "[全面审查] 自动生成的并行协作计划".into(),
        routing_hint: RoutingHint {
            domain: "patent".into(),
            complexity: "high".into(),
            workflow: WorkflowType::PlanPlusHitl,
            suggested_tools: vec![],
            suggested_agents: vec![
                "retriever".into(),
                "novelty_checker".into(),
                "creativity_checker".into(),
                "reviewer".into(),
                "quality_checker".into(),
            ],
            reasoning: "全面审查协作工作流（含并行分支）".into(),
        },
        retry_on_failure: Some(2),
    }
}

/// 协作模板注册表 — 按名称查找模板
pub struct CollaborationRegistry {
    templates: Vec<CollaborationTemplate>,
}

impl CollaborationRegistry {
    pub fn new() -> Self {
        Self {
            templates: CollaborationTemplate::all().to_vec(),
        }
    }

    /// 按名称查找模板
    pub fn find_by_name(&self, name: &str) -> Option<CollaborationTemplate> {
        self.templates.iter().find(|t| {
            let binding = serde_json::to_value(**t).unwrap();
            let snake = binding.as_str().unwrap_or("");
            snake == name || t.display_name() == name
        }).copied()
    }

    /// 列出所有可用模板
    pub fn list(&self) -> &[CollaborationTemplate] {
        &self.templates
    }

    /// 根据角色推荐模板
    pub fn suggest_for_role(&self, role: &str) -> Vec<CollaborationTemplate> {
        self.templates
            .iter()
            .filter(|t| t.roles().contains(&role))
            .copied()
            .collect()
    }

    /// 从目标自动选择模板
    pub fn suggest_for_goal(&self, goal: &str) -> CollaborationTemplate {
        let lower = goal.to_lowercase();

        if lower.contains("新颖") || lower.contains("novelty") {
            return CollaborationTemplate::NoveltyCheck;
        }
        if lower.contains("创造") || lower.contains("creativity") || lower.contains("obvious") {
            return CollaborationTemplate::CreativityCheck;
        }
        if lower.contains("侵权") || lower.contains("infringement") {
            return CollaborationTemplate::InfringementAnalysis;
        }
        if lower.contains("无效") || lower.contains("invalidity") {
            return CollaborationTemplate::InvalidityAnalysis;
        }
        if lower.contains("全面") || lower.contains("full review") || lower.contains("综合审查") {
            return CollaborationTemplate::FullReview;
        }

        CollaborationTemplate::SearchAnalyzeDraft
    }
}

impl Default for CollaborationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_templates_present() {
        let all = CollaborationTemplate::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_template_display_names() {
        assert_eq!(
            CollaborationTemplate::SearchAnalyzeDraft.display_name(),
            "检索分析撰写"
        );
        assert_eq!(
            CollaborationTemplate::FullReview.display_name(),
            "全面审查"
        );
    }

    #[test]
    fn test_template_roles() {
        let roles = CollaborationTemplate::SearchAnalyzeDraft.roles();
        assert_eq!(roles, vec!["retriever", "analyzer", "writer"]);

        let roles = CollaborationTemplate::FullReview.roles();
        assert_eq!(roles.len(), 5);
        assert!(roles.contains(&"novelty_checker"));
        assert!(roles.contains(&"creativity_checker"));
    }

    #[test]
    fn test_linear_plan_structure() {
        let plan = CollaborationTemplate::SearchAnalyzeDraft.to_plan("分析专利A的新颖性");

        assert!(!plan.id.is_empty());
        assert_eq!(plan.steps.len(), 3);
        assert!(plan.steps[0].depends_on.is_empty());
        assert_eq!(plan.steps[1].depends_on, vec!["retrieve"]);
        assert_eq!(plan.steps[2].depends_on, vec!["analyze"]);

        // 验证无循环
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_full_review_plan_parallel() {
        let plan = CollaborationTemplate::FullReview.to_plan("全面审查专利B");

        assert_eq!(plan.steps.len(), 5);

        // check_novelty 和 check_creativity 都依赖 retrieve
        assert_eq!(plan.steps[1].depends_on, vec!["retrieve"]);
        assert_eq!(plan.steps[2].depends_on, vec!["retrieve"]);

        // review 依赖两个检查步骤
        assert!(plan.steps[3].depends_on.contains(&"check_novelty".to_string()));
        assert!(plan.steps[3].depends_on.contains(&"check_creativity".to_string()));

        // quality_check 依赖 review
        assert_eq!(plan.steps[4].depends_on, vec!["review"]);

        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_registry_find_by_name() {
        let reg = CollaborationRegistry::new();

        assert!(reg.find_by_name("search_analyze_draft").is_some());
        assert!(reg.find_by_name("检索分析撰写").is_some());
        assert!(reg.find_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_registry_suggest_for_role() {
        let reg = CollaborationRegistry::new();

        let retriever_templates = reg.suggest_for_role("retriever");
        assert!(retriever_templates.len() >= 6); // retriever 在所有模板中

        let writer_templates = reg.suggest_for_role("writer");
        assert!(writer_templates.len() >= 3); // SearchAnalyzeDraft, Infringement, Invalidity
    }

    #[test]
    fn test_registry_suggest_for_goal() {
        let reg = CollaborationRegistry::new();

        assert_eq!(
            reg.suggest_for_goal("检查专利新颖性"),
            CollaborationTemplate::NoveltyCheck
        );
        assert_eq!(
            reg.suggest_for_goal("评估创造性"),
            CollaborationTemplate::CreativityCheck
        );
        assert_eq!(
            reg.suggest_for_goal("分析是否侵权"),
            CollaborationTemplate::InfringementAnalysis
        );
        assert_eq!(
            reg.suggest_for_goal("无效宣告"),
            CollaborationTemplate::InvalidityAnalysis
        );
        assert_eq!(
            reg.suggest_for_goal("全面审查"),
            CollaborationTemplate::FullReview
        );
        // 默认回退
        assert_eq!(
            reg.suggest_for_goal("检索相关专利"),
            CollaborationTemplate::SearchAnalyzeDraft
        );
    }

    #[test]
    fn test_topic_format() {
        let topic = CollaborationTemplate::NoveltyCheck.topic();
        assert_eq!(topic, "patent.collaboration.novelty_check");
    }

    #[test]
    fn test_plan_to_graph() {
        let plan = CollaborationTemplate::FullReview.to_plan("测试");
        let graph = plan.to_graph();

        assert_eq!(graph.nodes.len(), 5);
        // retrieve → novelty, retrieve → creativity, novelty → review, creativity → review, review → quality
        assert_eq!(graph.edges.len(), 5);
    }
}

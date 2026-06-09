//! LLM 驱动的计划生成器 + 预定义模板库。
//!
//! 根据用户目标关键词匹配预定义工作流模板，实例化为 `ExecutionPlan`。
//! 无匹配时回退到简单分析+执行两步计划。

use crate::flow::FlowStep;
use crate::plan::{ExecutionPlan, PlanGenerator, PlanStep, PlanStepStatus, RoutingHint};

/// 预定义工作流模板。
#[derive(Debug, Clone)]
pub struct WorkflowTemplate {
    pub name: String,
    pub trigger_keywords: Vec<String>,
    pub steps: Vec<TemplateStep>,
}

/// 模板中的单个步骤。
#[derive(Debug, Clone)]
pub struct TemplateStep {
    pub id: String,
    pub description: String,
    pub step: FlowStep,
    pub depends_on: Vec<String>,
}

/// LLM 驱动的计划生成器。
pub struct LlmPlanGenerator {
    templates: Vec<WorkflowTemplate>,
}

impl Default for LlmPlanGenerator {
    fn default() -> Self {
        Self::new()
    }
}
impl LlmPlanGenerator {
    pub fn new() -> Self {
        Self {
            templates: Self::builtin_templates(),
        }
    }

    fn builtin_templates() -> Vec<WorkflowTemplate> {
        vec![
            // 模板 1: 新颖性检索 + 分析
            WorkflowTemplate {
                name: "novelity_search_analysis".into(),
                trigger_keywords: vec!["新颖性".into(), "查新".into(), "prior art".into()],
                steps: vec![
                    TemplateStep {
                        id: "step_0".into(),
                        description: "检索现有技术".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "PatentSearch".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec![],
                    },
                    TemplateStep {
                        id: "step_1".into(),
                        description: "解析权利要求".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "ClaimParse".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec!["step_0".into()],
                    },
                    TemplateStep {
                        id: "step_2".into(),
                        description: "新颖性分析".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "NoveltyAnalysis".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec!["step_1".into()],
                    },
                    TemplateStep {
                        id: "step_3".into(),
                        description: "生成报告".into(),
                        step: FlowStep::AgentCall {
                            agent_name: "analyst".into(),
                            prompt: "基于检索和分析结果生成新颖性分析报告".into(),
                        },
                        depends_on: vec!["step_2".into()],
                    },
                ],
            },
            // 模板 2: OA 答复
            WorkflowTemplate {
                name: "oa_response".into(),
                trigger_keywords: vec![
                    "审查意见".into(),
                    "OA".into(),
                    "office action".into(),
                    "答复".into(),
                ],
                steps: vec![
                    TemplateStep {
                        id: "step_0".into(),
                        description: "解析审查意见".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "OaParser".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec![],
                    },
                    TemplateStep {
                        id: "step_1".into(),
                        description: "制定答复策略".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "OaStrategist".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec!["step_0".into()],
                    },
                    TemplateStep {
                        id: "step_2".into(),
                        description: "审查员模拟评估".into(),
                        step: FlowStep::QualityCheck {
                            criteria: vec!["persuasiveness".into(), "technical_depth".into()],
                        },
                        depends_on: vec!["step_1".into()],
                    },
                    TemplateStep {
                        id: "step_3".into(),
                        description: "人工审核".into(),
                        step: FlowStep::HumanApproval {
                            title: "审查意见答复".into(),
                            description: "请审核生成的答复".into(),
                            timeout_secs: Some(3600),
                            timeout_action: Default::default(),
                        },
                        depends_on: vec!["step_2".into()],
                    },
                ],
            },
            // 模板 3: 权利要求撰写
            WorkflowTemplate {
                name: "claim_drafting".into(),
                trigger_keywords: vec![
                    "撰写".into(),
                    "权利要求".into(),
                    "drafting".into(),
                    "写专利".into(),
                ],
                steps: vec![
                    TemplateStep {
                        id: "step_0".into(),
                        description: "提取发明内容".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "TechTripleExtractor".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec![],
                    },
                    TemplateStep {
                        id: "step_1".into(),
                        description: "生成权利要求".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "ClaimGenerator".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec!["step_0".into()],
                    },
                    TemplateStep {
                        id: "step_2".into(),
                        description: "说明书撰写".into(),
                        step: FlowStep::ToolCall {
                            tool_name: "SpecificationDrafter".into(),
                            input: serde_json::json!({}),
                        },
                        depends_on: vec!["step_1".into()],
                    },
                    TemplateStep {
                        id: "step_3".into(),
                        description: "质量检查".into(),
                        step: FlowStep::QualityCheck {
                            criteria: vec![
                                "sufficiency".into(),
                                "clarity".into(),
                                "support".into(),
                            ],
                        },
                        depends_on: vec!["step_2".into()],
                    },
                    TemplateStep {
                        id: "step_4".into(),
                        description: "合规检查".into(),
                        step: FlowStep::QualityCheck {
                            criteria: vec!["constitutional_compliance".into()],
                        },
                        depends_on: vec!["step_3".into()],
                    },
                ],
            },
        ]
    }

    /// 匹配模板 — 返回命中关键词数最多的模板（至少命中一个）。
    pub fn match_template(&self, goal: &str) -> Option<&WorkflowTemplate> {
        let goal_lower = goal.to_lowercase();
        self.templates
            .iter()
            .max_by_key(|t| {
                t.trigger_keywords
                    .iter()
                    .filter(|kw| goal_lower.contains(&kw.to_lowercase()))
                    .count()
            })
            .filter(|t| {
                t.trigger_keywords
                    .iter()
                    .any(|kw| goal_lower.contains(&kw.to_lowercase()))
            })
    }

    /// 从模板实例化执行计划。
    fn instantiate_template(&self, goal: &str, template: &WorkflowTemplate) -> ExecutionPlan {
        let steps: Vec<PlanStep> = template
            .steps
            .iter()
            .map(|ts| PlanStep {
                id: ts.id.clone(),
                description: ts.description.clone(),
                step: ts.step.clone(),
                depends_on: ts.depends_on.clone(),
                assigned_agent: match &ts.step {
                    FlowStep::AgentCall { agent_name, .. } => Some(agent_name.clone()),
                    FlowStep::AgentTool { agent_name, .. } => Some(agent_name.clone()),
                    _ => None,
                },
                status: PlanStepStatus::Pending,
            })
            .collect();

        ExecutionPlan {
            id: uuid::Uuid::new_v4().to_string(),
            goal: goal.to_string(),
            steps,
            reasoning: format!("[模板匹配] 使用模板: {}", template.name),
            routing_hint: RoutingHint::default(),
            retry_on_failure: Some(3),
        }
    }
}

impl PlanGenerator for LlmPlanGenerator {
    fn generate(&self, goal: &str) -> Result<ExecutionPlan, String> {
        if let Some(template) = self.match_template(goal) {
            return Ok(self.instantiate_template(goal, template));
        }

        // 无匹配模板时回退到简单分析+执行计划
        Ok(ExecutionPlan {
            id: uuid::Uuid::new_v4().to_string(),
            goal: goal.to_string(),
            steps: vec![
                PlanStep {
                    id: "step_0".into(),
                    description: "分析目标".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "analyst".into(),
                        prompt: goal.to_string(),
                    },
                    depends_on: vec![],
                    assigned_agent: Some("analyst".into()),
                    status: PlanStepStatus::Pending,
                },
                PlanStep {
                    id: "step_1".into(),
                    description: "执行任务".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "executor".into(),
                        prompt: format!("执行: {goal}"),
                    },
                    depends_on: vec!["step_0".into()],
                    assigned_agent: Some("executor".into()),
                    status: PlanStepStatus::Pending,
                },
            ],
            reasoning: format!("[LLM-Plan] 自动生成计划: {goal}"),
            routing_hint: RoutingHint::default(),
            retry_on_failure: Some(3),
        })
    }

    fn name(&self) -> &str {
        "llm_plan_generator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_match_novelty_template() {
        let gen = LlmPlanGenerator::new();
        let plan = gen.generate("进行新颖性查新分析").unwrap();
        assert_eq!(plan.steps.len(), 4, "novelty template should have 4 steps");
        assert!(
            plan.reasoning.contains("novelity_search_analysis"),
            "reasoning should reference novelty template"
        );
        assert_eq!(plan.steps[0].id, "step_0");
        assert!(plan.steps[0].depends_on.is_empty());
        assert_eq!(plan.steps[3].depends_on, vec!["step_2"]);
    }

    #[test]
    fn test_match_oa_template() {
        let gen = LlmPlanGenerator::new();
        let plan = gen.generate("我收到了一份审查意见需要答复").unwrap();
        assert_eq!(plan.steps.len(), 4, "OA template should have 4 steps");
        assert!(
            plan.reasoning.contains("oa_response"),
            "reasoning should reference OA template"
        );
        // 最后一步应为 HumanApproval
        match &plan.steps[3].step {
            FlowStep::HumanApproval { title, .. } => {
                assert_eq!(title, "审查意见答复");
            }
            other => panic!("expected HumanApproval, got {other:?}"),
        }
    }

    #[test]
    fn test_match_claim_drafting_template() {
        let gen = LlmPlanGenerator::new();
        let plan = gen.generate("我需要撰写权利要求").unwrap();
        assert_eq!(
            plan.steps.len(),
            5,
            "claim drafting template should have 5 steps"
        );
        assert!(
            plan.reasoning.contains("claim_drafting"),
            "reasoning should reference claim drafting template"
        );
    }

    #[test]
    fn test_fallback_when_no_match() {
        let gen = LlmPlanGenerator::new();
        let plan = gen.generate("随便做点什么完全无关的事情").unwrap();
        assert_eq!(plan.steps.len(), 2, "fallback plan should have 2 steps");
        assert!(
            plan.reasoning.contains("[LLM-Plan]"),
            "reasoning should indicate fallback"
        );
        assert_eq!(plan.steps[0].assigned_agent.as_deref(), Some("analyst"));
        assert_eq!(plan.steps[1].assigned_agent.as_deref(), Some("executor"));
    }
}

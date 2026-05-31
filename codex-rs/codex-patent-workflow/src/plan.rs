//! 执行计划定义 — 任务分解与计划生成。
//!
//! `ExecutionPlan` 由 LLM 生成，描述如何将高层用户目标分解为
//! 可执行的 Agent/Tool 步骤（支持 DAG 依赖）。

use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use super::flow::FlowStep;
use super::flow::StepResult;

/// 工作流类型 — 决定编排器的执行策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowType {
    #[default]
    Direct,
    Hitl,
    PlanPlusHitl,
}

/// 路由提示 — 从 Router 层注入的决策信息。
///
/// 由上层（cli/runtime）在调用 Orchestrator 时注入，
/// 告知编排器使用哪种执行策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingHint {
    #[serde(default)]
    pub domain: String,
    #[serde(default = "default_complexity")]
    pub complexity: String,
    #[serde(default)]
    pub workflow: WorkflowType,
    #[serde(default)]
    pub suggested_tools: Vec<String>,
    #[serde(default)]
    pub suggested_agents: Vec<String>,
    #[serde(default)]
    pub reasoning: String,
}

fn default_complexity() -> String {
    "medium".to_string()
}

impl Default for RoutingHint {
    fn default() -> Self {
        Self {
            domain: "general".into(),
            complexity: "medium".into(),
            workflow: WorkflowType::Direct,
            suggested_tools: vec![],
            suggested_agents: vec![],
            reasoning: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStepStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub step: FlowStep,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned_agent: Option<String>,
    #[serde(default = "default_status")]
    pub status: PlanStepStatus,
}

fn default_status() -> PlanStepStatus {
    PlanStepStatus::Pending
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub id: String,
    pub goal: String,
    pub steps: Vec<PlanStep>,
    pub reasoning: String,
    #[serde(default)]
    pub routing_hint: RoutingHint,
    #[serde(default)]
    pub retry_on_failure: Option<u32>,
}

impl ExecutionPlan {
    pub fn find_step(&self, id: &str) -> Option<&PlanStep> {
        self.steps.iter().find(|s| s.id == id)
    }

    pub fn find_step_mut(&mut self, id: &str) -> Option<&mut PlanStep> {
        self.steps.iter_mut().find(|s| s.id == id)
    }

    pub fn ready_steps(&self) -> Vec<&PlanStep> {
        let completed: HashSet<&str> = self
            .steps
            .iter()
            .filter(|s| s.status == PlanStepStatus::Completed)
            .map(|s| s.id.as_str())
            .collect();

        self.steps
            .iter()
            .filter(|s| {
                s.status == PlanStepStatus::Pending
                    && s.depends_on
                        .iter()
                        .all(|dep| completed.contains(dep.as_str()))
            })
            .collect()
    }

    pub fn update_step(&mut self, step_id: &str, result: &StepResult) {
        if let Some(step) = self.find_step_mut(step_id) {
            step.status = if result.success {
                PlanStepStatus::Completed
            } else {
                PlanStepStatus::Failed
            };
        }
    }

    pub fn progress(&self) -> f64 {
        let total = self.steps.len();
        if total == 0 {
            return 1.0;
        }
        let done = self
            .steps
            .iter()
            .filter(|s| s.status == PlanStepStatus::Completed)
            .count();
        done as f64 / total as f64
    }

    pub fn all_completed(&self) -> bool {
        self.steps.iter().all(|s| {
            matches!(
                s.status,
                PlanStepStatus::Completed | PlanStepStatus::Skipped
            )
        })
    }

    pub fn has_failures(&self) -> bool {
        self.steps
            .iter()
            .any(|s| s.status == PlanStepStatus::Failed)
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.goal.is_empty() {
            errors.push("计划目标不能为空".into());
        }
        if self.steps.is_empty() {
            errors.push("计划步骤不能为空".into());
        }

        let step_ids: HashSet<&str> = self.steps.iter().map(|s| s.id.as_str()).collect();
        for step in &self.steps {
            for dep in &step.depends_on {
                if !step_ids.contains(dep.as_str()) {
                    errors.push(format!("步骤 '{}' 依赖不存在的步骤 '{}'", step.id, dep));
                }
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        if Self::detect_cycle(&self.steps, &step_ids) {
            return Err(vec!["计划存在循环依赖".into()]);
        }

        Ok(())
    }

    fn detect_cycle(steps: &[PlanStep], all_ids: &HashSet<&str>) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for &id in all_ids {
            if !visited.contains(id) && Self::dfs(id, steps, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        false
    }

    fn dfs<'a>(
        current: &'a str,
        steps: &'a [PlanStep],
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
    ) -> bool {
        visited.insert(current);
        rec_stack.insert(current);

        if let Some(step) = steps.iter().find(|s| s.id == current) {
            for dep in &step.depends_on {
                if !visited.contains(dep.as_str()) {
                    if Self::dfs(dep, steps, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep.as_str()) {
                    return true;
                }
            }
        }

        rec_stack.remove(current);
        false
    }

    pub fn to_graph(&self) -> super::graph::FlowGraph {
        use super::graph::Condition;
        use super::graph::FlowEdge;
        use super::graph::FlowNode;

        let nodes: Vec<FlowNode> = self
            .steps
            .iter()
            .map(|s| FlowNode {
                id: s.id.clone(),
                step: s.step.clone(),
                label: Some(s.description.clone()),
            })
            .collect();

        let mut edges = Vec::new();
        for step in &self.steps {
            for dep in &step.depends_on {
                edges.push(FlowEdge {
                    from: dep.clone(),
                    to: step.id.clone(),
                    condition: Condition::OnSuccess,
                });
            }
        }

        if edges.is_empty() && nodes.len() > 1 {
            for i in 0..nodes.len() - 1 {
                edges.push(FlowEdge {
                    from: nodes[i].id.clone(),
                    to: nodes[i + 1].id.clone(),
                    condition: Condition::Always,
                });
            }
        }

        super::graph::FlowGraph {
            id: self.id.clone(),
            name: self.goal.clone(),
            entry_node: None,
            nodes,
            edges,
            retry_on_failure: self.retry_on_failure,
        }
    }
}

pub trait PlanGenerator: Send {
    fn generate(&self, goal: &str) -> Result<ExecutionPlan, String>;

    fn generate_with_hint(&self, goal: &str, hint: &RoutingHint) -> Result<ExecutionPlan, String> {
        let mut plan = self.generate(goal)?;
        plan.routing_hint = hint.clone();
        Ok(plan)
    }

    fn name(&self) -> &str;
}

pub struct NoopPlanGenerator {
    pub label: String,
}

impl PlanGenerator for NoopPlanGenerator {
    fn generate(&self, goal: &str) -> Result<ExecutionPlan, String> {
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
            reasoning: format!("[NOOP] 自动生成的计划: {goal}"),
            routing_hint: RoutingHint::default(),
            retry_on_failure: Some(3),
        })
    }

    fn name(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan() -> ExecutionPlan {
        ExecutionPlan {
            id: "test-plan".into(),
            goal: "专利检索分析".into(),
            steps: vec![
                PlanStep {
                    id: "search".into(),
                    description: "检索现有技术".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "searcher".into(),
                        prompt: "检索".into(),
                    },
                    depends_on: vec![],
                    assigned_agent: None,
                    status: PlanStepStatus::Pending,
                },
                PlanStep {
                    id: "analyze".into(),
                    description: "分析检索结果".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "analyst".into(),
                        prompt: "分析".into(),
                    },
                    depends_on: vec!["search".into()],
                    assigned_agent: None,
                    status: PlanStepStatus::Pending,
                },
            ],
            reasoning: "测试".into(),
            routing_hint: RoutingHint::default(),
            retry_on_failure: None,
        }
    }

    #[test]
    fn test_ready_steps() {
        let plan = sample_plan();
        let ready = plan.ready_steps();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "search");
    }

    #[test]
    fn test_progress() {
        let plan = sample_plan();
        assert_eq!(plan.progress(), 0.0);

        let mut plan = plan;
        plan.steps[0].status = PlanStepStatus::Completed;
        assert!((plan.progress() - 0.5).abs() < 0.01);

        plan.steps[1].status = PlanStepStatus::Completed;
        assert!((plan.progress() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_validate_ok() {
        let plan = sample_plan();
        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_validate_bad_dependency() {
        let plan = ExecutionPlan {
            id: "bad".into(),
            goal: "测试".into(),
            steps: vec![PlanStep {
                id: "only".into(),
                description: "唯一步骤".into(),
                step: FlowStep::QualityCheck {
                    criteria: vec!["c".into()],
                },
                depends_on: vec!["nonexistent".into()],
                assigned_agent: None,
                status: PlanStepStatus::Pending,
            }],
            reasoning: "".into(),
            routing_hint: RoutingHint::default(),
            retry_on_failure: None,
        };
        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_cycle_detection() {
        let plan = ExecutionPlan {
            id: "cycle".into(),
            goal: "循环测试".into(),
            steps: vec![
                PlanStep {
                    id: "A".into(),
                    description: "A".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["c".into()],
                    },
                    depends_on: vec!["B".into()],
                    assigned_agent: None,
                    status: PlanStepStatus::Pending,
                },
                PlanStep {
                    id: "B".into(),
                    description: "B".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["c".into()],
                    },
                    depends_on: vec!["A".into()],
                    assigned_agent: None,
                    status: PlanStepStatus::Pending,
                },
            ],
            reasoning: "".into(),
            routing_hint: RoutingHint::default(),
            retry_on_failure: None,
        };
        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_to_graph_preserves_dependencies() {
        let plan = sample_plan();
        let graph = plan.to_graph();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from, "search");
        assert_eq!(graph.edges[0].to, "analyze");
    }
}

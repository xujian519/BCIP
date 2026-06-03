//! 工作流编排器 — 组装 规划 → 执行 → 恢复 闭环。
//!
//! Orchestrator 是规划系统的顶层入口，串联：
//! 1. PlanGenerator（LLM 计划生成）
//! 2. ExecutionPlan（任务分解/DAG 依赖）
//! 3. GraphExecutor（图执行引擎）
//! 4. 重试/恢复策略

use super::agent_bridge::AgentExecutor;
use super::assignment::TaskAssignmentStrategy;
use super::checkpoint::CheckpointStore;
use super::graph_executor::CodeExecutor;
use super::graph_executor::GraphExecution;
use super::graph_executor::GraphExecutor;
use super::graph_executor::ToolExecutorFn;
use super::plan::ExecutionPlan;
use super::plan::PlanGenerator;
use super::plan::PlanStepStatus;
use super::plan::RoutingHint;

/// 编排器执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestrationStatus {
    Running,
    Completed,
    Failed,
    Suspended,
}

/// 编排器执行结果
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    pub plan_id: String,
    pub status: OrchestrationStatus,
    pub graph_execution: Option<GraphExecution>,
    pub completed_steps: Vec<String>,
    pub failed_steps: Vec<String>,
    pub progress: f64,
    pub errors: Vec<String>,
}

/// 工作流编排器 — 规划 → 执行 → 恢复 闭环
pub struct Orchestrator {
    plan_generator: Box<dyn PlanGenerator>,
    graph_executor: Option<GraphExecutor>,
    checkpoint_store: Option<CheckpointStore>,
    tool_executor: Option<ToolExecutorFn>,
    agent_executor: Option<Box<dyn AgentExecutor>>,
    code_executor: Option<Box<dyn CodeExecutor>>,
    assignment_strategy: Option<Box<dyn TaskAssignmentStrategy>>,
    max_retries: u32,
    current_plan: Option<ExecutionPlan>,
    routing_hint: Option<RoutingHint>,
}

impl Orchestrator {
    /// 创建编排器，注入计划生成器
    pub fn new(plan_generator: Box<dyn PlanGenerator>) -> Self {
        Self {
            plan_generator,
            graph_executor: None,
            checkpoint_store: None,
            tool_executor: None,
            agent_executor: None,
            code_executor: None,
            assignment_strategy: None,
            max_retries: 3,
            current_plan: None,
            routing_hint: None,
        }
    }

    /// 设置检查点存储
    pub fn with_checkpoint_store(mut self, store: CheckpointStore) -> Self {
        self.checkpoint_store = Some(store);
        self
    }

    /// 设置工具执行器
    pub fn with_tool_executor(mut self, executor: ToolExecutorFn) -> Self {
        self.tool_executor = Some(executor);
        self
    }

    /// 设置 Agent 执行器
    pub fn with_agent_executor(mut self, executor: Box<dyn AgentExecutor>) -> Self {
        self.agent_executor = Some(executor);
        self
    }

    /// 设置代码执行器
    pub fn with_code_executor(mut self, executor: Box<dyn CodeExecutor>) -> Self {
        self.code_executor = Some(executor);
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// 设置路由提示（影响执行策略）
    pub fn with_routing_hint(mut self, hint: RoutingHint) -> Self {
        self.routing_hint = Some(hint);
        self
    }

    /// 设置任务分配策略
    pub fn with_assignment_strategy(mut self, strategy: Box<dyn TaskAssignmentStrategy>) -> Self {
        self.assignment_strategy = Some(strategy);
        self
    }

    fn ensure_executor(&mut self) -> Result<&GraphExecutor, String> {
        if self.graph_executor.is_none() {
            let store = if let Some(s) = self.checkpoint_store.take() {
                s
            } else {
                CheckpointStore::open(std::path::Path::new(
                    ".codex-patent-workflow/checkpoints.db",
                ))
                .map_err(|e| format!("无法创建 CheckpointStore: {e}"))?
            };

            let mut executor = GraphExecutor::new(store);
            if let Some(tool_exec) = self.tool_executor.take() {
                executor = executor.with_tool_executor(tool_exec);
            }
            if let Some(agent_exec) = self.agent_executor.take() {
                executor = executor.with_agent_executor(agent_exec);
            }
            if let Some(code_exec) = self.code_executor.take() {
                executor = executor.with_code_executor(code_exec);
            }
            self.graph_executor = Some(executor);
        }
        self.graph_executor
            .as_ref()
            .ok_or_else(|| "GraphExecutor 未初始化".to_string())
    }

    /// 执行一次完整工作流编排（规划 → 执行 → 返回结果）
    pub fn orchestrate(&mut self, goal: &str) -> Result<OrchestrationResult, String> {
        let mut plan = if let Some(ref hint) = self.routing_hint {
            self.plan_generator.generate_with_hint(goal, hint)?
        } else {
            self.plan_generator.generate(goal)?
        };
        plan.validate().map_err(|errs| errs.join("; "))?;

        let graph = plan.to_graph();
        graph.validate().map_err(|errs| errs.join("; "))?;

        let result = self.execute_plan(&graph)?;

        for node_result in &result.node_results {
            plan.update_step(&node_result.node_id, &node_result.step_result);
        }

        let completed_steps: Vec<String> = plan
            .steps
            .iter()
            .filter(|s| s.status == PlanStepStatus::Completed)
            .map(|s| s.id.clone())
            .collect();

        let failed_steps: Vec<String> = plan
            .steps
            .iter()
            .filter(|s| s.status == PlanStepStatus::Failed)
            .map(|s| s.id.clone())
            .collect();

        let progress = plan.progress();
        let plan_id = plan.id.clone();
        self.current_plan = Some(plan);

        let status = match result.status {
            super::flow::FlowStatus::Completed => OrchestrationStatus::Completed,
            super::flow::FlowStatus::Failed => OrchestrationStatus::Failed,
            super::flow::FlowStatus::Suspended => OrchestrationStatus::Suspended,
            super::flow::FlowStatus::Running => OrchestrationStatus::Running,
            super::flow::FlowStatus::Pending => OrchestrationStatus::Running,
        };

        let errors: Vec<String> = result
            .node_results
            .iter()
            .filter_map(|r| r.step_result.error.clone())
            .collect();

        Ok(OrchestrationResult {
            plan_id,
            status,
            graph_execution: Some(result),
            completed_steps,
            failed_steps,
            progress,
            errors,
        })
    }

    /// 执行工作流编排并自动重试（按配置的最大重试次数）
    pub fn orchestrate_with_retry(&mut self, goal: &str) -> Result<OrchestrationResult, String> {
        let mut last_error = String::new();

        for attempt in 1..=self.max_retries {
            match self.orchestrate(goal) {
                Ok(result) => {
                    if result.status == OrchestrationStatus::Completed {
                        return Ok(result);
                    }
                    if result.status == OrchestrationStatus::Failed && attempt < self.max_retries {
                        continue;
                    }
                    return Ok(result);
                }
                Err(e) => {
                    last_error = e;
                    if attempt < self.max_retries {
                        continue;
                    }
                }
            }
        }

        Err(format!(
            "编排失败，已重试 {max} 次。最后错误: {last}",
            max = self.max_retries,
            last = last_error
        ))
    }

    pub fn resume_execution(
        &mut self,
        graph: &super::graph::FlowGraph,
    ) -> Result<OrchestrationResult, String> {
        let executor = self.ensure_executor()?;
        let result = executor.execute(graph)?;

        let status = match result.status {
            super::flow::FlowStatus::Completed => OrchestrationStatus::Completed,
            super::flow::FlowStatus::Failed => OrchestrationStatus::Failed,
            super::flow::FlowStatus::Suspended => OrchestrationStatus::Suspended,
            _ => OrchestrationStatus::Running,
        };

        Ok(OrchestrationResult {
            plan_id: graph.id.clone(),
            status,
            graph_execution: Some(result),
            completed_steps: vec![],
            failed_steps: vec![],
            progress: 1.0,
            errors: vec![],
        })
    }

    fn execute_plan(&mut self, graph: &super::graph::FlowGraph) -> Result<GraphExecution, String> {
        let executor = self.ensure_executor()?;
        executor.execute(graph)
    }

    pub fn current_plan(&self) -> Option<&ExecutionPlan> {
        self.current_plan.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::super::agent_bridge::NoopAgentExecutor;
    use super::super::plan::NoopPlanGenerator;
    use super::*;

    fn temp_db() -> (tempfile::NamedTempFile, CheckpointStore) {
        let file = tempfile::NamedTempFile::new().unwrap();
        let store = CheckpointStore::open(file.path()).unwrap();
        (file, store)
    }

    #[test]
    fn test_simple_orchestration() {
        let (_file, store) = temp_db();
        let mut orch = Orchestrator::new(Box::new(NoopPlanGenerator {
            label: "test".into(),
        }))
        .with_checkpoint_store(store)
        .with_agent_executor(Box::new(NoopAgentExecutor {
            label: "test".into(),
        }));

        let result = orch.orchestrate("分析专利 A 的新颖性").unwrap();
        assert_eq!(result.status, OrchestrationStatus::Completed);
        assert_eq!(result.completed_steps.len(), 2);
        assert!((result.progress - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_orchestration_with_retry() {
        let (_file, store) = temp_db();
        let mut orch = Orchestrator::new(Box::new(NoopPlanGenerator {
            label: "test".into(),
        }))
        .with_checkpoint_store(store)
        .with_agent_executor(Box::new(NoopAgentExecutor {
            label: "test".into(),
        }))
        .with_max_retries(3);

        let result = orch.orchestrate_with_retry("检索相关专利").unwrap();
        assert_eq!(result.status, OrchestrationStatus::Completed);
    }

    #[test]
    fn test_plan_accessible_after_execution() {
        let (_file, store) = temp_db();
        let mut orch = Orchestrator::new(Box::new(NoopPlanGenerator {
            label: "test".into(),
        }))
        .with_checkpoint_store(store)
        .with_agent_executor(Box::new(NoopAgentExecutor {
            label: "test".into(),
        }));

        orch.orchestrate("测试任务").unwrap();
        let plan = orch.current_plan().unwrap();
        assert!(plan.all_completed());
    }
}

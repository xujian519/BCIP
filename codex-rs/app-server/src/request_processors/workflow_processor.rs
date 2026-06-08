use super::*;
use codex_patent_workflow::agent_bridge::NoopAgentExecutor;
use codex_patent_workflow::graph_executor::ToolExecutorFn;
use codex_patent_workflow::orchestrator::OrchestrationStatus;
use codex_patent_workflow::orchestrator::Orchestrator;
use codex_patent_workflow::plan::NoopPlanGenerator;
use codex_patent_workflow::plan::PlanGenerator;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub(crate) struct WorkflowRequestProcessor {
    active_workflows: Arc<RwLock<HashMap<String, WorkflowState>>>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct WorkflowState {
    plan_id: String,
    status: String,
    progress: f64,
    completed_steps: Vec<String>,
    failed_steps: Vec<String>,
    errors: Vec<String>,
}

impl WorkflowRequestProcessor {
    pub(crate) fn new() -> Self {
        Self {
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub(crate) async fn workflow_start(
        &self,
        params: WorkflowStartParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let workflow_id = uuid::Uuid::new_v4().to_string();

        let plan_generator: Box<dyn PlanGenerator> = Box::new(NoopPlanGenerator {
            label: "default".into(),
        });
        let mut orchestrator = Orchestrator::new(plan_generator);

        // 注入工具执行器（将 async ToolHandler 转为同步 ToolExecutorFn）
        orchestrator = orchestrator.with_tool_executor(create_tool_executor());

        // 注入 Agent 执行器
        orchestrator = orchestrator.with_agent_executor(Box::new(NoopAgentExecutor {
            label: "default".into(),
        }));

        if let Some(max_retries) = params.max_retries {
            orchestrator = orchestrator.with_max_retries(max_retries);
        }

        let result = orchestrator
            .orchestrate_with_retry(&params.goal)
            .map_err(|err| invalid_request(format!("workflow orchestration failed: {err}")))?;

        let status_str = match result.status {
            OrchestrationStatus::Running => "running",
            OrchestrationStatus::Completed => "completed",
            OrchestrationStatus::Failed => "failed",
            OrchestrationStatus::Suspended => "suspended",
        };
        let status_for_state = status_str.to_string();
        let state = WorkflowState {
            plan_id: result.plan_id.clone(),
            status: status_for_state,
            progress: result.progress,
            completed_steps: result.completed_steps,
            failed_steps: result.failed_steps,
            errors: result.errors,
        };

        let mut workflows = self.active_workflows.write().await;
        workflows.insert(workflow_id.clone(), state);

        let status = workflows
            .get(&workflow_id)
            .map(|s| s.status.clone())
            .unwrap_or_default();

        let plan = workflows.get(&workflow_id).and_then(|_w| {
            // Plan details not available from state; return None
            None::<ExecutionPlanDto>
        });

        Ok(Some(
            WorkflowStartResponse {
                workflow_id,
                status,
                plan,
            }
            .into(),
        ))
    }

    pub(crate) async fn workflow_resume(
        &self,
        params: WorkflowResumeParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let workflows = self.active_workflows.read().await;
        let Some(_state) = workflows.get(&params.workflow_id) else {
            return Err(invalid_request(format!(
                "workflow not found: {}",
                params.workflow_id
            )));
        };

        Ok(Some(
            WorkflowResumeResponse {
                workflow_id: params.workflow_id,
                status: "running".to_string(),
            }
            .into(),
        ))
    }

    pub(crate) async fn workflow_status(
        &self,
        params: WorkflowStatusParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let workflows = self.active_workflows.read().await;
        let Some(state) = workflows.get(&params.workflow_id) else {
            return Err(invalid_request(format!(
                "workflow not found: {}",
                params.workflow_id
            )));
        };

        Ok(Some(
            WorkflowStatusResponse {
                workflow_id: params.workflow_id,
                status: state.status.clone(),
                progress: state.progress,
                completed_steps: state.completed_steps.clone(),
                failed_steps: state.failed_steps.clone(),
                errors: state.errors.clone(),
            }
            .into(),
        ))
    }
}

/// 将 patent-tools 的 async ToolHandler 映射转换为 workflow 的同步 ToolExecutorFn。
///
/// 使用 `tokio::task::block_in_place` 在 tokio runtime 中同步执行 async handler，
/// 避免 `Send` trait 约束冲突。
fn create_tool_executor() -> ToolExecutorFn {
    let handlers = Arc::new(codex_patent_tools::register_all_tools());
    Box::new(move |tool_name: &str, input: &serde_json::Value| {
        let handler = match handlers.get(tool_name) {
            Some(h) => *h,
            None => return Err(format!("未知工具: {tool_name}")),
        };
        let input = input.clone();
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(handler(input)))
            .map(|v| serde_json::to_string(&v).unwrap_or_default())
            .map_err(|e| e)
    })
}

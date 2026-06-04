use super::*;
use codex_patent_workflow::orchestrator::OrchestrationStatus;
use codex_patent_workflow::orchestrator::Orchestrator;
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

        let plan_generator = Box::new(SimplePlanGenerator);
        let mut orchestrator = Orchestrator::new(plan_generator);

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

struct SimplePlanGenerator;

impl PlanGenerator for SimplePlanGenerator {
    fn name(&self) -> &str {
        "SimplePlanGenerator"
    }

    fn generate(&self, _goal: &str) -> Result<codex_patent_workflow::plan::ExecutionPlan, String> {
        Err("workflow planning is not yet available".to_string())
    }

    fn generate_with_hint(
        &self,
        _goal: &str,
        _hint: &codex_patent_workflow::plan::RoutingHint,
    ) -> Result<codex_patent_workflow::plan::ExecutionPlan, String> {
        Err("workflow planning is not yet available".to_string())
    }
}

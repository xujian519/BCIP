use crate::checkpoint::Checkpoint;
use crate::flow::FlowResult;
use crate::flow::FlowStatus;
use crate::flow::StepResult;
use crate::graph::FlowGraph;
use crate::graph::GraphNodeResult;

use super::GraphExecution;
use super::GraphExecutor;

impl GraphExecutor {
    pub(super) fn build_checkpoint(
        &self,
        flow_id: &str,
        run_id: &str,
        step_index: usize,
        status: FlowStatus,
        node_results: &[GraphNodeResult],
    ) -> Checkpoint {
        let step_results: Vec<StepResult> = node_results
            .iter()
            .enumerate()
            .map(|(i, r)| StepResult {
                step_index: i,
                success: r.step_result.success,
                output: r.step_result.output.clone(),
                error: r.step_result.error.clone(),
            })
            .collect();

        Checkpoint {
            id: format!("{}-{}-{}", flow_id, run_id, step_index),
            flow_id: flow_id.to_string(),
            run_id: run_id.to_string(),
            step_index,
            state: FlowResult {
                flow_id: flow_id.to_string(),
                status,
                step_results,
                current_step: step_index,
            },
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub(super) fn save_checkpoint(&self, checkpoint: &Checkpoint) {
        if let Err(e) = self.checkpoint_store.save_checkpoint(checkpoint) {
            tracing::warn!(error = %e, "failed to save checkpoint");
        }
    }

    pub fn resume_from_checkpoint(
        &self,
        run_id: &str,
        graph: &FlowGraph,
    ) -> Result<GraphExecution, String> {
        let checkpoint = self
            .checkpoint_store
            .load_checkpoint(run_id)?
            .ok_or_else(|| format!("no checkpoint found for run {}", run_id))?;
        tracing::info!(
            run_id = %run_id,
            flow_id = %checkpoint.flow_id,
            step_index = checkpoint.step_index,
            "resuming from checkpoint"
        );
        graph.validate().map_err(|errs| errs.join("; "))?;
        let entry = graph
            .resolve_entry_node()
            .ok_or_else(|| "无法确定入口节点".to_string())?;
        let run_id = super::super::checkpoint::generate_run_id();
        let levels = graph.topological_levels()?;
        // 从 checkpoint 已完成的步骤恢复 ExecutionState
        let mut state = super::scheduler::ExecutionState::new(entry);
        let mut node_results: Vec<crate::graph::GraphNodeResult> = Vec::new();
        // 恢复已完成步骤的结果和状态
        for step_result in &checkpoint.state.step_results {
            let node_id = format!("node_{}", step_result.step_index);
            state.mark_completed(node_id.clone());
            node_results.push(crate::graph::GraphNodeResult {
                node_id,
                step_result: crate::flow::StepResult {
                    step_index: step_result.step_index,
                    success: step_result.success,
                    output: step_result.output.clone(),
                    error: step_result.error.clone(),
                },
            });
        }
        // 从 checkpoint.step_index 对应的层级继续执行（跳过已完成的层级）
        let start_level = checkpoint.step_index.min(levels.len());
        let max_retries = graph.retry_on_failure.unwrap_or(self.max_retries);
        for level in levels.iter().skip(start_level) {
            if !state.should_continue() {
                break;
            }
            super::scheduler::execute_level(
                self,
                graph,
                &run_id,
                level,
                &mut state,
                &mut node_results,
                max_retries,
            )?;
        }
        let final_status = super::status::determine_final_status(state.suspended, state.failed);
        Ok(super::status::build_execution_result(
            graph,
            run_id,
            final_status,
            node_results,
        ))
    }
}

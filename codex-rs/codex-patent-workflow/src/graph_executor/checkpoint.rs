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

        self.execute(graph)
    }
}

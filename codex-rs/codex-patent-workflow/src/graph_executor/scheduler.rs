//! 调度逻辑 - 层级执行、节点路由、下一节点激活

use std::collections::HashSet;
use std::sync::Arc;

use crate::flow::FlowStatus;
use crate::flow::HumanApprovalTimeoutAction;
use crate::flow::StepResult;
use crate::graph::FlowGraph;
use crate::graph::GraphNodeResult;

use super::step_runner;
use super::GraphExecutor;

/// 执行状态跟踪
#[derive(Debug, Clone, Default)]
pub struct ExecutionState {
    pub completed: HashSet<String>,
    pub active: HashSet<String>,
    pub suspended: bool,
    pub failed: bool,
}

impl ExecutionState {
    pub fn new(entry: String) -> Self {
        Self {
            completed: HashSet::new(),
            active: HashSet::from([entry]),
            suspended: false,
            failed: false,
        }
    }

    pub fn should_continue(&self) -> bool {
        !self.suspended && !self.failed
    }

    pub fn mark_completed(&mut self, node_id: String) {
        self.completed.insert(node_id);
    }

    pub fn activate_next(&mut self, node_ids: Vec<String>) {
        for id in node_ids {
            if !self.completed.contains(&id) {
                self.active.insert(id);
            }
        }
    }

    pub fn set_suspended(&mut self) {
        self.suspended = true;
    }

    pub fn set_failed(&mut self) {
        self.failed = true;
    }
}

/// 查找当前层级中就绪的节点
pub fn find_pending_nodes(level: &[String], state: &ExecutionState) -> Vec<String> {
    level
        .iter()
        .filter(|id| state.active.contains(id.as_str()) && !state.completed.contains(id.as_str()))
        .cloned()
        .collect()
}

/// 执行单个层级（单节点或并行多节点）
pub fn execute_level(
    executor: &GraphExecutor,
    graph: &FlowGraph,
    run_id: &str,
    level: &[String],
    state: &mut ExecutionState,
    node_results: &mut Vec<GraphNodeResult>,
    max_retries: u32,
) -> Result<(), String> {
    let pending = find_pending_nodes(level, state);

    if pending.is_empty() {
        return Ok(());
    }

    if pending.len() == 1 {
        execute_single_node(
            executor,
            graph,
            run_id,
            &pending[0],
            state,
            node_results,
            max_retries,
        )
    } else {
        execute_parallel_nodes(
            executor,
            graph,
            run_id,
            &pending,
            state,
            node_results,
            max_retries,
        )
    }
}

/// 执行单个节点（串行）
fn execute_single_node(
    executor: &GraphExecutor,
    graph: &FlowGraph,
    run_id: &str,
    node_id: &str,
    state: &mut ExecutionState,
    node_results: &mut Vec<GraphNodeResult>,
    max_retries: u32,
) -> Result<(), String> {
    let node = graph
        .find_node(node_id)
        .ok_or_else(|| format!("节点 {} 不存在", node_id))?;

    let step_result = executor.execute_step(&node.step, max_retries)?;

    let success = step_result.success;
    node_results.push(GraphNodeResult {
        node_id: node_id.to_string(),
        step_result,
    });
    state.mark_completed(node_id.to_string());

    executor.save_checkpoint(&executor.build_checkpoint(
        &graph.id,
        run_id,
        node_results.len(),
        FlowStatus::Running,
        node_results,
    ));

    // 检查 HumanApproval 步骤
    if step_runner::node_matches_step(
        &node.step,
        &crate::flow::FlowStep::HumanApproval {
            title: String::new(),
            description: String::new(),
            timeout_secs: None,
            timeout_action: HumanApprovalTimeoutAction::Fail,
        },
    ) {
        executor.save_checkpoint(&executor.build_checkpoint(
            &graph.id,
            run_id,
            node_results.len(),
            FlowStatus::Suspended,
            node_results,
        ));
        state.set_suspended();
        return Ok(());
    }

    // 激活下一节点
    if !success {
        let outgoing = graph.compute_next_nodes(node_id, false);
        let mut handled = false;
        for next_id in &outgoing {
            if !state.completed.contains(next_id) {
                state.active.insert(next_id.clone());
                handled = true;
            }
        }
        if !handled {
            executor.save_checkpoint(&executor.build_checkpoint(
                &graph.id,
                run_id,
                node_results.len(),
                FlowStatus::Failed,
                node_results,
            ));
            state.set_failed();
        }
    } else {
        let next_ids = graph.compute_next_nodes(node_id, true);
        state.activate_next(next_ids);
    }

    Ok(())
}

/// 并行执行多个节点
fn execute_parallel_nodes(
    executor: &GraphExecutor,
    graph: &FlowGraph,
    run_id: &str,
    pending: &[String],
    state: &mut ExecutionState,
    node_results: &mut Vec<GraphNodeResult>,
    max_retries: u32,
) -> Result<(), String> {
    let tool_exec = executor.tool_executor.as_ref().map(Arc::clone);
    let agent_exec = executor.agent_executor.as_ref().map(Arc::clone);
    let code_exec = executor.code_executor.as_ref().map(Arc::clone);

    let (tx, rx) = std::sync::mpsc::channel();
    let max_parallel = executor.max_parallel;
    for chunk in pending.chunks(max_parallel) {
        let tool_exec = tool_exec.clone();
        let agent_exec = agent_exec.clone();
        let code_exec = code_exec.clone();
        let tx = tx.clone();
        std::thread::scope(|s| {
            for node_id in chunk {
                let node = match graph.find_node(node_id) {
                    Some(n) => n,
                    None => {
                        let _ = tx.send((
                            (*node_id).clone(),
                            Err(format!("DAG 节点未找到: {node_id}")),
                        ));
                        continue;
                    }
                };
                let step = node.step.clone();
                let node_id = (*node_id).clone();
                let tx = tx.clone();
                let tool = tool_exec.clone();
                let agent = agent_exec.clone();
                let code = code_exec.clone();
                s.spawn(move || {
                    let result = step_runner::execute_step_from_parts(
                        &step,
                        &tool,
                        &agent,
                        &code,
                        max_retries,
                    );
                    let _ = tx.send((node_id, result));
                });
            }
        });
    }
    drop(tx);

    let mut level_results: Vec<(String, StepResult)> = rx
        .iter()
        .filter_map(|(id, r)| r.ok().map(|r| (id, r)))
        .collect();

    level_results.sort_by_key(|(id, _)| {
        pending
            .iter()
            .position(|p| p.as_str() == id.as_str())
            .unwrap_or(usize::MAX)
    });

    let mut level_suspended = false;
    let mut level_failed = false;

    for (node_id, step_result) in &level_results {
        node_results.push(GraphNodeResult {
            node_id: node_id.clone(),
            step_result: step_result.clone(),
        });
        state.mark_completed(node_id.clone());

        if let Some(node) = graph.find_node(node_id) {
            if step_runner::node_matches_step(
                &node.step,
                &crate::flow::FlowStep::HumanApproval {
                    title: String::new(),
                    description: String::new(),
                    timeout_secs: None,
                    timeout_action: HumanApprovalTimeoutAction::Fail,
                },
            ) {
                level_suspended = true;
            }
        }

        if step_result.success {
            let next_ids = graph.compute_next_nodes(node_id, true);
            state.activate_next(next_ids);
        } else {
            let outgoing = graph.compute_next_nodes(node_id, false);
            if outgoing.is_empty()
                || outgoing
                    .iter()
                    .all(|id| state.completed.contains(id.as_str()))
            {
                level_failed = true;
            } else {
                state.activate_next(outgoing);
            }
        }
    }

    executor.save_checkpoint(&executor.build_checkpoint(
        &graph.id,
        run_id,
        node_results.len(),
        FlowStatus::Running,
        node_results,
    ));

    if level_suspended {
        executor.save_checkpoint(&executor.build_checkpoint(
            &graph.id,
            run_id,
            node_results.len(),
            FlowStatus::Suspended,
            node_results,
        ));
        state.set_suspended();
    } else if level_failed {
        executor.save_checkpoint(&executor.build_checkpoint(
            &graph.id,
            run_id,
            node_results.len(),
            FlowStatus::Failed,
            node_results,
        ));
        state.set_failed();
    }

    Ok(())
}

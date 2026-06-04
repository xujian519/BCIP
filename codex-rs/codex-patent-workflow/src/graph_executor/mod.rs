//! DAG 图执行器 — 按拓扑层级执行 FlowGraph 中的节点。
//!
//! 同一层级内的节点并行执行（通过 std::thread::scope），
//! 拓扑分组按层级展开，各层之间顺序推进。
//!
//! 支持条件路由：节点完成后根据成功/失败出边决定下一层节点。

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use super::agent_bridge::AgentExecutor;
use super::checkpoint::generate_run_id;
use super::checkpoint::CheckpointStore;
use super::flow::FlowStatus;
use super::flow::FlowStep;
use super::flow::HumanApprovalTimeoutAction;
use super::flow::StepResult;
use super::graph::FlowGraph;
use super::graph::GraphNodeResult;

mod checkpoint;
mod code_executor;
mod step_runner;

pub use code_executor::{CodeExecutionResult, CodeExecutor, NoopCodeExecutor};

pub type ToolExecutorFn =
    Box<dyn Fn(&str, &serde_json::Value) -> Result<String, String> + Send + Sync>;

/// 图执行状态
#[derive(Debug, Clone)]
pub struct GraphExecution {
    pub flow_id: String,
    pub status: FlowStatus,
    pub run_id: String,
    pub node_results: Vec<GraphNodeResult>,
}

const MAX_PARALLEL_CAP: usize = 32;

/// DAG 图执行器 — 按拓扑层级并行执行 FlowGraph 节点
pub struct GraphExecutor {
    checkpoint_store: CheckpointStore,
    tool_executor: Option<Arc<ToolExecutorFn>>,
    agent_executor: Option<Arc<Mutex<Box<dyn AgentExecutor>>>>,
    code_executor: Option<Arc<Mutex<Box<dyn CodeExecutor>>>>,
    max_retries: u32,
    max_parallel: usize,
}

impl GraphExecutor {
    /// 创建图执行器，注入检查点存储
    pub fn new(checkpoint_store: CheckpointStore) -> Self {
        Self {
            checkpoint_store,
            tool_executor: None,
            agent_executor: None,
            code_executor: None,
            max_retries: 3,
            max_parallel: 4,
        }
    }

    /// 设置工具执行器
    pub fn with_tool_executor(mut self, executor: ToolExecutorFn) -> Self {
        self.tool_executor = Some(Arc::new(executor));
        self
    }

    /// 设置 Agent 执行器
    pub fn with_agent_executor(mut self, executor: Box<dyn AgentExecutor>) -> Self {
        self.agent_executor = Some(Arc::new(Mutex::new(executor)));
        self
    }

    /// 设置代码执行器
    pub fn with_code_executor(mut self, executor: Box<dyn CodeExecutor>) -> Self {
        self.code_executor = Some(Arc::new(Mutex::new(executor)));
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// 设置同层最大并行节点数（默认 4，上限 32）。
    pub fn with_max_parallel(mut self, limit: usize) -> Self {
        self.max_parallel = limit.max(1).min(MAX_PARALLEL_CAP);
        self
    }

    /// 执行 DAG 图，按拓扑层级并行推进各节点
    pub fn execute(&self, graph: &FlowGraph) -> Result<GraphExecution, String> {
        graph.validate().map_err(|errs| errs.join("; "))?;

        let entry = graph
            .resolve_entry_node()
            .ok_or_else(|| "无法确定入口节点".to_string())?;
        let run_id = generate_run_id();
        let mut node_results: Vec<GraphNodeResult> = Vec::new();

        let max_retries = graph.retry_on_failure.unwrap_or(self.max_retries);

        let levels = graph.topological_levels()?;

        let mut completed: HashSet<String> = HashSet::new();
        let mut active: HashSet<String> = HashSet::from([entry]);
        let mut suspended = false;
        let mut failed = false;

        for level in &levels {
            if suspended || failed {
                break;
            }

            let pending: Vec<&String> = level
                .iter()
                .filter(|id| active.contains(id.as_str()) && !completed.contains(id.as_str()))
                .collect();

            if pending.is_empty() {
                continue;
            }

            if pending.len() == 1 {
                // 单节点：串行执行
                let current = pending[0].clone();
                let node = graph
                    .find_node(&current)
                    .ok_or_else(|| format!("节点 {} 不存在", current))?;

                let step_result = self.execute_step(&node.step, max_retries)?;

                let success = step_result.success;
                node_results.push(GraphNodeResult {
                    node_id: current.clone(),
                    step_result,
                });
                completed.insert(current.clone());

                self.save_checkpoint(&self.build_checkpoint(
                    &graph.id,
                    &run_id,
                    node_results.len(),
                    FlowStatus::Running,
                    &node_results,
                ));

                if step_runner::node_matches_step(
                    &node.step,
                    &FlowStep::HumanApproval {
                        title: String::new(),
                        description: String::new(),
                        timeout_secs: None,
                        timeout_action: HumanApprovalTimeoutAction::Fail,
                    },
                ) {
                    self.save_checkpoint(&self.build_checkpoint(
                        &graph.id,
                        &run_id,
                        node_results.len(),
                        FlowStatus::Suspended,
                        &node_results,
                    ));
                    suspended = true;
                    break;
                }

                if !success {
                    let outgoing = graph.compute_next_nodes(&current, false);
                    let mut handled = false;
                    for next_id in &outgoing {
                        if !completed.contains(next_id) {
                            active.insert(next_id.clone());
                            handled = true;
                        }
                    }
                    if !handled {
                        self.save_checkpoint(&self.build_checkpoint(
                            &graph.id,
                            &run_id,
                            node_results.len(),
                            FlowStatus::Failed,
                            &node_results,
                        ));
                        failed = true;
                        break;
                    }
                } else {
                    for next_id in graph.compute_next_nodes(&current, true) {
                        active.insert(next_id);
                    }
                }
            } else {
                // 多节点：并行执行（受 max_parallel 舱壁限制）
                let tool_exec = self.tool_executor.as_ref().map(Arc::clone);
                let agent_exec = self.agent_executor.as_ref().map(Arc::clone);
                let code_exec = self.code_executor.as_ref().map(Arc::clone);

                let (tx, rx) = std::sync::mpsc::channel();
                let max_parallel = self.max_parallel;
                for chunk in pending.chunks(max_parallel) {
                    let tool_exec = tool_exec.clone();
                    let agent_exec = agent_exec.clone();
                    let code_exec = code_exec.clone();
                    let tx = tx.clone();
                    std::thread::scope(|s| {
                        for node_id in chunk {
                            let node = graph.find_node(node_id).unwrap();
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
                    pending.iter().position(|p| *p == id).unwrap_or(usize::MAX)
                });

                let mut level_suspended = false;
                let mut level_failed = false;

                for (node_id, step_result) in &level_results {
                    node_results.push(GraphNodeResult {
                        node_id: node_id.clone(),
                        step_result: step_result.clone(),
                    });
                    completed.insert(node_id.clone());

                    if let Some(node) = graph.find_node(node_id) {
                        if step_runner::node_matches_step(
                            &node.step,
                            &FlowStep::HumanApproval {
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
                        for next_id in graph.compute_next_nodes(node_id, true) {
                            active.insert(next_id);
                        }
                    } else {
                        let outgoing = graph.compute_next_nodes(node_id, false);
                        if outgoing.is_empty()
                            || outgoing.iter().all(|id| completed.contains(id.as_str()))
                        {
                            level_failed = true;
                        } else {
                            for next_id in &outgoing {
                                active.insert(next_id.clone());
                            }
                        }
                    }
                }

                self.save_checkpoint(&self.build_checkpoint(
                    &graph.id,
                    &run_id,
                    node_results.len(),
                    FlowStatus::Running,
                    &node_results,
                ));

                if level_suspended {
                    self.save_checkpoint(&self.build_checkpoint(
                        &graph.id,
                        &run_id,
                        node_results.len(),
                        FlowStatus::Suspended,
                        &node_results,
                    ));
                    suspended = true;
                    break;
                }
                if level_failed {
                    self.save_checkpoint(&self.build_checkpoint(
                        &graph.id,
                        &run_id,
                        node_results.len(),
                        FlowStatus::Failed,
                        &node_results,
                    ));
                    failed = true;
                    break;
                }
            }
        }

        let status = if suspended {
            FlowStatus::Suspended
        } else if failed {
            FlowStatus::Failed
        } else {
            FlowStatus::Completed
        };

        Ok(GraphExecution {
            flow_id: graph.id.clone(),
            status,
            run_id,
            node_results,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::step_runner::{classify_tool_error, ErrorKind};
    use super::*;
    use crate::agent_bridge::NoopAgentExecutor;
    use crate::checkpoint::CheckpointStore;
    use crate::flow::FlowStatus;
    use crate::flow::HumanApprovalTimeoutAction;
    use crate::graph::Condition;
    use crate::graph::FlowEdge;
    use crate::graph::FlowGraph;
    use crate::graph::FlowNode;

    fn temp_db() -> (tempfile::NamedTempFile, CheckpointStore) {
        let file = tempfile::NamedTempFile::new().unwrap();
        let store = CheckpointStore::open(file.path()).unwrap();
        (file, store)
    }

    fn parallel_graph() -> FlowGraph {
        FlowGraph {
            id: "parallel".into(),
            name: "并行测试".into(),
            entry_node: None,
            nodes: vec![
                FlowNode {
                    id: "start".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "coordinator".into(),
                        prompt: "启动".into(),
                    },
                    label: None,
                },
                FlowNode {
                    id: "branch_a".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "worker_a".into(),
                        prompt: "分支A".into(),
                    },
                    label: None,
                },
                FlowNode {
                    id: "branch_b".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "worker_b".into(),
                        prompt: "分支B".into(),
                    },
                    label: None,
                },
                FlowNode {
                    id: "merge".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["完整性".into()],
                    },
                    label: None,
                },
            ],
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "branch_a".into(),
                    condition: Condition::Always,
                },
                FlowEdge {
                    from: "start".into(),
                    to: "branch_b".into(),
                    condition: Condition::Always,
                },
                FlowEdge {
                    from: "branch_a".into(),
                    to: "merge".into(),
                    condition: Condition::OnSuccess,
                },
                FlowEdge {
                    from: "branch_b".into(),
                    to: "merge".into(),
                    condition: Condition::OnSuccess,
                },
            ],
            retry_on_failure: None,
        }
    }

    #[test]
    fn test_execute_parallel_graph() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
            label: "test".into(),
        }));

        let graph = parallel_graph();
        let result = executor.execute(&graph).unwrap();

        assert_eq!(result.status, FlowStatus::Completed);
        assert_eq!(result.node_results.len(), 4);
        assert!(!result.run_id.is_empty());

        let node_ids: Vec<_> = result
            .node_results
            .iter()
            .map(|r| r.node_id.clone())
            .collect();
        assert_eq!(node_ids[0], "start");
        assert!(node_ids.contains(&"branch_a".to_string()));
        assert!(node_ids.contains(&"branch_b".to_string()));
        assert_eq!(node_ids.last().unwrap(), "merge");
    }

    #[test]
    fn test_graph_with_agent_tool_delegation() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
            label: "test".into(),
        }));

        let graph = FlowGraph {
            id: "delegate".into(),
            name: "委托测试".into(),
            entry_node: None,
            nodes: vec![
                FlowNode {
                    id: "main".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "main_agent".into(),
                        prompt: "主任务".into(),
                    },
                    label: None,
                },
                FlowNode {
                    id: "sub".into(),
                    step: FlowStep::AgentTool {
                        agent_name: "specialist".into(),
                        input: serde_json::json!({"task": "子任务"}),
                    },
                    label: None,
                },
            ],
            edges: vec![FlowEdge {
                from: "main".into(),
                to: "sub".into(),
                condition: Condition::Always,
            }],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert_eq!(result.status, FlowStatus::Completed);
        assert_eq!(result.node_results.len(), 2);
    }

    #[test]
    fn test_conditional_routing_on_failure() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store).with_tool_executor(Box::new(|name, _input| {
            if name == "failing_tool" {
                Err("模拟失败".into())
            } else {
                Ok("成功".into())
            }
        }));

        let graph = FlowGraph {
            id: "conditional".into(),
            name: "条件路由测试".into(),
            entry_node: None,
            nodes: vec![
                FlowNode {
                    id: "check".into(),
                    step: FlowStep::ToolCall {
                        tool_name: "failing_tool".into(),
                        input: serde_json::json!({}),
                    },
                    label: None,
                },
                FlowNode {
                    id: "success_path".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["ok".into()],
                    },
                    label: None,
                },
                FlowNode {
                    id: "failure_path".into(),
                    step: FlowStep::HumanApproval {
                        title: "失败".into(),
                        description: "处理失败".into(),
                        timeout_secs: None,
                        timeout_action: HumanApprovalTimeoutAction::Fail,
                    },
                    label: None,
                },
            ],
            edges: vec![
                FlowEdge {
                    from: "check".into(),
                    to: "success_path".into(),
                    condition: Condition::OnSuccess,
                },
                FlowEdge {
                    from: "check".into(),
                    to: "failure_path".into(),
                    condition: Condition::OnFailure,
                },
            ],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert_eq!(result.status, FlowStatus::Suspended);
        assert_eq!(result.node_results.len(), 2);
        assert!(result
            .node_results
            .iter()
            .any(|r| r.node_id == "failure_path"));
        assert!(!result
            .node_results
            .iter()
            .any(|r| r.node_id == "success_path"));
    }

    #[test]
    fn test_hitl_suspension_in_graph() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
            label: "test".into(),
        }));

        let graph = FlowGraph {
            id: "hitl".into(),
            name: "HITL图".into(),
            entry_node: None,
            nodes: vec![
                FlowNode {
                    id: "step1".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "worker".into(),
                        prompt: "工作".into(),
                    },
                    label: None,
                },
                FlowNode {
                    id: "approval".into(),
                    step: FlowStep::HumanApproval {
                        title: "审批".into(),
                        description: "请审批".into(),
                        timeout_secs: None,
                        timeout_action: HumanApprovalTimeoutAction::Fail,
                    },
                    label: None,
                },
                FlowNode {
                    id: "step2".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "worker".into(),
                        prompt: "继续".into(),
                    },
                    label: None,
                },
            ],
            edges: vec![
                FlowEdge {
                    from: "step1".into(),
                    to: "approval".into(),
                    condition: Condition::Always,
                },
                FlowEdge {
                    from: "approval".into(),
                    to: "step2".into(),
                    condition: Condition::Always,
                },
            ],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert_eq!(result.status, FlowStatus::Suspended);
        assert_eq!(result.node_results.len(), 2);
        assert!(!result.node_results.iter().any(|r| r.node_id == "step2"));
    }

    #[test]
    fn test_classify_tool_error_retryable() {
        assert!(matches!(
            classify_tool_error("connection timed out"),
            ErrorKind::Retryable
        ));
        assert!(matches!(
            classify_tool_error("network error"),
            ErrorKind::Retryable
        ));
        assert!(matches!(
            classify_tool_error("rate limit exceeded"),
            ErrorKind::Retryable
        ));
        assert!(matches!(
            classify_tool_error("HTTP 429 Too Many Requests"),
            ErrorKind::Retryable
        ));
        assert!(matches!(
            classify_tool_error("502 Bad Gateway"),
            ErrorKind::Retryable
        ));
        assert!(matches!(
            classify_tool_error("connection refused"),
            ErrorKind::Retryable
        ));
        assert!(matches!(
            classify_tool_error("broken pipe"),
            ErrorKind::Retryable
        ));
    }

    #[test]
    fn test_classify_tool_error_fatal() {
        assert!(matches!(
            classify_tool_error("invalid argument"),
            ErrorKind::Fatal
        ));
        assert!(matches!(
            classify_tool_error("permission denied"),
            ErrorKind::Fatal
        ));
        assert!(matches!(
            classify_tool_error("file not found"),
            ErrorKind::Fatal
        ));
    }

    #[test]
    fn test_code_block_execution() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store).with_code_executor(Box::new(NoopCodeExecutor));

        let graph = FlowGraph {
            id: "code".into(),
            name: "代码执行测试".into(),
            entry_node: None,
            nodes: vec![FlowNode {
                id: "run_code".into(),
                step: FlowStep::CodeBlock {
                    language: "python".into(),
                    code: "print('hello')".into(),
                },
                label: None,
            }],
            edges: vec![],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert_eq!(result.status, FlowStatus::Completed);
        assert_eq!(result.node_results.len(), 1);
        assert!(result.node_results[0].step_result.success);
        let output = result.node_results[0].step_result.output.as_ref().unwrap();
        assert!(output["output"].as_str().unwrap().contains("NOOP"));
    }

    #[test]
    fn test_noop_code_executor() {
        let mut exec = NoopCodeExecutor;
        let result = exec.execute("rust", "fn main() {}").unwrap();
        assert!(result.success);
        assert!(result.output.contains("NOOP"));
        assert_eq!(result.language, "rust");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_code_block_without_executor() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store);

        let graph = FlowGraph {
            id: "no_exec".into(),
            name: "无执行器测试".into(),
            entry_node: None,
            nodes: vec![FlowNode {
                id: "code_step".into(),
                step: FlowStep::CodeBlock {
                    language: "python".into(),
                    code: "1+1".into(),
                },
                label: None,
            }],
            edges: vec![],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert_eq!(result.status, FlowStatus::Failed);
        assert!(!result.node_results[0].step_result.success);
        assert!(result.node_results[0]
            .step_result
            .error
            .as_ref()
            .unwrap()
            .contains("未注册"));
    }

    #[test]
    fn test_tool_call_without_executor() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store);

        let graph = FlowGraph {
            id: "no_tool".into(),
            name: "无工具执行器测试".into(),
            entry_node: None,
            nodes: vec![FlowNode {
                id: "tool_step".into(),
                step: FlowStep::ToolCall {
                    tool_name: "search".into(),
                    input: serde_json::json!({"q": "test"}),
                },
                label: None,
            }],
            edges: vec![],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert!(!result.node_results[0].step_result.success);
        assert!(result.node_results[0]
            .step_result
            .error
            .as_ref()
            .unwrap()
            .contains("未注册"));
    }

    #[test]
    fn test_with_max_retries_builder() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store)
            .with_tool_executor(Box::new(|_name, _input| Ok("done".into())))
            .with_max_retries(0);

        let graph = FlowGraph {
            id: "retries".into(),
            name: "重试测试".into(),
            entry_node: None,
            nodes: vec![FlowNode {
                id: "tool".into(),
                step: FlowStep::ToolCall {
                    tool_name: "test".into(),
                    input: serde_json::json!({}),
                },
                label: None,
            }],
            edges: vec![],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert_eq!(result.status, FlowStatus::Completed);
        assert!(result.node_results[0].step_result.success);
    }

    #[test]
    fn test_quality_check_always_succeeds() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store);

        let graph = FlowGraph {
            id: "qc".into(),
            name: "质量检查测试".into(),
            entry_node: None,
            nodes: vec![FlowNode {
                id: "check".into(),
                step: FlowStep::QualityCheck {
                    criteria: vec!["完整性".into(), "准确性".into()],
                },
                label: None,
            }],
            edges: vec![],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert_eq!(result.status, FlowStatus::Completed);
        assert!(result.node_results[0].step_result.success);
        let output = result.node_results[0].step_result.output.as_ref().unwrap();
        assert!(output["passed"].as_bool().unwrap());
    }

    #[test]
    fn test_agent_call_without_executor() {
        let (_file, store) = temp_db();
        let executor = GraphExecutor::new(store);

        let graph = FlowGraph {
            id: "no_agent".into(),
            name: "无Agent执行器".into(),
            entry_node: None,
            nodes: vec![FlowNode {
                id: "call".into(),
                step: FlowStep::AgentCall {
                    agent_name: "worker".into(),
                    prompt: "do something".into(),
                },
                label: None,
            }],
            edges: vec![],
            retry_on_failure: None,
        };

        let result = executor.execute(&graph).unwrap();
        assert!(!result.node_results[0].step_result.success);
        assert!(result.node_results[0]
            .step_result
            .error
            .as_ref()
            .unwrap()
            .contains("未注册"));
    }
}

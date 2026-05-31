//! DAG 图执行器 — 按拓扑层级执行 FlowGraph 中的节点。
//!
//! 同一层级内的节点语义上可并行执行；当前采用串行实现，
//! 但拓扑分组已为异步并行化做好准备。
//!
//! 支持条件路由：节点完成后根据成功/失败出边决定下一层节点。

use std::collections::HashSet;

use super::agent_bridge::AgentExecutor;
use super::checkpoint::generate_run_id;
use super::checkpoint::CheckpointStore;
use super::flow::FlowStatus;
use super::flow::FlowStep;
use super::flow::StepResult;
use super::graph::FlowGraph;
use super::graph::GraphNodeResult;

pub type ToolExecutorFn =
    Box<dyn Fn(&str, &serde_json::Value) -> Result<String, String> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct GraphExecution {
    pub flow_id: String,
    pub status: FlowStatus,
    pub run_id: String,
    pub node_results: Vec<GraphNodeResult>,
}

pub struct GraphExecutor {
    #[allow(dead_code)]
    checkpoint_store: CheckpointStore,
    tool_executor: Option<ToolExecutorFn>,
    agent_executor: Option<Box<dyn AgentExecutor>>,
    code_executor: Option<Box<dyn CodeExecutor>>,
}

impl GraphExecutor {
    pub fn new(checkpoint_store: CheckpointStore) -> Self {
        Self {
            checkpoint_store,
            tool_executor: None,
            agent_executor: None,
            code_executor: None,
        }
    }

    pub fn with_tool_executor(mut self, executor: ToolExecutorFn) -> Self {
        self.tool_executor = Some(executor);
        self
    }

    pub fn with_agent_executor(mut self, executor: Box<dyn AgentExecutor>) -> Self {
        self.agent_executor = Some(executor);
        self
    }

    pub fn with_code_executor(mut self, executor: Box<dyn CodeExecutor>) -> Self {
        self.code_executor = Some(executor);
        self
    }

    pub fn execute(&mut self, graph: &FlowGraph) -> Result<GraphExecution, String> {
        graph.validate().map_err(|errs| errs.join("; "))?;

        let _entry = graph
            .resolve_entry_node()
            .ok_or_else(|| "无法确定入口节点".to_string())?;
        let run_id = generate_run_id();
        let mut node_results: Vec<GraphNodeResult> = Vec::new();

        let levels = graph.topological_levels()?;

        let mut completed: HashSet<String> = HashSet::new();
        let mut suspended = false;
        let mut failed = false;

        for level in &levels {
            if suspended || failed {
                break;
            }

            let mut next_nodes: Vec<String> = level.clone();

            while !next_nodes.is_empty() {
                let current = next_nodes.remove(0);
                if completed.contains(&current) {
                    continue;
                }

                let node = graph
                    .find_node(&current)
                    .ok_or_else(|| format!("节点 {} 不存在", current))?;

                let step_result = self.execute_step(&node.step)?;

                let success = step_result.success;
                node_results.push(GraphNodeResult {
                    node_id: current.clone(),
                    step_result,
                });
                completed.insert(current.clone());

                if node_matches_step(
                    &node.step,
                    &FlowStep::HumanApproval {
                        title: String::new(),
                        description: String::new(),
                    },
                ) {
                    suspended = true;
                    break;
                }

                if !success {
                    let outgoing = graph.compute_next_nodes(&current, false);
                    let mut handled = false;
                    for next_id in outgoing {
                        if !completed.contains(&next_id) && !next_nodes.contains(&next_id) {
                            next_nodes.push(next_id);
                            handled = true;
                        }
                    }
                    if !handled {
                        failed = true;
                        break;
                    }
                    continue;
                }

                let outgoing = graph.compute_next_nodes(&current, success);
                for next_id in outgoing {
                    if !completed.contains(&next_id) && !next_nodes.contains(&next_id) {
                        next_nodes.push(next_id);
                    }
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

    fn execute_step(&mut self, step: &FlowStep) -> Result<StepResult, String> {
        match step {
            FlowStep::AgentCall { agent_name, prompt } => {
                if let Some(ref mut agent_exec) = self.agent_executor {
                    match agent_exec.execute(agent_name, prompt) {
                        Ok(result) => Ok(StepResult {
                            step_index: 0,
                            success: result.success,
                            output: Some(serde_json::json!({
                                "agent": result.agent_name,
                                "prompt": result.prompt,
                                "output": result.output,
                                "success": result.success,
                                "error": result.error,
                            })),
                            error: result.error,
                        }),
                        Err(e) => Ok(StepResult {
                            step_index: 0,
                            success: false,
                            output: None,
                            error: Some(format!("Agent 执行失败: {e}")),
                        }),
                    }
                } else {
                    Ok(StepResult {
                        step_index: 0,
                        success: false,
                        output: None,
                        error: Some(format!(
                            "未注册 Agent 执行器，无法执行 agent '{agent_name}'"
                        )),
                    })
                }
            }
            FlowStep::AgentTool { agent_name, input } => {
                if let Some(ref mut agent_exec) = self.agent_executor {
                    let prompt = serde_json::to_string(input).unwrap_or_default();
                    match agent_exec.delegate_to(agent_name, &prompt) {
                        Ok(result) => Ok(StepResult {
                            step_index: 0,
                            success: result.success,
                            output: Some(serde_json::json!({
                                "agent": result.agent_name,
                                "output": result.output,
                            })),
                            error: result.error,
                        }),
                        Err(e) => Ok(StepResult {
                            step_index: 0,
                            success: false,
                            output: None,
                            error: Some(format!("AgentTool 委托失败: {e}")),
                        }),
                    }
                } else {
                    Ok(StepResult {
                        step_index: 0,
                        success: false,
                        output: None,
                        error: Some("未注册 Agent 执行器，无法委托 AgentTool".into()),
                    })
                }
            }
            FlowStep::QualityCheck { criteria } => Ok(StepResult {
                step_index: 0,
                success: true,
                output: Some(serde_json::json!({
                    "criteria": criteria,
                    "passed": true
                })),
                error: None,
            }),
            FlowStep::HumanApproval { title, description } => Ok(StepResult {
                step_index: 0,
                success: true,
                output: Some(serde_json::json!({
                    "type": "human_approval_required",
                    "title": title,
                    "description": description,
                    "suspended": true,
                })),
                error: None,
            }),
            FlowStep::ToolCall { tool_name, input } => {
                if let Some(ref executor) = self.tool_executor {
                    match executor(tool_name, input) {
                        Ok(output) => Ok(StepResult {
                            step_index: 0,
                            success: true,
                            output: Some(serde_json::json!({ "output": output })),
                            error: None,
                        }),
                        Err(e) => Ok(StepResult {
                            step_index: 0,
                            success: false,
                            output: None,
                            error: Some(e),
                        }),
                    }
                } else {
                    Ok(StepResult {
                        step_index: 0,
                        success: false,
                        output: None,
                        error: Some(format!("未注册 Tool 执行器: {tool_name}")),
                    })
                }
            }
            FlowStep::CodeBlock { language, code } => {
                if let Some(ref mut exec) = self.code_executor {
                    match exec.execute(language, code) {
                        Ok(result) => Ok(StepResult {
                            step_index: 0,
                            success: result.success,
                            output: Some(serde_json::json!({
                                "output": result.output,
                                "language": result.language,
                            })),
                            error: result.error,
                        }),
                        Err(e) => Ok(StepResult {
                            step_index: 0,
                            success: false,
                            output: None,
                            error: Some(format!("代码执行失败: {e}")),
                        }),
                    }
                } else {
                    Ok(StepResult {
                        step_index: 0,
                        success: false,
                        output: None,
                        error: Some("未注册代码执行器".into()),
                    })
                }
            }
        }
    }
}

fn node_matches_step(node_step: &FlowStep, target: &FlowStep) -> bool {
    matches!(
        (node_step, target),
        (
            FlowStep::HumanApproval { .. },
            FlowStep::HumanApproval { .. }
        )
    )
}

pub trait CodeExecutor: Send {
    fn execute(&mut self, language: &str, code: &str) -> Result<CodeExecutionResult, String>;
}

#[derive(Debug, Clone)]
pub struct CodeExecutionResult {
    pub output: String,
    pub language: String,
    pub success: bool,
    pub error: Option<String>,
}

pub struct NoopCodeExecutor;

impl CodeExecutor for NoopCodeExecutor {
    fn execute(&mut self, language: &str, code: &str) -> Result<CodeExecutionResult, String> {
        Ok(CodeExecutionResult {
            output: format!("[NOOP] 执行了 {} 代码，长度={}", language, code.len()),
            language: language.to_string(),
            success: true,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_bridge::NoopAgentExecutor;
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
        let mut executor =
            GraphExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
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
        let mut executor =
            GraphExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
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
        let mut executor =
            GraphExecutor::new(store).with_tool_executor(Box::new(|name, _input| {
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
        let mut executor =
            GraphExecutor::new(store).with_agent_executor(Box::new(NoopAgentExecutor {
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
}

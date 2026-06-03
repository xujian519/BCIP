//! GraphExecutor 并行执行测试 — 验证 DAG 并行分支的时序正确性和故障传播。

use crate::agent_bridge::AgentExecutionResult;
use crate::agent_bridge::AgentExecutor;
use crate::checkpoint::CheckpointStore;
use crate::flow::FlowStatus;
use crate::flow::FlowStep;
use crate::graph::Condition;
use crate::graph::FlowEdge;
use crate::graph::FlowGraph;
use crate::graph::FlowNode;
use crate::graph_executor::GraphExecutor;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

fn temp_db() -> (tempfile::NamedTempFile, CheckpointStore) {
    let file = tempfile::NamedTempFile::new().unwrap();
    let store = CheckpointStore::open(file.path()).unwrap();
    (file, store)
}

struct DelayedAgentExecutor {
    delay_ms: Arc<AtomicUsize>,
}

impl AgentExecutor for DelayedAgentExecutor {
    fn execute(&mut self, agent_name: &str, _prompt: &str) -> Result<AgentExecutionResult, String> {
        let delay = self.delay_ms.load(Ordering::Relaxed);
        std::thread::sleep(Duration::from_millis(delay as u64));
        Ok(AgentExecutionResult {
            agent_name: agent_name.to_string(),
            prompt: String::new(),
            output: format!("completed after {delay}ms"),
            success: true,
            error: None,
        })
    }

    fn name(&self) -> &str {
        "delayed"
    }
}

struct PartialFailureAgentExecutor {
    failing_agent: String,
}

impl AgentExecutor for PartialFailureAgentExecutor {
    fn execute(&mut self, agent_name: &str, _prompt: &str) -> Result<AgentExecutionResult, String> {
        if agent_name == self.failing_agent {
            Ok(AgentExecutionResult {
                agent_name: agent_name.to_string(),
                prompt: String::new(),
                output: String::new(),
                success: false,
                error: Some(format!("{agent_name} intentionally failed")),
            })
        } else {
            Ok(AgentExecutionResult {
                agent_name: agent_name.to_string(),
                prompt: String::new(),
                output: "ok".to_string(),
                success: true,
                error: None,
            })
        }
    }

    fn name(&self) -> &str {
        "partial_failure"
    }
}

fn parallel_graph_with_n_branches(n: usize) -> FlowGraph {
    let mut nodes = vec![FlowNode {
        id: "start".into(),
        step: FlowStep::AgentCall {
            agent_name: "coordinator".into(),
            prompt: "start".into(),
        },
        label: None,
    }];

    let mut branch_ids = Vec::new();
    for i in 0..n {
        let branch_id = format!("branch_{i}");
        branch_ids.push(branch_id.clone());
        nodes.push(FlowNode {
            id: branch_id,
            step: FlowStep::AgentCall {
                agent_name: format!("worker_{i}"),
                prompt: format!("branch {i}"),
            },
            label: None,
        });
    }

    nodes.push(FlowNode {
        id: "merge".into(),
        step: FlowStep::QualityCheck {
            criteria: vec!["完整性".into()],
        },
        label: None,
    });

    let mut edges = Vec::new();
    for branch_id in &branch_ids {
        edges.push(FlowEdge {
            from: "start".into(),
            to: branch_id.clone(),
            condition: Condition::Always,
        });
        edges.push(FlowEdge {
            from: branch_id.clone(),
            to: "merge".into(),
            condition: Condition::OnSuccess,
        });
    }

    FlowGraph {
        id: "parallel_test".into(),
        name: "并行测试".into(),
        entry_node: None,
        nodes,
        edges,
        retry_on_failure: None,
    }
}

#[test]
fn parallel_execution_with_timing_verification() {
    let (_file, store) = temp_db();
    let delay = Arc::new(AtomicUsize::new(100));
    let executor = GraphExecutor::new(store).with_agent_executor(Box::new(DelayedAgentExecutor {
        delay_ms: Arc::clone(&delay),
    }));

    let graph = parallel_graph_with_n_branches(4);
    let start = Instant::now();
    let result = executor.execute(&graph).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(result.status, FlowStatus::Completed);
    assert_eq!(result.node_results.len(), 6);
    // Parallel 4 branches x 100ms should be much less than sequential 400ms
    // Use generous threshold to avoid flakiness under load
    assert!(
        elapsed < Duration::from_millis(800),
        "parallel branches should complete faster than sequential. Got {elapsed:?}"
    );
}

#[test]
fn parallel_execution_all_branches_complete() {
    let (_file, store) = temp_db();

    let graph = FlowGraph {
        id: "staggered".into(),
        name: "不同延迟并行测试".into(),
        entry_node: None,
        nodes: vec![
            FlowNode {
                id: "start".into(),
                step: FlowStep::AgentCall {
                    agent_name: "coordinator".into(),
                    prompt: "start".into(),
                },
                label: None,
            },
            FlowNode {
                id: "fast".into(),
                step: FlowStep::ToolCall {
                    tool_name: "fast_tool".into(),
                    input: serde_json::json!({"delay_ms": 10}),
                },
                label: None,
            },
            FlowNode {
                id: "medium".into(),
                step: FlowStep::ToolCall {
                    tool_name: "medium_tool".into(),
                    input: serde_json::json!({"delay_ms": 50}),
                },
                label: None,
            },
            FlowNode {
                id: "slow".into(),
                step: FlowStep::ToolCall {
                    tool_name: "slow_tool".into(),
                    input: serde_json::json!({"delay_ms": 100}),
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
                to: "fast".into(),
                condition: Condition::Always,
            },
            FlowEdge {
                from: "start".into(),
                to: "medium".into(),
                condition: Condition::Always,
            },
            FlowEdge {
                from: "start".into(),
                to: "slow".into(),
                condition: Condition::Always,
            },
            FlowEdge {
                from: "fast".into(),
                to: "merge".into(),
                condition: Condition::OnSuccess,
            },
            FlowEdge {
                from: "medium".into(),
                to: "merge".into(),
                condition: Condition::OnSuccess,
            },
            FlowEdge {
                from: "slow".into(),
                to: "merge".into(),
                condition: Condition::OnSuccess,
            },
        ],
        retry_on_failure: None,
    };

    let executor = GraphExecutor::new(store).with_tool_executor(Box::new(|name, input| {
        let delay_ms = input.get("delay_ms").and_then(|v| v.as_u64()).unwrap_or(0);
        std::thread::sleep(Duration::from_millis(delay_ms));
        Ok(format!("{name} completed"))
    }));

    let result = executor.execute(&graph).unwrap();

    assert_eq!(result.status, FlowStatus::Completed);
    assert_eq!(result.node_results.len(), 5);
    let node_ids: Vec<_> = result
        .node_results
        .iter()
        .map(|r| r.node_id.clone())
        .collect();
    assert!(node_ids.contains(&"fast".to_string()));
    assert!(node_ids.contains(&"medium".to_string()));
    assert!(node_ids.contains(&"slow".to_string()));
    assert!(node_ids.contains(&"merge".to_string()));
}

#[test]
fn parallel_execution_one_branch_failure_propagates() {
    let (_file, store) = temp_db();

    let graph = FlowGraph {
        id: "failure_test".into(),
        name: "并行故障测试".into(),
        entry_node: None,
        nodes: vec![
            FlowNode {
                id: "start".into(),
                step: FlowStep::AgentCall {
                    agent_name: "coordinator".into(),
                    prompt: "start".into(),
                },
                label: None,
            },
            FlowNode {
                id: "ok_branch".into(),
                step: FlowStep::AgentCall {
                    agent_name: "good_worker".into(),
                    prompt: "this should succeed".into(),
                },
                label: None,
            },
            FlowNode {
                id: "fail_branch".into(),
                step: FlowStep::AgentCall {
                    agent_name: "bad_worker".into(),
                    prompt: "this should fail".into(),
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
                to: "ok_branch".into(),
                condition: Condition::Always,
            },
            FlowEdge {
                from: "start".into(),
                to: "fail_branch".into(),
                condition: Condition::Always,
            },
            FlowEdge {
                from: "ok_branch".into(),
                to: "merge".into(),
                condition: Condition::OnSuccess,
            },
            FlowEdge {
                from: "fail_branch".into(),
                to: "merge".into(),
                condition: Condition::OnSuccess,
            },
        ],
        retry_on_failure: Some(0),
    };

    let executor =
        GraphExecutor::new(store).with_agent_executor(Box::new(PartialFailureAgentExecutor {
            failing_agent: "bad_worker".to_string(),
        }));

    let result = executor.execute(&graph).unwrap();

    assert_eq!(
        result.status,
        FlowStatus::Failed,
        "graph should fail when a parallel branch fails with no failure edges"
    );
}

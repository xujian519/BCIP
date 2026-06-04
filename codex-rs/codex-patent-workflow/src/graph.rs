//! DAG 图式编排 — 将线性 FlowStep 扩展为有向无环图。
//!
//! 支持并行分支执行与条件路由，参考 LangGraph StateGraph 设计。

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use serde::Deserialize;
use serde::Serialize;

use super::flow::FlowStatus;
use super::flow::FlowStep;
use super::flow::HumanApprovalTimeoutAction;
use super::flow::StepResult;

/// DAG 图节点 — 包含一个工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub step: FlowStep,
    #[serde(default)]
    pub label: Option<String>,
}

/// 边执行条件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Always,
    OnSuccess,
    OnFailure,
}

/// DAG 有向边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    #[serde(default = "default_condition")]
    pub condition: Condition,
}

fn default_condition() -> Condition {
    Condition::Always
}

/// DAG 图定义 — 节点 + 有向边 + 重试策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub entry_node: Option<String>,
    pub nodes: Vec<FlowNode>,
    #[serde(default)]
    pub edges: Vec<FlowEdge>,
    #[serde(default)]
    pub retry_on_failure: Option<u32>,
}

/// DAG 图节点执行结果
#[derive(Debug, Clone)]
pub struct GraphNodeResult {
    pub node_id: String,
    pub step_result: StepResult,
}

/// DAG 图整体执行结果
#[derive(Debug, Clone)]
pub struct GraphExecutionResult {
    pub flow_id: String,
    pub status: FlowStatus,
    pub node_results: Vec<GraphNodeResult>,
    pub completed_nodes: HashSet<String>,
}

impl FlowGraph {
    pub fn find_node(&self, node_id: &str) -> Option<&FlowNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    pub fn find_node_index(&self, node_id: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.id == node_id)
    }

    /// 获取从指定节点出发的有效出边（根据步骤结果筛选）。
    pub fn outgoing_edges(&self, node_id: &str, success: bool) -> Vec<&FlowEdge> {
        self.edges
            .iter()
            .filter(|e| {
                e.from == node_id
                    && match e.condition {
                        Condition::Always => true,
                        Condition::OnSuccess => success,
                        Condition::OnFailure => !success,
                    }
            })
            .collect()
    }

    /// 获取所有入边目标节点（即所有被引用的 "to" 节点）。
    pub fn target_nodes(&self) -> HashSet<&str> {
        self.edges.iter().map(|e| e.to.as_str()).collect()
    }

    /// 入口节点：显式指定 > 第一个不在任何边的 "to" 中的节点 > 第一个节点。
    pub fn resolve_entry_node(&self) -> Option<String> {
        if let Some(ref entry) = self.entry_node {
            if self.find_node(entry).is_some() {
                return Some(entry.clone());
            }
        }
        let targets = self.target_nodes();
        self.nodes
            .iter()
            .find(|n| !targets.contains(n.id.as_str()))
            .or_else(|| self.nodes.first())
            .map(|n| n.id.clone())
    }

    /// 拓扑排序 — 以层级形式返回，同一层内的节点可并行执行。
    /// 如果存在环则返回 Err。
    pub fn topological_levels(&self) -> Result<Vec<Vec<String>>, String> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for node in &self.nodes {
            in_degree.entry(node.id.as_str()).or_insert(0);
            adjacency.entry(node.id.as_str()).or_default();
        }

        for edge in &self.edges {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
            adjacency
                .entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }

        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut visited = 0usize;

        while !queue.is_empty() {
            let current_level: Vec<String> = queue.drain(..).map(|s| s.to_string()).collect();
            let mut next_queue: Vec<&str> = Vec::new();

            for node_id in &current_level {
                visited += 1;
                if let Some(neighbors) = adjacency.get(node_id.as_str()) {
                    for &neighbor in neighbors {
                        if let Some(deg) = in_degree.get_mut(neighbor) {
                            *deg -= 1;
                            if *deg == 0 {
                                next_queue.push(neighbor);
                            }
                        }
                    }
                }
            }

            levels.push(current_level);
            queue.extend(next_queue);
        }

        if visited != self.nodes.len() {
            return Err(format!(
                "图包含环：{} 个节点中仅访问了 {} 个",
                self.nodes.len(),
                visited
            ));
        }

        Ok(levels)
    }

    /// 计算每个节点的有效后继（根据其成功/失败状态）。
    pub fn compute_next_nodes(&self, current_node_id: &str, success: bool) -> Vec<String> {
        self.outgoing_edges(current_node_id, success)
            .iter()
            .map(|e| e.to.clone())
            .collect()
    }

    /// 验证图结构完整性。
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.id.is_empty() {
            errors.push("FlowGraph.id 不能为空".into());
        }
        if self.nodes.is_empty() {
            errors.push("FlowGraph.nodes 不能为空".into());
        }

        let node_ids: HashSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();

        for edge in &self.edges {
            if !node_ids.contains(edge.from.as_str()) {
                errors.push(format!("边引用不存在的 source 节点: {}", edge.from));
            }
            if !node_ids.contains(edge.to.as_str()) {
                errors.push(format!("边引用不存在的 target 节点: {}", edge.to));
            }
        }

        if let Some(ref entry) = self.entry_node {
            if !node_ids.contains(entry.as_str()) {
                errors.push(format!("入口节点 '{}' 不存在于 nodes 中", entry));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// 从线性 Flow 构建 FlowGraph（兼容旧接口）。
    pub fn from_flow(flow: &super::flow::Flow) -> Self {
        let nodes: Vec<FlowNode> = flow
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| FlowNode {
                id: format!("step_{i}"),
                step: step.clone(),
                label: None,
            })
            .collect();

        let edges: Vec<FlowEdge> = (0..flow.steps.len().saturating_sub(1))
            .map(|i| FlowEdge {
                from: format!("step_{i}"),
                to: format!("step_{}", i + 1),
                condition: Condition::Always,
            })
            .collect();

        FlowGraph {
            id: flow.id.clone(),
            name: flow.name.clone(),
            entry_node: None,
            nodes,
            edges,
            retry_on_failure: flow.retry_on_failure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow::FlowStep;

    fn sample_graph() -> FlowGraph {
        FlowGraph {
            id: "test-graph".into(),
            name: "测试图".into(),
            entry_node: None,
            nodes: vec![
                FlowNode {
                    id: "A".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "agent1".into(),
                        prompt: "步骤A".into(),
                    },
                    label: None,
                },
                FlowNode {
                    id: "B".into(),
                    step: FlowStep::AgentCall {
                        agent_name: "agent2".into(),
                        prompt: "步骤B".into(),
                    },
                    label: None,
                },
                FlowNode {
                    id: "C".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["check".into()],
                    },
                    label: None,
                },
            ],
            edges: vec![
                FlowEdge {
                    from: "A".into(),
                    to: "B".into(),
                    condition: Condition::Always,
                },
                FlowEdge {
                    from: "A".into(),
                    to: "C".into(),
                    condition: Condition::Always,
                },
            ],
            retry_on_failure: None,
        }
    }

    #[test]
    fn test_topological_levels_parallel_branches() {
        let graph = sample_graph();
        let levels = graph.topological_levels().unwrap();
        assert_eq!(levels.len(), 2, "应有 2 层：A → B,C 并行");
        assert_eq!(levels[0], vec!["A"]);
        assert_eq!(levels[1].len(), 2);
        assert!(levels[1].contains(&"B".to_string()));
        assert!(levels[1].contains(&"C".to_string()));
    }

    #[test]
    fn test_topological_levels_linear() {
        let graph = FlowGraph {
            id: "linear".into(),
            name: "线性".into(),
            entry_node: None,
            nodes: vec![
                FlowNode {
                    id: "0".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["c".into()],
                    },
                    label: None,
                },
                FlowNode {
                    id: "1".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["c".into()],
                    },
                    label: None,
                },
                FlowNode {
                    id: "2".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["c".into()],
                    },
                    label: None,
                },
            ],
            edges: vec![
                FlowEdge {
                    from: "0".into(),
                    to: "1".into(),
                    condition: Condition::Always,
                },
                FlowEdge {
                    from: "1".into(),
                    to: "2".into(),
                    condition: Condition::Always,
                },
            ],
            retry_on_failure: None,
        };
        let levels = graph.topological_levels().unwrap();
        assert_eq!(levels.len(), 3, "线性图应有 3 层");
    }

    #[test]
    fn test_cycle_detection() {
        let graph = FlowGraph {
            id: "cycle".into(),
            name: "环图".into(),
            entry_node: None,
            nodes: vec![
                FlowNode {
                    id: "X".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["x".into()],
                    },
                    label: None,
                },
                FlowNode {
                    id: "Y".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["y".into()],
                    },
                    label: None,
                },
            ],
            edges: vec![
                FlowEdge {
                    from: "X".into(),
                    to: "Y".into(),
                    condition: Condition::Always,
                },
                FlowEdge {
                    from: "Y".into(),
                    to: "X".into(),
                    condition: Condition::Always,
                },
            ],
            retry_on_failure: None,
        };
        assert!(graph.topological_levels().is_err());
    }

    #[test]
    fn test_resolve_entry_node() {
        let graph = sample_graph();
        assert_eq!(graph.resolve_entry_node().unwrap(), "A");
    }

    #[test]
    fn test_validate_ok() {
        let graph = sample_graph();
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_validate_bad_edge() {
        let mut graph = sample_graph();
        graph.edges.push(FlowEdge {
            from: "Z".into(),
            to: "A".into(),
            condition: Condition::Always,
        });
        let errs = graph.validate().unwrap_err();
        assert!(errs.iter().any(|e| e.contains("Z")), "应报告不存在的节点 Z");
    }

    #[test]
    fn test_compute_next_nodes_with_condition() {
        let graph = FlowGraph {
            id: "cond".into(),
            name: "条件路由".into(),
            entry_node: None,
            nodes: vec![
                FlowNode {
                    id: "start".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["c".into()],
                    },
                    label: None,
                },
                FlowNode {
                    id: "success_path".into(),
                    step: FlowStep::QualityCheck {
                        criteria: vec!["c".into()],
                    },
                    label: None,
                },
                FlowNode {
                    id: "failure_path".into(),
                    step: FlowStep::HumanApproval {
                        title: "审批".into(),
                        description: "需要人工介入".into(),
                        timeout_secs: None,
                        timeout_action: HumanApprovalTimeoutAction::Fail,
                    },
                    label: None,
                },
            ],
            edges: vec![
                FlowEdge {
                    from: "start".into(),
                    to: "success_path".into(),
                    condition: Condition::OnSuccess,
                },
                FlowEdge {
                    from: "start".into(),
                    to: "failure_path".into(),
                    condition: Condition::OnFailure,
                },
            ],
            retry_on_failure: None,
        };

        let next_on_success = graph.compute_next_nodes("start", true);
        assert_eq!(next_on_success, vec!["success_path"]);

        let next_on_fail = graph.compute_next_nodes("start", false);
        assert_eq!(next_on_fail, vec!["failure_path"]);
    }
}

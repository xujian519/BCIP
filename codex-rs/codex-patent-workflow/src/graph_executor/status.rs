//! 状态报告逻辑 - 最终状态确定、GraphExecution 构建

use crate::flow::FlowStatus;
use crate::graph::FlowGraph;
use crate::graph::GraphNodeResult;

use super::GraphExecution;

/// 根据执行状态确定最终流程状态
pub fn determine_final_status(suspended: bool, failed: bool) -> FlowStatus {
    if suspended {
        FlowStatus::Suspended
    } else if failed {
        FlowStatus::Failed
    } else {
        FlowStatus::Completed
    }
}

/// 构建最终的 GraphExecution 结果
pub fn build_execution_result(
    graph: &FlowGraph,
    run_id: String,
    status: FlowStatus,
    node_results: Vec<GraphNodeResult>,
) -> GraphExecution {
    GraphExecution {
        flow_id: graph.id.clone(),
        status,
        run_id,
        node_results,
    }
}

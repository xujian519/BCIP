//! 专利工作流编排系统
//!
//! 提供 DAG 图编排、执行计划生成、检查点持久化、Agent 桥接和角色间协作模板。
//! 上层通过 `Orchestrator` 入口驱动整个工作流。

pub mod agent_bridge;
pub mod assignment;
pub mod checkpoint;
pub mod collaboration;
pub mod config;
pub mod flow;
pub mod graph;
pub mod graph_executor;
pub mod llm_plan_generator;
pub mod orchestrator;
pub mod plan;
pub mod task;
pub mod types;

#[cfg(test)]
#[path = "graph_executor_concurrency_tests.rs"]
mod graph_executor_concurrency_tests;

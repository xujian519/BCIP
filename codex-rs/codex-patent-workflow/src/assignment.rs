//! Task assignment strategies — 策略化的 Agent 任务分配。
//!
//! 提供三种分配策略：
//! 1. RoundRobin: 轮询分配
//! 2. LoadBalanced: 最少负载优先
//! 3. CapabilityMatch: 按 task 关键词匹配 agent capabilities

use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::plan::PlanStep;

/// Agent 信息 — 策略分配的候选者。
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: String,
    pub role: String,
    pub current_load: Arc<AtomicUsize>,
    pub capabilities: Vec<String>,
}

/// 任务分配策略 trait。
pub trait TaskAssignmentStrategy: Send + Sync {
    fn assign(&self, task: &PlanStep, candidates: &[AgentInfo]) -> Option<String>;
}

/// 轮询策略 — 按 AtomicUsize 计数器循环分配。
pub struct RoundRobinStrategy {
    counter: AtomicUsize,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            counter: AtomicUsize::new(0),
        }
    }
}

impl Default for RoundRobinStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskAssignmentStrategy for RoundRobinStrategy {
    fn assign(&self, _task: &PlanStep, candidates: &[AgentInfo]) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }
        let idx = self.counter.fetch_add(1, Ordering::Relaxed) % candidates.len();
        Some(candidates[idx].name.clone())
    }
}

/// 负载均衡策略 — 选择当前负载最低的 agent。
pub struct LoadBalancedStrategy;

impl LoadBalancedStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoadBalancedStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskAssignmentStrategy for LoadBalancedStrategy {
    fn assign(&self, _task: &PlanStep, candidates: &[AgentInfo]) -> Option<String> {
        candidates
            .iter()
            .min_by_key(|c| c.current_load.load(Ordering::Relaxed))
            .map(|c| c.name.clone())
    }
}

/// 能力匹配策略 — 按 task 描述关键词匹配 agent capabilities。
pub struct CapabilityMatchStrategy;

impl CapabilityMatchStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CapabilityMatchStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskAssignmentStrategy for CapabilityMatchStrategy {
    fn assign(&self, task: &PlanStep, candidates: &[AgentInfo]) -> Option<String> {
        if candidates.is_empty() {
            return None;
        }

        let desc = task.description.to_lowercase();
        let mut best: Option<&AgentInfo> = None;
        let mut best_score: usize = 0;

        for candidate in candidates {
            let score = candidate
                .capabilities
                .iter()
                .filter(|cap| desc.contains(&cap.to_lowercase()))
                .count();
            if score > best_score {
                best_score = score;
                best = Some(candidate);
            }
        }

        // Fallback: no match → pick first candidate
        best.or_else(|| candidates.first()).map(|c| c.name.clone())
    }
}

#[cfg(test)]
#[path = "assignment_tests.rs"]
mod assignment_tests;

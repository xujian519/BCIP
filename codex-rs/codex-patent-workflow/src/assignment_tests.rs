//! TaskAssignmentStrategy 单元测试 — 验证轮询、负载均衡、能力匹配策略。

use super::*;
use crate::flow::FlowStep;
use crate::plan::PlanStep;
use crate::plan::PlanStepStatus;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

fn agent(name: &str, role: &str, load: usize, caps: &[&str]) -> AgentInfo {
    AgentInfo {
        name: name.to_string(),
        role: role.to_string(),
        current_load: Arc::new(AtomicUsize::new(load)),
        capabilities: caps.iter().map(|s| s.to_string()).collect(),
    }
}

fn task(id: &str, desc: &str) -> PlanStep {
    PlanStep {
        id: id.to_string(),
        description: desc.to_string(),
        step: FlowStep::AgentCall {
            agent_name: String::new(),
            prompt: String::new(),
        },
        depends_on: vec![],
        assigned_agent: None,
        status: PlanStepStatus::Pending,
    }
}

/// Test 22: 轮询策略均匀分配 10 个任务给 5 个 agent。
#[test]
fn round_robin_distributes_evenly() {
    let strategy = RoundRobinStrategy::new();
    let candidates: Vec<AgentInfo> = (0..5)
        .map(|i| agent(&format!("agent_{i}"), "worker", 0, &[]))
        .collect();

    let mut counts = std::collections::HashMap::new();
    for i in 0..10 {
        let t = task(&format!("t_{i}"), "do work");
        let assigned = strategy.assign(&t, &candidates).unwrap();
        *counts.entry(assigned).or_insert(0) += 1;
    }

    assert_eq!(counts.len(), 5, "all 5 agents should get tasks");
    for (_, count) in &counts {
        assert!(
            *count >= 1 && *count <= 3,
            "distribution should be roughly even"
        );
    }
}

/// Test 23: 轮询策略分配超过 agent 数后从头开始。
#[test]
fn round_robin_wraps_around() {
    let strategy = RoundRobinStrategy::new();
    let candidates = vec![
        agent("a", "w", 0, &[]),
        agent("b", "w", 0, &[]),
        agent("c", "w", 0, &[]),
    ];

    let assignments: Vec<String> = (0..6)
        .map(|i| {
            strategy
                .assign(&task(&format!("t_{i}"), "work"), &candidates)
                .unwrap()
        })
        .collect();

    // Should cycle: a, b, c, a, b, c
    assert_eq!(assignments[0], "a");
    assert_eq!(assignments[3], "a");
    assert_eq!(assignments[1], "b");
    assert_eq!(assignments[4], "b");
}

/// Test 24: 负载均衡策略选择负载最低的 agent。
#[test]
fn load_balanced_picks_least_loaded() {
    let strategy = LoadBalancedStrategy::new();
    let candidates = vec![
        agent("busy", "w", 5, &[]),
        agent("medium", "w", 3, &[]),
        agent("idle", "w", 0, &[]),
    ];

    let assigned = strategy.assign(&task("t1", "work"), &candidates).unwrap();
    assert_eq!(assigned, "idle");
}

/// Test 25: 并发负载均衡分配，总分配数等于任务数。
#[test]
fn load_balanced_under_concurrent_assignment() {
    use std::sync::atomic::Ordering;

    let strategy = Arc::new(LoadBalancedStrategy::new());
    let candidates: Arc<Vec<AgentInfo>> = Arc::new(vec![
        agent("a", "w", 0, &[]),
        agent("b", "w", 0, &[]),
        agent("c", "w", 0, &[]),
    ]);

    let total_assignments = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for i in 0..10 {
        let strategy = Arc::clone(&strategy);
        let candidates = Arc::clone(&candidates);
        let total = Arc::clone(&total_assignments);
        handles.push(std::thread::spawn(move || {
            let t = task(&format!("t_{i}"), "work");
            if strategy.assign(&t, &candidates).is_some() {
                total.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(
        total_assignments.load(Ordering::Relaxed),
        10,
        "all 10 assignments should succeed"
    );
}

/// Test 26: 能力匹配策略按 task 关键词选择 agent。
#[test]
fn capability_match_selects_relevant_agent() {
    let strategy = CapabilityMatchStrategy::new();
    let candidates = vec![
        agent("general", "worker", 0, &["search", "analyze"]),
        agent("specialist", "worker", 0, &["检索", "新颖性", "专利"]),
    ];

    let t = task("t1", "检查专利的新颖性");
    let assigned = strategy.assign(&t, &candidates).unwrap();
    assert_eq!(assigned, "specialist");
}

/// Test 27: 能力匹配无匹配时回退到第一个候选者。
#[test]
fn capability_match_fallback_when_no_match() {
    let strategy = CapabilityMatchStrategy::new();
    let candidates = vec![
        agent("a", "worker", 0, &["search"]),
        agent("b", "worker", 0, &["analyze"]),
    ];

    let t = task("t1", "完全无关的任务描述");
    let assigned = strategy.assign(&t, &candidates).unwrap();
    assert_eq!(assigned, "a", "should fallback to first candidate");
}

/// Test 28: 能力匹配选匹配能力最多的 agent。
#[test]
fn capability_match_prefers_more_capable_agent() {
    let strategy = CapabilityMatchStrategy::new();
    let candidates = vec![
        agent("weak", "worker", 0, &["novelty"]),
        agent("strong", "worker", 0, &["novelty", "patent", "检索"]),
    ];

    let t = task("t1", "新颖性检查和专利检索");
    let assigned = strategy.assign(&t, &candidates).unwrap();
    assert_eq!(assigned, "strong");
}

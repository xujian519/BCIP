//! 现有角色分配策略验证 — 确认 6 个协作模板的任务分配正确性。

use super::*;

/// Test 19: 6 个模板每步 assigned_agent 均为 Some 且匹配 roles()。
#[test]
fn role_based_assignment_all_templates() {
    for template in CollaborationTemplate::all() {
        let plan = template.to_plan("测试目标");
        let expected_roles = template.roles();

        assert_eq!(
            plan.steps.len(),
            expected_roles.len(),
            "template {:?} should have {} steps, got {}",
            template,
            expected_roles.len(),
            plan.steps.len()
        );

        for (i, step) in plan.steps.iter().enumerate() {
            assert!(
                step.assigned_agent.is_some(),
                "template {:?} step {} ({}) should have an assigned agent",
                template,
                i,
                step.id
            );
            let agent = step.assigned_agent.as_ref().unwrap();
            assert_eq!(
                agent, expected_roles[i],
                "template {:?} step {} agent mismatch: expected {}, got {}",
                template, i, expected_roles[i], agent
            );
        }
    }
}

/// Test 20: FullReview 模板中 check_novelty 和 check_creativity 分配给不同 agent。
#[test]
fn full_review_parallel_steps_different_agents() {
    let plan = CollaborationTemplate::FullReview.to_plan("测试全面审查");

    let novelty_step = plan
        .steps
        .iter()
        .find(|s| s.id == "check_novelty")
        .expect("check_novelty step");
    let creativity_step = plan
        .steps
        .iter()
        .find(|s| s.id == "check_creativity")
        .expect("check_creativity step");

    assert_ne!(
        novelty_step.assigned_agent, creativity_step.assigned_agent,
        "parallel steps should be assigned to different agents"
    );

    assert_eq!(
        novelty_step.assigned_agent,
        Some("novelty_checker".to_string())
    );
    assert_eq!(
        creativity_step.assigned_agent,
        Some("creativity_checker".to_string())
    );
}

/// Test 21: retrieve 完成后 ready_steps() 返回 2 个并行步骤。
#[test]
fn execution_plan_ready_steps_parallel_eligibility() {
    let mut plan = CollaborationTemplate::FullReview.to_plan("测试目标");

    // Initially only retrieve is ready
    let ready = plan.ready_steps();
    assert_eq!(ready.len(), 1, "only retrieve should be ready initially");
    assert_eq!(ready[0].id, "retrieve");

    // Complete retrieve
    plan.update_step(
        "retrieve",
        &super::super::flow::StepResult {
            step_index: 0,
            success: true,
            output: Some(serde_json::json!("done")),
            error: None,
        },
    );

    let ready = plan.ready_steps();
    assert_eq!(
        ready.len(),
        2,
        "after retrieve completes, both check_novelty and check_creativity should be ready"
    );

    let ready_ids: Vec<&str> = ready.iter().map(|s| s.id.as_str()).collect();
    assert!(ready_ids.contains(&"check_novelty"));
    assert!(ready_ids.contains(&"check_creativity"));
}

//! AgentRegistry 并发槽位测试 — 验证高并发下 CAS 槽位管理和 RAII 释放的正确性。

use super::*;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::Ordering;

fn agent_path(path: &str) -> AgentPath {
    AgentPath::try_from(path).expect("valid agent path")
}

fn agent_metadata(thread_id: ThreadId) -> AgentMetadata {
    AgentMetadata {
        agent_id: Some(thread_id),
        ..Default::default()
    }
}

/// Test 6: max_threads=10，10 任务同时 reserve，全部成功。再 reserve 应失败。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_slot_reservation_at_capacity() {
    let registry = Arc::new(AgentRegistry::default());

    // 10 tasks race for 10 slots — all should succeed
    let reservations: Arc<tokio::sync::Mutex<Vec<_>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let barrier = Arc::new(tokio::sync::Barrier::new(10));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let registry = Arc::clone(&registry);
        let reservations = Arc::clone(&reservations);
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            let reservation = registry
                .reserve_spawn_slot(Some(10))
                .expect("should get slot");
            reservations.lock().await.push(reservation);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // All 10 slots are held, one more should fail
    let result = registry.reserve_spawn_slot(Some(10));
    assert!(
        result.is_err(),
        "should fail to reserve beyond max_threads=10"
    );

    // total_count should be exactly 10
    let count = registry.total_count.load(Ordering::Acquire);
    assert_eq!(count, 10, "total_count should be 10 after 10 reservations");
}

/// Test 7: max_threads=10，50 次 reserve→commit→release 循环，total_count 归零。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_reserve_and_release_no_leak() {
    let registry = Arc::new(AgentRegistry::default());
    let iterations = 50;

    let mut handles = Vec::new();
    for _ in 0..5 {
        let registry = Arc::clone(&registry);
        handles.push(tokio::spawn(async move {
            for _ in 0..(iterations / 5) {
                let reservation = registry.reserve_spawn_slot(Some(10)).expect("reserve slot");
                let thread_id = ThreadId::new();
                reservation.commit(agent_metadata(thread_id));
                registry.release_spawned_thread(thread_id);
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let count = registry.total_count.load(Ordering::Acquire);
    assert_eq!(
        count, 0,
        "all slots should be released after concurrent reserve/commit/release cycles"
    );
}

/// Test 8: 5 任务用同一名称池，nickname 不重复（池重置前）。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_nickname_reservation_no_double_assign() {
    let registry = Arc::new(AgentRegistry::default());
    let barrier = Arc::new(tokio::sync::Barrier::new(5));

    let nicknames = Arc::new(std::sync::Mutex::new(Vec::new()));

    let mut handles = Vec::new();
    for _ in 0..5 {
        let registry = Arc::clone(&registry);
        let barrier = Arc::clone(&barrier);
        let nicknames = Arc::clone(&nicknames);
        handles.push(tokio::spawn(async move {
            let mut reservation = registry.reserve_spawn_slot(None).expect("reserve slot");
            barrier.wait().await;
            let name = reservation
                .reserve_agent_nickname_with_preference(&["alpha"], None)
                .expect("nickname");
            nicknames.lock().unwrap().push(name.clone());
            let thread_id = ThreadId::new();
            reservation.commit(agent_metadata(thread_id));
            registry.release_spawned_thread(thread_id);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let assigned: Vec<String> = nicknames.lock().unwrap().clone();
    assert_eq!(assigned.len(), 5, "all 5 tasks should get nicknames");

    let unique: HashSet<String> = assigned.into_iter().collect();
    assert_eq!(
        unique.len(),
        5,
        "all assigned nicknames must be unique (via pool reset)"
    );
}

/// Test 9: 多任务抢同一路径，仅第一个成功，后续都失败。验证路径互斥。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_path_reservation_collision() {
    let registry = Arc::new(AgentRegistry::default());

    // First reservation succeeds
    let mut reservation = registry.reserve_spawn_slot(None).expect("reserve slot");
    reservation
        .reserve_agent_path(&agent_path("/root/researcher"))
        .expect("first path reserve");

    // All subsequent attempts should fail while the first holds the path
    for _ in 0..5 {
        let mut r = registry.reserve_spawn_slot(None).expect("reserve slot");
        let result = r.reserve_agent_path(&agent_path("/root/researcher"));
        assert!(
            result.is_err(),
            "subsequent path reservations should fail while path is held"
        );
    }

    // Drop the first reservation — path should be released
    drop(reservation);

    // Now a new reservation should succeed
    let mut r = registry.reserve_spawn_slot(None).expect("reserve slot");
    r.reserve_agent_path(&agent_path("/root/researcher"))
        .expect("path should be available after release");
}

/// Test 10: max_threads=10，20 任务×100 次循环，total_count 始终 ≤10。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn registry_total_count_monotonic_under_concurrency() {
    let registry = Arc::new(AgentRegistry::default());
    let violations = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::new();
    for _ in 0..20 {
        let registry = Arc::clone(&registry);
        let violations = Arc::clone(&violations);
        handles.push(tokio::spawn(async move {
            for _ in 0..100 {
                let reservation = registry.reserve_spawn_slot(Some(10));
                match reservation {
                    Ok(reservation) => {
                        let count = registry.total_count.load(Ordering::Acquire);
                        if count > 10 {
                            violations.fetch_add(1, Ordering::Relaxed);
                        }
                        let thread_id = ThreadId::new();
                        reservation.commit(agent_metadata(thread_id));
                        registry.release_spawned_thread(thread_id);
                    }
                    Err(CodexErr::AgentLimitReached { .. }) => {
                        // Expected when at capacity
                    }
                    Err(e) => panic!("unexpected error: {e}"),
                }
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let violation_count = violations.load(Ordering::Relaxed);
    assert_eq!(
        violation_count, 0,
        "total_count should never exceed max_threads"
    );
}

/// Test 11: catch_unwind 模拟 panic，验证 RAII Drop 释放槽位。
#[test]
fn spawn_reservation_drop_on_panic_releases_slot() {
    let registry = Arc::new(AgentRegistry::default());
    let count_before = registry.total_count.load(Ordering::Acquire);

    let registry_clone = Arc::clone(&registry);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _reservation = registry_clone
            .reserve_spawn_slot(Some(5))
            .expect("reserve slot");
        // Simulate panic while holding the reservation (not committed)
        panic!("simulated failure");
    }));

    assert!(result.is_err(), "should have caught the panic");

    let count_after = registry.total_count.load(Ordering::Acquire);
    assert_eq!(
        count_before, count_after,
        "slot should be released after panic via RAII Drop"
    );

    // Verify the slot is actually available again
    let _reservation = registry
        .reserve_spawn_slot(Some(1))
        .expect("slot should be available after panic cleanup");
}

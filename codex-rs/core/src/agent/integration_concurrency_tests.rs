//! AgentBus + AgentRegistry 集成无死锁测试 — 验证总线与注册表交互不产生死锁。

use crate::agent::bus::AgentBus;
use crate::agent::bus::MessageFilter;
use crate::agent::registry::AgentMetadata;
use crate::agent::registry::AgentRegistry;
use codex_protocol::AgentPath;
use codex_protocol::ThreadId;
use codex_protocol::agent_bus::AgentBusMessage;
use codex_protocol::agent_bus::AgentBusMessageType;
use codex_protocol::agent_bus::AgentBusRecipient;
use std::sync::Arc;
use std::sync::atomic::Ordering;

fn test_path(name: &str) -> AgentPath {
    AgentPath::try_from(format!("/root/{name}")).unwrap()
}

fn agent_metadata(thread_id: ThreadId) -> AgentMetadata {
    AgentMetadata {
        agent_id: Some(thread_id),
        ..Default::default()
    }
}

/// Test 12: 5 任务各自 reserve→bus.send×10→release，5s 超时完成。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn no_deadlock_on_bus_and_registry_interaction() {
    let registry = Arc::new(AgentRegistry::default());
    let bus = Arc::new(AgentBus::new(256, 1000));
    let barrier = Arc::new(tokio::sync::Barrier::new(5));

    let mut handles = Vec::new();
    for i in 0..5 {
        let registry = Arc::clone(&registry);
        let bus = Arc::clone(&bus);
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            // Reserve a slot
            let reservation = registry.reserve_spawn_slot(Some(10)).expect("reserve slot");
            let thread_id = ThreadId::new();
            reservation.commit(agent_metadata(thread_id));

            // Send messages through bus while holding a registry slot
            for j in 0..10 {
                bus.send(AgentBusMessage::new(
                    test_path(&format!("agent_{i}")),
                    AgentBusRecipient::Broadcast,
                    AgentBusMessageType::SystemEvent,
                    serde_json::json!({"agent": i, "seq": j}),
                ))
                .await
                .unwrap();
            }

            // Release slot
            registry.release_spawned_thread(thread_id);
        }));
    }

    let result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        for h in handles {
            h.await.unwrap();
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "deadlock detected: bus + registry interaction did not complete within 5s"
    );

    // Verify all slots released
    let count = registry.total_count.load(Ordering::Acquire);
    assert_eq!(count, 0, "all registry slots should be released");

    // Verify bus recorded messages (try_write may lose some under contention)
    let history = bus.history(MessageFilter::default()).await;
    assert!(
        history.len() >= 40,
        "most messages should be in history (got {})",
        history.len()
    );
}

/// Test 13: A 任务循环 subscribe/unsubscribe，B 任务并发 publish 100 条。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn no_deadlock_concurrent_bus_publish_and_topic_subscribe() {
    let bus = Arc::new(AgentBus::new(256, 1000));

    // Task A: subscribe/unsubscribe loop
    let bus_a = Arc::clone(&bus);
    let handle_a = tokio::spawn(async move {
        for i in 0..50 {
            let agent = test_path(&format!("agent_{i}"));
            bus_a
                .subscribe_topic("concurrent.topic", agent.clone())
                .await;
            bus_a.unsubscribe_topic("concurrent.topic", &agent).await;
        }
    });

    // Task B: publish to topic concurrently
    let bus_b = Arc::clone(&bus);
    let handle_b = tokio::spawn(async move {
        for i in 0..100 {
            bus_b
                .publish(
                    test_path("publisher"),
                    "concurrent.topic",
                    serde_json::json!(i),
                )
                .await
                .unwrap();
            if i % 10 == 0 {
                tokio::task::yield_now().await;
            }
        }
    });

    let result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        handle_a.await.unwrap();
        handle_b.await.unwrap();
    })
    .await;

    assert!(
        result.is_ok(),
        "deadlock detected: concurrent subscribe/unsubscribe + publish did not complete within 5s"
    );
}

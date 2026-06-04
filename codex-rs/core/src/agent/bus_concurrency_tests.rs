//! AgentBus 并发压力测试 — 验证高并发下消息总线的正确性和稳定性。

use super::*;
use codex_protocol::agent_bus::AgentBusMessageType;
use codex_protocol::agent_bus::MessagePriority;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

fn test_path(name: &str) -> AgentPath {
    AgentPath::try_from(format!("/root/{name}")).unwrap()
}

/// Test 1: 8 个订阅者同时接收 8 个发布者共 80 条消息。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_broadcast_to_many_subscribers() {
    let bus = Arc::new(AgentBus::new(256, 1000));

    // 8 subscribers
    let mut receivers: Vec<_> = (0..8).map(|_| bus.subscribe()).collect();

    // Barrier to synchronize publishers
    let barrier = Arc::new(tokio::sync::Barrier::new(8));
    let received_count = Arc::new(AtomicUsize::new(0));

    // 8 publishers, each sends 10 messages
    let mut handles = Vec::new();
    for i in 0..8 {
        let bus = Arc::clone(&bus);
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            for j in 0..10 {
                bus.send(AgentBusMessage::new(
                    test_path(&format!("pub_{i}")),
                    AgentBusRecipient::Broadcast,
                    AgentBusMessageType::SystemEvent,
                    serde_json::json!({"publisher": i, "seq": j}),
                ))
                .await
                .unwrap();
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    // Give receivers time to process
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Each subscriber should have received all 80 messages
    for rx in &mut receivers {
        let mut count = 0;
        while rx.try_recv().is_ok() {
            count += 1;
            received_count.fetch_add(1, Ordering::Relaxed);
        }
        assert_eq!(count, 80, "each subscriber should receive all 80 messages");
    }
}

/// Test 2: 并发订阅/取消订阅 + 并发发布不死锁。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_topic_subscribe_unsubscribe() {
    let bus = Arc::new(AgentBus::new(256, 1000));
    let iterations = 50;

    // Task A: subscribe/unsubscribe in a loop
    let bus_a = Arc::clone(&bus);
    let handle_a = tokio::spawn(async move {
        for i in 0..iterations {
            let agent = test_path(&format!("agent_{i}"));
            bus_a.subscribe_topic("stress.topic", agent.clone()).await;
            bus_a.unsubscribe_topic("stress.topic", &agent).await;
        }
    });

    // Task B: publish to topic
    let bus_b = Arc::clone(&bus);
    let handle_b = tokio::spawn(async move {
        for i in 0..100 {
            bus_b
                .publish(test_path("publisher"), "stress.topic", serde_json::json!(i))
                .await.unwrap();
            tokio::task::yield_now().await;
        }
    });

    // 5-second deadline — if deadlock, this will timeout
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        handle_a.await.unwrap();
        handle_b.await.unwrap();
    })
    .await;

    assert!(
        result.is_ok(),
        "deadlock detected: subscribe/unsubscribe + publish did not complete within 5s"
    );
}

/// Test 3: 16 个任务各发 100 条消息，history 不超 max_history 且无重复。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_history_writes_under_load() {
    let bus = Arc::new(AgentBus::new(512, 1000));
    let barrier = Arc::new(tokio::sync::Barrier::new(16));

    let mut handles = Vec::new();
    for i in 0..16 {
        let bus = Arc::clone(&bus);
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            for j in 0..100 {
                bus.send(AgentBusMessage::new(
                    test_path(&format!("sender_{i}")),
                    AgentBusRecipient::Broadcast,
                    AgentBusMessageType::Custom(format!("test_{i}_{j}")),
                    serde_json::json!({"sender": i, "seq": j}),
                ))
                .await
                .unwrap();
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let history = bus.history(MessageFilter::default()).await;
    // try_write may drop some entries under high contention, but must not exceed max_history
    assert!(
        history.len() <= 1000,
        "history should not exceed max_history, got {}",
        history.len()
    );

    // Verify no duplicate message IDs among recorded entries
    let ids: HashSet<uuid::Uuid> = history.iter().map(|m| m.id).collect();
    assert_eq!(
        ids.len(),
        history.len(),
        "all message IDs in history should be unique"
    );
}

/// Test 4: 10 个任务各发 5 条无接收者消息，DLQ 不超 500 上限。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn dlq_growth_under_concurrent_failed_retries() {
    let bus = Arc::new(AgentBus::new(256, 1000));
    let barrier = Arc::new(tokio::sync::Barrier::new(10));

    let config = RetryConfig {
        max_retries: 1,
        initial_delay_ms: 1,
        max_delay_ms: 1,
        backoff_multiplier: 1.0,
    };

    let mut handles = Vec::new();
    for i in 0..10 {
        let bus = Arc::clone(&bus);
        let barrier = Arc::clone(&barrier);
        let config = config.clone();
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            for j in 0..5 {
                let msg = AgentBusMessage::direct(
                    test_path(&format!("sender_{i}")),
                    test_path("nonexistent"),
                    serde_json::json!(j),
                );
                let _ = bus.send_with_retry(msg, config.clone()).await;
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let dlq_count = bus.dead_letter_count().await;
    assert!(
        dlq_count <= 500,
        "DLQ should not exceed DEFAULT_DLQ_CAPACITY (500), got {dlq_count}"
    );
    assert!(dlq_count > 0, "some messages should have entered the DLQ");
}

/// Test 5: 10 任务发 10000 条消息，max_history=50 下 history.len()==50。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn history_does_not_exceed_max_under_high_throughput() {
    let bus = Arc::new(AgentBus::new(512, 50));
    let barrier = Arc::new(tokio::sync::Barrier::new(10));

    let mut handles = Vec::new();
    for i in 0..10 {
        let bus = Arc::clone(&bus);
        let barrier = Arc::clone(&barrier);
        handles.push(tokio::spawn(async move {
            barrier.wait().await;
            for j in 0..1000 {
                bus.publish(
                    test_path(&format!("sender_{i}")),
                    "high.throughput",
                    serde_json::json!(j),
                )
                .await
                .unwrap();
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let history = bus.history(MessageFilter::default()).await;
    assert_eq!(
        history.len(),
        50,
        "history must be capped at max_history=50 even under high throughput"
    );
}

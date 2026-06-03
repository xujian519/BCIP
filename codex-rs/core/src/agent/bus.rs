//! AgentBus — tokio broadcast-based message bus for cross-agent communication.
//!
//! Supports:
//! - Point-to-point (Direct)
//! - Broadcast to all agents
//! - Topic-based pub/sub
//! - Role-based broadcast
//! - Bounded message history for debugging

use codex_protocol::AgentPath;
use codex_protocol::agent_bus::AgentBusMessage;
use codex_protocol::agent_bus::AgentBusRecipient;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;

const DEFAULT_BUFFER_SIZE: usize = 256;
const DEFAULT_MAX_HISTORY: usize = 1000;
const DEFAULT_DLQ_CAPACITY: usize = 500;

/// Filter for querying message history.
#[derive(Debug, Clone, Default)]
pub(crate) struct MessageFilter {
    pub(crate) from: Option<AgentPath>,
    pub(crate) topic: Option<String>,
    pub(crate) limit: Option<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("no active receivers")]
    NoReceivers,
    #[error("retry exhausted: {attempts} attempts")]
    RetryExhausted { attempts: u32 },
}

#[derive(Debug, Clone)]
pub(crate) struct RetryConfig {
    pub(crate) max_retries: u32,
    pub(crate) initial_delay_ms: u64,
    pub(crate) max_delay_ms: u64,
    pub(crate) backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

pub(crate) struct AgentBus {
    tx: broadcast::Sender<AgentBusMessage>,
    topics: Arc<RwLock<HashMap<String, Vec<AgentPath>>>>,
    history: Arc<RwLock<VecDeque<AgentBusMessage>>>,
    dead_letter: Arc<RwLock<VecDeque<AgentBusMessage>>>,
    max_history: usize,
}

impl AgentBus {
    pub(crate) fn new(buffer_size: usize, max_history: usize) -> Self {
        let (tx, _) = broadcast::channel(buffer_size);
        Self {
            tx,
            topics: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history))),
            dead_letter: Arc::new(RwLock::new(VecDeque::with_capacity(DEFAULT_DLQ_CAPACITY))),
            max_history,
        }
    }

    /// Create with default settings.
    pub(crate) fn default_config() -> Self {
        Self::new(DEFAULT_BUFFER_SIZE, DEFAULT_MAX_HISTORY)
    }

    /// Subscribe to the bus. Returns a [`Receiver`] that gets all broadcast messages.
    /// The subscriber should filter messages locally based on recipient.
    pub(crate) fn subscribe(&self) -> Receiver<AgentBusMessage> {
        self.tx.subscribe()
    }

    pub(crate) async fn subscribe_topic(&self, topic: &str, agent: AgentPath) {
        tracing::debug!(topic, agent = %agent, "agent subscribing to topic");
        let mut topics = self.topics.write().await;
        let subscribers = topics.entry(topic.to_string()).or_default();
        if !subscribers.contains(&agent) {
            subscribers.push(agent);
        }
    }

    /// Unregister an agent from a specific topic.
    pub(crate) async fn unsubscribe_topic(&self, topic: &str, agent: &AgentPath) {
        let mut topics = self.topics.write().await;
        if let Some(subscribers) = topics.get_mut(topic) {
            subscribers.retain(|p| p != agent);
            if subscribers.is_empty() {
                topics.remove(topic);
            }
        }
    }

    pub(crate) fn send(&self, message: AgentBusMessage) -> Result<usize, BusError> {
        let _span = tracing::debug_span!(
            "agent_bus.send",
            msg_id = %message.id,
            msg_type = ?message.message_type,
            from = %message.from,
        )
        .entered();

        let result = self.tx.send(message.clone());
        if let Ok(mut history) = self.history.try_write() {
            if history.len() >= self.max_history {
                history.pop_front();
            }
            history.push_back(message);
        }
        match result {
            Ok(n) => Ok(n),
            Err(_) => Ok(0), // No receivers is not an error — message still recorded
        }
    }

    /// Convenience: send a direct (point-to-point) message.
    pub(crate) fn send_direct(
        &self,
        from: AgentPath,
        to: AgentPath,
        payload: serde_json::Value,
    ) -> Result<usize, BusError> {
        let msg = AgentBusMessage::direct(from, to, payload);
        self.send(msg)
    }

    /// Convenience: publish to a topic.
    pub(crate) fn publish(
        &self,
        from: AgentPath,
        topic: &str,
        payload: serde_json::Value,
    ) -> Result<usize, BusError> {
        let msg = AgentBusMessage::topic(from, topic, payload);
        self.send(msg)
    }

    /// Convenience: broadcast to all agents of a role.
    pub(crate) fn send_to_role(
        &self,
        from: AgentPath,
        role: &str,
        payload: serde_json::Value,
    ) -> Result<usize, BusError> {
        let msg = AgentBusMessage::role(from, role, payload);
        self.send(msg)
    }

    /// Query message history with optional filter.
    pub(crate) async fn history(&self, filter: MessageFilter) -> Vec<AgentBusMessage> {
        let history = self.history.read().await;
        let mut results: Vec<AgentBusMessage> = history
            .iter()
            .filter(|msg| {
                if let Some(ref from) = filter.from {
                    if msg.from != *from {
                        return false;
                    }
                }
                if let Some(ref topic) = filter.topic {
                    match &msg.to {
                        AgentBusRecipient::Topic(t) if t == topic => {}
                        _ => return false,
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Return most recent entries when limit is set.
        if let Some(limit) = filter.limit {
            let start = results.len().saturating_sub(limit);
            results = results.split_off(start);
        }
        results
    }

    /// Get topic subscriber summary.
    pub(crate) async fn topic_summary(&self) -> HashMap<String, usize> {
        let topics = self.topics.read().await;
        topics.iter().map(|(k, v)| (k.clone(), v.len())).collect()
    }

    pub(crate) async fn is_subscribed(&self, topic: &str, agent: &AgentPath) -> bool {
        let topics = self.topics.read().await;
        topics.get(topic).map_or(false, |subs| subs.contains(agent))
    }

    pub(crate) async fn send_with_retry(
        &self,
        message: AgentBusMessage,
        config: RetryConfig,
    ) -> Result<usize, BusError> {
        let _span = tracing::debug_span!(
            "agent_bus.send_with_retry",
            msg_id = %message.id,
            max_retries = config.max_retries,
        )
        .entered();

        let mut delay_ms = config.initial_delay_ms;
        let mut attempts = 0;

        loop {
            let n = self.send(message.clone())?;
            if n > 0 {
                tracing::debug!(attempts, receivers = n, "send_with_retry succeeded");
                return Ok(n);
            }

            attempts += 1;
            if attempts >= config.max_retries {
                tracing::warn!(attempts, "send_with_retry exhausted, moving to DLQ");
                self.record_dead_letter(message);
                return Err(BusError::RetryExhausted { attempts });
            }

            tracing::debug!(attempt = attempts, delay_ms, "send_with_retry backing off");
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            delay_ms = ((delay_ms as f64) * config.backoff_multiplier).min(config.max_delay_ms as f64) as u64;
        }
    }

    fn record_dead_letter(&self, message: AgentBusMessage) {
        if let Ok(mut dlq) = self.dead_letter.try_write() {
            if dlq.len() >= DEFAULT_DLQ_CAPACITY {
                dlq.pop_front();
            }
            dlq.push_back(message);
        }
    }

    pub(crate) async fn dead_letter_queue(&self) -> Vec<AgentBusMessage> {
        self.dead_letter.read().await.iter().cloned().collect()
    }

    pub(crate) async fn dead_letter_count(&self) -> usize {
        self.dead_letter.read().await.len()
    }

    pub(crate) fn replay_dead_letter(&self, message_id: uuid::Uuid) -> Result<usize, BusError> {
        let msg = {
            let dlq = self.dead_letter.try_read();
            match dlq {
                Ok(q) => q.iter().find(|m| m.id == message_id).cloned(),
                Err(_) => None,
            }
        };
        match msg {
            Some(m) => self.send(m),
            None => Ok(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::agent_bus::AgentBusMessageType;
    use codex_protocol::agent_bus::MessagePriority;
    use pretty_assertions::assert_eq;

    fn test_path(name: &str) -> AgentPath {
        AgentPath::try_from(format!("/root/{name}")).unwrap()
    }

    #[tokio::test]
    async fn subscribe_and_receive_broadcast() {
        let bus = AgentBus::new(16, 100);
        let mut rx = bus.subscribe();

        let from = test_path("sender");
        let payload = serde_json::json!({"key": "value"});

        bus.send(AgentBusMessage::new(
            from.clone(),
            AgentBusRecipient::Broadcast,
            AgentBusMessageType::SystemEvent,
            payload.clone(),
        ))
        .unwrap();

        let msg = rx.try_recv().unwrap();
        assert_eq!(msg.from, from);
        assert_eq!(msg.payload, payload);
    }

    #[tokio::test]
    async fn topic_subscribe_unsubscribe() {
        let bus = AgentBus::new(16, 100);
        let agent = test_path("worker");

        bus.subscribe_topic("patent.search", agent.clone()).await;
        assert!(bus.is_subscribed("patent.search", &agent).await);

        bus.unsubscribe_topic("patent.search", &agent).await;
        assert!(!bus.is_subscribed("patent.search", &agent).await);
    }

    #[tokio::test]
    async fn history_records_messages() {
        let bus = AgentBus::new(16, 100);
        let from = test_path("agent");

        bus.publish(
            from.clone(),
            "test.topic",
            serde_json::json!("data1"),
        )
        .unwrap();
        bus.publish(
            from.clone(),
            "test.topic",
            serde_json::json!("data2"),
        )
        .unwrap();

        let filter = MessageFilter {
            topic: Some("test.topic".to_string()),
            limit: None,
            from: None,
        };
        let h = bus.history(filter).await;
        assert_eq!(h.len(), 2);
    }

    #[tokio::test]
    async fn history_respects_max_limit() {
        let bus = AgentBus::new(16, 3); // max_history = 3
        let from = test_path("agent");

        for i in 0..5 {
            bus.publish(from.clone(), "test", serde_json::json!(i))
                .unwrap();
        }

        let all = bus.history(MessageFilter::default()).await;
        assert_eq!(all.len(), 3); // Only last 3 retained
    }

    #[tokio::test]
    async fn direct_message_convenience() {
        let bus = AgentBus::new(16, 100);
        let mut rx = bus.subscribe();

        let from = test_path("a");
        let to = test_path("b");

        bus.send_direct(from.clone(), to.clone(), serde_json::json!("hello"))
            .unwrap();

        let msg = rx.try_recv().unwrap();
        assert_eq!(msg.from, from);
        match msg.to {
            AgentBusRecipient::Direct(p) => assert_eq!(p, to),
            _ => panic!("expected Direct recipient"),
        }
    }

    #[tokio::test]
    async fn send_with_no_receivers_still_records_history() {
        let bus = AgentBus::new(16, 100);
        // No subscribers — message should still be recorded
        let from = test_path("lonely");
        bus.publish(from, "orphan.topic", serde_json::json!("data"))
            .unwrap();

        let h = bus
            .history(MessageFilter {
                topic: Some("orphan.topic".to_string()),
                ..Default::default()
            })
            .await;
        assert_eq!(h.len(), 1);
    }

    #[tokio::test]
    async fn topic_summary() {
        let bus = AgentBus::new(16, 100);
        bus.subscribe_topic("a", test_path("x")).await;
        bus.subscribe_topic("a", test_path("y")).await;
        bus.subscribe_topic("b", test_path("z")).await;

        let summary = bus.topic_summary().await;
        assert_eq!(summary["a"], 2);
        assert_eq!(summary["b"], 1);
    }

    #[tokio::test]
    async fn send_with_retry_succeeds_immediately() {
        let bus = AgentBus::new(16, 100);
        let _rx = bus.subscribe();

        let from = test_path("sender");
        let msg = AgentBusMessage::direct(
            from,
            test_path("receiver"),
            serde_json::json!("hello"),
        );

        let result = bus
            .send_with_retry(msg, RetryConfig::default())
            .await
            .unwrap();
        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn send_with_retry_exhausted_goes_to_dlq() {
        let bus = AgentBus::new(16, 100);

        let from = test_path("sender");
        let msg = AgentBusMessage::direct(
            from,
            test_path("nonexistent"),
            serde_json::json!("lost"),
        );

        let config = RetryConfig {
            max_retries: 2,
            initial_delay_ms: 10,
            max_delay_ms: 50,
            backoff_multiplier: 2.0,
        };

        let result = bus.send_with_retry(msg, config).await;
        assert!(result.is_err());

        let dlq = bus.dead_letter_queue().await;
        assert_eq!(dlq.len(), 1);
    }

    #[tokio::test]
    async fn dead_letter_count_tracks() {
        let bus = AgentBus::new(16, 100);
        let config = RetryConfig {
            max_retries: 1,
            initial_delay_ms: 1,
            max_delay_ms: 1,
            backoff_multiplier: 1.0,
        };

        for i in 0..3 {
            let msg = AgentBusMessage::topic(
                test_path("sender"),
                "orphan",
                serde_json::json!(i),
            );
            let _ = bus.send_with_retry(msg, config.clone()).await;
        }

        assert_eq!(bus.dead_letter_count().await, 3);
    }

    #[tokio::test]
    async fn replay_dead_letter_succeeds() {
        let bus = AgentBus::new(16, 100);
        let config = RetryConfig {
            max_retries: 1,
            initial_delay_ms: 1,
            max_delay_ms: 1,
            backoff_multiplier: 1.0,
        };

        let msg = AgentBusMessage::topic(
            test_path("sender"),
            "test.replay",
            serde_json::json!("payload"),
        );
        let msg_id = msg.id;

        let _ = bus.send_with_retry(msg, config).await;
        assert_eq!(bus.dead_letter_count().await, 1);

        let _rx = bus.subscribe();
        let result = bus.replay_dead_letter(msg_id).unwrap();
        assert_eq!(result, 1);
    }
}

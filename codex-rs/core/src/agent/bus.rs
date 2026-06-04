//! AgentBus — tokio broadcast-based message bus for cross-agent communication.
//!
//! Supports:
//! - Point-to-point (Direct)
//! - Broadcast to all agents
//! - Topic-based pub/sub
//! - Role-based broadcast
//! - Bounded message history for debugging
//! - Heartbeat-based liveness monitoring

use codex_protocol::AgentPath;
use codex_protocol::agent_bus::AgentBusMessage;
use codex_protocol::agent_bus::AgentBusMessageType;
use codex_protocol::agent_bus::AgentBusRecipient;
use codex_protocol::agent_bus::AgentLiveness;
use codex_protocol::agent_bus::HeartbeatConfig;
use codex_protocol::agent_bus::TaskDescriptor;
use codex_protocol::transport::LocalTransport;
use codex_protocol::transport::Transport;
use codex_protocol::transport::TransportError;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;

const DEFAULT_BUFFER_SIZE: usize = 256;
const DEFAULT_MAX_HISTORY: usize = 1000;
const DEFAULT_DLQ_CAPACITY: usize = 500;
const DEFAULT_MAX_PAYLOAD_BYTES: usize = 1024 * 1024; // 1MB

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
    #[error("payload too large: {size} bytes (max {max})")]
    PayloadTooLarge { size: usize, max: usize },
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

/// Tracked liveness state for a single agent on the bus.
struct LivenessState {
    last_seen: Instant,
    missed_count: u32,
    status: AgentLiveness,
}

pub(crate) struct AgentBus {
    transport: Arc<dyn Transport>,
    topics: Arc<RwLock<HashMap<String, Vec<AgentPath>>>>,
    history: Arc<RwLock<VecDeque<AgentBusMessage>>>,
    dead_letter: Arc<RwLock<VecDeque<AgentBusMessage>>>,
    liveness: Arc<RwLock<HashMap<AgentPath, LivenessState>>>,
    pending_tasks: Arc<RwLock<HashMap<uuid::Uuid, TaskDescriptor>>>,
    on_unresponsive: Option<Arc<dyn Fn(AgentPath) + Send + Sync>>,
    max_history: usize,
    max_payload_bytes: usize,
}

impl AgentBus {
    pub(crate) fn new(buffer_size: usize, max_history: usize) -> Self {
        Self::with_transport(Arc::new(LocalTransport::new(buffer_size)), max_history)
    }

    pub(crate) fn with_transport(transport: Arc<dyn Transport>, max_history: usize) -> Self {
        Self {
            transport,
            topics: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(VecDeque::with_capacity(max_history))),
            dead_letter: Arc::new(RwLock::new(VecDeque::with_capacity(DEFAULT_DLQ_CAPACITY))),
            liveness: Arc::new(RwLock::new(HashMap::new())),
            pending_tasks: Arc::new(RwLock::new(HashMap::new())),
            on_unresponsive: None,
            max_history,
            max_payload_bytes: DEFAULT_MAX_PAYLOAD_BYTES,
        }
    }

    /// Create with default settings.
    pub(crate) fn default_config() -> Self {
        Self::new(DEFAULT_BUFFER_SIZE, DEFAULT_MAX_HISTORY)
    }

    /// Subscribe to the bus. Returns a [`Receiver`] that gets all broadcast messages.
    /// The subscriber should filter messages locally based on recipient.
    pub(crate) fn subscribe(&self) -> Receiver<AgentBusMessage> {
        self.transport.subscribe()
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

    pub(crate) async fn send(&self, message: AgentBusMessage) -> Result<usize, BusError> {
        let payload_size = message.payload.to_string().len();
        if payload_size > self.max_payload_bytes {
            return Err(BusError::PayloadTooLarge {
                size: payload_size,
                max: self.max_payload_bytes,
            });
        }

        let result = self.transport.send(message.clone()).await;
        if let Ok(mut history) = self.history.try_write() {
            if history.len() >= self.max_history {
                history.pop_front();
            }
            history.push_back(message);
        }
        match result {
            Ok(n) => Ok(n),
            Err(TransportError::SendFailed(_)) => Ok(0),
            Err(_) => Ok(0),
        }
    }

    /// Convenience: send a direct (point-to-point) message.
    pub(crate) async fn send_direct(
        &self,
        from: AgentPath,
        to: AgentPath,
        payload: serde_json::Value,
    ) -> Result<usize, BusError> {
        let msg = AgentBusMessage::direct(from, to, payload);
        self.send(msg).await
    }

    /// Convenience: publish to a topic.
    pub(crate) async fn publish(
        &self,
        from: AgentPath,
        topic: &str,
        payload: serde_json::Value,
    ) -> Result<usize, BusError> {
        let msg = AgentBusMessage::topic(from, topic, payload);
        self.send(msg).await
    }

    /// Convenience: broadcast to all agents of a role.
    pub(crate) async fn send_to_role(
        &self,
        from: AgentPath,
        role: &str,
        payload: serde_json::Value,
    ) -> Result<usize, BusError> {
        let msg = AgentBusMessage::role(from, role, payload);
        self.send(msg).await
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
        let mut delay_ms = config.initial_delay_ms;
        let mut attempts = 0;

        loop {
            let n = self.send(message.clone()).await?;
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
            delay_ms = ((delay_ms as f64) * config.backoff_multiplier)
                .min(config.max_delay_ms as f64) as u64;
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

    pub(crate) async fn replay_dead_letter(&self, message_id: uuid::Uuid) -> Result<usize, BusError> {
        let msg = {
            let dlq = self.dead_letter.try_read();
            match dlq {
                Ok(q) => q.iter().find(|m| m.id == message_id).cloned(),
                Err(_) => None,
            }
        };
        match msg {
            Some(m) => self.send(m).await,
            None => Ok(0),
        }
    }

    // ── Liveness tracking ──────────────────────────────────────────

    pub(crate) async fn register_agent(&self, path: AgentPath) {
        let mut liveness = self.liveness.write().await;
        liveness.entry(path).or_insert(LivenessState {
            last_seen: Instant::now(),
            missed_count: 0,
            status: AgentLiveness::Unknown,
        });
    }

    pub(crate) async fn unregister_agent(&self, path: &AgentPath) {
        self.liveness.write().await.remove(path);
    }

    pub(crate) async fn record_agent_alive(&self, path: &AgentPath) {
        let mut liveness = self.liveness.write().await;
        if let Some(state) = liveness.get_mut(path) {
            state.last_seen = Instant::now();
            state.missed_count = 0;
            state.status = AgentLiveness::Alive;
        }
    }

    pub(crate) async fn handle_heartbeat_ack(&self, from: &AgentPath) {
        self.record_agent_alive(from).await;
    }

    pub(crate) async fn agent_liveness(&self, path: &AgentPath) -> AgentLiveness {
        let liveness = self.liveness.read().await;
        liveness
            .get(path)
            .map(|s| s.status)
            .unwrap_or(AgentLiveness::Unknown)
    }

    pub(crate) async fn liveness_snapshot(&self) -> HashMap<AgentPath, AgentLiveness> {
        let liveness = self.liveness.read().await;
        liveness
            .iter()
            .map(|(k, v)| (k.clone(), v.status))
            .collect()
    }

    pub(crate) fn set_on_unresponsive(&mut self, callback: Arc<dyn Fn(AgentPath) + Send + Sync>) {
        self.on_unresponsive = Some(callback);
    }

    pub(crate) async fn check_liveness(&self, config: &HeartbeatConfig) {
        let mut liveness = self.liveness.write().await;
        let timeout = std::time::Duration::from_secs(config.timeout_secs);
        let now = Instant::now();
        let mut newly_unresponsive: Vec<AgentPath> = Vec::new();

        for (path, state) in liveness.iter_mut() {
            if now.duration_since(state.last_seen) > timeout {
                state.missed_count += 1;
                if state.missed_count >= config.max_missed
                    && state.status != AgentLiveness::Unresponsive
                {
                    state.status = AgentLiveness::Unresponsive;
                    newly_unresponsive.push(path.clone());
                }
            }
        }
        drop(liveness);

        if let Some(ref callback) = self.on_unresponsive {
            for path in newly_unresponsive {
                tracing::warn!(agent = %path, "agent became unresponsive, triggering callback");
                callback(path);
            }
        }
    }

    // ── Task tracking ─────────────────────────────────────────────

    pub(crate) async fn track_task(&self, descriptor: TaskDescriptor) {
        self.pending_tasks.write().await.insert(descriptor.task_id, descriptor);
    }

    pub(crate) async fn complete_task(&self, task_id: &uuid::Uuid) -> Option<TaskDescriptor> {
        self.pending_tasks.write().await.remove(task_id)
    }

    pub(crate) async fn pending_tasks_for(&self, path: &AgentPath) -> Vec<TaskDescriptor> {
        self.pending_tasks
            .read()
            .await
            .values()
            .filter(|t| &t.assigned_to == path)
            .cloned()
            .collect()
    }

    pub(crate) async fn reassign_orphaned_tasks(
        &self,
        dead_agent: &AgentPath,
        new_agent: &AgentPath,
    ) -> usize {
        let mut tasks = self.pending_tasks.write().await;
        let mut reassigned = 0;
        for (_, descriptor) in tasks.iter_mut() {
            if &descriptor.assigned_to == dead_agent {
                descriptor.assigned_to = new_agent.clone();
                reassigned += 1;
            }
        }
        if reassigned > 0 {
            tracing::info!(
                dead_agent = %dead_agent,
                new_agent = %new_agent,
                reassigned,
                "reassigned orphaned tasks"
            );
        }
        reassigned
    }

    pub(crate) async fn orphaned_tasks_by_role(
        &self,
        dead_agent: &AgentPath,
    ) -> HashMap<String, Vec<TaskDescriptor>> {
        let tasks = self.pending_tasks.read().await;
        let mut by_role: HashMap<String, Vec<TaskDescriptor>> = HashMap::new();
        for (_, descriptor) in tasks.iter() {
            if &descriptor.assigned_to == dead_agent {
                if let Some(ref role) = descriptor.agent_role {
                    by_role.entry(role.clone()).or_default().push(descriptor.clone());
                }
            }
        }
        by_role
    }

    pub(crate) async fn pending_task_count(&self) -> usize {
        self.pending_tasks.read().await.len()
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
        .await
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

        bus.publish(from.clone(), "test.topic", serde_json::json!("data1"))
            .await.unwrap();
        bus.publish(from.clone(), "test.topic", serde_json::json!("data2"))
            .await.unwrap();

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
                .await.unwrap();
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
            .await.unwrap();

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
            .await.unwrap();

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
        let msg = AgentBusMessage::direct(from, test_path("receiver"), serde_json::json!("hello"));

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
        let msg =
            AgentBusMessage::direct(from, test_path("nonexistent"), serde_json::json!("lost"));

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
            let msg = AgentBusMessage::topic(test_path("sender"), "orphan", serde_json::json!(i));
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
        let result = bus.replay_dead_letter(msg_id).await.unwrap();
        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn liveness_register_and_query() {
        let bus = AgentBus::new(16, 100);
        let agent = test_path("worker");

        assert_eq!(bus.agent_liveness(&agent).await, AgentLiveness::Unknown);

        bus.register_agent(agent.clone()).await;
        assert_eq!(bus.agent_liveness(&agent).await, AgentLiveness::Unknown);

        bus.handle_heartbeat_ack(&agent).await;
        assert_eq!(bus.agent_liveness(&agent).await, AgentLiveness::Alive);
    }

    #[tokio::test]
    async fn liveness_snapshot_returns_all() {
        let bus = AgentBus::new(16, 100);
        let a = test_path("a");
        let b = test_path("b");

        bus.register_agent(a.clone()).await;
        bus.register_agent(b.clone()).await;
        bus.handle_heartbeat_ack(&a).await;

        let snap = bus.liveness_snapshot().await;
        assert_eq!(snap.len(), 2);
        assert_eq!(snap[&a], AgentLiveness::Alive);
        assert_eq!(snap[&b], AgentLiveness::Unknown);
    }

    #[tokio::test]
    async fn liveness_unregister_removes() {
        let bus = AgentBus::new(16, 100);
        let agent = test_path("temp");

        bus.register_agent(agent.clone()).await;
        bus.handle_heartbeat_ack(&agent).await;
        assert_eq!(bus.agent_liveness(&agent).await, AgentLiveness::Alive);

        bus.unregister_agent(&agent).await;
        assert_eq!(bus.agent_liveness(&agent).await, AgentLiveness::Unknown);
    }

    #[tokio::test]
    async fn check_liveness_marks_unresponsive() {
        let bus = AgentBus::new(16, 100);
        let agent = test_path("stale");

        bus.register_agent(agent.clone()).await;
        bus.handle_heartbeat_ack(&agent).await;

        // Manually backdate last_seen beyond timeout
        {
            let mut lv = bus.liveness.write().await;
            let state = lv.get_mut(&agent).unwrap();
            state.last_seen = Instant::now() - std::time::Duration::from_secs(60);
        }

        let config = HeartbeatConfig {
            timeout_secs: 30,
            max_missed: 1,
            ..Default::default()
        };
        bus.check_liveness(&config).await;

        assert_eq!(
            bus.agent_liveness(&agent).await,
            AgentLiveness::Unresponsive
        );
    }

    #[tokio::test]
    async fn oversized_payload_rejected() {
        let bus = AgentBus::new(16, 100);
        let big_payload = serde_json::json!("x".repeat(DEFAULT_MAX_PAYLOAD_BYTES + 1));
        let result = bus.send(AgentBusMessage::direct(
            test_path("a"),
            test_path("b"),
            big_payload,
        ))
        .await;
        match result {
            Err(BusError::PayloadTooLarge { size, max }) => {
                assert!(size > max);
                assert_eq!(max, DEFAULT_MAX_PAYLOAD_BYTES);
            }
            other => panic!("expected PayloadTooLarge, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn normal_payload_accepted() {
        let bus = AgentBus::new(16, 100);
        let result = bus.send(AgentBusMessage::direct(
            test_path("a"),
            test_path("b"),
            serde_json::json!({"key": "value"}),
        ))
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn track_and_complete_task() {
        let bus = AgentBus::default_config();
        let agent = test_path("worker");
        let task_id = uuid::Uuid::new_v4();

        let descriptor = TaskDescriptor {
            task_id,
            assigned_to: agent.clone(),
            agent_role: Some("search".to_string()),
            request_message: AgentBusMessage::direct(
                test_path("orchestrator"),
                agent.clone(),
                serde_json::json!("search patent"),
            ),
            tracked_at_ms: 1000,
        };

        bus.track_task(descriptor).await;
        assert_eq!(bus.pending_task_count().await, 1);

        let pending = bus.pending_tasks_for(&agent).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].task_id, task_id);

        let completed = bus.complete_task(&task_id).await;
        assert!(completed.is_some());
        assert_eq!(bus.pending_task_count().await, 0);
    }

    #[tokio::test]
    async fn reassign_orphaned_tasks() {
        let bus = AgentBus::default_config();
        let dead_agent = test_path("worker-1");
        let new_agent = test_path("worker-2");

        for i in 0..3 {
            let descriptor = TaskDescriptor {
                task_id: uuid::Uuid::new_v4(),
                assigned_to: dead_agent.clone(),
                agent_role: Some("search".to_string()),
                request_message: AgentBusMessage::direct(
                    test_path("orchestrator"),
                    dead_agent.clone(),
                    serde_json::json!(format!("task-{i}")),
                ),
                tracked_at_ms: 1000 + i as i64,
            };
            bus.track_task(descriptor).await;
        }

        let other_task = TaskDescriptor {
            task_id: uuid::Uuid::new_v4(),
            assigned_to: test_path("other"),
            agent_role: Some("search".to_string()),
            request_message: AgentBusMessage::direct(
                test_path("orchestrator"),
                test_path("other"),
                serde_json::json!("other-task"),
            ),
            tracked_at_ms: 2000,
        };
        bus.track_task(other_task).await;

        assert_eq!(bus.pending_task_count().await, 4);

        let reassigned = bus.reassign_orphaned_tasks(&dead_agent, &new_agent).await;
        assert_eq!(reassigned, 3);

        assert_eq!(bus.pending_tasks_for(&dead_agent).await.len(), 0);
        assert_eq!(bus.pending_tasks_for(&new_agent).await.len(), 3);
        assert_eq!(bus.pending_task_count().await, 4);
    }

    #[tokio::test]
    async fn orphaned_tasks_by_role() {
        let bus = AgentBus::default_config();
        let dead_agent = test_path("dead-worker");

        for i in 0..2 {
            let descriptor = TaskDescriptor {
                task_id: uuid::Uuid::new_v4(),
                assigned_to: dead_agent.clone(),
                agent_role: Some("search".to_string()),
                request_message: AgentBusMessage::direct(
                    test_path("orch"),
                    dead_agent.clone(),
                    serde_json::json!(i),
                ),
                tracked_at_ms: 1000,
            };
            bus.track_task(descriptor).await;
        }

        let descriptor = TaskDescriptor {
            task_id: uuid::Uuid::new_v4(),
            assigned_to: dead_agent.clone(),
            agent_role: Some("analysis".to_string()),
            request_message: AgentBusMessage::direct(
                test_path("orch"),
                dead_agent.clone(),
                serde_json::json!("analyze"),
            ),
            tracked_at_ms: 2000,
        };
        bus.track_task(descriptor).await;

        let by_role = bus.orphaned_tasks_by_role(&dead_agent).await;
        assert_eq!(by_role.get("search").map(|v| v.len()), Some(2));
        assert_eq!(by_role.get("analysis").map(|v| v.len()), Some(1));
    }
}

#[cfg(test)]
#[path = "bus_concurrency_tests.rs"]
mod bus_concurrency_tests;

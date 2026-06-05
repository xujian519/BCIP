//! 事件总线：Agent 间松耦合通信
//!
//! 替代直接的函数调用，提供发布-订阅模式的事件通信：
//! - `EventBus`：异步事件总线，支持多订阅者
//! - `AgentEvent`：统一的 agent 生命周期事件类型
//! - 支持通配符和 topic 过滤

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::sync::Mutex;

/// Agent 事件类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AgentEvent {
    /// Agent 被创建
    AgentSpawned {
        agent_id: String,
        role: String,
        parent_id: Option<String>,
    },
    /// Agent 开始执行
    AgentStarted {
        agent_id: String,
        task: String,
    },
    /// Agent 执行完成
    AgentCompleted {
        agent_id: String,
        success: bool,
        duration_ms: u64,
        quality_score: Option<f64>,
    },
    /// Agent 产生输出
    AgentOutput {
        agent_id: String,
        output_type: String,
        summary: String,
    },
    /// Agent 请求协作
    CollaborationRequest {
        from_agent: String,
        to_agent: String,
        request_type: String,
        payload: serde_json::Value,
    },
    /// 协作响应
    CollaborationResponse {
        from_agent: String,
        to_agent: String,
        success: bool,
        payload: serde_json::Value,
    },
    /// Agent 遇到错误
    AgentError {
        agent_id: String,
        error: String,
        recoverable: bool,
    },
    /// 学习事件（反馈收集完成、策略更新等）
    LearningEvent {
        event_type: String,
        agent_id: String,
        data: serde_json::Value,
    },
    /// 自定义事件
    Custom {
        topic: String,
        source: String,
        payload: serde_json::Value,
    },
}

impl AgentEvent {
    /// 事件的 topic 标识，用于订阅过滤
    pub fn topic(&self) -> &str {
        match self {
            Self::AgentSpawned { .. } => "agent.spawned",
            Self::AgentStarted { .. } => "agent.started",
            Self::AgentCompleted { .. } => "agent.completed",
            Self::AgentOutput { .. } => "agent.output",
            Self::CollaborationRequest { .. } => "collaboration.request",
            Self::CollaborationResponse { .. } => "collaboration.response",
            Self::AgentError { .. } => "agent.error",
            Self::LearningEvent { .. } => "learning",
            Self::Custom { topic, .. } => topic,
        }
    }

    /// 事件来源 agent
    pub fn source(&self) -> &str {
        match self {
            Self::AgentSpawned { agent_id, .. } => agent_id,
            Self::AgentStarted { agent_id, .. } => agent_id,
            Self::AgentCompleted { agent_id, .. } => agent_id,
            Self::AgentOutput { agent_id, .. } => agent_id,
            Self::CollaborationRequest { from_agent, .. } => from_agent,
            Self::CollaborationResponse { from_agent, .. } => from_agent,
            Self::AgentError { agent_id, .. } => agent_id,
            Self::LearningEvent { agent_id, .. } => agent_id,
            Self::Custom { source, .. } => source,
        }
    }
}

impl fmt::Display for AgentEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AgentSpawned { agent_id, role, .. } => {
                write!(f, "[spawned] {agent_id} ({role})")
            }
            Self::AgentStarted { agent_id, task } => {
                write!(f, "[started] {agent_id}: {task}")
            }
            Self::AgentCompleted { agent_id, success, duration_ms, .. } => {
                let status = if *success { "✓" } else { "✗" };
                write!(f, "[completed] {agent_id} {status} ({duration_ms}ms)")
            }
            Self::AgentOutput { agent_id, output_type, summary } => {
                write!(f, "[output] {agent_id} {output_type}: {summary}")
            }
            Self::CollaborationRequest { from_agent, to_agent, request_type, .. } => {
                write!(f, "[collab] {from_agent} → {to_agent}: {request_type}")
            }
            Self::CollaborationResponse { from_agent, to_agent, success, .. } => {
                let status = if *success { "ok" } else { "fail" };
                write!(f, "[collab-reply] {from_agent} → {to_agent}: {status}")
            }
            Self::AgentError { agent_id, error, recoverable } => {
                let rec = if *recoverable { "recoverable" } else { "fatal" };
                write!(f, "[error] {agent_id}: {error} ({rec})")
            }
            Self::LearningEvent { event_type, agent_id, .. } => {
                write!(f, "[learning] {agent_id}: {event_type}")
            }
            Self::Custom { topic, source, .. } => {
                write!(f, "[custom] {source} → {topic}")
            }
        }
    }
}

/// 事件订阅者
struct Subscriber {
    topic_filter: Option<String>,
    sender: broadcast::Sender<AgentEvent>,
}

/// 异步事件总线
pub struct EventBus {
    subscribers: Arc<Mutex<HashMap<String, Subscriber>>>,
    default_channel: broadcast::Sender<AgentEvent>,
}

/// 事件总线容量
const DEFAULT_CHANNEL_CAPACITY: usize = 1024;

impl EventBus {
    /// 创建新的事件总线
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(DEFAULT_CHANNEL_CAPACITY);
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            default_channel: tx,
        }
    }

    /// 发布事件到总线
    pub async fn publish(&self, event: AgentEvent) {
        tracing::debug!(topic = %event.topic(), source = %event.source(), "发布事件: {event}");

        // 发送到默认 channel
        let _ = self.default_channel.send(event.clone());

        // 发送到匹配的订阅者
        let subs = self.subscribers.lock().await;
        for (_, subscriber) in subs.iter() {
            // 检查 topic 过滤
            if let Some(ref filter) = subscriber.topic_filter {
                if !event.topic().starts_with(filter) && filter != "*" {
                    continue;
                }
            }
            let _ = subscriber.sender.send(event.clone());
        }
    }

    /// 订阅所有事件
    pub fn subscribe_all(&self) -> broadcast::Receiver<AgentEvent> {
        self.default_channel.subscribe()
    }

    /// 按topic 订阅事件
    pub async fn subscribe(&self, subscriber_id: &str, topic_filter: Option<&str>) -> broadcast::Receiver<AgentEvent> {
        let (tx, rx) = broadcast::channel(256);
        let mut subs = self.subscribers.lock().await;
        subs.insert(
            subscriber_id.to_string(),
            Subscriber {
                topic_filter: topic_filter.map(|s| s.to_string()),
                sender: tx,
            },
        );
        rx
    }

    /// 取消订阅
    pub async fn unsubscribe(&self, subscriber_id: &str) {
        let mut subs = self.subscribers.lock().await;
        subs.remove(subscriber_id);
    }

    /// 获取当前订阅者数量
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.lock().await.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_all();

        bus.publish(AgentEvent::AgentSpawned {
            agent_id: "test-1".to_string(),
            role: "analyzer".to_string(),
            parent_id: None,
        }).await;

        let event = rx.try_recv().unwrap();
        assert!(matches!(event, AgentEvent::AgentSpawned { .. }));
    }

    #[tokio::test]
    async fn test_topic_filter() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe("sub-1", Some("collaboration")).await;

        // 发布不匹配的事件
        bus.publish(AgentEvent::AgentStarted {
            agent_id: "a1".to_string(),
            task: "test".to_string(),
        }).await;

        // 发布匹配的事件
        bus.publish(AgentEvent::CollaborationRequest {
            from_agent: "a1".to_string(),
            to_agent: "a2".to_string(),
            request_type: "help".to_string(),
            payload: serde_json::json!({}),
        }).await;

        // 应该只收到匹配的事件
        let event = rx.try_recv().unwrap();
        assert!(matches!(event, AgentEvent::CollaborationRequest { .. }));
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let bus = EventBus::new();
        bus.subscribe("sub-1", None).await;
        assert_eq!(bus.subscriber_count().await, 1);

        bus.unsubscribe("sub-1").await;
        assert_eq!(bus.subscriber_count().await, 0);
    }

    #[tokio::test]
    async fn test_event_display() {
        let event = AgentEvent::AgentCompleted {
            agent_id: "a1".to_string(),
            success: true,
            duration_ms: 1500,
            quality_score: Some(0.85),
        };
        let display = format!("{event}");
        assert!(display.contains("a1"));
        assert!(display.contains("✓"));
    }

    #[tokio::test]
    async fn test_event_topic() {
        let event = AgentEvent::AgentError {
            agent_id: "a1".to_string(),
            error: "timeout".to_string(),
            recoverable: true,
        };
        assert_eq!(event.topic(), "agent.error");
    }

    #[tokio::test]
    async fn test_custom_event() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_all();

        bus.publish(AgentEvent::Custom {
            topic: "patent.search".to_string(),
            source: "retriever".to_string(),
            payload: serde_json::json!({"query": "AI patent"}),
        }).await;

        let event = rx.try_recv().unwrap();
        assert_eq!(event.topic(), "patent.search");
        assert_eq!(event.source(), "retriever");
    }

    #[tokio::test]
    async fn test_wildcard_filter() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe("wildcard", Some("*")).await;

        bus.publish(AgentEvent::AgentStarted {
            agent_id: "a1".to_string(),
            task: "test".to_string(),
        }).await;

        let event = rx.try_recv().unwrap();
        assert!(matches!(event, AgentEvent::AgentStarted { .. }));
    }
}

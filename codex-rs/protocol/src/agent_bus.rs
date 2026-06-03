//! AgentBus message envelope types for cross-agent communication.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::AgentPath;

/// Unified message envelope for AgentBus.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct AgentBusMessage {
    /// Unique message ID.
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub id: Uuid,
    /// Sender agent path.
    pub from: AgentPath,
    /// Recipient addressing mode.
    pub to: AgentBusRecipient,
    /// Message type classification.
    pub message_type: AgentBusMessageType,
    /// JSON payload.
    pub payload: serde_json::Value,
    /// Unix timestamp in seconds.
    pub timestamp: i64,
    /// Correlation ID for request-response pairing.
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub correlation_id: Option<Uuid>,
    /// Priority level.
    pub priority: MessagePriority,
}

/// Recipient addressing modes.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub enum AgentBusRecipient {
    /// Point-to-point message to a specific agent.
    Direct(AgentPath),
    /// Broadcast to all agents on the bus.
    Broadcast,
    /// Topic-based publish/subscribe (e.g., "patent.search.results").
    Topic(String),
    /// Broadcast to all agents of a given role.
    Role(String),
}

/// Message type classification.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub enum AgentBusMessageType {
    /// Task request.
    TaskRequest,
    /// Task result/response.
    TaskResult,
    /// Progress update.
    Progress,
    /// Error notification.
    Error,
    /// System lifecycle event (agent spawn/shutdown).
    SystemEvent,
    /// Custom application-specific type.
    Custom(String),
}

/// Message priority (higher = more urgent).
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, TS, PartialEq, Eq, PartialOrd, Ord,
)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl AgentBusMessage {
    /// Create a new message with auto-generated ID and current timestamp.
    pub fn new(
        from: AgentPath,
        to: AgentBusRecipient,
        message_type: AgentBusMessageType,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            message_type,
            payload,
            timestamp: chrono::Utc::now().timestamp(),
            correlation_id: None,
            priority: MessagePriority::Normal,
        }
    }

    /// Create a direct (point-to-point) message.
    pub fn direct(from: AgentPath, to: AgentPath, payload: serde_json::Value) -> Self {
        Self::new(
            from,
            AgentBusRecipient::Direct(to),
            AgentBusMessageType::TaskRequest,
            payload,
        )
    }

    /// Create a topic broadcast message.
    pub fn topic(
        from: AgentPath,
        topic: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self::new(
            from,
            AgentBusRecipient::Topic(topic.into()),
            AgentBusMessageType::TaskRequest,
            payload,
        )
    }

    /// Create a role broadcast message.
    pub fn role(from: AgentPath, role: impl Into<String>, payload: serde_json::Value) -> Self {
        Self::new(
            from,
            AgentBusRecipient::Role(role.into()),
            AgentBusMessageType::TaskRequest,
            payload,
        )
    }

    /// Set correlation ID for request-response pairing.
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Convert from [`crate::protocol::InterAgentCommunication`] for backwards compatibility.
    pub fn from_inter_agent_comm(comm: &crate::protocol::InterAgentCommunication) -> Self {
        Self {
            id: Uuid::new_v4(),
            from: comm.author.clone(),
            to: AgentBusRecipient::Direct(comm.recipient.clone()),
            message_type: AgentBusMessageType::TaskRequest,
            payload: serde_json::json!({
                "content": comm.content,
                "trigger_turn": comm.trigger_turn,
                "other_recipients": comm.other_recipients.iter().map(|p| p.as_str()).collect::<Vec<_>>(),
            }),
            timestamp: chrono::Utc::now().timestamp(),
            correlation_id: None,
            priority: MessagePriority::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn test_path(name: &str) -> AgentPath {
        AgentPath::try_from(format!("/root/{name}")).unwrap()
    }

    #[test]
    fn message_serialization_roundtrip() {
        let from = test_path("sender");
        let to = test_path("receiver");
        let msg = AgentBusMessage::direct(
            from.clone(),
            to.clone(),
            serde_json::json!({"task": "search"}),
        );

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: AgentBusMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.from, from);
        assert_eq!(deserialized.id, msg.id);
        assert_eq!(deserialized.payload["task"], "search");
    }

    #[test]
    fn recipient_topic_serialization() {
        let recipient = AgentBusRecipient::Topic("patent.search.results".to_string());
        let json = serde_json::to_string(&recipient).unwrap();
        let back: AgentBusRecipient = serde_json::from_str(&json).unwrap();
        assert_eq!(recipient, back);
    }

    #[test]
    fn from_inter_agent_comm_conversion() {
        let author = test_path("alice");
        let recipient = test_path("bob");
        let comm = crate::protocol::InterAgentCommunication::new(
            author.clone(),
            recipient.clone(),
            vec![],
            "hello".to_string(),
            false,
        );

        let msg = AgentBusMessage::from_inter_agent_comm(&comm);
        assert_eq!(msg.from, author);
        match msg.to {
            AgentBusRecipient::Direct(p) => assert_eq!(p, recipient),
            other => panic!("expected Direct, got {other:?}"),
        }
        assert_eq!(msg.payload["content"], "hello");
    }

    #[test]
    fn priority_ordering() {
        assert!(MessagePriority::Critical > MessagePriority::High);
        assert!(MessagePriority::High > MessagePriority::Normal);
        assert!(MessagePriority::Normal > MessagePriority::Low);
    }

    #[test]
    fn with_correlation_id_and_priority() {
        let from = test_path("a");
        let id = Uuid::new_v4();
        let msg = AgentBusMessage::direct(from, test_path("b"), serde_json::json!(null))
            .with_correlation_id(id)
            .with_priority(MessagePriority::Critical);

        assert_eq!(msg.correlation_id, Some(id));
        assert_eq!(msg.priority, MessagePriority::Critical);
    }
}

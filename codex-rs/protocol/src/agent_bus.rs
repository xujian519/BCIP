//! AgentBus message envelope types for cross-agent communication.

use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::AgentPath;

/// Current protocol version for AgentBus messages.
pub const PROTOCOL_VERSION: u32 = 1;

fn default_protocol_version() -> u32 {
    PROTOCOL_VERSION
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &i64) -> bool {
    *v == 0
}

/// Unified message envelope for AgentBus.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct AgentBusMessage {
    /// Unique message ID.
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub id: Uuid,
    /// Protocol version for forward compatibility.
    #[serde(default = "default_protocol_version")]
    pub version: u32,
    /// Sender agent path.
    pub from: AgentPath,
    /// Recipient addressing mode.
    pub to: AgentBusRecipient,
    /// Message type classification.
    pub message_type: AgentBusMessageType,
    /// JSON payload.
    pub payload: serde_json::Value,
    /// Optional payload type hint for typed deserialization.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload_type: Option<String>,
    /// Unix timestamp in seconds (deprecated, use timestamp_ms).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub timestamp: i64,
    /// Unix timestamp in milliseconds (higher precision).
    pub timestamp_ms: i64,
    /// Correlation ID for request-response pairing.
    #[schemars(with = "Option<String>")]
    #[ts(type = "string | null")]
    pub correlation_id: Option<Uuid>,
    /// Priority level.
    pub priority: MessagePriority,
}

/// Recipient addressing modes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
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
    /// Agent heartbeat probe.
    Heartbeat,
    /// Agent heartbeat acknowledgement.
    HeartbeatAck,
    /// Custom application-specific type.
    Custom(String),
}

/// Message priority (higher = more urgent).
#[derive(
    Debug, Clone, Serialize, Deserialize, JsonSchema, TS, PartialEq, Eq, PartialOrd, Ord, Default,
)]
pub enum MessagePriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Heartbeat configuration for agent liveness detection.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct HeartbeatConfig {
    /// Interval between heartbeat probes in seconds.
    pub interval_secs: u64,
    /// Time without a response before an agent is considered unresponsive.
    pub timeout_secs: u64,
    /// Maximum number of consecutive missed heartbeats before marking unresponsive.
    pub max_missed: u32,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval_secs: 10,
            timeout_secs: 30,
            max_missed: 3,
        }
    }
}

/// Error type for payload decoding.
#[derive(Debug, thiserror::Error)]
pub enum PayloadDecodeError {
    #[error("payload decode failed: {0}")]
    DecodeFailed(#[from] serde_json::Error),
}

/// Agent liveness status as observed by the heartbeat monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
pub enum AgentLiveness {
    /// Agent is responding to heartbeats.
    Alive,
    /// Agent has missed heartbeats and is considered unresponsive.
    Unresponsive,
    /// Liveness state is unknown (not yet probed).
    Unknown,
}

/// Describes a task currently in-flight on the bus.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
pub struct TaskDescriptor {
    /// Unique task ID.
    #[schemars(with = "String")]
    #[ts(type = "string")]
    pub task_id: Uuid,
    /// Agent that owns (or owned) this task.
    pub assigned_to: AgentPath,
    /// Role of the agent (for reassignment matching).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_role: Option<String>,
    /// Original TaskRequest message.
    pub request_message: AgentBusMessage,
    /// When the task was tracked (Unix milliseconds).
    pub tracked_at_ms: i64,
}

impl AgentBusMessage {
    /// Create a new message with auto-generated ID and current timestamp.
    pub fn new(
        from: AgentPath,
        to: AgentBusRecipient,
        message_type: AgentBusMessageType,
        payload: serde_json::Value,
    ) -> Self {
        let now_ms = chrono::Utc::now().timestamp_millis();
        Self {
            id: Uuid::new_v4(),
            version: PROTOCOL_VERSION,
            from,
            to,
            message_type,
            payload,
            payload_type: None,
            timestamp: now_ms / 1000,
            timestamp_ms: now_ms,
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
    pub fn topic(from: AgentPath, topic: impl Into<String>, payload: serde_json::Value) -> Self {
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

    /// Set payload type hint.
    pub fn with_payload_type(mut self, ty: impl Into<String>) -> Self {
        self.payload_type = Some(ty.into());
        self
    }

    /// Decode payload into a typed value.
    pub fn decode_payload<T: serde::de::DeserializeOwned>(&self) -> Result<T, PayloadDecodeError> {
        serde_json::from_value(self.payload.clone()).map_err(PayloadDecodeError::DecodeFailed)
    }

    /// Convert from [`crate::protocol::InterAgentCommunication`] for backwards compatibility.
    pub fn from_inter_agent_comm(comm: &crate::protocol::InterAgentCommunication) -> Self {
        let now_ms = chrono::Utc::now().timestamp_millis();
        Self {
            id: Uuid::new_v4(),
            version: PROTOCOL_VERSION,
            from: comm.author.clone(),
            to: AgentBusRecipient::Direct(comm.recipient.clone()),
            message_type: AgentBusMessageType::TaskRequest,
            payload: serde_json::json!({
                "content": comm.content,
                "trigger_turn": comm.trigger_turn,
                "other_recipients": comm.other_recipients.iter().map(AgentPath::as_str).collect::<Vec<_>>(),
            }),
            payload_type: None,
            timestamp: now_ms / 1000,
            timestamp_ms: now_ms,
            correlation_id: None,
            priority: MessagePriority::Normal,
        }
    }

    /// Create a heartbeat probe message.
    pub fn heartbeat(from: AgentPath, seq: u64) -> Self {
        Self::new(
            from,
            AgentBusRecipient::Broadcast,
            AgentBusMessageType::Heartbeat,
            serde_json::json!({ "seq": seq }),
        )
    }

    /// Create a heartbeat acknowledgement message.
    pub fn heartbeat_ack(from: AgentPath, to: AgentPath, seq: u64) -> Self {
        Self::new(
            from,
            AgentBusRecipient::Direct(to),
            AgentBusMessageType::HeartbeatAck,
            serde_json::json!({ "seq": seq }),
        )
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

    #[test]
    fn heartbeat_message_type_serialization() {
        let from = test_path("agent");
        let msg = AgentBusMessage::heartbeat(from.clone(), 42);

        assert_eq!(msg.message_type, AgentBusMessageType::Heartbeat);
        assert_eq!(msg.to, AgentBusRecipient::Broadcast);
        assert_eq!(msg.payload["seq"], 42);

        let json = serde_json::to_string(&msg).unwrap();
        let back: AgentBusMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.message_type, AgentBusMessageType::Heartbeat);
        assert_eq!(back.payload["seq"], 42);
    }

    #[test]
    fn heartbeat_ack_message_type() {
        let from = test_path("agent");
        let to = test_path("monitor");
        let msg = AgentBusMessage::heartbeat_ack(from.clone(), to.clone(), 7);

        assert_eq!(msg.message_type, AgentBusMessageType::HeartbeatAck);
        match msg.to {
            AgentBusRecipient::Direct(p) => assert_eq!(p, to),
            other => panic!("expected Direct, got {other:?}"),
        }
        assert_eq!(msg.payload["seq"], 7);
    }

    #[test]
    fn heartbeat_config_default_values() {
        let config = HeartbeatConfig::default();
        assert_eq!(config.interval_secs, 10);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_missed, 3);
    }

    #[test]
    fn heartbeat_config_serialization_roundtrip() {
        let config = HeartbeatConfig {
            interval_secs: 5,
            timeout_secs: 15,
            max_missed: 2,
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: HeartbeatConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.interval_secs, 5);
        assert_eq!(back.timeout_secs, 15);
        assert_eq!(back.max_missed, 2);
    }

    #[test]
    fn agent_liveness_serialization() {
        assert_eq!(
            serde_json::to_string(&AgentLiveness::Alive).unwrap(),
            "\"Alive\""
        );
        let json = "\"Unresponsive\"";
        let back: AgentLiveness = serde_json::from_str(json).unwrap();
        assert_eq!(back, AgentLiveness::Unresponsive);
    }

    #[test]
    fn new_message_has_protocol_version() {
        let msg = AgentBusMessage::direct(test_path("a"), test_path("b"), serde_json::json!(null));
        assert_eq!(msg.version, PROTOCOL_VERSION);
        assert!(msg.timestamp_ms > 0);
        assert_eq!(msg.timestamp, msg.timestamp_ms / 1000);
    }

    #[test]
    fn legacy_message_without_version_deserializes() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "from": "/root/a",
            "to": {"Direct": "/root/b"},
            "message_type": "TaskRequest",
            "payload": {"task": "search"},
            "timestamp": 1700000000,
            "timestamp_ms": 1700000000000,
            "correlation_id": null,
            "priority": "Normal"
        }"#;
        let msg: AgentBusMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.version, PROTOCOL_VERSION); // default
        assert_eq!(msg.payload["task"], "search");
    }

    #[test]
    fn payload_type_roundtrip() {
        let msg = AgentBusMessage::direct(
            test_path("a"),
            test_path("b"),
            serde_json::json!({"key": "value"}),
        )
        .with_payload_type("SearchRequest");

        assert_eq!(msg.payload_type.as_deref(), Some("SearchRequest"));

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("payload_type"));
    }

    #[test]
    fn decode_payload_success() {
        #[derive(serde::Deserialize)]
        struct SearchQuery {
            query: String,
        }
        let msg = AgentBusMessage::direct(
            test_path("a"),
            test_path("b"),
            serde_json::json!({"query": "patent AI"}),
        );
        let decoded: SearchQuery = msg.decode_payload().unwrap();
        assert_eq!(decoded.query, "patent AI");
    }

    #[test]
    fn decode_payload_type_mismatch_fails() {
        let msg = AgentBusMessage::direct(
            test_path("a"),
            test_path("b"),
            serde_json::json!("not an object"),
        );
        #[derive(serde::Deserialize)]
        struct NeedsObject {
            field: String,
        }
        assert!(msg.decode_payload::<NeedsObject>().is_err());
    }

    #[test]
    fn timestamp_ms_has_millisecond_precision() {
        let msg = AgentBusMessage::direct(test_path("a"), test_path("b"), serde_json::json!(null));
        assert!(msg.timestamp_ms % 1000 != msg.timestamp_ms);
        let json = serde_json::to_string(&msg).unwrap();
        let back: AgentBusMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(back.timestamp_ms, msg.timestamp_ms);
    }
}

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    Idle,
    Thinking,
    Streaming,
    ToolExecuting,
    PermissionPending,
    Compacting,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    Allow,
    AlwaysAllow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AttachmentType {
    #[serde(rename = "image")]
    Image(ImageAttachment),
    #[serde(rename = "document")]
    Document(DocumentAttachment),
    #[serde(rename = "audio")]
    Audio(AudioAttachment),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAttachment {
    pub mime_type: String,
    pub data_base64: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentAttachment {
    pub filename: String,
    pub mime_type: String,
    pub data_base64: String,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAttachment {
    pub mime_type: String,
    pub data_base64: String,
    pub duration_secs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub r#type: AttachmentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "user_message")]
    UserMessage {
        text: String,
        attachments: Option<Vec<Attachment>>,
    },
    #[serde(rename = "permission_response")]
    PermissionResponse {
        request_id: String,
        decision: PermissionDecision,
    },
    #[serde(rename = "stop_generation")]
    StopGeneration,
    #[serde(rename = "set_permission_mode")]
    SetPermissionMode { mode: String },
    #[serde(rename = "set_runtime_config")]
    SetRuntimeConfig {
        provider_id: Option<String>,
        model: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_input_tokens: Option<u64>,
    pub cache_creation_input_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "connected")]
    Connected { session_id: String },
    #[serde(rename = "content_start")]
    ContentStart {
        block_id: String,
        content_type: ContentBlockType,
    },
    #[serde(rename = "content_delta")]
    ContentDelta { block_id: String, delta: String },
    #[serde(rename = "thinking")]
    Thinking { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        block_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
    },
    #[serde(rename = "tool_use_complete")]
    ToolUseComplete { block_id: String },
    #[serde(rename = "tool_result")]
    ToolResult {
        block_id: String,
        output: String,
        is_error: bool,
    },
    #[serde(rename = "permission_request")]
    PermissionRequest {
        request_id: String,
        tool_name: String,
        tool_input: serde_json::Value,
        risk_level: RiskLevel,
        description: Option<String>,
    },
    #[serde(rename = "message_complete")]
    MessageComplete { stop_reason: String, usage: Usage },
    #[serde(rename = "status")]
    Status {
        state: SessionState,
        detail: Option<String>,
    },
    #[serde(rename = "error")]
    Error { code: String, message: String },
    #[serde(rename = "system_notification")]
    SystemNotification { level: String, text: String },
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "compact_start")]
    CompactStart,
    #[serde(rename = "compact_end")]
    CompactEnd,
    #[serde(rename = "image")]
    Image {
        mime_type: String,
        data_base64: String,
        alt_text: Option<String>,
    },
    #[serde(rename = "session_changed")]
    SessionChanged { session_id: String, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentBlockType {
    Text,
    ToolUse,
    Thinking,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedUser {
    pub platform: String,
    pub user_id: String,
    pub paired_at: DateTime<Utc>,
    pub display_name: Option<String>,
}

impl PairedUser {
    pub fn new(platform: impl Into<String>, user_id: impl Into<String>) -> Self {
        Self {
            platform: platform.into(),
            user_id: user_id.into(),
            paired_at: Utc::now(),
            display_name: None,
        }
    }
}

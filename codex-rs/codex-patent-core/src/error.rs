use thiserror::Error;

/// 工具错误分类：决定是否应该重试。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolErrorKind {
    /// 可重试：网络超时、连接拒绝、服务暂不可用等临时性错误
    Retryable,
    /// 致命错误：参数错误、权限不足、资源不存在等不可恢复错误
    Fatal,
}

/// 根据工具错误消息的文本模式进行分类。
///
/// 匹配常见的超时、网络、限流等可重试错误关键字。
/// 未匹配任何模式时默认为 `Fatal`（保守策略：不重试未知错误）。
pub fn classify_tool_error(msg: &str) -> ToolErrorKind {
    let lower = msg.to_lowercase();
    if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("connection")
        || lower.contains("network")
        || lower.contains("temporary")
        || lower.contains("rate limit")
        || lower.contains("429")
        || lower.contains("503")
        || lower.contains("502")
        || lower.contains("gateway")
        || lower.contains("unavailable")
        || lower.contains("eof")
        || lower.contains("reset")
        || lower.contains("refused")
        || lower.contains("broken pipe")
        || lower.contains("io error")
        || lower.contains("interrupted")
        || lower.contains("try again")
    {
        ToolErrorKind::Retryable
    } else {
        ToolErrorKind::Fatal
    }
}

/// API key 解析错误。
///
/// 用于阻断"代理 URL 被误当作 API key"这类典型故障。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyError {
    /// 环境变量未设置。
    Missing(String),
    /// 环境变量值为空或仅空白。
    Empty(String),
    /// 环境变量值看起来是代理 URL。
    SuspectedProxyValue { env_var: String, len: usize },
}

impl std::fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiKeyError::Missing(name) => write!(f, "environment variable `{name}` is not set"),
            ApiKeyError::Empty(name) => write!(f, "environment variable `{name}` is empty"),
            ApiKeyError::SuspectedProxyValue { env_var, len } => write!(
                f,
                "environment variable `{env_var}` (len={len}) looks like a proxy URL/placeholder, refusing to send as API key"
            ),
        }
    }
}

impl std::error::Error for ApiKeyError {}

#[derive(Error, Debug)]
pub enum PatentError {
    #[error("kg error: {0}")]
    KnowledgeGraph(String),
    #[error("law db error: {0}")]
    LawDb(String),
    #[error("search error: {0}")]
    Search(String),
    #[error("claim parse error: {0}")]
    ClaimParse(String),
    #[error("rule engine error: {0}")]
    RuleEngine(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("agent error: {0}")]
    Agent(String),
    #[error("api key error: {0}")]
    ApiKey(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("learning error: {0}")]
    Learning(String),
    #[error("reflection error: {0}")]
    Reflection(String),
}

impl From<serde_json::Error> for PatentError {
    fn from(e: serde_json::Error) -> Self {
        PatentError::Serialization(e.to_string())
    }
}

impl From<ApiKeyError> for PatentError {
    fn from(e: ApiKeyError) -> Self {
        PatentError::ApiKey(e.to_string())
    }
}

impl PatentError {
    /// 根据错误类别判断是否可重试。
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            PatentError::KnowledgeGraph(_)
                | PatentError::LawDb(_)
                | PatentError::Search(_)
                | PatentError::Io(_)
                | PatentError::Provider(_)
        )
    }
}

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

/// BCIP 专利系统统一错误类型。
///
/// 每个变体携带简明的字符串描述。部分变体通过 `is_retryable()` 标记可重试性。
#[derive(Error, Debug)]
pub enum PatentError {
    /// 知识图谱操作错误（查询、写入、连接等）。
    #[error("kg error: {0}")]
    KnowledgeGraph(String),
    /// 法律数据库操作错误。
    #[error("law db error: {0}")]
    LawDb(String),
    /// 检索/搜索失败。
    #[error("search error: {0}")]
    Search(String),
    /// 权利要求解析错误。
    #[error("claim parse error: {0}")]
    ClaimParse(String),
    /// 规则引擎执行错误。
    #[error("rule engine error: {0}")]
    RuleEngine(String),
    /// 底层 IO 错误。
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// 资源未找到（文档、配置项等）。
    #[error("not found: {0}")]
    NotFound(String),
    /// 配置解析/校验错误。
    #[error("config error: {0}")]
    Config(String),
    /// Agent 执行期间的错误。
    #[error("agent error: {0}")]
    Agent(String),
    /// API key 配置错误（缺失/无效）。
    #[error("api key error: {0}")]
    ApiKey(String),
    /// AI 服务提供商调用错误。
    #[error("provider error: {0}")]
    Provider(String),
    /// 序列化/反序列化错误。
    #[error("serialization error: {0}")]
    Serialization(String),
    /// 数据校验未通过。
    #[error("validation error: {0}")]
    Validation(String),
    /// 学习/训练流程错误。
    #[error("learning error: {0}")]
    Learning(String),
    /// 反思/自评流程错误。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_retryable_timeout() {
        assert_eq!(classify_tool_error("request timeout"), ToolErrorKind::Retryable);
    }

    #[test]
    fn classify_retryable_connection() {
        assert_eq!(classify_tool_error("connection refused"), ToolErrorKind::Retryable);
    }

    #[test]
    fn classify_retryable_rate_limit() {
        assert_eq!(classify_tool_error("rate limit exceeded"), ToolErrorKind::Retryable);
    }

    #[test]
    fn classify_retryable_429() {
        assert_eq!(classify_tool_error("HTTP 429"), ToolErrorKind::Retryable);
    }

    #[test]
    fn classify_retryable_gateway() {
        assert_eq!(classify_tool_error("502 bad gateway"), ToolErrorKind::Retryable);
    }

    #[test]
    fn classify_retryable_broken_pipe() {
        assert_eq!(classify_tool_error("broken pipe"), ToolErrorKind::Retryable);
    }

    #[test]
    fn classify_retryable_try_again() {
        assert_eq!(classify_tool_error("please try again"), ToolErrorKind::Retryable);
    }

    #[test]
    fn classify_fatal_invalid_arg() {
        assert_eq!(classify_tool_error("invalid argument"), ToolErrorKind::Fatal);
    }

    #[test]
    fn classify_fatal_permission() {
        assert_eq!(classify_tool_error("permission denied"), ToolErrorKind::Fatal);
    }

    #[test]
    fn classify_fatal_unknown() {
        assert_eq!(classify_tool_error("something went wrong"), ToolErrorKind::Fatal);
    }

    #[test]
    fn api_key_error_display_missing() {
        let err = ApiKeyError::Missing("OPENAI_API_KEY".into());
        assert!(err.to_string().contains("not set"));
    }

    #[test]
    fn api_key_error_display_empty() {
        let err = ApiKeyError::Empty("OPENAI_API_KEY".into());
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn api_key_error_display_proxy() {
        let err = ApiKeyError::SuspectedProxyValue {
            env_var: "OPENAI_API_KEY".into(),
            len: 42,
        };
        assert!(err.to_string().contains("proxy"));
    }

    #[test]
    fn patent_error_is_retryable_kg() {
        let err = PatentError::KnowledgeGraph("test".into());
        assert!(err.is_retryable());
    }

    #[test]
    fn patent_error_is_retryable_io() {
        let err = PatentError::Io(std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout"));
        assert!(err.is_retryable());
    }

    #[test]
    fn patent_error_not_retryable_config() {
        let err = PatentError::Config("bad config".into());
        assert!(!err.is_retryable());
    }

    #[test]
    fn patent_error_not_retryable_validation() {
        let err = PatentError::Validation("invalid".into());
        assert!(!err.is_retryable());
    }

    #[test]
    fn from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("{bad}");
        let err = PatentError::from(json_err.unwrap_err());
        assert!(matches!(err, PatentError::Serialization(_)));
    }

    #[test]
    fn from_api_key_error() {
        let err = PatentError::from(ApiKeyError::Missing("KEY".into()));
        assert!(matches!(err, PatentError::ApiKey(_)));
    }
}

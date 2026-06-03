use thiserror::Error;

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

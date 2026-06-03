//! 多 LLM provider 路由
//!
//! 支持检测模型类型并路由到对应的 LLM provider。
//! 默认 base URL 从内置 registry 提供，可通过 `$BCIP_HOME/config.toml`
//! 的 `[providers.*]` 段覆盖。

use std::collections::HashMap;
use std::fmt;

/// 一个 provider 的可配置条目。
///
/// 用于构建默认 registry 以及从 `config.toml` 加载用户覆盖。
#[derive(Debug, Clone)]
pub struct ProviderEntry {
    /// API base URL。
    pub base_url: String,
    /// 存放 API key 的环境变量名。
    pub api_key_env: String,
}

/// 返回内置的 6 个默认 provider 条目。
fn default_providers() -> HashMap<String, ProviderEntry> {
    let mut m = HashMap::new();
    m.insert(
        "deepseek".to_string(),
        ProviderEntry {
            base_url: "https://api.deepseek.com".to_string(),
            api_key_env: "DEEPSEEK_API_KEY".to_string(),
        },
    );
    m.insert(
        "qwen".to_string(),
        ProviderEntry {
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key_env: "QWEN_API_KEY".to_string(),
        },
    );
    m.insert(
        "moonshot".to_string(),
        ProviderEntry {
            base_url: "https://api.moonshot.cn/v1".to_string(),
            api_key_env: "MOONSHOT_API_KEY".to_string(),
        },
    );
    m.insert(
        "glm".to_string(),
        ProviderEntry {
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            api_key_env: "GLM_API_KEY".to_string(),
        },
    );
    m.insert(
        "openai".to_string(),
        ProviderEntry {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
        },
    );
    m.insert(
        "anthropic".to_string(),
        ProviderEntry {
            base_url: "https://api.anthropic.com".to_string(),
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
        },
    );
    m
}

/// 从 `$BCIP_HOME/config.toml` 的 `[providers.*]` 段加载用户覆盖。
///
/// 支持覆盖 `base_url` 和 `api_key_env`。
/// 若配置文件不存在或 `[providers]` 段缺失，返回空 map。
fn load_provider_overrides() -> HashMap<String, ProviderEntry> {
    let config_path = bcip_home().join("config.toml");
    if !config_path.exists() {
        return HashMap::new();
    }

    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    let root: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(_) => return HashMap::new(),
    };

    let providers = match root.get("providers").and_then(|v| v.as_table()) {
        Some(t) => t,
        None => return HashMap::new(),
    };

    let mut overrides = HashMap::new();
    for (name, val) in providers {
        let table = match val.as_table() {
            Some(t) => t,
            None => continue,
        };

        let mut entry = ProviderEntry {
            base_url: String::new(),
            api_key_env: String::new(),
        };

        if let Some(url) = table.get("base_url").and_then(|v| v.as_str()) {
            entry.base_url = url.trim().to_string();
        }
        if let Some(env) = table.get("api_key_env").and_then(|v| v.as_str()) {
            entry.api_key_env = env.trim().to_string();
        }

        overrides.insert(name.clone(), entry);
    }

    overrides
}

/// 构建合并后的 provider registry：默认值 + 用户覆盖。
///
/// 用户覆盖只替换显式设置的字段，未设置的字段保留默认值。
fn build_provider_registry() -> HashMap<String, ProviderEntry> {
    let mut registry = default_providers();
    let overrides = load_provider_overrides();

    for (name, override_entry) in overrides {
        // 只覆盖已知 provider 名
        if let Some(default) = registry.get(&name) {
            let merged = ProviderEntry {
                base_url: if override_entry.base_url.is_empty() {
                    default.base_url.clone()
                } else {
                    override_entry.base_url
                },
                api_key_env: if override_entry.api_key_env.is_empty() {
                    default.api_key_env.clone()
                } else {
                    override_entry.api_key_env
                },
            };
            registry.insert(name, merged);
        }
    }

    registry
}

/// API key 解析错误。
///
/// 主要用于阻断"代理 URL 被误当作 API key"这类典型故障：当环境变量值
/// 形如 `http://127.0.0.1:56186` 或 `Proxy` 等占位字符串时，直接向 LLM
/// 服务发送会触发 401，且日志只显示 `****roxy` 后缀，难以排查。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyError {
    /// 环境变量未设置。
    Missing(String),
    /// 环境变量值为空或仅空白。
    Empty(String),
    /// 环境变量值看起来是代理 URL（含 `http://` / `https://` / `proxy` 字面）。
    SuspectedProxyValue { env_var: String, len: usize },
}

impl fmt::Display for ApiKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[derive(Debug, Clone)]
pub enum AgentProvider {
    Anthropic {
        api_key_env: String,
    },
    OpenAiCompatible {
        base_url: String,
        api_key_env: String,
    },
}

impl AgentProvider {
    /// 返回此 provider 的 base URL。
    ///
    /// Anthropic 的 base URL 默认为 `https://api.anthropic.com`，
    /// 可通过 config.toml 覆盖。
    pub fn base_url(&self) -> &str {
        match self {
            AgentProvider::Anthropic { .. } => {
                // Anthropic 的 base_url 存储在 registry 中但不在此变体。
                // 这里返回默认值；实际调用时应使用 registry 中的值。
                // 为保持兼容，直接返回默认；如需覆盖则通过 registry 查找。
                "https://api.anthropic.com"
            }
            AgentProvider::OpenAiCompatible { base_url, .. } => base_url,
        }
    }
}

/// 根据模型名称推断 provider 名（用于 registry 查找）。
fn model_to_provider_name(model: &str) -> &'static str {
    let lower = model.to_ascii_lowercase();

    if lower.starts_with("claude-")
        || lower.contains("opus")
        || lower.contains("sonnet")
        || lower.contains("haiku")
    {
        return "anthropic";
    }

    if lower.contains("deepseek") || lower.starts_with("ds-") {
        return "deepseek";
    }

    if lower.contains("qwen") {
        return "qwen";
    }

    if lower.contains("kimi") || lower.contains("moonshot") {
        return "moonshot";
    }

    if lower.contains("glm") || lower.contains("chatglm") {
        return "glm";
    }

    if lower.starts_with("gpt-")
        || lower.starts_with("o1-")
        || lower.starts_with("o3-")
        || lower.starts_with("o4-")
        || lower.contains("chatgpt")
    {
        return "openai";
    }

    // 默认回退到 Anthropic
    "anthropic"
}

/// 根据模型名称检测对应的 provider，使用可配置 registry。
///
/// 若 `$BCIP_HOME/config.toml` 中有对应 provider 的 `base_url` 覆盖，
/// 则使用覆盖值；否则使用内置默认值。
/// 无配置文件时行为与原来完全一致。
pub fn detect_provider(model: &str) -> AgentProvider {
    let provider_name = model_to_provider_name(model);
    let registry = build_provider_registry();

    let entry = match registry.get(provider_name) {
        Some(e) => e,
        None => {
            // 不应发生：model_to_provider_name 总是返回已知 provider。
            return AgentProvider::Anthropic {
                api_key_env: "ANTHROPIC_API_KEY".to_string(),
            };
        }
    };

    if provider_name == "anthropic" {
        AgentProvider::Anthropic {
            api_key_env: entry.api_key_env.clone(),
        }
    } else {
        AgentProvider::OpenAiCompatible {
            base_url: entry.base_url.clone(),
            api_key_env: entry.api_key_env.clone(),
        }
    }
}

/// 根据模型名称获取 base URL（从 registry 查找，包含 Anthropic）。
///
/// 与 `AgentProvider::base_url()` 不同，此方法从 registry 获取，
/// 因此会反映 config.toml 中对 Anthropic base_url 的覆盖。
pub fn resolve_base_url(model: &str) -> String {
    let provider_name = model_to_provider_name(model);
    let registry = build_provider_registry();

    match registry.get(provider_name) {
        Some(entry) => entry.base_url.clone(),
        None => "https://api.anthropic.com".to_string(),
    }
}

/// 从环境变量解析 API key，遇到代理 URL/占位值时拒绝并返回错误。
///
/// 解析顺序：
/// 1. `std::env::var(env_var)`：未设置 → `Missing`；空 → `Empty`
/// 2. 合法性检查：值若含 `http://` / `https://` / `proxy`（忽略大小写）→ `SuspectedProxyValue`
/// 3. 通过 → 返回 `Ok(key)`
///
/// 调用方应当把 `Err` 当作硬错误中止请求，而非 fallback 到其他环境变量，
/// 以避免把代理 URL 误发送到 LLM 服务造成 401。
pub fn resolve_api_key(env_var: &str) -> Result<String, ApiKeyError> {
    let raw = std::env::var(env_var).map_err(|_| ApiKeyError::Missing(env_var.to_string()))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ApiKeyError::Empty(env_var.to_string()));
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("http://")
        || lower.contains("https://")
        || lower.contains("proxy")
        || lower.contains("socks")
    {
        return Err(ApiKeyError::SuspectedProxyValue {
            env_var: env_var.to_string(),
            len: trimmed.len(),
        });
    }
    Ok(trimmed.to_string())
}

/// 返回 API key 的脱敏指纹（前 4 + … + 后 4），用于日志/错误诊断。
/// 空/短 key 返回 `<empty>` / `<short:N>`。
pub fn mask_api_key(key: &str) -> String {
    let len = key.chars().count();
    if len == 0 {
        return "<empty>".to_string();
    }
    if len < 8 {
        return format!("<short:{len}>");
    }
    let head: String = key.chars().take(4).collect();
    let tail: String = key
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    format!("{head}…{tail} (len={len})")
}

/// 从配置文件解析 API key。
///
/// 配置文件路径：`$BCIP_HOME/config.toml` 或 `~/.bcip/config.toml`
///
/// 配置格式：
/// ```toml
/// [providers.deepseek]
/// api_key = "sk-xxx"
///
/// [providers.anthropic]
/// api_key = "sk-ant-xxx"
/// ```
pub fn resolve_api_key_from_config(env_var: &str) -> Option<String> {
    let provider_name = env_var_to_provider_name(env_var);
    if provider_name.is_empty() {
        return None;
    }

    let config_path = bcip_home().join("config.toml");
    if !config_path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(&config_path).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;

    value
        .get("providers")?
        .get(provider_name)?
        .get("api_key")?
        .as_str()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// 统一 API key 解析：环境变量优先，其次配置文件。
///
/// 1. 尝试环境变量 `env_var`
/// 2. 合法性检查（代理 URL 检测）
/// 3. 若环境变量缺失，尝试从 `~/.bcip/config.toml` 读取
pub fn resolve_provider_api_key(env_var: &str) -> Result<String, ApiKeyError> {
    match resolve_api_key(env_var) {
        Ok(key) => Ok(key),
        Err(ApiKeyError::Missing(_)) => resolve_api_key_from_config(env_var)
            .ok_or_else(|| ApiKeyError::Missing(env_var.to_string())),
        Err(e) => Err(e),
    }
}

fn env_var_to_provider_name(env_var: &str) -> &'static str {
    match env_var {
        "DEEPSEEK_API_KEY" => "deepseek",
        "QWEN_API_KEY" => "qwen",
        "MOONSHOT_API_KEY" => "moonshot",
        "GLM_API_KEY" => "glm",
        "OPENAI_API_KEY" => "openai",
        "ANTHROPIC_API_KEY" => "anthropic",
        _ => "",
    }
}

fn bcip_home() -> std::path::PathBuf {
    std::env::var("BCIP_HOME")
        .or_else(|_| std::env::var("CODEX_HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| dirs_home().join(".bcip"))
}

fn dirs_home() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_claude() {
        let provider = detect_provider("claude-3-5-sonnet-20241022");
        assert!(matches!(provider, AgentProvider::Anthropic { .. }));
    }

    #[test]
    fn test_detect_deepseek() {
        let provider = detect_provider("deepseek-v4-pro");
        assert!(matches!(provider, AgentProvider::OpenAiCompatible { .. }));
        if let AgentProvider::OpenAiCompatible {
            api_key_env,
            base_url,
            ..
        } = provider
        {
            assert_eq!(api_key_env, "DEEPSEEK_API_KEY");
            assert_eq!(base_url, "https://api.deepseek.com");
        }
    }

    #[test]
    fn test_detect_qwen() {
        let provider = detect_provider("qwen-turbo");
        assert!(matches!(provider, AgentProvider::OpenAiCompatible { .. }));
        if let AgentProvider::OpenAiCompatible {
            api_key_env,
            base_url,
            ..
        } = provider
        {
            assert_eq!(api_key_env, "QWEN_API_KEY");
            assert_eq!(
                base_url,
                "https://dashscope.aliyuncs.com/compatible-mode/v1"
            );
        }
    }

    #[test]
    fn test_detect_gpt() {
        let provider = detect_provider("gpt-4o");
        assert!(matches!(provider, AgentProvider::OpenAiCompatible { .. }));
        if let AgentProvider::OpenAiCompatible {
            api_key_env,
            base_url,
            ..
        } = provider
        {
            assert_eq!(api_key_env, "OPENAI_API_KEY");
            assert_eq!(base_url, "https://api.openai.com/v1");
        }
    }

    #[test]
    fn test_detect_default() {
        let provider = detect_provider("unknown-model");
        assert!(matches!(provider, AgentProvider::Anthropic { .. }));
    }

    #[test]
    fn test_default_providers_has_six_entries() {
        let defaults = default_providers();
        assert_eq!(defaults.len(), 6);
        assert!(defaults.contains_key("deepseek"));
        assert!(defaults.contains_key("qwen"));
        assert!(defaults.contains_key("moonshot"));
        assert!(defaults.contains_key("glm"));
        assert!(defaults.contains_key("openai"));
        assert!(defaults.contains_key("anthropic"));
    }

    #[test]
    fn test_model_to_provider_name() {
        assert_eq!(model_to_provider_name("claude-3-opus"), "anthropic");
        assert_eq!(model_to_provider_name("deepseek-v4"), "deepseek");
        assert_eq!(model_to_provider_name("ds-xxx"), "deepseek");
        assert_eq!(model_to_provider_name("qwen-turbo"), "qwen");
        assert_eq!(model_to_provider_name("moonshot-v1"), "moonshot");
        assert_eq!(model_to_provider_name("kimi-latest"), "moonshot");
        assert_eq!(model_to_provider_name("glm-4"), "glm");
        assert_eq!(model_to_provider_name("chatglm-3"), "glm");
        assert_eq!(model_to_provider_name("gpt-4o"), "openai");
        assert_eq!(model_to_provider_name("o1-preview"), "openai");
        assert_eq!(model_to_provider_name("unknown"), "anthropic");
    }

    #[test]
    fn test_base_url_method() {
        let provider = AgentProvider::OpenAiCompatible {
            base_url: "https://custom.example.com/v1".to_string(),
            api_key_env: "TEST_KEY".to_string(),
        };
        assert_eq!(provider.base_url(), "https://custom.example.com/v1");

        let anthropic = AgentProvider::Anthropic {
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
        };
        assert_eq!(anthropic.base_url(), "https://api.anthropic.com");
    }

    #[test]
    fn test_resolve_base_url() {
        // Without config overrides, should return default URLs
        let url = resolve_base_url("deepseek-v4");
        assert_eq!(url, "https://api.deepseek.com");

        let url = resolve_base_url("claude-3-opus");
        assert_eq!(url, "https://api.anthropic.com");
    }

    fn with_env<F: FnOnce()>(name: &str, value: Option<&str>, f: F) {
        let prev = std::env::var(name).ok();
        match value {
            Some(v) => unsafe { std::env::set_var(name, v) },
            None => unsafe { std::env::remove_var(name) },
        }
        f();
        match prev {
            Some(v) => unsafe { std::env::set_var(name, v) },
            None => unsafe { std::env::remove_var(name) },
        }
    }

    #[test]
    fn resolve_api_key_returns_missing_when_unset() {
        with_env("BCIP_TEST_KEY_NONE", None, || {
            let err = resolve_api_key("BCIP_TEST_KEY_NONE").expect_err("must error when unset");
            assert!(matches!(err, ApiKeyError::Missing(_)));
        });
    }

    #[test]
    fn resolve_api_key_rejects_blank() {
        with_env("BCIP_TEST_KEY_BLANK", Some("   "), || {
            let err = resolve_api_key("BCIP_TEST_KEY_BLANK").expect_err("must error on blank");
            assert!(matches!(err, ApiKeyError::Empty(_)));
        });
    }

    #[test]
    fn resolve_api_key_rejects_proxy_url() {
        with_env(
            "BCIP_TEST_KEY_PROXY",
            Some("http://127.0.0.1:56186"),
            || {
                let err =
                    resolve_api_key("BCIP_TEST_KEY_PROXY").expect_err("must error on proxy URL");
                assert!(matches!(err, ApiKeyError::SuspectedProxyValue { .. }));
            },
        );
    }

    #[test]
    fn resolve_api_key_rejects_proxy_literal() {
        with_env("BCIP_TEST_KEY_LITERAL", Some("Proxy"), || {
            let err =
                resolve_api_key("BCIP_TEST_KEY_LITERAL").expect_err("must error on Proxy literal");
            assert!(matches!(err, ApiKeyError::SuspectedProxyValue { .. }));
        });
    }

    #[test]
    fn resolve_api_key_accepts_normal_value() {
        with_env("BCIP_TEST_KEY_OK", Some("sk-deadbeef-1234"), || {
            let v = resolve_api_key("BCIP_TEST_KEY_OK").expect("normal key must pass");
            assert_eq!(v, "sk-deadbeef-1234");
        });
    }

    #[test]
    fn mask_api_key_truncates_middle() {
        assert_eq!(mask_api_key(""), "<empty>");
        assert_eq!(mask_api_key("abc"), "<short:3>");
        assert_eq!(mask_api_key("abcdefgh"), "abcd…efgh (len=8)");
        assert_eq!(mask_api_key("sk-deadbeef-12345678"), "sk-d…5678 (len=20)");
    }
}

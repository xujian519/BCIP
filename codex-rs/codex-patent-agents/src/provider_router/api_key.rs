//! API key 解析：环境变量、配置文件、脱敏。

use codex_patent_core::ApiKeyError;

use super::config::bcip_home;
use super::config::env_var_to_provider_name;

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

//! Provider 注册表：默认条目、配置加载、合并、模型检测。

use std::collections::HashMap;

use super::ProviderEntry;
use super::config::bcip_home;

/// 返回内置的 6 个默认 provider 条目。
pub(crate) fn default_providers() -> HashMap<String, ProviderEntry> {
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
pub(crate) fn load_provider_overrides() -> HashMap<String, ProviderEntry> {
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
pub(crate) fn build_provider_registry() -> HashMap<String, ProviderEntry> {
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

/// 根据模型名称推断 provider 名（用于 registry 查找）。
pub(crate) fn model_to_provider_name(model: &str) -> &'static str {
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
pub fn detect_provider(model: &str) -> super::AgentProvider {
    let provider_name = model_to_provider_name(model);
    let registry = build_provider_registry();

    let entry = match registry.get(provider_name) {
        Some(e) => e,
        None => {
            // 不应发生：model_to_provider_name 总是返回已知 provider。
            return super::AgentProvider::Anthropic {
                api_key_env: "ANTHROPIC_API_KEY".to_string(),
            };
        }
    };

    if provider_name == "anthropic" {
        super::AgentProvider::Anthropic {
            api_key_env: entry.api_key_env.clone(),
        }
    } else {
        super::AgentProvider::OpenAiCompatible {
            base_url: entry.base_url.clone(),
            api_key_env: entry.api_key_env.clone(),
        }
    }
}

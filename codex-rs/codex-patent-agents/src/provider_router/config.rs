//! 配置辅助函数：BCIP 路径、环境变量映射、base URL 解析。

use super::registry::{build_provider_registry, model_to_provider_name};

/// 返回 `$BCIP_HOME`（或 `$CODEX_HOME`）路径，默认 `~/.bcip`。
pub(crate) fn bcip_home() -> std::path::PathBuf {
    std::env::var("BCIP_HOME")
        .or_else(|_| std::env::var("CODEX_HOME"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| dirs_home().join(".bcip"))
}

pub(crate) fn dirs_home() -> std::path::PathBuf {
    std::env::var("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/"))
}

/// 将 API key 环境变量名映射到 provider 名。
pub(crate) fn env_var_to_provider_name(env_var: &str) -> &'static str {
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

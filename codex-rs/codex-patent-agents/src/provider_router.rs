//! 多 LLM provider 路由
//!
//! 支持检测模型类型并路由到对应的 LLM provider。

#[derive(Debug, Clone)]
pub enum AgentProvider {
    Anthropic,
    OpenAiCompatible {
        base_url: String,
        api_key_env: String,
    },
}

/// 根据模型名称检测对应的 provider
pub fn detect_provider(model: &str) -> AgentProvider {
    let lower = model.to_ascii_lowercase();

    if lower.starts_with("claude-") || lower.contains("opus") || lower.contains("sonnet") || lower.contains("haiku") {
        return AgentProvider::Anthropic;
    }

    if lower.contains("deepseek") || lower.starts_with("ds-") {
        return AgentProvider::OpenAiCompatible {
            base_url: "https://api.deepseek.com".to_string(),
            api_key_env: "DEEPSEEK_API_KEY".to_string(),
        };
    }

    if lower.contains("qwen") {
        return AgentProvider::OpenAiCompatible {
            base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key_env: "QWEN_API_KEY".to_string(),
        };
    }

    if lower.contains("kimi") || lower.contains("moonshot") {
        return AgentProvider::OpenAiCompatible {
            base_url: "https://api.moonshot.cn/v1".to_string(),
            api_key_env: "MOONSHOT_API_KEY".to_string(),
        };
    }

    if lower.contains("glm") || lower.contains("chatglm") {
        return AgentProvider::OpenAiCompatible {
            base_url: "https://open.bigmodel.cn/api/paas/v4".to_string(),
            api_key_env: "GLM_API_KEY".to_string(),
        };
    }

    if lower.starts_with("gpt-") || lower.starts_with("o1-") || lower.starts_with("o3-") || lower.starts_with("o4-") || lower.contains("chatgpt") {
        return AgentProvider::OpenAiCompatible {
            base_url: "https://api.openai.com/v1".to_string(),
            api_key_env: "OPENAI_API_KEY".to_string(),
        };
    }

    AgentProvider::Anthropic
}

/// 从环境变量解析 API key
pub fn resolve_api_key(env_var: &str) -> String {
    if let Ok(val) = std::env::var(env_var)
        && !val.is_empty() {
            return val;
        }

    resolve_api_key_from_config(env_var)
}

/// 从配置文件解析 API key（预留）
pub fn resolve_api_key_from_config(_env_var: &str) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_claude() {
        let provider = detect_provider("claude-3-5-sonnet-20241022");
        assert!(matches!(provider, AgentProvider::Anthropic));
    }

    #[test]
    fn test_detect_deepseek() {
        let provider = detect_provider("deepseek-v4-pro");
        assert!(matches!(provider, AgentProvider::OpenAiCompatible { .. }));
        if let AgentProvider::OpenAiCompatible { api_key_env, .. } = provider {
            assert_eq!(api_key_env, "DEEPSEEK_API_KEY");
        }
    }

    #[test]
    fn test_detect_qwen() {
        let provider = detect_provider("qwen-turbo");
        assert!(matches!(provider, AgentProvider::OpenAiCompatible { .. }));
        if let AgentProvider::OpenAiCompatible { api_key_env, .. } = provider {
            assert_eq!(api_key_env, "QWEN_API_KEY");
        }
    }

    #[test]
    fn test_detect_gpt() {
        let provider = detect_provider("gpt-4o");
        assert!(matches!(provider, AgentProvider::OpenAiCompatible { .. }));
        if let AgentProvider::OpenAiCompatible { api_key_env, .. } = provider {
            assert_eq!(api_key_env, "OPENAI_API_KEY");
        }
    }

    #[test]
    fn test_detect_default() {
        let provider = detect_provider("unknown-model");
        assert!(matches!(provider, AgentProvider::Anthropic));
    }
}
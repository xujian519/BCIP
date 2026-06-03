//! 多 LLM provider 路由
//!
//! 支持检测模型类型并路由到对应的 LLM provider。
//! 默认 base URL 从内置 registry 提供，可通过 `$BCIP_HOME/config.toml`
//! 的 `[providers.*]` 段覆盖。

pub mod api_key;
pub mod config;
pub mod registry;

// ── 公共类型 ──────────────────────────────────────────────────────────

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

// ── Re-export 公共 API ────────────────────────────────────────────────

pub use api_key::mask_api_key;
pub use api_key::resolve_api_key;
pub use api_key::resolve_api_key_from_config;
pub use api_key::resolve_provider_api_key;
pub use config::resolve_base_url;
pub use registry::detect_provider;

// ── 测试 ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use codex_patent_core::ApiKeyError;

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
        let defaults = registry::default_providers();
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
        assert_eq!(
            registry::model_to_provider_name("claude-3-opus"),
            "anthropic"
        );
        assert_eq!(registry::model_to_provider_name("deepseek-v4"), "deepseek");
        assert_eq!(registry::model_to_provider_name("ds-xxx"), "deepseek");
        assert_eq!(registry::model_to_provider_name("qwen-turbo"), "qwen");
        assert_eq!(registry::model_to_provider_name("moonshot-v1"), "moonshot");
        assert_eq!(registry::model_to_provider_name("kimi-latest"), "moonshot");
        assert_eq!(registry::model_to_provider_name("glm-4"), "glm");
        assert_eq!(registry::model_to_provider_name("chatglm-3"), "glm");
        assert_eq!(registry::model_to_provider_name("gpt-4o"), "openai");
        assert_eq!(registry::model_to_provider_name("o1-preview"), "openai");
        assert_eq!(registry::model_to_provider_name("unknown"), "anthropic");
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

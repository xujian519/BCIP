use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::Duration;

/// 可重试的错误类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)] // RateLimit/Transient variants reserved for future retry policies
pub enum RetryableError {
    Timeout,
    Network,
    RateLimit,
    Transient,
}

/// 单个工具的重试策略。
#[derive(Debug, Clone)]
pub struct ToolRetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub retry_on: Vec<RetryableError>,
    /// 降级链：主工具失败后依次尝试的替代工具名。
    #[allow(dead_code)] // 待降级链功能接线后移除
    pub fallback_chain: Vec<String>,
}

impl ToolRetryPolicy {
    /// 是否应该对该错误进行重试。
    pub fn should_retry(&self, error: &str) -> bool {
        if self.max_retries == 0 {
            return false;
        }
        let lower = error.to_lowercase();
        self.retry_on.iter().any(|kind| match kind {
            RetryableError::Timeout => lower.contains("timeout") || lower.contains("timed out"),
            RetryableError::Network => lower.contains("network") || lower.contains("connection"),
            RetryableError::RateLimit => lower.contains("429") || lower.contains("rate limit"),
            RetryableError::Transient => lower.contains("transient") || lower.contains("temporary"),
        })
    }
}

/// 指数退避 + 抖动（委托到 `ExponentialBackoff`）。
pub fn backoff(base: Duration, attempt: u32) -> Duration {
    let preset =
        crate::resilience::ExponentialBackoff::new(base.as_millis() as u64, 30_000, 2.0, 0.1);
    preset.delay_for_attempt(attempt)
}

// ── 静态策略配置表 ──

static TOOL_RETRY_POLICIES: LazyLock<HashMap<&'static str, ToolRetryPolicy>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();

        m.insert(
            "patent_search",
            ToolRetryPolicy {
                max_retries: 2,
                base_delay: Duration::from_millis(500),
                retry_on: vec![RetryableError::Timeout, RetryableError::Network],
                fallback_chain: vec!["google_patents_fetch".into(), "knowledge_search".into()],
            },
        );

        m.insert(
            "google_patents_fetch",
            ToolRetryPolicy {
                max_retries: 2,
                base_delay: Duration::from_millis(500),
                retry_on: vec![RetryableError::Timeout, RetryableError::Network],
                fallback_chain: vec!["knowledge_search".into()],
            },
        );

        m.insert(
            "ocr_bridge",
            ToolRetryPolicy {
                max_retries: 1,
                base_delay: Duration::from_millis(800),
                retry_on: vec![RetryableError::Timeout],
                fallback_chain: vec![],
            },
        );

        m
    });

/// 获取指定工具的重试策略，无配置返回 None。
pub fn get_retry_policy(tool_name: &str) -> Option<&'static ToolRetryPolicy> {
    TOOL_RETRY_POLICIES.get(tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_retry_timeout() {
        let policy = ToolRetryPolicy {
            max_retries: 2,
            base_delay: Duration::from_millis(100),
            retry_on: vec![RetryableError::Timeout],
            fallback_chain: vec![],
        };
        assert!(policy.should_retry("request timeout after 30s"));
        assert!(!policy.should_retry("invalid parameter"));
    }

    #[test]
    fn no_retry_when_max_zero() {
        let policy = ToolRetryPolicy {
            max_retries: 0,
            base_delay: Duration::from_millis(100),
            retry_on: vec![RetryableError::Timeout],
            fallback_chain: vec![],
        };
        assert!(!policy.should_retry("timeout"));
    }

    #[test]
    fn backoff_increases() {
        let base = Duration::from_millis(100);
        let d0 = backoff(base, 0);
        let d1 = backoff(base, 1);
        let d2 = backoff(base, 2);
        assert!(d0 < d1);
        assert!(d1 < d2);
    }

    #[test]
    fn known_policies_exist() {
        assert!(get_retry_policy("patent_search").is_some());
        assert!(get_retry_policy("google_patents_fetch").is_some());
        assert!(get_retry_policy("ocr_bridge").is_some());
        assert!(get_retry_policy("unknown_tool").is_none());
    }

    #[test]
    fn fallback_chain_order() {
        let policy = get_retry_policy("patent_search").unwrap();
        assert_eq!(
            policy.fallback_chain,
            vec!["google_patents_fetch", "knowledge_search"]
        );
    }
}

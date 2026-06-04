//! 弹性调用辅助工具。
//!
//! 提供带熔断器保护的调用辅助函数，减少每个外部 API 调用点的样板代码。
//! 支持同步（blocking）和异步两种调用模式。

use std::sync::Arc;

use super::backoff::ExponentialBackoff;
use super::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};

/// 弹性调用配置。
#[derive(Debug, Clone)]
pub struct ResilientCallConfig {
    /// 服务名称（用于熔断器注册和日志）。
    pub service_name: String,
    /// 最大重试次数（不含首次调用）。
    pub max_retries: u32,
    /// 退避策略。
    pub backoff: ExponentialBackoff,
    /// 熔断器配置。
    pub circuit_breaker_config: CircuitBreakerConfig,
}

impl ResilientCallConfig {
    /// Agent LLM 调用预设（同步阻塞）。
    ///
    /// max_retries=3, standard退避(1s-30s), 熔断阈值5次失败。
    pub fn agent_llm() -> Self {
        Self {
            service_name: "agent-llm".into(),
            max_retries: 3,
            backoff: ExponentialBackoff::standard(),
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 5,
                failure_rate_threshold: 0.5,
                window_size: 20,
                reset_timeout: std::time::Duration::from_secs(60),
                half_open_max_calls: 3,
            },
        }
    }

    /// Google Patents 调用预设（异步）。
    ///
    /// max_retries=2, conservative退避(2s-60s), 熔断阈值3次失败。
    pub fn google_patents() -> Self {
        Self {
            service_name: "google-patents".into(),
            max_retries: 2,
            backoff: ExponentialBackoff::conservative(),
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 3,
                failure_rate_threshold: 0.6,
                window_size: 10,
                reset_timeout: std::time::Duration::from_secs(120),
                half_open_max_calls: 2,
            },
        }
    }

    /// Embedding 服务调用预设（同步阻塞）。
    ///
    /// max_retries=2, aggressive退避(100ms-5s), 熔断阈值5次失败。
    pub fn embedding() -> Self {
        Self {
            service_name: "embedding".into(),
            max_retries: 2,
            backoff: ExponentialBackoff::aggressive(),
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 5,
                failure_rate_threshold: 0.5,
                window_size: 20,
                reset_timeout: std::time::Duration::from_secs(30),
                half_open_max_calls: 3,
            },
        }
    }

    /// LLM Responses API 预设（流式）。
    ///
    /// max_retries=0（流式调用由 SSE 重连处理）, standard退避, 熔断阈值5次。
    pub fn responses_api() -> Self {
        Self {
            service_name: "responses-api".into(),
            max_retries: 0,
            backoff: ExponentialBackoff::standard(),
            circuit_breaker_config: CircuitBreakerConfig {
                failure_threshold: 5,
                failure_rate_threshold: 0.5,
                window_size: 20,
                reset_timeout: std::time::Duration::from_secs(60),
                half_open_max_calls: 3,
            },
        }
    }
}

/// 熔断器拒绝错误。
#[derive(Debug)]
pub struct CircuitOpenError {
    pub service: String,
    pub state: super::circuit_breaker::CircuitState,
}

impl std::fmt::Display for CircuitOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "circuit breaker open for service '{}', state={:?}",
            self.service, self.state
        )
    }
}

impl std::error::Error for CircuitOpenError {}

/// 使用熔断器保护的同步调用。
///
/// 如果熔断器处于 Open 状态，立即返回错误（不发出网络请求）。
/// 调用失败时记录到熔断器，成功时重置。
///
/// **注意**: 重试逻辑由调用方自行控制（各调用点已有不同的重试策略）。
/// 此函数仅提供熔断器保护层。
pub fn call_with_breaker<F, T, E>(
    breaker: &Arc<dyn CircuitBreaker>,
    service_name: &str,
    f: F,
) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: From<CircuitOpenError>,
{
    if !breaker.allow_request() {
        return Err(E::from(CircuitOpenError {
            service: service_name.to_string(),
            state: breaker.state(),
        }));
    }

    match f() {
        Ok(val) => {
            breaker.record_success();
            Ok(val)
        }
        Err(e) => {
            breaker.record_failure();
            Err(e)
        }
    }
}

/// 使用熔断器保护的异步调用。
///
/// 与 [`call_with_breaker`] 相同的语义，但用于 async 上下文。
pub async fn call_with_breaker_async<F, Fut, T, E>(
    breaker: &Arc<dyn CircuitBreaker>,
    service_name: &str,
    f: F,
) -> Result<T, E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: From<CircuitOpenError>,
{
    if !breaker.allow_request() {
        return Err(E::from(CircuitOpenError {
            service: service_name.to_string(),
            state: breaker.state(),
        }));
    }

    match f().await {
        Ok(val) => {
            breaker.record_success();
            Ok(val)
        }
        Err(e) => {
            breaker.record_failure();
            Err(e)
        }
    }
}

/// 计算重试退避延迟并同步 sleep。
pub fn sleep_backoff(backoff: &ExponentialBackoff, attempt: u32) {
    let delay = backoff.delay_for_attempt(attempt);
    std::thread::sleep(delay);
}

/// 计算重试退避延迟并异步 sleep。
pub async fn sleep_backoff_async(backoff: &ExponentialBackoff, attempt: u32) {
    let delay = backoff.delay_for_attempt(attempt);
    tokio::time::sleep(delay).await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resilience::circuit_breaker::{CircuitBreakerConfig, StdCircuitBreaker};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    fn test_breaker() -> Arc<dyn CircuitBreaker> {
        Arc::new(StdCircuitBreaker::new(
            "test".into(),
            CircuitBreakerConfig {
                failure_threshold: 2,
                failure_rate_threshold: 0.0,
                window_size: 0,
                reset_timeout: std::time::Duration::from_millis(50),
                half_open_max_calls: 1,
            },
        ))
    }

    #[test]
    fn call_with_breaker_success() {
        let breaker = test_breaker();
        let result: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || Ok(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn call_with_breaker_failure_records() {
        let breaker = test_breaker();
        let call_count = AtomicU32::new(0);
        let _: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || {
                call_count.fetch_add(1, Ordering::SeqCst);
                Err(CircuitOpenError {
                    service: "test".into(),
                    state: super::super::circuit_breaker::CircuitState::Open,
                })
            });
        let _: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || {
                call_count.fetch_add(1, Ordering::SeqCst);
                Err(CircuitOpenError {
                    service: "test".into(),
                    state: super::super::circuit_breaker::CircuitState::Open,
                })
            });

        // 2 failures → breaker should be Open
        assert_eq!(
            breaker.state(),
            super::super::circuit_breaker::CircuitState::Open
        );
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn call_with_breaker_open_rejects() {
        let breaker = test_breaker();

        // 触发熔断
        let _: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || {
                Err(CircuitOpenError {
                    service: "test".into(),
                    state: super::super::circuit_breaker::CircuitState::Open,
                })
            });
        let _: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || {
                Err(CircuitOpenError {
                    service: "test".into(),
                    state: super::super::circuit_breaker::CircuitState::Open,
                })
            });

        // 第3次应该被拒绝（不调用闭包）
        let called = AtomicU32::new(0);
        let result: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || {
                called.fetch_add(1, Ordering::SeqCst);
                Ok(1)
            });
        assert!(result.is_err());
        assert_eq!(called.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn call_with_breaker_success_resets_consecutive() {
        let breaker = test_breaker();

        // 1 failure
        let _: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || {
                Err(CircuitOpenError {
                    service: "test".into(),
                    state: super::super::circuit_breaker::CircuitState::Open,
                })
            });

        // 1 success → resets consecutive
        let result: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || Ok(42));
        assert_eq!(result.unwrap(), 42);

        // 1 more failure → NOT tripped (consecutive was reset)
        let _: Result<i32, CircuitOpenError> =
            call_with_breaker(&breaker, "test", || {
                Err(CircuitOpenError {
                    service: "test".into(),
                    state: super::super::circuit_breaker::CircuitState::Open,
                })
            });

        // Should still be Closed
        assert_eq!(
            breaker.state(),
            super::super::circuit_breaker::CircuitState::Closed
        );
    }

    #[tokio::test]
    async fn call_with_breaker_async_success() {
        let breaker = test_breaker();
        let result: Result<i32, CircuitOpenError> = call_with_breaker_async(
            &breaker,
            "test",
            || async { Ok::<i32, CircuitOpenError>(42) },
        )
        .await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn call_with_breaker_async_open_rejects() {
        let breaker = test_breaker();

        // 触发熔断
        let _ = call_with_breaker_async(&breaker, "test", || async {
            Err::<i32, CircuitOpenError>(CircuitOpenError {
                service: "test".into(),
                state: super::super::circuit_breaker::CircuitState::Open,
            })
        })
        .await;
        let _ = call_with_breaker_async(&breaker, "test", || async {
            Err::<i32, CircuitOpenError>(CircuitOpenError {
                service: "test".into(),
                state: super::super::circuit_breaker::CircuitState::Open,
            })
        })
        .await;

        // 应被拒绝
        let result: Result<i32, CircuitOpenError> =
            call_with_breaker_async(&breaker, "test", || async { Ok::<i32, CircuitOpenError>(42) })
                .await;
        assert!(result.is_err());
    }

    #[test]
    fn preset_configs_have_service_names() {
        assert_eq!(ResilientCallConfig::agent_llm().service_name, "agent-llm");
        assert_eq!(
            ResilientCallConfig::google_patents().service_name,
            "google-patents"
        );
        assert_eq!(ResilientCallConfig::embedding().service_name, "embedding");
        assert_eq!(
            ResilientCallConfig::responses_api().service_name,
            "responses-api"
        );
    }
}
